#!/usr/bin/env python3
"""Run the full wow-ui-sim test suite for one commit, in the background, and record results.

Usage:
  full_suite.py run [REF]      Test REF (default origin/master) now; blocks until done.
  full_suite.py submit [REF]   Same, detached via systemd-run --user; prints the unit name.
  full_suite.py status [REF]   Print the stored result for REF's commit (or the latest run).

Runs serialize on a lock. Each run uses a dedicated checkout and target dir so agents'
working copies are never touched. Results: RESULTS/<sha>.json plus <sha>.log; master runs
also update RESULTS/master-latest.json, which later runs compare against to list NEW failures.
"""

import fcntl
import json
import os
import re
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path.home() / "Projects/wow"
REPO = ROOT / "wow-ui-sim"
CHECKOUT = ROOT / "full-suite-checkout"
RESULTS = ROOT / "full-suite-results"
LOCK = RESULTS / ".lock"
JOBS = os.environ.get("FULL_SUITE_JOBS", "16")
FAILED = re.compile(
    r"^\s*(?:FAIL|SIGSEGV|SIGABRT|TIMEOUT) \[.*?\] (?:\(\s*\d+/\d+\) )?\S+ (\S+)", re.M
)
PREFORK_FAILED = re.compile(r"^test (\S+) \.\.\. FAILED", re.M)


def git(*args, cwd=REPO):
    return subprocess.run(
        ["git", *args], cwd=cwd, check=True, capture_output=True, text=True
    ).stdout.strip()


def resolve(ref):
    git("fetch", "-q", "origin")
    return git("rev-parse", f"{ref}^{{commit}}")


def prepare_checkout(sha):
    if not (CHECKOUT / ".git").exists():
        subprocess.run(
            ["git", "clone", "-q", "--shared", str(REPO), str(CHECKOUT)], check=True
        )
    git("fetch", "-q", str(REPO), sha, cwd=CHECKOUT)
    git("checkout", "-q", "--force", sha, cwd=CHECKOUT)
    git("clean", "-q", "-fdx", "-e", "target", cwd=CHECKOUT)


def run_step(name, cmd, log, env):
    cmd = [*cmd, "--offline", "--locked"]
    start = time.monotonic()
    proc = subprocess.run(cmd, cwd=CHECKOUT, env=env, capture_output=True, text=True)
    output = proc.stdout + proc.stderr
    log.write(f"===== {name}: {' '.join(cmd)} (exit {proc.returncode})\n{output}\n")
    return {
        "exit": proc.returncode,
        "seconds": round(time.monotonic() - start, 1),
    }, output


def summarize(output, pattern):
    return sorted(set(pattern.findall(output)))


def run(ref):
    RESULTS.mkdir(parents=True, exist_ok=True)
    with open(LOCK, "w") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        sha = resolve(ref)
        prepare_checkout(sha)
        env = dict(os.environ, CARGO_BUILD_JOBS="12", CARGO_TERM_COLOR="never")
        result = {
            "ref": ref,
            "sha": sha,
            "started": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
            "steps": {},
            "failures": {},
        }
        with open(RESULTS / f"{sha}.log", "w") as log:
            steps = [
                (
                    "integration",
                    [
                        "cargo",
                        "nextest",
                        "run",
                        "--test",
                        "integration",
                        "--no-fail-fast",
                        "--test-threads",
                        JOBS,
                    ],
                    FAILED,
                ),
                (
                    "prefork",
                    ["cargo", "test", "--test", "prefork_full_ui"],
                    PREFORK_FAILED,
                ),
                (
                    "lib",
                    [
                        "cargo",
                        "nextest",
                        "run",
                        "--lib",
                        "--no-fail-fast",
                        "--test-threads",
                        JOBS,
                    ],
                    FAILED,
                ),
            ]
            for name, cmd, pattern in steps:
                result["steps"][name], output = run_step(name, cmd, log, env)
                result["failures"][name] = summarize(output, pattern)
        result["finished"] = time.strftime("%Y-%m-%dT%H:%M:%S%z")
        result["new_failures"] = new_failures(result)
        (RESULTS / f"{sha}.json").write_text(json.dumps(result, indent=2) + "\n")
        if ref in ("origin/master", "master"):
            (RESULTS / "master-latest.json").write_text(
                json.dumps(result, indent=2) + "\n"
            )
        print(
            json.dumps(
                {k: result[k] for k in ("sha", "steps", "new_failures")}, indent=2
            )
        )


def new_failures(result):
    base_path = RESULTS / "master-latest.json"
    if not base_path.exists():
        return None
    base = json.loads(base_path.read_text())
    return {
        name: sorted(set(fails) - set(base["failures"].get(name, [])))
        for name, fails in result["failures"].items()
    }


def submit(ref):
    unit = f"full-suite-{int(time.time())}"
    subprocess.run(
        [
            "systemd-run",
            "--user",
            f"--unit={unit}",
            "--collect",
            "--slice=agents.slice",
            "-p",
            "CPUQuota=1200%",
            "-p",
            "MemoryHigh=16G",
            "-p",
            "MemoryMax=16G",
            sys.executable,
            os.path.abspath(__file__),
            "run",
            ref,
        ],
        check=True,
    )
    print(unit)


def status(ref):
    if ref:
        path = RESULTS / f"{resolve(ref)}.json"
    else:
        runs = sorted(RESULTS.glob("*.json"), key=lambda p: p.stat().st_mtime)
        runs = [p for p in runs if p.name != "master-latest.json"]
        path = runs[-1] if runs else None
    if not path or not path.exists():
        print("no result yet")
        return
    print(path.read_text())


if __name__ == "__main__":
    action = sys.argv[1] if len(sys.argv) > 1 else "status"
    target = sys.argv[2] if len(sys.argv) > 2 else None
    if action == "run":
        run(target or "origin/master")
    elif action == "submit":
        submit(target or "origin/master")
    elif action == "status":
        status(target)
    else:
        sys.exit(__doc__)
