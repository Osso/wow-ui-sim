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


def with_client_line(register, client_line):
    """Keep legacy bytes by default; explicitly label separate client histories."""
    return register if client_line is None else dict(register, client_line=client_line)


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


def parse_legacy_caption_tables(text):
    """Parse older unheaded and repeated inventories independently by table caption."""
    sections = {'Global API': 'global-api', 'Widget API': 'widgets',
                'Widget Handlers': 'widgets', 'Events': 'events',
                'Console variables': 'cvars', 'Console commands': 'commands'}
    entries, counts, table = [], [], []
    section = None
    for number, line in enumerate(text.splitlines(), 1):
        if line.startswith('{|'):
            table, section = [], None
        elif line.startswith('|+'):
            section = next((value for caption, value in sections.items()
                            if line.startswith('|+ ' + caption + ' ')), None)
        elif line == '|}' and section:
            rows, headers = parse_section('cvars' if section == 'commands' else section,
                                          table, legacy_column_headers=True)
            if section == 'commands':
                for row in rows:
                    row['kind'] = 'command'
                for header in headers:
                    header['section'] = 'commands'
            entries.extend(rows)
            counts.extend(headers)
            section = None
        else:
            table.append((number, line))
    return entries, counts


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


def parse_historical_api_headings(text):
    """Retain 2010 retail heading inventories, including signature-bearing bullets."""
    headings = {'New API functions': ('global-api', 'added'),
                'New events': ('events', 'added'),
                'API function changes': ('global-api', 'changed'),
                'Removed API': ('global-api', 'removed')}
    entries, category, direction = [], None, None
    for number, line in enumerate(text.splitlines(), 1):
        heading = re.fullmatch(r'==\s*([^=]+?)\s*==', line)
        if heading:
            category, direction = headings.get(heading[1], (None, None))
        elif category and line.startswith('* '):
            for reference in TEMPLATE.finditer(line):
                symbol, params = parse_symbol(reference[0])
                section = 'widgets' if params.get('t') == 'w' else category
                entry = make_entry(section, direction, number, reference[0])
                entries.append(dict(entry, annotation=line))
    return entries


def parse_cataclysm_change_bullets(text):
    """Retain 4.1-style NEW/REMOVED groups and explicit changed event identities."""
    entries = []
    section, direction = None, None
    for number, line in enumerate(text.splitlines(), 1):
        heading = re.fullmatch(r'==\s*([^=]+?)\s*==', line)
        if heading:
            section, direction = heading[1], None
            continue
        if section not in ('Breaking changes', 'API changes', 'Event changes'):
            continue
        if not line.startswith('*'):
            continue
        if line.startswith('* '):
            marker = re.match(r'\* (NEW|REMOVED)\b', line)
            direction = {'NEW': 'added', 'REMOVED': 'removed'}.get(
                marker[1] if marker else '', 'changed')
        if section == 'Event changes' or 'COMBAT_LOG_EVENT' in line:
            category = 'events'
            names = re.findall(r'\b[A-Z][A-Z_]+\b', line)
            symbols = list(dict.fromkeys(name for name in names if name not in ('NEW', 'REMOVED', 'API')))
        else:
            category = 'global-api'
            symbols = [match[2].split('|')[0].strip() for match in TEMPLATE.finditer(line)]
        for symbol in symbols:
            entry = make_entry(category, direction, number, '{{api|' + symbol + '}}')
            entries.append(dict(entry, annotation=line))
    return entries


def launch_entry(number, line, symbol, direction, *, kind=None):
    section = ('widgets' if ':' in symbol or kind == 'widget-script' else
               'events' if kind == 'event' else 'cvars' if kind == 'cvar' else
               'commands' if kind == 'click-modifier' else
               'framexml' if symbol.startswith('InterfaceOptionsFrame_') else 'global-api')
    entry = dict(make_entry(section, direction, number, '{{api|' + symbol + '}}'),
                 annotation=line)
    if kind:
        entry['kind'] = kind
    return entry


def parse_wrath_launch_inventory(text):
    """Retain 3.0.2 labeled rows and literal summary references, not linked domains."""
    entries = []
    for number, line in enumerate(text.splitlines(), 1):
        normalized = re.sub(r'\{\{api\|([^{}|]+)\}\}', r'\1', line)
        marker = re.match(r'\s*\*?\s*(NEW|UPDATED|MODIFIED|REMOVED)( HANDLER)?\s*-?\s*(.*)',
                          normalized)
        if marker:
            direction = {'NEW': 'added', 'REMOVED': 'removed'}.get(marker[1], 'changed')
            body = marker[3].split(' -- ', 1)[0]
            if '=' in body:
                body = body.split('=', 1)[1].strip()
            symbol = re.match(r'([A-Za-z_]\w*(?::[A-Za-z_]\w*)?(?:\.[A-Za-z_]\w*)?)', body)
            if not symbol:
                raise ValueError(f'launch inventory lacks literal identity at {number}: {line}')
            entries.append(launch_entry(number, line, symbol[1], direction,
                                        kind='widget-script' if marker[2] else None))
            continue
        direction = ('removed' if 'functions have been replaced' in line else 'changed')
        for call in re.finditer(r'\b([A-Za-z_]\w*(?::[A-Za-z_]\w*)?)\(', normalized):
            entries.append(launch_entry(number, line, call[1], direction))
        if 'new event' in line:
            for symbol in re.findall(r'\b[A-Z]+(?:_[A-Z]+)+\b', line):
                entries.append(launch_entry(number, line, symbol, 'added', kind='event'))
        for symbol in re.findall(r'"([A-Za-z_]\w*)" cvar', line):
            entries.append(launch_entry(number, line, symbol, 'changed', kind='cvar'))
        if 'new FOCUSCAST click modifier' in line:
            entries.append(launch_entry(number, line, 'FOCUSCAST', 'added', kind='click-modifier'))
        rename = re.search(r'(InterfaceOptionsFrame_\w+) has been renamed (InterfaceOptionsFrame_\w+)', line)
        if rename:
            entries.extend(launch_entry(number, line, symbol, action) for symbol, action in
                           zip(rename.groups(), ('removed', 'added')))
        # Bare references without call syntax remain literal changed occurrences.
        for symbol in ('GetInventoryItemsForSlot', 'UnitExists'):
            if symbol in line and not any(r['symbol'] == symbol and r['wikitext_line'] == number
                                          for r in entries):
                entries.append(launch_entry(number, line, symbol, 'changed'))
    return entries


