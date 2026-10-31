#!/usr/bin/env python3
"""Run a certified full suite for one frozen commit."""
import argparse
import fcntl
import hashlib
import json
import os
import random
import re
import subprocess
import sys
import tempfile
import time
from contextlib import contextmanager
from pathlib import Path

ROOT = Path.home() / "Projects/wow"
REPO = ROOT / "wow-ui-sim"
RESULTS = ROOT / "full-suite-results"
LOCK = RESULTS / ".lock"
JOBS = os.environ.get("FULL_SUITE_JOBS", "16")
PROFILES = ("retail", "ptr", "wrath", "mists", "wowforever", "era", "anniversary")
# Slot 2 was retired on 2026-10-27 (user decision: three build slots).
SLOTS = (0, 1, 3)
FAILED = re.compile(r"^test (\S+) \.\.\. FAILED", re.M)


def git(*args, cwd=REPO):
    return subprocess.run(["git", *args], cwd=cwd, check=True, capture_output=True, text=True).stdout.strip()


def resolve(ref):
    git("fetch", "-q", "origin")
    return git("rev-parse", f"{ref}^{{commit}}")


def prepare_checkout(sha):
    checkout = Path.home() / ".worktrees" / f"wow-ui-sim-fullsuite-{sha[:12]}"
    if not checkout.exists():
        checkout.parent.mkdir(parents=True, exist_ok=True)
        git("worktree", "add", "--detach", str(checkout), sha)
    if git("rev-parse", "HEAD", cwd=checkout) != sha:
        raise ValueError(f"worktree has wrong HEAD: {checkout}")
    if git("status", "--porcelain", "--untracked-files=all", cwd=checkout):
        raise ValueError(f"worktree has changes: {checkout}")
    return checkout


def validate_target_dir(value):
    target = Path(value)
    if not target.is_absolute():
        raise ValueError("target dir must be absolute")
    target = target.resolve()
    slots = {Path.home() / f".cache/wow-ui-sim-targets/slot-{i}" for i in SLOTS}
    if target not in slots:
        raise ValueError(f"target dir must resolve to an approved cache slot {SLOTS}")
    return target


def positive_build_jobs(value):
    jobs = int(value)
    if jobs < 1:
        raise argparse.ArgumentTypeError("build jobs must be positive")
    return jobs


