#!/usr/bin/env python3
"""Run a certified full suite for one frozen commit."""
import argparse
from collections import deque
from concurrent.futures import ThreadPoolExecutor
import heapq
import io
import fcntl
import hashlib
import json
import os
import random
import re
import subprocess
import sys
import tempfile
import threading
import time
from contextlib import contextmanager
from dataclasses import dataclass
from pathlib import Path

ROOT = Path.home() / "Projects/wow"
REPO = ROOT / "wow-ui-sim"
RESULTS = ROOT / "full-suite-results"
LOCK = RESULTS / ".lock"
JOBS = os.environ.get("FULL_SUITE_JOBS", "16")
# Integration runs many full Blizzard-UI tests (~1.1-1.5 GiB each) in one process; 16 threads exceed
# the 9G unit (cgroup oom-kill, 2026-11-03). User decision: fewer threads for this target.
INTEGRATION_JOBS = os.environ.get("FULL_SUITE_INTEGRATION_JOBS", "3")
MAX_TEST_CPUS = 16
MAX_TEST_MEM_GIB = 24
GIB = 1024 ** 3
# Unknown integration cases are admitted at this share, matching the measured
# 16-worker/24GiB policy rather than reserving each unit's 5GiB ceiling.
WORKER_SHARE_BYTES = 3 * GIB // 2
TAIL_MEMORY_BYTES = 9 * GIB
BATCH_TARGET_SECONDS = 20.0
BATCH_MAX_NAMES = 25
# Exact per-name CPU and memory is only worth a dedicated process where it pays:
# measured expensive cases and cases with a reserved memory envelope.
ISOLATE_SECONDS = 5.0
WORKER = Path(__file__).with_name("full_suite_worker.py")
PROFILES = ("retail", "ptr", "wrath", "mists", "wowforever", "era", "anniversary")
# Slot 2 was retired on 2026-10-27 (user decision: three build slots).
SLOTS = (0, 1, 3)
FAILED = re.compile(r"^test (\S+) \.\.\. FAILED", re.M)