def parse_wrath_retail_summary(text):
    """Retain literal bare-call, method and event identities in 2009 summaries."""
    entries = []
    for number, line in enumerate(text.splitlines(), 1):
        if not line.startswith('* '):
            continue
        symbols = []
        if line.startswith('* NEW - '):
            call = re.search(r'\b((?:region|Button|Texture):)?([A-Za-z_]\w*)\(', line)
            if not call:
                raise ValueError(f'NEW bullet lacks an API call: {line}')
            owner, name = call.groups()
            symbol = ('Region:' if owner == 'region:' else owner or '') + name
            symbols.append(('widgets' if owner else 'global-api', symbol, 'added'))
            symbols.extend(('events', event, 'added') for event in
                           re.findall(r'\b([A-Z][A-Z_]+) event\b', line))
            replacement = re.search(r'replaces the ([A-Za-z_]\w*) FrameXML', line)
            if replacement:
                symbols.append(('framexml', replacement[1], 'changed'))
        elif 'deprecated GetFrameType method has been completely removed' in line:
            symbols = [('widgets', 'Frame:GetFrameType', 'removed'),
                       ('widgets', 'Frame:GetObjectType', 'changed')]
        for section, symbol, direction in symbols:
            entry = make_entry(section, direction, number, '{{api|' + symbol + '}}')
            entries.append(dict(entry, annotation=line))
    return entries


def parse_legacy_api_bullets(text):
    """Retain New/Removals API bullets and unqualified event additions (8.1.5)."""
    entries = []
    section, direction = None, None
    for number, line in enumerate(text.splitlines(), 1):
        heading = re.fullmatch(r'(=+)\s*(.*?)\s*\1', line)
        if heading:
            title = heading[2]
            if len(heading[1]) == 2:
                section = {'API': 'global-api', 'Events': 'events'}.get(title)
                direction = 'added' if section == 'events' else None
            elif section == 'global-api':
                direction = {'New': 'added', 'Removals': 'removed'}.get(title)
        elif section and direction and line.startswith('* ') and TEMPLATE.search(line):
            entries.append(make_entry(section, direction, number, line))
    return entries


def parse_legacy_api_renames(text):
    """Retain old and successor identities from API/Renamed bullet pairs."""
    entries = []
    in_api, renamed = False, False
    for number, line in enumerate(text.splitlines(), 1):
        heading = re.fullmatch(r'(=+)\s*(.*?)\s*\1', line)
        if heading:
            if len(heading[1]) == 2:
                in_api, renamed = heading[2] == 'API', False
            else:
                renamed = in_api and heading[2] == 'Renamed'
        elif renamed and line.startswith('* '):
            references = list(TEMPLATE.finditer(line))
            if len(references) != 2:
                raise ValueError(f'expected old/successor rename pair: {line}')
            entries.extend(make_entry('global-api', direction, number, reference[0])
                           for reference, direction in zip(references, ('removed', 'added')))
    return entries


def bfa_removal_entries(number, line):
    """Do not mistake replacement API links after 'Use' for removals."""
    removed = re.split(r'\b(?:Use|use)\b', line, maxsplit=1)[0]
    entries = [make_entry('global-api', 'removed', number, match[0])
               for match in TEMPLATE.finditer(removed)]
    entries.extend(make_entry('events', 'removed', number, '{{api|' + name + '}}')
                   for name in re.findall(r'\[\[(GLYPH_[A-Z]+)\]\]', removed))
    if not entries:
        names = re.findall(r'\b(?:FindSpellOverrideNameByName|FindBaseSpellNameByName|SearchGuildRecipes)\b', removed)
        entries.extend(make_entry('global-api', 'removed', number, '{{api|' + name + '}}')
                       for name in names)
    return entries


def parse_bfa_prepatch(text):
    """8.0.1: top-level nested namespaces, prose removals, four-level events."""
    entries = []
    section, direction = None, None
    for number, line in enumerate(text.splitlines(), 1):
        heading = re.fullmatch(r'(=+)\s*(.*?)\s*\1', line)
        if heading:
            title = heading[2]
            if len(heading[1]) == 2:
                section = title
                direction = 'added' if title == 'New' else None
            elif section == 'Events':
                direction = {'Added': 'added', 'Removed': 'removed'}.get(title)
            continue
        if not line.startswith('*'):
            continue
        if section == 'Removals':
            entries.extend(bfa_removal_entries(number, line))
        elif direction and section in ('New', 'Events'):
            category = 'events' if section == 'Events' else 'global-api'
            entries.extend(make_entry(category, direction, number, match[0])
                           for match in TEMPLATE.finditer(line))
    return entries


