#!/usr/bin/env python3
"""Run the complete Retail suite before publishing an annotated release tag."""

import argparse
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parent.parent


def git(*args):
    return subprocess.run(["git", *args], cwd=ROOT, check=True, text=True,
                          stdout=subprocess.PIPE).stdout.strip()


def checked_master():
    git("fetch", "origin", "master")
    if git("branch", "--show-current") != "master":
        raise ValueError("release requires master")
    if git("status", "--porcelain", "--untracked-files=all"):
        raise ValueError("release requires a clean tree")
    head = git("rev-parse", "HEAD")
    if head != git("rev-parse", "origin/master"):
        raise ValueError("master must match origin/master")
    return head


def release(version, target_dir, dry_run=False):
    if not re.fullmatch(r"(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)", version):
        raise ValueError("version must be X.Y.Z")
    head = checked_master()
    command = [sys.executable, str(ROOT / "tools/full_suite.py"), "run", head,
               "--profile", "retail", "--target-dir", target_dir, "--optional"]
    result = subprocess.run(command, cwd=ROOT, text=True, capture_output=True)
    print(result.stdout, end="")
    print(result.stderr, end="", file=sys.stderr)
    if result.returncode:
        raise ValueError("full suite failed; release aborted")
    if checked_master() != head:
        raise ValueError("master changed during the suite; release aborted")
    if dry_run:
        print(f"dry run: would create and push v{version}")
        return
    tag = f"v{version}"
    git("tag", "-a", tag, "-m", tag, head)
    git("push", "origin", f"refs/tags/{tag}:refs/tags/{tag}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("version")
    parser.add_argument("--target-dir", required=True)
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()
    try:
        release(args.version, args.target_dir, args.dry_run)
    except (ValueError, subprocess.CalledProcessError) as error:
        print(f"release failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
