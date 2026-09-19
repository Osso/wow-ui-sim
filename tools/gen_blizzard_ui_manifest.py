#!/usr/bin/env python3
"""Generate data/blizzard-ui-files/<profile>.txt from a Gethe wow-ui-source branch.

Each per-profile Blizzard UI manifest is the *full* file list of the matching
Gethe branch's `Interface/AddOns` tree — no filtering. A Gethe branch already
is the per-client dump, so the branch tree maps 1:1 to a profile manifest:

    retail      -> live
    ptr         -> ptr2
    mists       -> classic          (current Classic is Mists of Pandaria)
    era         -> classic_era
    anniversary -> classic_anniversary
    wowforever  -> forever

By default the script refreshes the local wow-ui-source cache from upstream
(shallow) before enumerating, so the manifest matches the current build of the
branch. Pass --no-refresh to enumerate the existing cache as-is.

The community listfile is deliberately NOT used as a source: it is a periodic
snapshot and lags new/renamed files after a patch. The Gethe branch tree is the
authoritative file list.
"""

from __future__ import annotations

import argparse
import subprocess
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
CACHE_ROOT = Path.home() / ".cache/wow-ui-sim/wow-ui-source"
GIT_ROOT = Path.home() / ".cache/wow-ui-sim/wow-ui-source-git"
GETHE_REMOTE = "https://github.com/Gethe/wow-ui-source.git"

# profile manifest name -> Gethe branch
PROFILE_BRANCH = {
    "retail": "live",
    "ptr": "ptr2",
    "mists": "classic",
    "era": "classic_era",
    "anniversary": "classic_anniversary",
    "wowforever": "forever",
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "profiles",
        nargs="*",
        default=sorted(PROFILE_BRANCH),
        help="profiles to regenerate (default: all)",
    )
    parser.add_argument(
        "--no-refresh",
        action="store_true",
        help="skip the upstream fetch; enumerate the existing cache",
    )
    return parser.parse_args()


def run(cmd: list[str], cwd: Path | None = None) -> None:
    subprocess.run(cmd, cwd=cwd, check=True)


def refresh_branch(branch: str) -> None:
    """Shallow-checkout `branch` and sync its Interface tree into the cache."""
    work = GIT_ROOT / branch
    if (work / ".git").is_dir():
        run(["git", "-C", str(work), "fetch", "--depth", "1", "origin", branch])
        run(["git", "-C", str(work), "reset", "--hard", "FETCH_HEAD"])
    else:
        work.parent.mkdir(parents=True, exist_ok=True)
        run(
            [
                "git", "clone", "--depth", "1", "--single-branch",
                "--branch", branch, GETHE_REMOTE, str(work),
            ]
        )

    dest = CACHE_ROOT / branch
    dest.mkdir(parents=True, exist_ok=True)
    sources = [work / "Interface"]
    sources += [work / name for name in ("version.txt", "README.md") if (work / name).is_file()]
    run(["rsync", "-a", "--delete", "--exclude", ".git", *map(str, sources), f"{dest}/"])


def enumerate_files(branch: str) -> list[str]:
    addons = CACHE_ROOT / branch / "Interface/AddOns"
    if not addons.is_dir():
        raise SystemExit(f"AddOns tree not found: {addons} (run without --no-refresh?)")
    entries = sorted(
        str(path.relative_to(addons)).replace("\\", "/")
        for path in addons.rglob("*")
        if path.is_file()
    )
    return entries


def write_manifest(profile: str, entries: list[str]) -> Path:
    out = REPO_ROOT / "data/blizzard-ui-files" / f"{profile}.txt"
    out.write_text("\n".join(entries) + "\n", encoding="utf-8")
    return out


def main() -> None:
    args = parse_args()
    for profile in args.profiles:
        branch = PROFILE_BRANCH.get(profile)
        if branch is None:
            raise SystemExit(
                f"unknown profile '{profile}'; known: {', '.join(sorted(PROFILE_BRANCH))}"
            )
        if not args.no_refresh:
            refresh_branch(branch)
        entries = enumerate_files(branch)
        out = write_manifest(profile, entries)
        version = (CACHE_ROOT / branch / "version.txt").read_text().strip() \
            if (CACHE_ROOT / branch / "version.txt").is_file() else "unknown"
        print(f"{profile:<12} <- {branch:<20} {len(entries):>5} files (build {version}) -> {out}")


if __name__ == "__main__":
    main()