def parse_legacy_summary_tables(text):
    """Retain bare table names in legacy New summaries, not their prose claims."""
    entries = []
    in_new = False
    for number, line in enumerate(text.splitlines(), 1):
        heading = re.fullmatch(r'==\s*([^=]+?)\s*==', line)
        if heading:
            in_new = heading[1] == 'New'
        elif in_new:
            match = re.fullmatch(r'\* New (?:global table|API tables): (.*)', line)
            if not match:
                continue
            names = match[1].split(' - ', 1)[0].split(', ')
            if not all(re.fullmatch(r'[A-Za-z_][A-Za-z0-9_]*', name) for name in names):
                raise ValueError(f'invalid table summary: {line}')
            entries.extend(make_entry('global-api', 'added', number, '{{api|' + name + '}}')
                           for name in names)
    return entries


def parse_legacy_widget_summaries(text):
    """Keep explicit New widget links; normalize instance names to API owners."""
    entries = []
    in_new = False
    for number, line in enumerate(text.splitlines(), 1):
        heading = re.fullmatch(r'==\s*([^=]+?)\s*==', line)
        if heading:
            in_new = heading[1] == 'New'
        elif in_new:
            kind = re.fullmatch(r'\* New widget type: \[\[UIOBJECT ([^|\]]+)(?:\|[^\]]+)?\]\]', line)
            method = re.fullmatch(r'\* New texture method: \[\[API_Texture_([A-Za-z_][A-Za-z0-9_]*)\|[^\]]+\]\]', line)
            symbol = kind[1] if kind else ('Texture:' + method[1] if method else None)
            if symbol:
                entries.append(make_entry('widgets', 'added', number, '{{api|' + symbol + '}}'))
    return entries


def parse_prose_namespace_migrations(text):
    """Publish the named destination table, never infer unnamed member retirements."""
    entries = []
    in_changes = False
    for number, line in enumerate(text.splitlines(), 1):
        heading = re.fullmatch(r'==\s*([^=]+?)\s*==', line)
        if heading:
            in_changes = heading[1] == 'Changes'
        elif in_changes:
            match = re.search(r'API have been moved to the new (C_[A-Za-z0-9_]+) table\.', line)
            if match:
                entry = make_entry('global-api', 'changed', number, '{{api|' + match[1] + '}}')
                entry['annotation'] = line
                entries.append(entry)
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


def parse_top_level_api_bullets(text):
    """Retain explicit New/Changes summary identities, not shortened labels (7.2.5)."""
    entries = []
    section = None
    for number, line in enumerate(text.splitlines(), 1):
        heading = re.fullmatch(r'==\s*([^=]+?)\s*==', line)
        if heading:
            section = heading[1]
            continue
        if section not in ('New', 'Changes') or not line.startswith('* '):
            continue
        references = list(TEMPLATE.finditer(line))
        renamed = ' renamed to ' in line
        if renamed and len(references) != 2:
            raise ValueError(f'expected old/successor rename pair: {line}')
        for index, match in enumerate(references):
            parts = match[2].split('|')
            symbol = next(part for part in parts if '=' not in part)
            namespace = 't=n' in parts
            if renamed:
                direction = 'removed' if index == 0 else 'added'
            else:
                direction = 'changed' if section == 'Changes' and namespace else 'added'
            entry = make_entry('global-api', direction, number, '{{api|' + symbol + '}}')
            entry['annotation'] = line
            entries.append(entry)
    return entries


def parse_legacy_widget_cvar_bullets(text):
    """Retain linked frame methods, nested CVar names and underscore API links."""
    entries = []
    section, cvars = None, False
    for number, line in enumerate(text.splitlines(), 1):
        heading = re.fullmatch(r'==\s*([^=]+?)\s*==', line)
        if heading:
            section, cvars = heading[1], False
            continue
        if section == 'New' and line.startswith('* '):
            cvars = line == '* New CVars:'
            if line.startswith('* New frame methods:'):
                for match in re.finditer(r'\[\[API_Frame_\w+\|frame:(\w+)\([^]]*\)\]\]', line):
                    entries.append(make_entry('widgets', 'added', number,
                                              '{{api|Frame:' + match[1] + '}}'))
        elif section == 'New' and cvars:
            match = re.match(r"\*\* '''([A-Za-z_][A-Za-z0-9_]*)''' - ", line)
            if match:
                entries.append(make_entry('cvars', 'added', number, '{{api|' + match[1] + '}}'))
        elif section == 'Changes':
            for match in re.finditer(r'\[\[API_([^|\]]+)(?:\|[^\]]+)?\]\]', line):
                entry = make_entry('global-api', 'changed', number, '{{api|' + match[1] + '}}')
                entry['annotation'] = line
                entries.append(entry)
    return entries


