#!/usr/bin/env python3
"""Run one integration batch and record per-name wall time and resources.

Isolated mode runs one selected name in its own process, so wait4 CPU and the
kernel memory peak belong to that name. Bulk mode runs the whole batch in one
process and derives each name's wall time from its libtest completion line,
which avoids paying harness startup 11,986 times.
"""
import json
from pathlib import Path
import re
import resource
import subprocess
import sys
import time
from contextlib import nullcontext

# A case may print its own text after "...", so the status can arrive on a
# separate line; both forms are accepted.
TEST_LINE = re.compile(r"^test (?P<name>[^ ]+?)(?: - [^\n]*)? \.\.\.(?P<rest>.*)$")
LEADING_STATUS = re.compile(r"^(ok|ignored|FAILED)\b")
STATUS_WORD = re.compile(r"^(ok|ignored|FAILED)$")
SUMMARY = re.compile(r"^test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored", re.M)


def open_memory_peak():
    relative = next(line.split(":", 2)[2].strip()
                    for line in Path("/proc/self/cgroup").read_text().splitlines()
                    if line.startswith("0:"))
    return (Path("/sys/fs/cgroup") / relative.lstrip("/") / "memory.peak").open("r+")


def reset_peak(peak):
    if peak is not None:
        peak.seek(0)
        peak.write("0")
        peak.flush()


def read_peak(peak):
    if peak is None:
        return None
    peak.seek(0)
    return int(peak.read())


def child_usage():
    usage = resource.getrusage(resource.RUSAGE_CHILDREN)
    return usage.ru_utime + usage.ru_stime, usage.ru_maxrss


def native_command(binary, names):
    return [binary, "--test-threads=1", "--format=pretty", "--color=never", "--exact", *names]


def run_isolated(manifest, peak):
    records = []
    for name in manifest["names"]:
        reset_peak(peak)
        before = child_usage()
        started = time.monotonic()
        process = subprocess.Popen(native_command(manifest["binary"], [name]),
                                   stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                   text=True, bufsize=1)
        text = process.communicate()[0]
        elapsed = time.monotonic() - started
        after = child_usage()
        sys.stdout.write(text)
        sys.stdout.flush()
        status = "failed" if process.returncode else outcome_from_summary(text, name)
        records.append({"name": name, "status": status, "exit": process.returncode,
                        "seconds": elapsed, "cpu_seconds": after[0] - before[0],
                        "max_rss_kib": after[1], "memory_peak_bytes": read_peak(peak),
                        "cpu_source": "isolated"})
        publish(manifest["result"], records, None)
    return records


def outcome_from_summary(text, name):
    if not re.search(r"^test " + re.escape(name) + r"(?: - [^\n]*)? \.\.\.", text, re.M):
        return "invalid"
    summaries = SUMMARY.findall(text)
    if not summaries:
        return "invalid"
    passed, failed, ignored = (int(value) for value in summaries[-1])
    if ignored:
        return "ignored"
    return "passed" if passed == 1 else "invalid"


def run_bulk(manifest, peak):
    """One process for the whole batch; per-name wall from completion lines."""
    reset_peak(peak)
    names = manifest["names"]
    wanted = set(names)
    before = child_usage()
    started = time.monotonic()
    process = subprocess.Popen(native_command(manifest["binary"], names),
                               stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                               text=True, bufsize=1)
    finished = []
    pending = None
    transcript = []
    for line in process.stdout:
        transcript.append(line)
        stripped = line.strip()
        match = TEST_LINE.match(stripped)
        if match and match.group("name") in wanted:
            status = LEADING_STATUS.match(match.group("rest").strip())
            if status:
                finished.append((match.group("name"), time.monotonic() - started,
                                 status.group(1)))
                pending = None
            else:
                pending = match.group("name")
            continue
        if pending is not None and STATUS_WORD.match(stripped):
            finished.append((pending, time.monotonic() - started, stripped))
            pending = None
    process.wait()
    elapsed = time.monotonic() - started
    after = child_usage()
    text = "".join(transcript)
    sys.stdout.write(text)
    sys.stdout.flush()
    cpu_total = after[0] - before[0]
    records = []
    previous = 0.0
    total_wall = 0.0
    for name, at, status in finished:
        duration = max(at - previous, 0.0)
        previous = at
        total_wall += duration
        outcome = {"ok": "passed", "ignored": "ignored"}.get(status.lower(), "failed")
        records.append({"name": name, "status": outcome, "exit": 0,
                        "seconds": duration, "cpu_seconds": None,
                        "max_rss_kib": after[1], "memory_peak_bytes": None,
                        "cpu_source": "batch_share"})
    for record in records:
        if total_wall > 0:
            record["cpu_seconds"] = cpu_total * record["seconds"] / total_wall
        if record["status"] == "failed":
            record["exit"] = process.returncode or 1
    missing = [name for name in names if name not in {entry[0] for entry in finished}]
    for name in missing:
        records.append({"name": name, "status": "invalid", "exit": 2, "seconds": 0.0,
                        "cpu_seconds": None, "max_rss_kib": after[1],
                        "memory_peak_bytes": None, "cpu_source": "batch_share"})
        print(f"Invalid native result: selected case {name} was not reported",
              file=sys.stderr)
    publish(manifest["result"], records, None)
    failed = any(record["status"] in ("failed", "invalid") for record in records)
    if missing:
        return 2
    if process.returncode:
        return process.returncode
    return 1 if failed else 0


def publish(path, records, current):
    result = {"exit": next((record["exit"] for record in records if record.get("exit")), 0),
              "records": records, "current_name": current}
    destination = Path(path)
    temporary = destination.with_suffix(".tmp")
    temporary.write_text(json.dumps(result, indent=2) + "\n")
    temporary.replace(destination)


def run_batch(manifest):
    peak_context = open_memory_peak() if manifest.get("measure_memory", False) else nullcontext()
    with peak_context as peak:
        if manifest.get("isolate", False):
            records = run_isolated(manifest, peak)
            return next((record["exit"] for record in records if record["exit"]), 0)
        return run_bulk(manifest, peak)


if __name__ == "__main__":
    raise SystemExit(run_batch(json.loads(Path(sys.argv[1]).read_text())))
