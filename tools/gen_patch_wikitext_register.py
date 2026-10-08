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
    if section == "cvars" and re.fullmatch(r":\s+[A-Za-z_][A-Za-z0-9_]*", text):
        symbol, params = text[1:].strip(), {}
    else:
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
    if section == "widgets" and re.search(r"\[\[UIHANDLER[ _]", text):
        entry["kind"] = "widget-script"
    if params.get("type") == "command":
        entry["kind"] = "command"
    if section == "cvars" and "default" in params:
        entry["page_default"] = params["default"]
    elif section == "cvars":
        default = re.search(r'Default: <code><span class="apitype">([^<]*)</span></code>', text)
        if default:
            entry["page_default"] = default.group(1)
    if "{{test-inline}}" in text.lower():
        entry["test_inline"] = True
    return entry


def parse_section(section, lines, *, expand_shared_changes=False, capture_span_defaults=False,
                  skip_plain_scripts_label=False, legacy_column_headers=False,
                  legacy_inventory_labels=False):
    """lines: [(line_no, text)] between this heading and the next."""
    entries, headers, columns = [], [], 0
    mode = None
    inline_commands = False
    command_columns = set()
    for line_no, text in lines:
        if text.startswith("! "):
            legacy_header = re.search(r'\|\s*(\d+)\s+(?:new|removed(?:/renamed)?)\s+(\w+)', text)
            if legacy_column_headers and legacy_header:
                headers.append(int(legacy_header[1]))
                if legacy_header[2] in ('command', 'commands'):
                    command_columns.add(len(headers))
            else:
                headers += [int(n) for n in COUNT.findall(text)]
        elif text.startswith('| <font') and FONT.sub('', text).strip('| ').lower() in ('added', 'removed'):
            mode = FONT.sub('', text).strip('| ').lower()
        elif text.startswith('| valign="top"'):
            columns += 1
            inline_commands = columns in command_columns
            mode = "added" if columns == 1 else "removed"
        elif text.startswith("</div>"):
            mode = None
        elif text.startswith("|}") or "'''Changed'''" in text:
            mode = "changed"
        elif legacy_inventory_labels and section == 'cvars' and text.strip() == ': CVar':
            inline_commands = False
        elif legacy_inventory_labels and section == 'cvars' and text.strip() == ': Command':
            inline_commands = True
        elif section == "cvars" and text.strip() in (": '''Commands'''", ": Commands"):
            inline_commands = True
        elif section == "widgets" and (text.strip() == ": Widget Scripts" or
                                       (skip_plain_scripts_label and text.strip() == ": Scripts")):
            continue  # Category label, not an API occurrence.
        elif mode in ("added", "removed") and text.startswith(":"):
            entry = make_entry(section, mode, line_no, text)
            if section == 'cvars' and capture_span_defaults:
                default = re.search(r'Default:\s*<code>(.*?)</code>', text)
                if default:
                    entry['page_default'] = re.sub(r'<[^>]+>', '', default[1]).strip()
            if inline_commands:
                entry["kind"] = "command"
            entries.append(entry)
        # Changed entries may carry a documentation-system label (" PlayerScript {{api|...}}").
        elif mode == "changed" and re.match(r"^\s+(\w+ )?(\{\{|\[\[)", text):
            references = [match.group(0) for match in TEMPLATE.finditer(text)]
            source_refs = references if expand_shared_changes and references else [text]
            entries.extend(make_entry(section, "changed", line_no, ref) for ref in source_refs)
        # Annotations are operator lines or bare colored renames ("   <font ...>A -> B</font>").
        elif mode == "changed" and re.match(r"^\s+([#+-] |<font)", text):
            note = FONT.sub("", text).strip()
            last = entries[-1]
            group = [last]
            if expand_shared_changes:
                group = [entry for entry in entries
                         if entry["wikitext_line"] == last["wikitext_line"]]
            for entry in group:
                entry["annotation"] = f"{entry['annotation']}\n{note}".lstrip("\n")
    counts = []
    for direction, header in zip(("added", "removed"), headers):
        parsed = sum(1 for e in entries if e["direction"] == direction)
        counts.append(
            {"section": section, "direction": direction, "header_count": header, "parsed_count": parsed}
        )
    if legacy_inventory_labels:
        counts = legacy_inventory_counts(section, lines, entries)
    return entries, counts