def parse_prose_api_links(text):
    """Retain each API-linked identity in Changes prose, without inventing an addition."""
    entries = []
    in_changes = False
    for line_no, line in enumerate(text.splitlines(), 1):
        heading = re.fullmatch(r'=+\s*([^=]+?)\s*=+', line)
        if heading:
            in_changes = heading[1] == 'Changes'
        elif in_changes:
            for match in re.finditer(r'\[\[API ([^|\]]+)(?:\|[^\]]+)?\]\]', line):
                symbol = match[1]
                entry = make_entry('global-api', 'changed', line_no, f'{{{{api|{symbol}}}}}')
                entry['annotation'] = line
                entries.append(entry)
    return entries


def legion_reference_entries(number, line, section):
    """Normalize canonical API owners; replacement links are not retirements."""
    references = []
    for match in TEMPLATE.finditer(line):
        parts = match[2].split('|')
        symbol = next(part for part in parts if '=' not in part)
        category = 'events' if 't=e' in parts else 'global-api'
        references.append((match.start(), category, symbol))
    for match in re.finditer(r'\[\[API ([^|\]]+)(?:\|[^\]]+)?\]\]', line):
        words = match[1].replace('_', ' ').split()
        symbol = ':'.join(words) if len(words) == 2 else words[0]
        references.append((match.start(), 'widgets' if ':' in symbol else 'global-api', symbol))
    renamed = ' renamed to ' in line
    replaced = ' replaced by ' in line
    removed = ' has been removed' in line
    entries = []
    seen = set()
    for index, (_, category, symbol) in enumerate(sorted(references)):
        if symbol in seen:
            continue
        seen.add(symbol)
        direction = 'added' if section == 'New' else 'changed'
        if renamed or replaced or removed or section == 'Removals':
            direction = 'removed' if index == 0 or (section == 'Removals' and not replaced) else 'added'
        entry = make_entry(category, direction, number, '{{api|' + symbol + '}}')
        entry['annotation'] = line
        entries.append(entry)
    return entries


def legion_summary_entries(number, line, section):
    """Retain explicitly named bare inventories, never expand etc. or domains."""
    symbols = []
    category, direction = 'global-api', 'added' if section == 'New' else 'changed'
    if line.startswith('* New Widget types:'):
        category = 'widgets'
        symbols = re.findall(r'\[\[UIOBJECT ([^|\]]+)', line)
    elif section == 'New' and line.startswith('* '):
        symbols = re.findall(r'\bC_[A-Za-z0-9_]+\b', line)
    elif section == 'Removals':
        direction = 'removed'
        if line.startswith('* Event '):
            category, symbols = 'events', [line.split()[2]]
        elif line.startswith('* CVar '):
            category = 'cvars'
            symbols = re.findall(r'\[\[CVar ([^|\]]+)', line)[:1]
        else:
            prefix = line.split(' (', 1)[0]
            symbols = re.findall(r'\b(?:CastGlyph|SetGlyph|GetGlyphInfo|GetInventoryItemGems|GetContainerItemGems|GetQuestLogRewardTalents|ShowHelm|ShowCloak|ShowingHelm|ShowingCloak)\b', prefix)
    elif 'Functions like ' in line:
        symbols = re.findall(r'\b(?:GetNumNamePlateMotionTypes|GetNameplateFrames)\b', line)
    return [dict(make_entry(category, direction, number, '{{api|' + symbol + '}}'), annotation=line)
            for symbol in dict.fromkeys(symbols)]


def parse_legion_prepatch(text):
    """Retain 7.0.3 nested inventories, widget owners and explicit removals."""
    entries, section = [], None
    for number, line in enumerate(text.splitlines(), 1):
        heading = re.fullmatch(r'==\s*([^=]+?)\s*==', line)
        if heading:
            section = heading[1]
        elif section in ('New', 'Changes', 'Removals') and line.startswith('*'):
            rows = legion_reference_entries(number, line, section)
            rows.extend(legion_summary_entries(number, line, section))
            entries.extend({row['id']: row for row in rows}.values())
    return entries


def parse_warlords_diff(text):
    """Parse the separately pinned 6.0.2 diff; enum values remain prose targets."""
    sections = {'Global API': 'global-api', 'FrameXML': 'framexml',
                'Events': 'events', 'Widget API': 'widgets'}
    entries, counts, table, section = [], [], [], None
    for number, line in enumerate(text.splitlines(), 1):
        if line.startswith('{|'):
            table, section = [], None
        elif line.startswith('|+'):
            section = next((value for caption, value in sections.items()
                            if line.startswith('|+ ' + caption + ' ')), None)
        elif line == '|}' and section:
            # Bare removal identities need the same reference syntax as additions.
            normalized = [(n, ': {{api|' + s[2:] + '}}')
                          if re.fullmatch(r': [A-Za-z_][A-Za-z0-9_:.]*', s) else (n, s)
                          for n, s in table]
            rows, headers = parse_section(section, normalized, legacy_column_headers=True)
            for row in rows:
                row['id'] = 'diff-' + row['id']
                row['source_path'] = 'data/patch-api/sources/6.0.2-api-changes.diff.wikitext'
            entries.extend(rows)
            counts.extend(headers)
            section = None
        else:
            table.append((number, line))
    return entries, counts


