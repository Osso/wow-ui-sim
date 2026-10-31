#!/usr/bin/env python3
"""Generate data/blizzard-ui-files/<profile>.txt from a Gethe wow-ui-source branch.

Each per-profile Blizzard UI manifest is the *full* file list of the matching
Gethe branch's `Interface/AddOns` tree — no filtering. A Gethe branch already
is the per-client dump, so the branch tree maps 1:1 to a profile manifest:

    retail      -> live
    ptr         -> ptr
    mists       -> classic          (current Classic is Mists of Pandaria)
    era         -> classic_era
    anniversary -> classic_anniversary
    wowforever  -> forever
    wrath       -> immutable pinned snapshot in wow-ui-source/wrath

Wrath's pin lives in data/blizzard-ui-builds/wrath.json. Its snapshot has
AddOns and FrameXML roots rather than Interface/AddOns. Wrath enumeration
validates that local snapshot; it never refreshes another profile's branch.

By default the script refreshes branch-backed snapshots from upstream
(shallow). PTR checks out the immutable revision in its committed content index;
other profiles follow their branch. Pass --no-refresh to enumerate the existing cache as-is.

The community listfile is deliberately NOT used as a source: it is a periodic
snapshot and lags new/renamed files after a patch. The Gethe branch tree is the
authoritative file list.
"""

from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
CACHE_ROOT = Path.home() / ".cache/wow-ui-sim/wow-ui-source"
GIT_ROOT = Path.home() / ".cache/wow-ui-sim/wow-ui-source-git"
GETHE_REMOTE = "https://github.com/Gethe/wow-ui-source.git"

# profile manifest name -> Gethe branch
PROFILE_BRANCH = {
    "retail": "live",
    "ptr": "ptr",
    "mists": "classic",
    "era": "classic_era",
    "anniversary": "classic_anniversary",
    "wowforever": "forever",
}
SUPPORTED_PROFILES = tuple(sorted((*PROFILE_BRANCH, "wrath")))


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "profiles",
        nargs="*",
        default=SUPPORTED_PROFILES,
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


def refresh_branch(branch: str, revision: str | None = None) -> None:
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

    if revision is not None:
        run(["git", "-C", str(work), "fetch", "--depth", "1", "origin", revision])
        run(["git", "-C", str(work), "reset", "--hard", "FETCH_HEAD"])

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


def enumerate_wrath_files() -> list[str]:
    snapshot = CACHE_ROOT / "wrath"
    try:
        pin_path = REPO_ROOT / "data/blizzard-ui-builds/wrath.json"
        revision = json.loads(pin_path.read_text())["gethe_revision"]
        provenance = json.loads(
            (snapshot / ".wow-ui-sim-wrath-source.json").read_text()
        )
    except (OSError, ValueError, KeyError, TypeError) as cause:
        raise SystemExit(f"Cannot read pinned Wrath source identity: {cause}") from cause
    if not isinstance(revision, str) or len(revision) != 40 or any(
        character not in "0123456789abcdef" for character in revision
    ):
        raise SystemExit("Invalid committed Wrath Gethe revision")
    if not isinstance(provenance, dict) or provenance.get("gethe_revision") != revision:
        raise SystemExit("Wrath source snapshot does not match the committed Gethe pin")
    addons = snapshot / "AddOns"
    framexml = snapshot / "FrameXML"
    if not addons.is_dir() or not (framexml / "FrameXML.toc").is_file():
        raise SystemExit("Wrath snapshot requires AddOns and FrameXML/FrameXML.toc")
    entries = [str(path.relative_to(addons)) for path in addons.rglob("*") if path.is_file()]
    entries.extend(
        str(path.relative_to(snapshot)) for path in framexml.rglob("*") if path.is_file()
    )
    return sorted(entries)


def write_manifest(profile: str, entries: list[str]) -> Path:
    out = REPO_ROOT / "data/blizzard-ui-files" / f"{profile}.txt"
    out.write_text("\n".join(entries) + "\n", encoding="utf-8")
    return out


def main() -> None:
    args = parse_args()
    for profile in args.profiles:
        if profile == "wrath":
            entries = enumerate_wrath_files()
            out = write_manifest(profile, entries)
            print(f"wrath <- pinned snapshot {len(entries)} files -> {out}")
            continue
        branch = PROFILE_BRANCH.get(profile)
        if branch is None:
            raise SystemExit(
                f"unknown profile '{profile}'; known: {', '.join(SUPPORTED_PROFILES)}"
            )
        if not args.no_refresh:
            revision = None
            if profile in ("retail", "ptr"):
                pin_path = REPO_ROOT / f"data/blizzard-ui-builds/{profile}.json"
                pin = json.loads(pin_path.read_text())
                revision = pin["gethe_revision"]
            refresh_branch(branch, revision)
        entries = enumerate_files(branch)
        out = write_manifest(profile, entries)
        version = (CACHE_ROOT / branch / "version.txt").read_text().strip() \
            if (CACHE_ROOT / branch / "version.txt").is_file() else "unknown"
        print(f"{profile:<12} <- {branch:<20} {len(entries):>5} files (build {version}) -> {out}")


if __name__ == "__main__":
    main()