@dataclass(frozen=True)
class TestBudget:
    cpus: int = MAX_TEST_CPUS
    mem_gib: int = MAX_TEST_MEM_GIB

    def __post_init__(self):
        if not 1 <= self.cpus <= MAX_TEST_CPUS or not 1 <= self.mem_gib <= MAX_TEST_MEM_GIB:
            raise ValueError("test budget requires 1-16 CPUs and 1-24 GiB")

    @property
    def memory_bytes(self):
        return self.mem_gib * GIB

    @property
    def shard_count(self):
        return max(1, min(self.cpus, self.mem_gib * MAX_TEST_CPUS // MAX_TEST_MEM_GIB))

    @property
    def timeout_slots(self):
        return max(2, min(6, self.cpus * 6 // MAX_TEST_CPUS,
                          self.mem_gib * 6 // MAX_TEST_MEM_GIB))

    def record(self):
        return {"cpus": self.cpus, "mem_gib": self.mem_gib,
                "integration_shards": self.shard_count, "timeout_child_slots": self.timeout_slots}


def configure_test_slice(unit, budget):
    subprocess.run(["systemctl", "--user", "set-property", "--runtime", unit,
                    f"CPUQuota={budget.cpus * 100}%", f"MemoryMax={budget.mem_gib}G",
                    "MemorySwapMax=0"], check=True, capture_output=True, text=True)


def finish_test_slice(unit):
    try:
        result = subprocess.run(["systemctl", "--user", "show", unit, "-p", "CPUUsageNSec", "-p", "MemoryPeak"],
                                check=True, capture_output=True, text=True)
        fields = dict(line.split("=", 1) for line in result.stdout.splitlines() if "=" in line)
        cpu = fields.get("CPUUsageNSec", "")
        peak = fields.get("MemoryPeak", "")
        return {"cpu_seconds": int(cpu) / 1e9 if cpu.isdecimal() else None,
                "memory_peak_bytes": int(peak) if peak.isdecimal() else None}
    finally:
        subprocess.run(["systemctl", "--user", "stop", unit], check=True, capture_output=True, text=True)


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


def require_capacity(sha, profile):
    capacity = Path.home() / ".worktrees/logs/capacity.sh"
    env = dict(os.environ, CAPACITY_ID=f"full-suite-{sha[:12]}-{profile}")
    env.pop("CAPACITY_PEEK", None)
    result = subprocess.run([str(capacity)], env=env, capture_output=True, text=True)
    if result.returncode or "| ADD:" not in result.stdout:
        raise ValueError(f"capacity refused: {result.stdout.strip()} {result.stderr.strip()}")


def claim_slot(target, sha, profile):
    require_capacity(sha, profile)
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


def build(checkout, target, sha, profile, jobs, owner, log, optional=False):
    slot = target.name.removeprefix("slot-")
    unit = f"full-suite-build-{sha[:12]}-{profile}-{time.time_ns()}"
    try:
        return build_unit(checkout, target, profile, jobs, owner, log, unit, slot, optional)
    finally:
        confirm_terminated(unit, slot)


def build_unit(checkout, target, profile, jobs, owner, log, unit, slot, optional=False):
    features = [] if profile == "retail" else ["--no-default-features", "--features", f"sound,gui,casc,client-{profile}"]
    if optional:
        features += ["--features", "optional-tests"]
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


def additional_test_targets(receipt):
    targets = []
    for artifact in receipt.get("reported_artifacts", []):
        target = artifact.get("target", {})
        kind = target.get("kind", [])
        name = target.get("name")
        if (artifact.get("executable") and artifact.get("profile", {}).get("test")
                and name not in ("integration", "prefork_full_ui", "wow_ui_sim") and kind):
            targets.append((f"{kind[0]}:{name}", name, kind[0]))
    return targets


def run_step(name, cmd, checkout, log, env):
    start = time.monotonic()
    proc = subprocess.run(cmd, cwd=checkout, env=env, capture_output=True, text=True)
    output = proc.stdout + proc.stderr
    log.write(f"===== {name}: {' '.join(cmd)} (exit {proc.returncode})\n{output}\n")
    log.flush()
    return {"exit": proc.returncode, "seconds": round(time.monotonic() - start, 1),
            "command": cmd, "environment": {"WOW_SIM_TIMEOUT_FIXTURE_CAPACITY": env.get("WOW_SIM_TIMEOUT_FIXTURE_CAPACITY")}}, output


def shard_memory_bytes(names, memory_costs):
    """Per-unit ceiling: measured envelope plus headroom, never below the conservative floor."""
    peaks = memory_costs["peak_bytes"]
    if not peaks or min(peaks.values()) <= 0 or memory_costs["headroom_percent"] < 0:
        raise ValueError("memory costs require positive byte peaks and nonnegative headroom")
    # Unmeasured cases use the largest observed envelope, not a smaller guess.
    largest = max(peaks.values())
    peak = max(peaks.get(name, largest) for name in names)
    required = (peak * (100 + memory_costs["headroom_percent"]) + 99) // 100
    # A measured small case can still exceed its own envelope when it bootstraps a
    # second full environment, so keep the established 5 GiB conservative cap.
    return max(5 * GIB, ((required + GIB - 1) // GIB) * GIB)


def admission_bytes(names, memory_costs):
    """Admit unknown work at the worker share; reserve measured envelopes only when known."""
    peaks = memory_costs["peak_bytes"]
    measured = max((peaks[name] for name in names if name in peaks), default=0)
    if not measured:
        return WORKER_SHARE_BYTES
    required = (measured * (100 + memory_costs["headroom_percent"]) + 99) // 100
    return max(WORKER_SHARE_BYTES, ((required + GIB - 1) // GIB) * GIB)


def lane_plan(budget, has_tail):
    """Keep integration lanes dense unless a tail job needs a reserved 9GiB slot."""
    if not has_tail:
        return budget.shard_count, 0
    by_memory = (budget.memory_bytes - TAIL_MEMORY_BYTES) // WORKER_SHARE_BYTES
    lanes = max(2, min(budget.shard_count, by_memory, budget.cpus - 1))
    return lanes, max(1, budget.cpus - lanes)


def family_of(name):
    return name.split("::", 1)[0]


def family_queues(names, costs, grouped_families=()):
    """Keep verified grouped families contiguous; cost-order everything else.

    Only a family that installs a shared forked fixture needs its cases in one
    process. Every other family is batched by cost, so a run does not pay one
    process launch per small family.
    """
    verified = set(grouped_families)
    families = {}
    loose = []
    for name in names:
        family = family_of(name)
        if family in verified:
            families.setdefault(family, []).append(name)
        else:
            loose.append(name)
    ordered = sorted(families.items(),
                     key=lambda item: (-sum(costs.get(name, 1) for name in item[1]), item[0]))
    queues = [deque(sorted(cases, key=lambda name: (-costs.get(name, 1), name)))
              for _, cases in ordered]
    if loose:
        queues.append(deque(sorted(loose, key=lambda name: (-costs.get(name, 1), name))))
    return deque(queues)


def take_batch(families, costs, single=False):
    """Take a small cost-bounded batch from the head family."""
    family = families[0]
    batch = [family.popleft()]
    load = costs.get(batch[0], 1)
    while not single and family and len(batch) < BATCH_MAX_NAMES and load < BATCH_TARGET_SECONDS:
        name = family.popleft()
        batch.append(name)
        load += costs.get(name, 1)
    if not family:
        families.popleft()
    return batch


def run_dynamic_suite(command, checkout, log, env, costs, budget, timeout_tests,
                      memory_costs, tail_jobs=(), timing_path=None, grouped_families=()):
    """One shared queue: lanes pull small batches; tail steps overlap where they fit."""
    started = time.monotonic()
    discovery = subprocess.run([command[-1], "--list", "--format=terse"],
                               cwd=checkout, env=env, capture_output=True, text=True)
    names = re.findall(r"^(.+): test$", discovery.stdout, re.M)
    if discovery.returncode or not names or len(names) != len(set(names)):
        raise ValueError(f"integration discovery failed: {discovery.stdout}{discovery.stderr}")
    # These cases reserve exclusive capacity while testing the configured child
    # semaphore. Shared workloads can otherwise starve their exclusive flock.
    reserved = [name for name in names if name.startswith("timeout_reexec::")]
    timeout_tests = set(timeout_tests)
    guarded = [name for name in names if name not in reserved and name in timeout_tests]
    ordinary = [name for name in names if name not in reserved and name not in timeout_tests]
    pending = family_queues(ordinary, costs, grouped_families)
    guards = family_queues(guarded, costs)
    lanes, tail_threads = lane_plan(budget, bool(tail_jobs))
    # Guarded cases may wait on the global permit semaphore; keep running room
    # for ordinary work, and never exceed the configured permit bound.
    guard_cap = max(2, min(budget.timeout_slots, max(1, lanes - 1)))
    if admission_bytes(names, memory_costs) > budget.memory_bytes:
        raise ValueError("test memory budget cannot fit a measured single-case envelope")
    base_unit = next(arg for arg in command if arg.startswith("--unit="))
    slot = Path(env.get("CARGO_TARGET_DIR", "integration")).name.removeprefix("slot-")
    workspace = Path(tempfile.mkdtemp(prefix="full-suite-batches-"))
    lock = threading.Condition()
    state = {"admitted": 0, "guarded": 0}
    records = {}
    steps = []
    tail_results = []
    sequence = [0]

    def next_job():
        with lock:
            while pending or guards:
                if guards and state["guarded"] < guard_cap:
                    families, kind = guards, "guarded"
                elif pending:
                    families, kind = pending, "ordinary"
                else:
                    lock.wait(0.2)
                    continue
                head = families[0][0]
                single = (costs.get(head, 0) >= ISOLATE_SECONDS
                          or head in memory_costs["peak_bytes"])
                # Families are ordered heaviest-first, so the head name bounds the
                # batch's admission; check before taking so nothing must be requeued.
                admission = admission_bytes([head], memory_costs)
                if state["admitted"] + admission > budget.memory_bytes:
                    lock.wait(0.2)
                    continue
                batch = take_batch(families, costs, single)
                admission = admission_bytes(batch, memory_costs)
                state["admitted"] += admission
                if kind == "guarded":
                    state["guarded"] += 1
                return kind, batch, admission
            return None, None, 0

    def settle(kind, admission):
        with lock:
            state["admitted"] -= admission
            if kind == "guarded":
                state["guarded"] -= 1
            lock.notify_all()

    def run_batch(kind, batch, admission):
        with lock:
            sequence[0] += 1
            index = sequence[0]
        unit = f"{base_unit.removeprefix('--unit=')}-batch-{index}"
        memory = shard_memory_bytes(batch, memory_costs)
        batch_command = [f"--unit={unit}" if arg == base_unit else arg for arg in command]
        memory_index = next(i for i, arg in enumerate(batch_command) if arg.startswith("MemoryMax="))
        batch_command[memory_index] = f"MemoryMax={memory}"
        manifest = workspace / f"batch-{index}.json"
        result_path = workspace / f"batch-{index}.result.json"
        manifest.write_text(json.dumps({"binary": command[-1], "names": batch,
                                        "result": str(result_path), "measure_memory": True,
                                        "isolate": len(batch) == 1 and kind != "reserved"}))
        try:
            batch_env = dict(env, RUST_TEST_THREADS="1",
                             WOW_SIM_TIMEOUT_CHILD_SLOTS=str(budget.timeout_slots))
            if grouped_families:
                batch_env["WOW_SIM_GROUPED_FIXTURE_FAMILIES"] = ",".join(grouped_families)
            step, output = run_step(
                f"integration:{index}",
                batch_command[:-1] + [sys.executable, str(WORKER), str(manifest)],
                checkout, io.StringIO(), batch_env)
            batch_records = json.loads(result_path.read_text()).get("records", []) if result_path.exists() else []
            with lock:
                records.update({record["name"]: record for record in batch_records if record.get("name")})
            step["tests"] = len(batch)
            step["kind"] = kind
            step["memory_max_bytes"] = memory
            step["admission_bytes"] = admission
            step["isolated"] = len(batch) == 1
            return step, output
        finally:
            for path in (manifest, result_path):
                path.unlink(missing_ok=True)
            confirm_terminated(unit, slot)

    def lane():
        while True:
            kind, batch, admission = next_job()
            if batch is None:
                return
            try:
                step, output = run_batch(kind, batch, admission)
            finally:
                settle(kind, admission)
            with lock:
                steps.append({"step": step, "output": output})

    def run_tail():
        for job in tail_jobs:
            step, output = run_step(job["name"], job["command"], checkout, io.StringIO(),
                                    dict(env, RUST_TEST_THREADS=str(tail_threads),
                                         WOW_SIM_TIMEOUT_CHILD_SLOTS=str(budget.timeout_slots)))
            tail_results.append((job["name"], step, output))
            log.write(output)
            log.flush()

    def run_reserved():
        if not reserved:
            return
        unit = f"{base_unit.removeprefix('--unit=')}-conformance"
        conformance = [f"--unit={unit}" if arg == base_unit else arg for arg in command]
        step, output = run_step("integration:conformance", conformance + ["--exact", *reserved],
                                checkout, io.StringIO(),
                                dict(env, RUST_TEST_THREADS="1",
                                     WOW_SIM_TIMEOUT_CHILD_SLOTS=str(budget.timeout_slots)))
        step["tests"] = len(reserved)
        step["kind"] = "reserved"
        steps.append({"step": step, "output": output})

    try:
        run_reserved()
        tail_worker = threading.Thread(target=run_tail, name="tail-overlap") if tail_jobs else None
        if tail_worker is not None:
            tail_worker.start()
        try:
            with ThreadPoolExecutor(max_workers=lanes) as executor:
                list(executor.map(lambda _: lane(), range(lanes)))
        finally:
            if tail_worker is not None:
                tail_worker.join()
    finally:
        for path in workspace.glob("*"):
            path.unlink(missing_ok=True)
        workspace.rmdir()
    ordered = sorted(steps, key=lambda item: item["step"]["kind"] != "reserved")
    for item in ordered:
        log.write(f"===== integration batch ({item['step'].get('kind')}): "
                  f"(exit {item['step']['exit']})\n{item['output']}\n")
    log.flush()
    if timing_path is not None:
        timing_path.write_text(json.dumps(records, indent=2) + "\n")
    all_steps = [item["step"] for item in ordered]
    integration = {
        "exit": next((step["exit"] for step in all_steps if step["exit"]), 0),
        "seconds": round(time.monotonic() - started, 1), "tests": len(names),
        "lanes": lanes, "tail_threads": tail_threads, "batches": len(all_steps),
        "batches_detail": all_steps}
    return (integration, {name: (step, output) for name, step, output in tail_results},
            "\n".join(item["output"] for item in ordered))


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


def build_plan(name, artifact, kind, receipt, env, budget, test_slice, sha, profile, checkout, target_dir):
    """Resolve one certified test target into a systemd-run test plan."""
    threads = INTEGRATION_JOBS if name == "integration" else JOBS
    step_env = dict(env, RUST_TEST_THREADS=str(min(int(threads), budget.cpus)))
    if artifact == "wow-sim" and kind == "bin":
        # Three full-UI bin cases peaked at 5.45 GiB; reserve 2 GiB per thread.
        step_env["RUST_TEST_THREADS"] = str(min(3, int(JOBS), budget.cpus,
                                               max(1, budget.mem_gib // 2)))
    binary = test_artifact(receipt, artifact, kind)
    # Linux test executables link libwow_ui_sim_dylib.so from the same deps directory.
    step_env["LD_LIBRARY_PATH"] = str(Path(binary).parent)
    mount = Path.home() / ".cache/wow-ui-sim-srcmount" / target_dir.name
    unit = f"full-suite-test-{sha[:12]}-{profile}-{name}-{time.time_ns()}"
    command = ["systemd-run", "--user", f"--slice={test_slice}", f"--unit={unit}", "--wait", "--pipe", "--collect", "--quiet",
               "-p", f"MemoryMax={min(9, budget.mem_gib)}G", "-p", "MemorySwapMax=0",
               "-p", f"BindPaths={checkout}:{mount}", f"--working-directory={mount}",
               *(f"--setenv={key}" for key in step_env), "--", binary]
    return {"name": name, "command": command, "unit": unit, "env": step_env}


def run(ref, build_jobs, target_dir, profile="retail", optional=False,
        test_cpus=MAX_TEST_CPUS, test_mem_gib=MAX_TEST_MEM_GIB):
    budget = TestBudget(test_cpus, test_mem_gib)
    target_dir = validate_target_dir(target_dir)
    RESULTS.mkdir(parents=True, exist_ok=True)
    with open(LOCK, "w") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        sha = resolve(ref)
        steps = {name: {"status": "not_run"} for name in ("integration", "prefork", "lib")}
        if profile not in ("retail", "mists"):
            steps["prefork"] = {"status": "not_applicable", "reason": "prefork_full_ui supports only Retail/Mists"}
        failures = {name: [] for name, step in steps.items() if step["status"] != "not_applicable"}
        result = {"ref": ref, "sha": sha, "profile": profile, "optional": optional, "started": time.strftime("%Y-%m-%dT%H:%M:%S%z"), "steps": steps, "failures": failures}
        phase = "setup"
        output_path = RESULTS / f"{sha}-{profile}.json"
        attempt = f"{sha}-{profile}-attempt-{time.time_ns()}"
        log_path = RESULTS / f"{attempt}.log"
        result["log"] = str(log_path)
        result["test_budget"] = budget.record()
        test_slice = None
        owner = None
        try:
            with open(log_path, "w") as log:
                try:
                    checkout = prepare_checkout(sha)
                    owner = claim_slot(target_dir, sha, profile)
                    phase = "build"
                    receipt = build(checkout, target_dir, sha, profile, build_jobs, owner, log, optional)
                    phase = "setup"
                    require_capacity(sha, profile)
                    test_slice = f"agents-fullsuite-{time.time_ns()}.slice"
                    result["test_slice"] = test_slice
                    configure_test_slice(test_slice, budget)
                    env = dict(os.environ, CARGO_TARGET_DIR=str(target_dir), RUST_TEST_THREADS=JOBS,
                               WOW_SIM_TIMEOUT_CHILD_SLOTS=str(budget.timeout_slots), CARGO_TERM_COLOR="never")
                    with isolated_cvars(env, log) as env:
                        targets = [("integration", "integration", "test"), ("prefork", "prefork_full_ui", "test"), ("lib", "wow_ui_sim", "lib")]
                        extra_targets = additional_test_targets(receipt)
                        targets.extend(extra_targets)
                        for step, _, _ in extra_targets:
                            result["steps"][step] = {"status": "not_run"}
                            result["failures"][step] = []
                        plans = []
                        if profile == "retail":
                            for name, artifact, kind in targets:
                                if name == "prefork" and profile not in ("retail", "mists"):
                                    result["steps"][name] = {"status": "not_applicable", "reason": "prefork_full_ui supports only Retail/Mists"}
                                    continue
                                plans.append(build_plan(name, artifact, kind, receipt, env, budget,
                                                        test_slice, sha, profile, checkout, target_dir))
                            integration_plan = next(plan for plan in plans if plan["name"] == "integration")
                            tail_plans = [plan for plan in plans if plan is not integration_plan]
                            costs = json.loads((checkout / "tools/integration-costs.json").read_text())
                            grouped = json.loads(
                                (checkout / "tools/grouped-fixture-families.json").read_text())["families"]
                            result["grouped_fixture_families"] = grouped
                            timeout_tests = json.loads((checkout / "tools/integration-timeout-tests.json").read_text())
                            memory_costs = json.loads((checkout / "tools/integration-memory-peaks.json").read_text())
                            timings = RESULTS / f"{attempt}-timings.json"
                            phase = "integration"
                            (result["steps"]["integration"], tail_results,
                             output) = run_dynamic_suite(
                                integration_plan["command"], checkout, log, integration_plan["env"],
                                costs, budget, timeout_tests, memory_costs, tail_plans, timings,
                                grouped)
                            result["timings"] = str(timings)
                            result["failures"]["integration"] = sorted(set(FAILED.findall(output)))
                            if result["steps"]["integration"]["exit"]:
                                result["steps"]["integration"]["status"] = "FAILED"
                            for plan in tail_plans:
                                name = plan["name"]
                                step, tail_output = tail_results.get(name, (
                                    {"status": "FAILED", "exit": 1,
                                     "error": "overlapped tail step did not report"}, ""))
                                try:
                                    result["steps"][name] = step
                                finally:
                                    confirm_terminated(plan["unit"], target_dir.name.removeprefix("slot-"))
                                result["failures"][name] = sorted(set(FAILED.findall(tail_output)))
                                if step["exit"]:
                                    result["steps"][name]["status"] = "FAILED"
                        else:
                            for name, artifact, kind in targets:
                                if name == "prefork" and profile not in ("retail", "mists"):
                                    result["steps"][name] = {"status": "not_applicable", "reason": "prefork_full_ui supports only Retail/Mists"}
                                    continue
                                plan = build_plan(name, artifact, kind, receipt, env, budget,
                                                  test_slice, sha, profile, checkout, target_dir)
                                phase = name
                                try:
                                    result["steps"][name], output = run_step(
                                        name, plan["command"], checkout, log, plan["env"])
                                finally:
                                    confirm_terminated(plan["unit"], target_dir.name.removeprefix("slot-"))
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
        if test_slice is not None and owner is not None:
            try:
                result["test_resources"] = finish_test_slice(test_slice)
            except Exception as error:
                result["status"] = "FAILED"
                result["cleanup_error"] = f"test slice cleanup failed: {error}"
                owner = None  # Retain ownership when cleanup cannot be established.
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
        if result["status"] == "PASSED" and not optional:
            try:
                baseline = RESULTS / f"master-latest-{profile}.json" if ref in ("origin/master", "master") else None
                promote_result(output_path, baseline, result)
            except Exception as error:
                result["status"] = "FAILED"
                result["error"] = f"promotion failed: {error}"
        write_result(attempt_path, result)
        print(json.dumps(result, indent=2))
        return 0 if result["status"] == "PASSED" else 1



def submit(ref, build_jobs, target_dir, profile="retail", optional=False,
           test_cpus=MAX_TEST_CPUS, test_mem_gib=MAX_TEST_MEM_GIB):
    TestBudget(test_cpus, test_mem_gib)
    target_dir = validate_target_dir(target_dir)
    unit = f"full-suite-{profile}-{int(time.time())}"
    subprocess.run(["systemd-run", "--user", f"--unit={unit}", "--collect", "--slice=agents.slice", "-p", "CPUQuota=1200%", "-p", "MemoryMax=9G", "-p", "MemorySwapMax=0", f"--setenv=FULL_SUITE_JOBS={JOBS}", f"--setenv=FULL_SUITE_INTEGRATION_JOBS={INTEGRATION_JOBS}", sys.executable, os.path.abspath(__file__), "run", ref, "--profile", profile, "--build-jobs", str(build_jobs), "--target-dir", str(target_dir), "--test-cpus", str(test_cpus), "--test-mem-gib", str(test_mem_gib), *(["--optional"] if optional else [])], check=True)
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
        command.add_argument("--optional", action="store_true")
        command.add_argument("--test-cpus", type=int, default=MAX_TEST_CPUS)
        command.add_argument("--test-mem-gib", type=int, default=MAX_TEST_MEM_GIB)
    status_parser = actions.add_parser("status")
    status_parser.add_argument("ref", nargs="?")
    args = parser.parse_args(sys.argv[1:] or ["status"])
    if args.action == "status":
        status(args.ref)
    else:
        try:
            if args.action == "run":
                raise SystemExit(run(args.ref, args.build_jobs, args.target_dir, args.profile, args.optional,
                                     args.test_cpus, args.test_mem_gib))
            else:
                submit(args.ref, args.build_jobs, args.target_dir, args.profile, args.optional,
                       args.test_cpus, args.test_mem_gib)
        except ValueError as error:
            parser.error(str(error))
