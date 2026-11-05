#!/usr/bin/env python3
"""Merge measured per-test durations into the saved integration costs."""
import argparse
import json
import os
from pathlib import Path
import sys
import tempfile


def merge(timings, costs):
    refreshed = dict(costs)
    for name, record in timings.items():
        seconds = record.get("seconds") if isinstance(record, dict) else None
        if not isinstance(seconds, (int, float)) or not seconds > 0:
            raise ValueError(f"invalid measured duration for {name}: {seconds!r}")
        refreshed[name] = round(float(seconds), 6)
    return dict(sorted(refreshed.items()))


def write_atomic(path, data):
    with tempfile.NamedTemporaryFile("w", dir=path.parent, delete=False) as stream:
        temporary = Path(stream.name)
        try:
            json.dump(data, stream, indent=2)
            stream.write("\n")
            stream.flush()
            os.fsync(stream.fileno())
        except BaseException:
            temporary.unlink(missing_ok=True)
            raise
    temporary.replace(path)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("timings", type=Path, help="per-test timing JSON from a finished run")
    parser.add_argument("costs", type=Path, help="integration-costs.json to update in place")
    args = parser.parse_args(argv)
    timings = json.loads(args.timings.read_text())
    costs = json.loads(args.costs.read_text()) if args.costs.exists() else {}
    refreshed = merge(timings, costs)
    write_atomic(args.costs, refreshed)
    observed = sum(1 for name in timings if name in refreshed)
    print(f"updated {observed} measured cases; {len(refreshed)} total saved costs")
    return 0


if __name__ == "__main__":
    sys.exit(main())
