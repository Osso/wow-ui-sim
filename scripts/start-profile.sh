#!/usr/bin/env python3
"""Build/run live, PTR, Mists, or all on the selected desktop/local host.

Usage: scripts/start-profile.sh [--build-host desktop|local] PROFILE [runtime args]
WOW_SIM_START_RELEASE=1 selects release; WOW_SIM_START_FOREGROUND=1 runs one
profile in foreground; WOW_SIM_START_NO_BUILD=1 uses its existing host binary.
"""

import argparse
import os
import subprocess
import sys
from pathlib import Path

PROFILES = {
    "live": "client-retail",
    "retail": "client-retail",
    "ptr": "client-ptr",
    "mists": "client-mists",
}


def helper_command(root, profile, host, runtime):
    command = [sys.executable, str(root / "scripts/build-host.py")]
    if host:
        command.extend(["--build-host", host])
    command.extend(
        ["--no-default-features", "--features", "sound,gui,casc," + PROFILES[profile]]
    )
    if os.environ.get("WOW_SIM_START_RELEASE") == "1":
        command.append("--release")
    if os.environ.get("WOW_SIM_START_NO_BUILD") == "1":
        command.append("--no-build")
    return command + ["--run", "--", *runtime]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--build-host", choices=("desktop", "local"))
    parser.add_argument("profile", choices=(*PROFILES, "all"))
    parser.add_argument("runtime", nargs=argparse.REMAINDER)
    if len(sys.argv) == 1:
        parser.print_help()
        return 0
    args = parser.parse_args()
    runtime = args.runtime[1:] if args.runtime[:1] == ["--"] else args.runtime
    foreground = os.environ.get("WOW_SIM_START_FOREGROUND") == "1"
    if args.profile == "all" and foreground:
        parser.error("WOW_SIM_START_FOREGROUND=1 only works with a single profile")
    root = Path(__file__).resolve().parents[1]
    os.chdir(root)
    profiles = ("live", "ptr", "mists") if args.profile == "all" else (args.profile,)
    for profile in profiles:
        command = helper_command(root, profile, args.build_host, runtime)
        if foreground:
            os.execv(sys.executable, command)
        label = "live" if profile == "retail" else profile
        output = root / "target/profile-runs"
        output.mkdir(parents=True, exist_ok=True)
        logfile = output / (label + ".log")
        with logfile.open("w") as log:
            child = subprocess.Popen(
                command,
                cwd=root,
                stdout=log,
                stderr=log,
                stdin=subprocess.DEVNULL,
                start_new_session=True,
            )
        (output / (label + ".pid")).write_text(str(child.pid) + "\n")
        print(f"{label:<5} pid={child.pid} log={logfile}", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