def claim_slot(target, sha, profile):
    capacity = Path.home() / ".worktrees/logs/capacity.sh"
    env = dict(os.environ, CAPACITY_ID=f"full-suite-{sha[:12]}-{profile}")
    env.pop("CAPACITY_PEEK", None)
    result = subprocess.run([str(capacity)], env=env, capture_output=True, text=True)
    if result.returncode or "| ADD:" not in result.stdout:
        raise ValueError(f"capacity refused: {result.stdout.strip()} {result.stderr.strip()}")
    owner = f"full-suite {sha[:12]} {profile}"
    try:
        fd = os.open(target / ".owner", os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    except FileExistsError as error:
        raise ValueError(f"slot owner already exists: {target / '.owner'}") from error
    with os.fdopen(fd, "w") as stream:
        stream.write(owner)
    return owner


def release_slot(target, sha, profile, owner):
    claim = target / ".owner"
    if claim.read_text() != owner:
        raise ValueError(f"slot owner changed: {claim}")
    dest = Path.home() / ".local/state/wow-ui-sim/handoff/full-suite" / f"{sha[:12]}-{profile}"
    dest.mkdir(parents=True, exist_ok=True)
    claim.replace(dest / f"{target.name}.owner.released")


class SlotStateUnknown(Exception):
    pass


def show_unit(unit, slot):
    for attempt in range(3):
        try:
            state = subprocess.run(["systemctl", "--user", "show", unit, "-p", "LoadState", "-p", "ActiveState", "-p", "SubState", "-p", "Result"], capture_output=True, text=True)
        except OSError:
            state = None
        if state is not None and state.returncode == 0:
            fields = dict(line.split("=", 1) for line in state.stdout.splitlines() if "=" in line)
            if fields.get("ActiveState") or fields.get("LoadState") == "not-found":
                return fields
        if attempt < 2:
            time.sleep((0.1 * 2 ** attempt) + random.uniform(0, 0.05))
    raise SlotStateUnknown(f"slot {slot} retained: unit {unit} state unknown")


def confirm_terminated(unit, slot):
    fields = show_unit(unit, slot)
    if fields.get("ActiveState") in ("inactive", "failed") or fields.get("LoadState") == "not-found":
        return fields
    try:
        stopped = subprocess.run(["systemctl", "--user", "stop", unit], capture_output=True, text=True)
    except OSError as error:
        raise SlotStateUnknown(f"slot {slot} retained: unit {unit} state unknown") from error
    if stopped.returncode:
        raise SlotStateUnknown(f"slot {slot} retained: unit {unit} state unknown")
    fields = show_unit(unit, slot)
    if fields.get("ActiveState") not in ("inactive", "failed") and fields.get("LoadState") != "not-found":
        raise SlotStateUnknown(f"slot {slot} retained: unit {unit} state unknown")
    return fields


def build(checkout, target, sha, profile, jobs, owner, log):
    slot = target.name.removeprefix("slot-")
    unit = f"full-suite-build-{sha[:12]}-{profile}-{time.time_ns()}"
    try:
        return build_unit(checkout, target, profile, jobs, owner, log, unit, slot)
    finally:
        confirm_terminated(unit, slot)


def build_unit(checkout, target, profile, jobs, owner, log, unit, slot):
    features = [] if profile == "retail" else ["--no-default-features", "--features", f"sound,gui,casc,client-{profile}"]
    cmd = ["slot-build.py", "--worktree", str(checkout), "--slot", slot, "--unit", unit,
           "--owner", owner, "--capacity-id", unit, "--", "cargo", "test", "--no-run",
           "--all-targets", "--locked", "--offline", "--message-format=json", f"-j{jobs}", *features]
    env = dict(os.environ, PATH=os.environ.get("PATH", "") + os.pathsep + str(REPO / "scripts"))
    proc = subprocess.run(cmd, env=env, capture_output=True, text=True)
    log.write(f"===== build: {' '.join(cmd)} (exit {proc.returncode})\n{proc.stdout}{proc.stderr}\n")
    log.flush()
    if proc.returncode:
        raise ValueError(f"slot-build guard rejected: {proc.stderr.strip() or proc.stdout.strip()}")
    receipt = target / "receipts" / f"{unit}.json"
    while True:
        fields = show_unit(unit, slot)
        if fields.get("ActiveState") in ("inactive", "failed") or fields.get("LoadState") == "not-found":
            if fields.get("Result") != "success":
                detail = ""
                if receipt.exists():
                    failed_receipt = json.loads(receipt.read_text())
                    detail = f": {failed_receipt.get('result')}: {failed_receipt.get('dep_info_errors')}"
                raise ValueError(f"slot-build unit failed: {fields.get('Result')}{detail}")
            break
        if fields.get("SubState") == "exited":
            confirm_terminated(unit, slot)
        time.sleep(1)
    if not receipt.exists():
        raise ValueError("slot-build certification receipt missing")
    data = json.loads(receipt.read_text())
    if data.get("result") != "success" or data.get("returncode") != 0:
        raise ValueError(f"slot-build certification failed: {data.get('result')}: {data.get('dep_info_errors')}")
    return data


def test_artifact(receipt, name, kind):
    matches = [r["executable"] for r in receipt.get("reported_artifacts", [])
               if r.get("executable") and r.get("profile", {}).get("test")
               and r.get("target", {}).get("name") == name
               and kind in r.get("target", {}).get("kind", [])]
    if len(matches) != 1:
        raise ValueError(f"certified test artifact missing or ambiguous: {name}")
    return matches[0]


def run_step(name, cmd, checkout, log, env):
    start = time.monotonic()
    proc = subprocess.run(cmd, cwd=checkout, env=env, capture_output=True, text=True)
    output = proc.stdout + proc.stderr
    log.write(f"===== {name}: {' '.join(cmd)} (exit {proc.returncode})\n{output}\n")
    log.flush()
    return {"exit": proc.returncode, "seconds": round(time.monotonic() - start, 1),
            "command": cmd, "environment": {"WOW_SIM_TIMEOUT_FIXTURE_CAPACITY": env.get("WOW_SIM_TIMEOUT_FIXTURE_CAPACITY")}}, output


def new_failures(result):
    base_path = RESULTS / f"master-latest-{result['profile']}.json"
    if not base_path.exists():
        return None
    base = json.loads(base_path.read_text())
    return {name: sorted(set(fails) - set(base["failures"].get(name, []))) for name, fails in result["failures"].items()}


def read_cvars(source):
    fd = os.open(source, os.O_RDONLY | os.O_NOATIME)
    with os.fdopen(fd, "rb") as stream:
        data = stream.read()
        stat = os.fstat(stream.fileno())
    return data, {"sha256": hashlib.sha256(data).hexdigest(), "device": stat.st_dev,
                  "inode": stat.st_ino, "size": stat.st_size, "atime_ns": stat.st_atime_ns,
                  "mtime_ns": stat.st_mtime_ns, "ctime_ns": stat.st_ctime_ns}


@contextmanager
def isolated_cvars(env, log):
    source = Path(env.get("XDG_DATA_HOME", str(Path.home() / ".local/share"))) / "wow-sim/cvars.json"
    before_data, before = read_cvars(source)
    log.write(f"cvar-before {json.dumps(before)}\n")
    try:
        with tempfile.TemporaryDirectory(prefix="full-suite-cvars-") as tmp:
            dest = Path(tmp) / "wow-sim/cvars.json"
            dest.parent.mkdir(parents=True)
            dest.write_bytes(before_data)
            yield dict(env, XDG_DATA_HOME=tmp)
    finally:
        active_error = sys.exc_info()[1]
        try:
            _, after = read_cvars(source)
            log.write(f"cvar-after {json.dumps(after)}\n")
            if before != after:
                raise ValueError("persisted CVar file changed during suite")
        except (OSError, ValueError) as error:
            if not isinstance(active_error, SlotStateUnknown):
                raise
            active_error.cleanup_error = str(error)


def write_result(path, result):
    with tempfile.NamedTemporaryFile(mode="w", dir=path.parent, delete=False) as stream:
        temporary = Path(stream.name)
        try:
            stream.write(json.dumps(result, indent=2) + "\n")
        except BaseException:
            temporary.unlink(missing_ok=True)
            raise
    try:
        temporary.replace(path)
    finally:
        temporary.unlink(missing_ok=True)


def promote_result(canonical, baseline, result):
    destinations = ([baseline] if baseline else []) + [canonical]
    staged = {}
    backups = {}
    replaced = []
    try:
        for path in destinations:
            with tempfile.NamedTemporaryFile(mode="w", dir=path.parent, delete=False) as stream:
                staged[path] = Path(stream.name)
                stream.write(json.dumps(result, indent=2) + "\n")
                stream.flush()
                os.fsync(stream.fileno())
        for path in destinations:
            if path.exists() or path.is_symlink():
                with tempfile.NamedTemporaryFile(dir=path.parent, delete=False) as stream:
                    backup = Path(stream.name)
                backup.unlink()
                os.link(path, backup, follow_symlinks=False)
                backups[path] = backup
        for path in destinations:
            staged[path].replace(path)
            replaced.append(path)
    except BaseException:
        for path in reversed(replaced):
            if path in backups:
                backups[path].replace(path)
            else:
                path.unlink(missing_ok=True)
        raise
    finally:
        for path in (*staged.values(), *backups.values()):
            path.unlink(missing_ok=True)


def run(ref, build_jobs, target_dir, profile="retail"):
    target_dir = validate_target_dir(target_dir)
    RESULTS.mkdir(parents=True, exist_ok=True)
    with open(LOCK, "w") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        sha = resolve(ref)
        steps = {name: {"status": "not_run"} for name in ("integration", "prefork", "lib")}
        if profile not in ("retail", "mists"):
            steps["prefork"] = {"status": "not_applicable", "reason": "prefork_full_ui supports only Retail/Mists"}
        failures = {name: [] for name, step in steps.items() if step["status"] != "not_applicable"}
        result = {"ref": ref, "sha": sha, "profile": profile, "started": time.strftime("%Y-%m-%dT%H:%M:%S%z"), "steps": steps, "failures": failures}
        phase = "setup"
        output_path = RESULTS / f"{sha}-{profile}.json"
        attempt = f"{sha}-{profile}-attempt-{time.time_ns()}"
        log_path = RESULTS / f"{attempt}.log"
        result["log"] = str(log_path)
        owner = None
        try:
            with open(log_path, "w") as log:
                try:
                    checkout = prepare_checkout(sha)
                    owner = claim_slot(target_dir, sha, profile)
                    phase = "build"
                    receipt = build(checkout, target_dir, sha, profile, build_jobs, owner, log)
                    phase = "setup"
                    env = dict(os.environ, CARGO_TARGET_DIR=str(target_dir), RUST_TEST_THREADS=JOBS, CARGO_TERM_COLOR="never")
                    with isolated_cvars(env, log) as env:
                        for name, artifact, kind in [("integration", "integration", "test"), ("prefork", "prefork_full_ui", "test"), ("lib", "wow_ui_sim", "lib")]:
                            if name == "prefork" and profile not in ("retail", "mists"):
                                result["steps"][name] = {"status": "not_applicable", "reason": "prefork_full_ui supports only Retail/Mists"}
                                continue
                            phase = name
                            binary = test_artifact(receipt, artifact, kind)
                            # Linux test executables link libwow_ui_sim_dylib.so from the same deps directory.
                            env["LD_LIBRARY_PATH"] = str(Path(binary).parent)
                            mount = Path.home() / ".cache/wow-ui-sim-srcmount" / target_dir.name
                            unit = f"full-suite-test-{sha[:12]}-{profile}-{name}-{time.time_ns()}"
                            command = ["systemd-run", "--user", f"--unit={unit}", "--wait", "--pipe", "--collect", "--quiet",
                                       "-p", "MemoryMax=9G", "-p", "MemorySwapMax=0",
                                       "-p", f"BindPaths={checkout}:{mount}", f"--working-directory={mount}",
                                       *(f"--setenv={name}" for name in env), "--", binary]
                            try:
                                result["steps"][name], output = run_step(name, command, checkout, log, env)
                            finally:
                                confirm_terminated(unit, target_dir.name.removeprefix("slot-"))
                            if result["steps"][name]["exit"]:
                                result["steps"][name]["status"] = "FAILED"
                            result["failures"][name] = sorted(set(FAILED.findall(output)))
                    phase = "setup"
                    result["status"] = "FAILED" if any(step.get("exit", 0) for step in result["steps"].values()) else "PASSED"
                except SlotStateUnknown as error:
                    result["steps"].setdefault(phase, {})["status"] = "FAILED"
                    result["status"] = "FAILED"
                    result["error"] = str(error)
                    if hasattr(error, "cleanup_error"):
                        result["cleanup_error"] = error.cleanup_error
                    owner = None  # Keep the live or unknown claim for reconciliation.
                    try:
                        log.write(f"FAILED: {error}\n")
                    except OSError as log_error:
                        result["log_error"] = str(log_error)
                except Exception as error:
                    result["steps"].setdefault(phase, {})["status"] = "FAILED"
                    result["status"] = "FAILED"
                    result["error"] = str(error)
                    try:
                        log.write(f"FAILED: {error}\n")
                    except OSError as log_error:
                        result["log_error"] = str(log_error)
        except Exception as error:
            result["steps"].setdefault(phase, {})["status"] = "FAILED"
            result["status"] = "FAILED"
            result["log_error"] = str(error)
            result.setdefault("error", str(error))
        result["finished"] = time.strftime("%Y-%m-%dT%H:%M:%S%z")
        attempt_path = RESULTS / f"{attempt}.json"
        try:
            result["new_failures"] = new_failures(result)
        except Exception as error:
            result["status"] = "FAILED"
            result["error"] = f"baseline comparison failed: {error}"
        # Record before releasing ownership, so receipt storage failure retains it.
        write_result(attempt_path, result)
        if owner is not None:
            try:
                release_slot(target_dir, sha, profile, owner)
            except Exception as error:
                result["status"] = "FAILED"
                result["error"] = f"slot release failed: {error}"
        if result["status"] == "PASSED":
            try:
                baseline = RESULTS / f"master-latest-{profile}.json" if ref in ("origin/master", "master") else None
                promote_result(output_path, baseline, result)
            except Exception as error:
                result["status"] = "FAILED"
                result["error"] = f"promotion failed: {error}"
        write_result(attempt_path, result)
        print(json.dumps(result, indent=2))
        return 0 if result["status"] == "PASSED" else 1



def submit(ref, build_jobs, target_dir, profile="retail"):
    target_dir = validate_target_dir(target_dir)
    unit = f"full-suite-{profile}-{int(time.time())}"
    subprocess.run(["systemd-run", "--user", f"--unit={unit}", "--collect", "--slice=agents.slice", "-p", "CPUQuota=1200%", "-p", "MemoryMax=9G", "-p", "MemorySwapMax=0", f"--setenv=FULL_SUITE_JOBS={JOBS}", sys.executable, os.path.abspath(__file__), "run", ref, "--profile", profile, "--build-jobs", str(build_jobs), "--target-dir", str(target_dir)], check=True)
    print(unit)


def attempt_phase(attempt):
    steps = attempt.get("steps", {})
    for name, step in steps.items():
        if step.get("status") == "FAILED":
            return name
    if attempt.get("status") == "FAILED":
        return "setup"
    completed = [name for name, step in steps.items()
                 if step.get("status") not in ("not_run", "not_applicable")]
    return completed[-1] if completed else "setup"


def status(ref):
    runs = [p for p in RESULTS.glob("*.json") if not p.name.startswith("master-latest")]
    attempts = [p for p in runs if "-attempt-" in p.name]
    canonicals = [p for p in runs if p not in attempts]
    if ref:
        sha = resolve(ref)
    elif attempts:
        sha = json.loads(max(attempts, key=lambda p: p.stat().st_mtime_ns).read_text())["sha"]
    elif canonicals:
        sha = json.loads(max(canonicals, key=lambda p: p.stat().st_mtime_ns).read_text())["sha"]
    else:
        print("no result yet")
        return
    profiles = {profile: json.loads(path.read_text()) if (path := RESULTS / f"{sha}-{profile}.json").exists() else None for profile in PROFILES}
    result = {"sha": sha, "profiles": profiles}
    legacy = RESULTS / f"{sha}.json"
    if legacy.exists():
        result["legacy_retail"] = json.loads(legacy.read_text())
    selected = [p for p in attempts if p.name.startswith(f"{sha}-")]
    if selected:
        attempt = json.loads(max(selected, key=lambda p: p.stat().st_mtime_ns).read_text())
        result["latest_attempt"] = {**attempt, "phase": attempt_phase(attempt)}
    if not ref and canonicals:
        passed = [(p, json.loads(p.read_text())) for p in canonicals]
        passed = [(p, record) for p, record in passed if record.get("status") == "PASSED"]
        if passed:
            _, newest = max(passed, key=lambda item: item[0].stat().st_mtime_ns)
            if newest["sha"] != sha:
                result["latest_canonical"] = newest
    if not any(profiles.values()) and not legacy.exists() and not selected:
        print("no result yet")
        return
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    actions = parser.add_subparsers(dest="action", required=True)
    for action in ("run", "submit"):
        command = actions.add_parser(action)
        command.add_argument("ref", nargs="?", default="origin/master")
        command.add_argument("--profile", choices=PROFILES, default="retail")
        command.add_argument("--build-jobs", type=positive_build_jobs, default=1)
        command.add_argument("--target-dir", required=True)
    status_parser = actions.add_parser("status")
    status_parser.add_argument("ref", nargs="?")
    args = parser.parse_args(sys.argv[1:] or ["status"])
    if args.action == "status":
        status(args.ref)
    else:
        try:
            if args.action == "run":
                raise SystemExit(run(args.ref, args.build_jobs, args.target_dir, args.profile))
            else:
                submit(args.ref, args.build_jobs, args.target_dir, args.profile)
        except ValueError as error:
            parser.error(str(error))
