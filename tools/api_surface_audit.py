#!/usr/bin/env python3
"""Maintain tests/data/api_surface/<profile>-missing.txt and rank its gaps.

bless <profile> <log>: apply the failure output of the api_surface_audit test
  (unlisted, stale and relabel sections) to the profile allowlist.
rank <profile> [limit]: rank allowlisted not-yet names by call sites in the
  profile's cached Blizzard UI Lua (one rg pass with an explicit path).

Contract: docs/specs/api-surface-audit.md.
"""
import collections
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CACHE = Path.home() / ".cache/wow-ui-sim/blizzard-ui"
CSV = ROOT / "data/secure-publication/retail-12-1-5-70688.csv"
SECTIONS = ("documented names missing", "stale entries", "entries to relabel")
# Native world camera and model-scene camera control: the simulator renders
# 2D UI only (AGENTS.md "Intentional Gaps"; permanent commentator_camera.rs).
CAMERA = re.compile(
    r"Camera|^(Pitch(Up|Down)(Start|Stop)|Mouselook(Start|Stop)|IsMouselooking"
    r"|SetMouselookOverrideBinding|(Next|Prev|Reset|Save)View|ZoomIn|ZoomOut)"
)
FUNCTION = re.compile(r'Name = "(\w+)",\n\t\t\tType = "Function",\n((?:\t\t\t\w+ = [^\n]*\n)*)')
# rg has no look-around: keep a leading "."/":" and drop those method calls later.
CALL = r"[.:]?(?:function\s+)?[A-Za-z_]\w*(?:\.[A-Za-z_]\w*)?\s*\("


def allowlist_path(profile):
    return ROOT / f"tests/data/api_surface/{profile}-missing.txt"


def read_allowlist(profile):
    lines = allowlist_path(profile).read_text().splitlines()
    header = [line for line in lines if line.startswith("#")]
    entries = dict(line.split(" # ", 1) for line in lines if line and not line.startswith("#"))
    return header, entries


def parse_sections(log_text):
    """Return {section: [line, ...]} from the test's assertion message."""
    sections, current = {}, None
    for line in log_text.splitlines():
        match = re.match(r"^(?:.*: )?(\d+) (.+?):$", line)
        if match and match.group(2).startswith(SECTIONS):
            current = next(s for s in SECTIONS if match.group(2).startswith(s))
            sections[current] = []
        elif current and line and not line.startswith(("note:", "test result")):
            sections[current].append(line)
        elif current:
            current = None
    return sections


def secure_only_names(profile):
    """Functions in SecureOnly systems or with a SecureOnly function Environment."""
    names = set()
    for path in sorted((CACHE / profile / "AddOns/Blizzard_APIDocumentationGenerated").glob("*.lua")):
        text = path.read_text(encoding="utf-8", errors="replace")
        namespace = re.search(r'^\tNamespace = "(\w+)"', text, re.M)
        prefix = f"{namespace.group(1)}." if namespace else ""
        system_secure = re.search(r'^\tEnvironment = "SecureOnly"', text, re.M) is not None
        for name, fields in FUNCTION.findall(text):
            if system_secure or 'Environment = "SecureOnly"' in fields:
                names.add(prefix + name)
    return names


def glue_only_names():
    rows = [line.split(",") for line in CSV.read_text().splitlines()[1:]]
    return {r[1] if r[0] == "_G" else f"{r[0]}.{r[1]}" for r in rows if r[3] == "Glue"}


def categorize(name, suggestion, secure_only, glue_only):
    if suggestion != "not-yet":
        return suggestion
    if name in secure_only:
        return "intentionally-unsupported: SecureOnly, native secure environment only"
    if name in glue_only:
        return "intentionally-unsupported: Glue-only in retail-12-1-5-70688.csv"
    if CAMERA.search(name.rsplit(".", 1)[-1]):
        return "intentionally-unsupported: 3D camera/model"
    return suggestion


def bless(profile, log_path):
    header, entries = read_allowlist(profile)
    sections = parse_sections(Path(log_path).read_text())
    if not sections:
        raise SystemExit(f"no api_surface_audit failure sections in {log_path}")
    secure_only, glue_only = secure_only_names(profile), glue_only_names()
    for name in sections.get("stale entries", []):
        entries.pop(name, None)
    for line in sections.get("documented names missing", []) + sections.get("entries to relabel", []):
        name, suggestion = line.split(" # ", 1)
        entries[name] = categorize(name, suggestion, secure_only, glue_only)
    body = [f"{name} # {entries[name]}" for name in sorted(entries)]
    allowlist_path(profile).write_text("\n".join(header + body) + "\n")
    print(f"{profile}: {len(entries)} entries")


def call_site_counts(profile):
    addons = CACHE / profile / "AddOns"
    argv = ["rg", "-o", "-N", "--no-filename", "-g", "*.lua",
            "-g", "!Blizzard_APIDocumentation*/**", CALL, str(addons)]
    output = subprocess.run(argv, check=True, capture_output=True, text=True).stdout
    tokens = (re.sub(r"\s*\($", "", token) for token in output.splitlines())
    return collections.Counter(t for t in tokens if not t.startswith(("function", ".", ":")))


def rank(profile, limit):
    _, entries = read_allowlist(profile)
    counts = call_site_counts(profile)
    ranked = sorted(entries, key=lambda name: (-counts[name], name))
    for position, name in enumerate(ranked[:limit], 1):
        print(f"| {position} | `{name}` | {counts[name]} | {entries[name]} |")


def main(argv):
    if len(argv) >= 3 and argv[0] == "bless":
        bless(argv[1], argv[2])
    elif len(argv) >= 2 and argv[0] == "rank":
        rank(argv[1], int(argv[2]) if len(argv) > 2 else 40)
    else:
        raise SystemExit(__doc__)


if __name__ == "__main__":
    main(sys.argv[1:])
