#!/usr/bin/env python3
"""Build a patch-api-wikitext-register/v1 from a Patch_X/API_changes raw wikitext.

The MediaWiki plaintext extract drops the collapsed "Consolidated changes"
inventories; this parses them from the raw wikitext instead.

Usage: gen_patch_wikitext_register.py PATCH WIKITEXT REVID OUT
"""

import argparse
import hashlib
import json
import os
import re
from pathlib import Path

SECTIONS = {
    "Global API": "global-api",
    "FrameXML": "framexml",
    "ScriptObjects": "scriptobjects",
    "Widgets": "widgets",
    "Events": "events",
    "CVars": "cvars",
    "Commands": "commands",
}
TEMPLATE = re.compile(r"\{\{(api|apilink|tlygo|apitooltip)\|([^{}]*)\}\}")
LINK = re.compile(r"\[\[[^|\]]*\|([^\]]+)\]\]")
FONT = re.compile(r"</?font[^>]*>")
COUNT = re.compile(r"<small>\((\d+)\)</small>")


def parse_symbol(text):
    """Return (symbol, params) for the inventory reference on one line."""
    match = TEMPLATE.search(text)
    if match:
        parts = match.group(2).split("|")
        named = dict(part.split("=", 1) for part in parts if "=" in part)
        if match.group(1) == "apitooltip":
            return named["name"], named
        return [part for part in parts if "=" not in part][-1], named
    match = LINK.search(text)
    if match:
        return match.group(1), {}
    raise ValueError(f"no symbol reference in: {text!r}")


def make_entry(section, direction, line_no, text):
    symbol, params = parse_symbol(text)
    entry = {
        "id": f"wt-{section}-{symbol}-{line_no}",
        "section": section,
        "direction": direction,
        "symbol": symbol,
        "annotation": "",
        "wikitext_line": line_no,
    }
    # CVar metadata the 12.1.0 register omitted; optional fields stay absent there.
    if section == "widgets" and "[[UIHANDLER " in text:
        entry["kind"] = "widget-script"
    if params.get("type") == "command":
        entry["kind"] = "command"
    if section == "cvars" and "default" in params:
        entry["page_default"] = params["default"]
    if "{{test-inline}}" in text.lower():
        entry["test_inline"] = True
    return entry


def parse_section(section, lines):
    """lines: [(line_no, text)] between this heading and the next."""
    entries, headers, columns = [], [], 0
    mode = None
    for line_no, text in lines:
        if text.startswith("! "):
            headers += [int(n) for n in COUNT.findall(text)]
        elif text.startswith('| <font') and FONT.sub('', text).strip('| ').lower() in ('added', 'removed'):
            mode = FONT.sub('', text).strip('| ').lower()
        elif text.startswith('| valign="top"'):
            columns += 1
            mode = "added" if columns == 1 else "removed"
        elif text.startswith("</div>"):
            mode = None
        elif text.startswith("|}") or "'''Changed'''" in text:
            mode = "changed"
        elif section == "widgets" and text.strip() == ": Widget Scripts":
            continue  # Category label, not an API occurrence.
        elif mode in ("added", "removed") and text.startswith(":"):
            entries.append(make_entry(section, mode, line_no, text))
        # Changed entries may carry a documentation-system label (" PlayerScript {{api|...}}").
        elif mode == "changed" and re.match(r"^\s+(\w+ )?(\{\{|\[\[)", text):
            entries.append(make_entry(section, "changed", line_no, text))
        # Annotations are operator lines or bare colored renames ("   <font ...>A -> B</font>").
        elif mode == "changed" and re.match(r"^\s+([#+-] |<font)", text):
            note = FONT.sub("", text).strip()
            last = entries[-1]
            last["annotation"] = f"{last['annotation']}\n{note}".lstrip("\n")
    counts = []
    for direction, header in zip(("added", "removed"), headers):
        parsed = sum(1 for e in entries if e["direction"] == direction)
        counts.append(
            {"section": section, "direction": direction, "header_count": header, "parsed_count": parsed}
        )
    return entries, counts


def split_sections(text):
    current, buckets = None, {}
    for line_no, line in enumerate(text.split("\n"), start=1):
        heading = re.match(r"^===([^=]+)===\s*$", line)
        if heading:
            current = SECTIONS.get(heading.group(1).strip())
            if current:
                buckets[current] = []
            continue
        if line.strip() == "==Consolidated changes==":
            # Older pages start Global API directly, without a subsection heading.
            current = "global-api"
            buckets.setdefault(current, [])
            continue
        if line.startswith("==") and not line.startswith("==="):
            current = None
        if current:
            buckets[current].append((line_no, line))
    return buckets


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("patch", "path", "revid", "out"):
        parser.add_argument(name)
    parser.add_argument("--inventory-only", action="store_true",
                        help="Omit optional metadata for legacy inventory-only registers")
    args = parser.parse_args()
    patch, path, revid, out = args.patch, args.path, args.revid, args.out
    raw = Path(path).read_bytes()
    buckets = split_sections(raw.decode("utf-8"))
    entries, counts = [], []
    for section in SECTIONS.values():
        entry_section = "cvars" if section == "commands" else section
        section_entries, section_counts = parse_section(entry_section, buckets.get(section, []))
        if section == "commands":
            for count in section_counts:
                count["section"] = "commands"
        entries += section_entries
        counts += section_counts
    if args.inventory_only:
        for entry in entries:
            for key in ("kind", "page_default", "test_inline"):
                entry.pop(key, None)
    source_path = os.path.relpath(Path(path).resolve(), Path(__file__).resolve().parent.parent)
    register = {
        "schema": "patch-api-wikitext-register/v1",
        "patch": patch,
        "source": {"path": source_path, "revid": int(revid), "sha256": hashlib.sha256(raw).hexdigest()},
        "header_counts": counts,
        "entries": entries,
    }
    with open(out, "w") as handle:
        json.dump(register, handle, indent=2, ensure_ascii=False)
        handle.write("\n")


if __name__ == "__main__":
    main()