def parse_warlords_prepatch(text):
    """Retain compact/nested 6.0.2 references; unnamed removals stay prose."""
    entries, section = [], None
    for number, line in enumerate(text.splitlines(), 1):
        heading = re.fullmatch(r'==\s*([^=]+?)\s*==', line)
        if heading:
            section = heading[1]
            continue
        if section not in ('New', 'Changes') or not line.startswith('*'):
            continue
        rows = legion_reference_entries(number, line, section)
        for row in rows:
            if row['section'] == 'global-api' and ' ' in row['symbol']:
                row['symbol'] = row['symbol'].replace(' ', ':')
                row['section'] = 'widgets'
                row['id'] = f"wt-widgets-{row['symbol']}-{number}"
        names = re.findall(r'\b(C_[A-Za-z0-9_]+)\.\*', line) if section == 'New' else []
        kind = re.search(r'New Widget type: ([A-Za-z_][A-Za-z0-9_]*)', line)
        if kind:
            rows.append(make_entry('widgets', 'added', number, '{{api|' + kind[1] + '}}'))
        rows.extend(make_entry('global-api', 'added', number, '{{api|' + name + '}}')
                    for name in names)
        entries.extend({row['id']: dict(row, annotation=line) for row in rows}.values())
    return entries


def parse_indented_api_lists(text):
    """Retain standalone legacy lists, explicit rename pairs and named CVar removals."""
    entries = []
    section = None
    for number, line in enumerate(text.splitlines(), 1):
        heading = re.fullmatch(r'==\s*([^=]+?)\s*==', line)
        if heading:
            section = heading[1]
            continue
        if section == 'New' and re.fullmatch(r'\s+\{\{api\|[^{}]+\}\}', line):
            entries.append(make_entry('global-api', 'added', number, line))
        elif section == 'Changes' and re.fullmatch(
                r'\s+\{\{api\|[^{}]+\}\}\s*->\s*\{\{api\|[^{}]+\}\}', line):
            entries.extend(make_entry('global-api', direction, number, match[0])
                           for direction, match in zip(('removed', 'added'), TEMPLATE.finditer(line)))
        elif section == 'Removals':
            name = re.fullmatch(r'\s+([A-Za-z_][A-Za-z0-9_]*)', line)
            cvar = re.search(r'“([A-Za-z_][A-Za-z0-9_]*)” \[\[CVar\]\] no longer exists', line)
            if name or cvar:
                category = 'cvars' if cvar else 'global-api'
                symbol = (cvar or name)[1]
                entry = make_entry(category, 'removed', number, '{{api|' + symbol + '}}')
                entry['annotation'] = line
                entries.append(entry)
    return entries


def parse_cataclysm_labeled_inventory(text):
    """Retain explicit NEW/REMOVED rows and typed Breaking changes references."""
    entries = []
    section = None
    for number, line in enumerate(text.splitlines(), 1):
        heading = re.fullmatch(r'==\s*([^=]+?)\s*==\s*(?:<!--.*?-->)?', line)
        if heading:
            section = heading[1].strip()
            continue
        labeled = re.fullmatch(r': (NEW|REMOVED) (\{\{api\|[^{}]+\}\})', line)
        if labeled and section in ('Global API', 'Events'):
            category = 'events' if section == 'Events' else 'global-api'
            entries.append(make_entry(category, 'added' if labeled[1] == 'NEW' else 'removed',
                                      number, labeled[2]))
        elif section == 'Breaking changes' and line.startswith('*'):
            for match in TEMPLATE.finditer(line):
                symbol, params = parse_symbol(match[0])
                category = 'events' if params.get('t') == 'e' else 'global-api'
                entry = make_entry(category, 'changed', number, match[0])
                entry['annotation'] = line
                entries.append(entry)
    return entries


def parse_colon_api_bullets(text):
    """Retain standalone colon-prefixed API additions, not addon descriptions."""
    entries = []
    in_new = False
    for number, line in enumerate(text.splitlines(), 1):
        heading = re.fullmatch(r'==\s*([^=]+?)\s*==', line)
        if heading:
            in_new = heading[1] == 'New'
        elif in_new and re.fullmatch(r':[*:]+\s*\{\{api\|[^{}]+\}\}', line):
            entries.append(make_entry('global-api', 'added', number, line))
    return entries


def parse_legacy_section_lists(text):
    """Retain 2010 sectioned API/event lists and literal signature fragments."""
    sections = {
        'New API functions': ('global-api', 'added'),
        'New FrameXML API': ('framexml', 'added'),
        'New Events': ('events', 'added'),
        'API Changes': ('global-api', 'changed'),
        'Removed FrameXML API': ('framexml', 'removed'),
    }
    entries, current = [], None
    for number, line in enumerate(text.splitlines(), 1):
        heading = re.fullmatch(r'==\s*([^=]+?)\s*==', line)
        if heading:
            current = sections.get(heading[1])
            continue
        if current is None or not line.startswith((':', '*')):
            continue
        for match in re.finditer(r'\[\[API [^\]]+\]\]|\{\{api\|[^{}]+\}\}', line):
            entry = make_entry(*current, number, match[0])
            entry['annotation'] = line
            signature = re.match(r'\([^\n)]*\)', line[match.end():])
            if signature:
                entry['signature'] = signature[0]
            returns = re.fullmatch(r':\s*(.*?)\s*=\s*', line[:match.start()])
            if returns:
                entry['returns'] = returns[1]
            entries.append(entry)
    return entries