def legacy_inventory_counts(section, lines, entries):
    """Retain each prose numerical header, including mixed CVar/command counts."""
    counts = []
    for _, text in lines:
        if not text.startswith('! '):
            continue
        for count, change, kind in re.findall(
                r'(\d+)\s+(new|removed(?:/renamed)?)\s+(\w+)', text):
            direction = 'added' if change == 'new' else 'removed'
            command = kind in ('command', 'commands')
            parsed = sum(entry['direction'] == direction and
                         (section != 'cvars' or (entry.get('kind') == 'command') == command)
                         for entry in entries)
            counts.append({'section': 'commands' if command else section,
                           'direction': direction, 'header_count': int(count),
                           'parsed_count': parsed})
    return counts


def split_sections(text, *, separate_inline_structures=False):
    current, buckets = None, {}
    for line_no, line in enumerate(text.split("\n"), start=1):
        if separate_inline_structures and line.strip() == 'Structures':
            current = None
            continue
        if line.strip() == "==Type Changes==":
            break  # Historical type annotations are extract rows, not publication inventories.
        heading = re.match(r"^(={2,3})([^=]+)\1\s*$", line)
        if heading and heading.group(2).strip() != "Consolidated changes":
            current = SECTIONS.get(heading.group(2).strip())
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


def parse_simple_api_list(text):
    """Retain the bullet additions under API/New (8.3.7's inventory format)."""
    entries = []
    in_api, added = False, False
    for line_no, line in enumerate(text.splitlines(), 1):
        heading = re.fullmatch(r'(={2,3})\s*(.*?)\s*\1', line)
        if heading:
            if len(heading[1]) == 2:
                in_api, added = heading[2] == 'API', False
            else:
                added = in_api and heading[2] == 'New'
        elif added and line.startswith('* '):
            section = 'cvars' if line.startswith('* CVar ') else 'global-api'
            entries.append(make_entry(section, 'added', line_no, line))
    return entries


def parse_diff_api_additions(text):
    """Retain each explicit late-build addition outside consolidated inventories."""
    entries = []
    in_diffs = False
    for line_no, line in enumerate(text.splitlines(), 1):
        heading = re.fullmatch(r'==\s*([^=]+?)\s*==', line)
        if heading:
            in_diffs = heading[1] == 'Diffs'
        elif in_diffs and line.startswith(('* New functions:', '* New event:')):
            section = 'events' if line.startswith('* New event:') else 'global-api'
            entries.extend(make_entry(section, 'added', line_no, match[0])
                           for match in TEMPLATE.finditer(line))
    return entries


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("patch", "path", "revid", "out"):
        parser.add_argument(name)
    parser.add_argument("--inventory-only", action="store_true",
                        help="Omit optional metadata for legacy inventory-only registers")
    parser.add_argument("--expand-shared-changes", action="store_true",
                        help="Account for every API on shared changed lines; opt-in preserves prior registers")
    parser.add_argument('--separate-inline-structures', action='store_true',
                        help='Exclude unheaded Structures from API annotations; opt-in preserves prior registers')
    parser.add_argument('--capture-span-defaults', action='store_true',
                        help='Retain hidden-span CVar defaults; opt-in preserves prior registers')
    parser.add_argument('--skip-plain-scripts-label', action='store_true',
                        help='Skip the plain widget Scripts label; opt-in preserves prior registers')
    parser.add_argument('--simple-api-list', action='store_true',
                        help='Retain API/New bullet additions; opt-in preserves prior registers')
    parser.add_argument('--legacy-column-headers', action='store_true',
                        help='Retain prose numerical headers and command-column kinds; opt-in preserves prior registers')
    parser.add_argument('--diff-api-additions', action='store_true',
                        help='Retain explicit late-build additions in Diffs; opt-in preserves prior registers')
    parser.add_argument('--legacy-inventory-labels', action='store_true',
                        help='Retain prose counts and singular CVar/Command labels; opt-in preserves prior registers')
    args = parser.parse_args()
    patch, path, revid, out = args.patch, args.path, args.revid, args.out
    raw = Path(path).read_bytes()
    buckets = split_sections(raw.decode("utf-8"),
                             separate_inline_structures=args.separate_inline_structures)
    entries, counts = [], []
    for section in SECTIONS.values():
        entry_section = "cvars" if section == "commands" else section
        section_entries, section_counts = parse_section(
            entry_section, buckets.get(section, []),
            expand_shared_changes=args.expand_shared_changes,
            capture_span_defaults=args.capture_span_defaults,
            skip_plain_scripts_label=args.skip_plain_scripts_label,
            legacy_column_headers=args.legacy_column_headers,
            legacy_inventory_labels=args.legacy_inventory_labels)
        if section == "commands":
            for count in section_counts:
                count["section"] = "commands"
        entries += section_entries
        counts += section_counts
    if args.simple_api_list:
        entries.extend(parse_simple_api_list(raw.decode('utf-8')))
    if args.diff_api_additions:
        entries.extend(parse_diff_api_additions(raw.decode('utf-8')))
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