def parse_combat_restriction_bullets(text):
    """Retain 5.4.8 Breaking changes identities, never infer removal."""
    entries = []
    in_breaking = False
    for number, line in enumerate(text.splitlines(), 1):
        heading = re.fullmatch(r'==\s*([^=]+?)\s*==', line)
        if heading:
            in_breaking = heading[1] == 'Breaking changes'
            continue
        if not in_breaking or not line.startswith('*'):
            continue
        for match in re.finditer(r'\[\[CVar ([^|\]]+)(?:\|[^\]]+)?\]\]', line):
            entry = make_entry('cvars', 'changed', number, '{{api|' + match[1] + '}}')
            entry['annotation'] = line
            entries.append(entry)
        for match in TEMPLATE.finditer(line):
            entry = make_entry('global-api', 'changed', number, match[0])
            entry['annotation'] = line
            entries.append(entry)
    return entries


def normalize_mists_code_inventory(text):
    """Replace code-wrapped inventory names without moving original source lines."""
    return re.sub(r'^: <code>([A-Za-z_][A-Za-z0-9_:.]*)</code>$',
                  r': {{api|\1}}', text, flags=re.M)


def mists_prepatch_category(symbol, params, section):
    if section == 'FrameXML':
        return 'framexml'
    if params.get('t') == 'w':
        return 'widgets'
    if params.get('t') == 'e' or (symbol.isupper() and not symbol.startswith('LE_')):
        return 'events'
    return 'global-api'


def mists_prepatch_references(line):
    """Keep code names and API references in their original occurrence order."""
    references = []
    for match in re.finditer(r'<code>([A-Za-z_][A-Za-z0-9_, :.]*)</code>', line):
        for symbol in match[1].split(', '):
            references.append((match.start(), match.end(), symbol, {}))
    for match in TEMPLATE.finditer(line):
        symbol, params = parse_symbol(match[0])
        references.append((match.start(), match.end(), symbol, params))
    for match in re.finditer(r'\bLE_PARTY_CATEGORY_(?:HOME|INSTANCE)\b', line):
        references.append((match.start(), match.end(), match[0], {}))
    return sorted(references, key=lambda reference: reference[0])


def parse_mists_prepatch(text):
    """Keep 5.0.4 prose identities; only explicit rename/removal clauses retire names."""
    entries = []
    section = None
    for number, line in enumerate(text.splitlines(), 1):
        heading = re.fullmatch(r'==\s*([^=]+?)\s*==', line)
        if heading:
            section = heading[1]
            continue
        if not line.startswith('*') or section == 'Automated diff':
            continue
        rename = re.search(r'&rarr;| is renamed(?:/merged)? to ', line)
        removal = re.search(r' is removed\b', line)
        successor_seen = False
        for start, end, symbol, params in mists_prepatch_references(line):
            direction = 'added' if re.match(r'\* NEW\b', line) else 'changed'
            if rename:
                if end <= rename.start():
                    direction = 'removed'
                elif not successor_seen:
                    direction, successor_seen = 'added', True
            elif removal and end <= removal.start():
                direction = 'removed'
            category = mists_prepatch_category(symbol, params, section)
            entry = make_entry(category, direction, number, '{{api|' + symbol + '}}')
            entry['annotation'] = line
            signature = re.match(r'\([^\n)]*\)', line[end:])
            if signature:
                entry['signature'] = signature[0]
            entries.append(entry)
    return entries


def parse_mists_automated_diff(text):
    """Parse 2013 captioned inventories; enum values stay in the prose extract."""
    sections = {'Global API': 'global-api', 'FrameXML': 'framexml',
                'Events': 'events', 'Widget API': 'widgets'}
    entries, counts = [], []
    for table in re.finditer(r'^\{\|[^\n]*\n([\s\S]*?)^\|\}', text, re.M):
        lines = table[1].splitlines()
        caption = next((line for line in lines if line.startswith('|+ ')), '')
        section = next((value for name, value in sections.items()
                        if caption.startswith('|+ ' + name + ' ')), None)
        if section is None:
            continue
        first_line = text[:table.start(1)].count('\n') + 1
        normalized = [(first_line + offset, ': {{api|' + line[2:] + '}}')
                      if re.fullmatch(r': [A-Za-z_][A-Za-z0-9_:.]*', line)
                      else (first_line + offset, line)
                      for offset, line in enumerate(lines)]
        rows, headers = parse_section(section, normalized, legacy_column_headers=True)
        entries.extend(rows)
        counts.extend(headers)
    return entries, counts


def parse_mists_summary(text):
    """Keep explicit 5.4.0 prose references without inferring historical removals."""
    entries = []
    section = None
    for number, line in enumerate(text.splitlines(), 1):
        heading = re.fullmatch(r'==\s*([^=]+?)\s*==', line)
        if heading:
            section = heading[1]
        elif section in ('Breaking changes', 'Modified API', 'Bugs', 'New features'):
            for match in TEMPLATE.finditer(line):
                category = 'widgets' if '|t=w|' in match[0] else (
                    'events' if '|t=e|' in match[0] else 'global-api')
                direction = 'added' if section == 'New features' else 'changed'
                symbol = next(part for part in match[2].split('|') if '=' not in part)
                entry = make_entry(category, direction, number, '{{api|' + symbol + '}}')
                entry['annotation'] = line
                entries.append(entry)
    return entries


def parse_mists_widget_handlers(text):
    """Keep Browser handler owner/name pairs in the dedicated 2013 table."""
    entries, counts = [], []
    for table in re.finditer(r'^\{\|[^\n]*\n([\s\S]*?)^\|\}', text, re.M):
        lines = table[1].splitlines()
        if not any(line.startswith('|+ Widget Handlers ') for line in lines):
            continue
        first_line = text[:table.start(1)].count('\n') + 1
        rows, headers = parse_section(
            'widgets', list(enumerate(lines, first_line)), legacy_column_headers=True)
        for row in rows:
            row['kind'] = 'widget-script'
        entries.extend(rows)
        counts.extend(headers)
    return entries, counts


def parse_mists_bare_widget_handlers(text):
    """Normalize bare owner/handler removals without changing source line numbers."""
    normalized = re.sub(r'^: ([A-Za-z_][A-Za-z0-9_]* On[A-Za-z0-9_]+)$',
                        r': {{api|t=wh|\1}}', text, flags=re.M)
    return parse_mists_widget_handlers(normalized)


def normalize_mists_code_removals(text):
    """Keep 2012 code-wrapped API/handler identities at their original line numbers."""
    return re.sub(r'^: <code>([A-Za-z_][A-Za-z0-9_:.]*(?: On[A-Za-z0-9_]+)?)</code>$',
                  r': {{api|\1}}', text, flags=re.M)


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
    parser.add_argument('--legacy-api-tables', action='store_true',
                        help='Parse older unheaded/repeated caption inventories independently')
    parser.add_argument('--historical-api-headings', action='store_true',
                        help='Retain New API functions/New events/Removed API headings; opt-in')
    parser.add_argument('--cataclysm-change-bullets', action='store_true',
                        help='Retain NEW/REMOVED API groups and changed events in Cataclysm summaries; opt-in')
    parser.add_argument('--legacy-api-bullets', action='store_true',
                        help='Retain API New/Removals and event bullet inventories; opt-in')
    parser.add_argument('--legacy-api-renames', action='store_true',
                        help='Retain both identities in API/Renamed bullet pairs; opt-in')

    parser.add_argument('--bfa-prepatch', action='store_true',
                        help='Parse 8.0.1 nested namespaces, prose removals and event lists; opt-in')
    parser.add_argument('--prose-api-links', action='store_true',
                        help='Retain API-linked identities in Changes prose; opt-in')
    parser.add_argument('--legacy-summary-tables', action='store_true',
                        help='Retain bare New global/API table summary names; opt-in')
    parser.add_argument('--top-level-api-bullets', action='store_true',
                        help='Retain top-level New/Changes summary identities and rename pairs; opt-in')
    parser.add_argument('--legacy-widget-summaries', action='store_true',
                        help='Retain explicit New widget type/texture method links; opt-in')
    parser.add_argument('--prose-namespace-migrations', action='store_true',
                        help='Retain named Changes destination tables without inferring removals; opt-in')
    parser.add_argument('--legacy-widget-cvar-bullets', action='store_true',
                        help='Retain frame-method/CVar summaries and underscore API links; opt-in')
    parser.add_argument('--legion-prepatch', action='store_true',
                        help='Parse Legion nested inventories, canonical widget owners and explicit removals; opt-in')
    parser.add_argument('--indented-api-lists', action='store_true',
                        help='Retain standalone legacy API lists, rename pairs and CVar removals; opt-in')
    parser.add_argument('--cataclysm-labeled-inventory', action='store_true',
                        help='Retain NEW/REMOVED colon inventories and typed breaking references; opt-in')
    parser.add_argument('--colon-api-bullets', action='store_true',
                        help='Retain standalone colon-prefixed New API bullets; opt-in')
    parser.add_argument('--warlords-prepatch', action='store_true',
                        help='Retain compact Warlords summary references and canonical widget owners; opt-in')
    parser.add_argument('--warlords-diff', help='Separately pinned Warlords transcluded inventory; opt-in')
    parser.add_argument('--legacy-section-lists', action='store_true',
                        help='Retain 2010 sectioned API/event lists and literal signatures; opt-in')
    parser.add_argument('--combat-restriction-bullets', action='store_true',
                        help='Retain changed CVar/API identities in Breaking changes; opt-in')
    parser.add_argument('--client-line', choices=('retail', 'mists-classic', 'classic-era'),
                        help='Label the client history for isolated supersession; opt-in')
    parser.add_argument('--mists-automated-diff', action='store_true',
                        help='Parse 2013 Mists captioned APIs and bare removals; opt-in')
    parser.add_argument('--mists-prepatch', action='store_true',
                        help='Retain 5.0.4 prose/rename identities and code-wrapped diff names; opt-in')
    parser.add_argument('--mists-summary', action='store_true',
                        help='Retain explicit 5.4.0 summary API occurrences; opt-in')
    parser.add_argument('--mists-diff', help='Separately pinned Mists transcluded inventory; opt-in')
    parser.add_argument('--mists-widget-handlers', action='store_true',
                        help='Retain captioned Mists widget handler ownership; opt-in')
    parser.add_argument('--mists-bare-widget-handlers', action='store_true',
                        help='Retain bare Mists owner/handler removals; opt-in')
    parser.add_argument('--mists-code-removals', action='store_true',
                        help='Normalize 2012 code-wrapped removals in the pinned diff; opt-in')
    parser.add_argument('--wrath-retail-summary', action='store_true',
                        help='Retain 2009 bare-call/method/event summary identities; opt-in')
    parser.add_argument('--wrath-launch-inventory', action='store_true',
                        help='Retain literal 3.0.2 labeled and summary inventory; opt-in')
    args = parser.parse_args()
    patch, path, revid, out = args.patch, args.path, args.revid, args.out
    raw = Path(path).read_bytes()
    buckets = split_sections(raw.decode("utf-8"),
                             separate_inline_structures=args.separate_inline_structures)
    if args.mists_automated_diff:
        buckets = {}
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
    if args.legacy_api_tables:
        entries, counts = parse_legacy_caption_tables(raw.decode('utf-8'))
    if args.historical_api_headings:
        entries.extend(parse_historical_api_headings(raw.decode('utf-8')))
    if args.cataclysm_change_bullets:
        entries.extend(parse_cataclysm_change_bullets(raw.decode('utf-8')))
    if args.wrath_retail_summary:
        entries.extend(parse_wrath_retail_summary(raw.decode('utf-8')))
    if args.wrath_launch_inventory:
        entries.extend(parse_wrath_launch_inventory(raw.decode('utf-8')))
    if args.legacy_api_bullets:
        entries.extend(parse_legacy_api_bullets(raw.decode('utf-8')))
    if args.legacy_api_renames:
        entries.extend(parse_legacy_api_renames(raw.decode('utf-8')))
    if args.bfa_prepatch:
        entries, counts = parse_bfa_prepatch(raw.decode('utf-8')), []
    if args.simple_api_list:
        entries.extend(parse_simple_api_list(raw.decode('utf-8')))
    if args.diff_api_additions:
        entries.extend(parse_diff_api_additions(raw.decode('utf-8')))
    if args.prose_api_links:
        entries.extend(parse_prose_api_links(raw.decode('utf-8')))
    if args.legacy_summary_tables:
        entries.extend(parse_legacy_summary_tables(raw.decode('utf-8')))
    if args.top_level_api_bullets:
        entries.extend(parse_top_level_api_bullets(raw.decode('utf-8')))
    if args.legacy_widget_summaries:
        entries.extend(parse_legacy_widget_summaries(raw.decode('utf-8')))
    if args.prose_namespace_migrations:
        entries.extend(parse_prose_namespace_migrations(raw.decode('utf-8')))
    if args.legacy_widget_cvar_bullets:
        entries.extend(parse_legacy_widget_cvar_bullets(raw.decode('utf-8')))
    if args.legion_prepatch:
        entries, counts = parse_legion_prepatch(raw.decode('utf-8')), []
    if args.warlords_prepatch:
        entries, counts = parse_warlords_prepatch(raw.decode('utf-8')), []
    if args.indented_api_lists:
        entries.extend(parse_indented_api_lists(raw.decode('utf-8')))
    if args.cataclysm_labeled_inventory:
        entries, counts = parse_cataclysm_labeled_inventory(raw.decode('utf-8')), []
    if args.colon_api_bullets:
        entries.extend(parse_colon_api_bullets(raw.decode('utf-8')))
    if args.warlords_diff:
        diff_entries, diff_counts = parse_warlords_diff(Path(args.warlords_diff).read_text())
        entries.extend(diff_entries)
        counts.extend(diff_counts)
    if args.legacy_section_lists:
        entries.extend(parse_legacy_section_lists(raw.decode('utf-8')))
    if args.combat_restriction_bullets:
        entries.extend(parse_combat_restriction_bullets(raw.decode('utf-8')))
    if args.mists_automated_diff:
        entries, counts = parse_mists_automated_diff(raw.decode('utf-8'))
    if args.mists_prepatch:
        entries, counts = parse_mists_prepatch(raw.decode('utf-8')), []
    if args.mists_summary:
        entries.extend(parse_mists_summary(raw.decode('utf-8')))
    if args.mists_diff:
        diff_raw = Path(args.mists_diff).read_text()
        if args.mists_code_removals:
            diff_raw = normalize_mists_code_removals(diff_raw)
        if args.mists_prepatch:
            diff_raw = normalize_mists_code_inventory(diff_raw)
        diff_entries, diff_counts = parse_mists_automated_diff(diff_raw)
        for entry in diff_entries:
            entry['id'] = 'diff-' + entry['id']
        entries.extend(diff_entries)
        counts.extend(diff_counts)
    if args.mists_widget_handlers:
        handler_raw = diff_raw if args.mists_diff else raw.decode('utf-8')
        handler_entries, handler_counts = parse_mists_widget_handlers(handler_raw)
        if args.mists_diff:
            for entry in handler_entries:
                entry['id'] = 'diff-' + entry['id']
        entries.extend(handler_entries)
        counts.extend(handler_counts)
    if args.mists_bare_widget_handlers:
        handler_raw = Path(args.mists_diff).read_text() if args.mists_diff else raw.decode('utf-8')
        handler_entries, handler_counts = parse_mists_bare_widget_handlers(handler_raw)
        if args.mists_diff:
            for entry in handler_entries:
                entry['id'] = 'diff-' + entry['id']
        entries.extend(handler_entries)
        counts.extend(handler_counts)
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
    register = with_client_line(register, args.client_line)
    with open(out, "w") as handle:
        json.dump(register, handle, indent=2, ensure_ascii=False)
        handle.write("\n")


if __name__ == "__main__":
    main()
