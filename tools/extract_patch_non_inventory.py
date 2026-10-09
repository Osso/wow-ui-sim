#!/usr/bin/env python3
"""Extract retained patch non-inventory wikitext and seed supplemental rows.

No network or dependencies. Line suffixes refer to the generated plaintext,
including blank lines, as in the 12.1.0 page ledger. --check never writes.
The 12.0.0 capture has no Notes/Blue posts; 12.0.1 retains Blue posts.
Linked pages are not expanded. --text-only never accesses a coverage ledger.
"""

import argparse
import hashlib
import html
import json
import re
from collections import Counter
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent


def render_api_reference(match):
    """Keep the API identity, not its optional shortened display label."""
    parts = [part for part in match[1].split('|') if '=' not in part]
    return parts[0]


def render_line(line):
    """Render the markup forms present in the retained revisions."""
    line = re.sub(r"\{\{(?:apisummary.header|text\|blizz)\|([^{}]+)\}\}",
                  r"=== \1 ===", line)
    line = line.replace('{{apisummary.blizzquote}}', '')
    line = line.replace('{{clrr}}', '')
    line = line.replace('{{Reflist}}', '[References list; not expanded]')
    line = line.replace('{{:Settings_API}}',
                        '[Transcluded source: Settings_API; not expanded]')
    line = line.replace('{{:Enum.AddOnProfilerMetric}}',
                        '[Transcluded source: Enum.AddOnProfilerMetric; not expanded]')
    # Retain the TOC example's symbolic interface, not today's expanded version.
    line = line.replace('{{API LatestInterface}}', '[API LatestInterface]')
    line = re.sub(r"\{\{keypress\|([^{}]+)\}\}", r"\1", line)
    line = re.sub(r"\{\{(?:g|tlygo|api.inline|apisummary.title)\|([^{}]+)\}\}",
                  r"\1", line)
    line = re.sub(r"\{\{api.system\|[^|{}]+\|([^{}]+)\}\}", r"\1", line)
    line = re.sub(r"\[\[File:[^\]]*\]\]", "", line, flags=re.I)
    line = re.sub(r"\[\[([^]|]+)(?:\|([^]]+))?\]\]",
                  lambda m: m[2] or m[1], line)
    line = re.sub(r"\[https?://\S+\s+([^]]+)\]", r"\1", line)
    line = re.sub(r"\{\{api\|([^{}]+)\}\}", render_api_reference, line)
    line = re.sub(r"\{\{apichanges\|([^|]+)\|[^{}]+\}\}",
                  r"Patch \1 API changes", line)
    line = re.sub(r"\{\{ambox\|.*?<br>(.*?)\|format=tiny\}\}", r"\1", line)
    line = re.sub(r"</?[A-Za-z][^>]*>", "", line)
    line = line.replace("'''", "").replace("''", "")
    return html.unescape(line).rstrip()


def strip_legacy_cvar_tables(raw):
    """Remove CVar inventory tables but keep captions and their source citations."""
    lines = []
    in_cvars, in_table = False, False
    for line in raw.splitlines():
        heading = re.fullmatch(r'==\s*([^=]+?)\s*==', line)
        if heading:
            in_cvars = heading[1] == 'CVars'
        if in_cvars and line.startswith('{|'):
            in_table = True
        elif in_table and line == '|}':
            in_table = False
        elif not in_table or line.startswith('|+'):
            lines.append(line)
    if in_table:
        raise ValueError('unterminated legacy CVar inventory table')
    return '\n'.join(lines) + '\n'


def strip_bfa_inventories(raw):
    """Keep 8.0.1 prose, migration notes and event documentation qualifications."""
    lines = []
    section = None
    for line in raw.splitlines():
        heading = re.fullmatch(r'==([^=]+)==', line)
        if heading:
            section = heading[1].strip()
        if section == 'New' or (section == 'Events' and line.startswith('*')):
            continue
        lines.append(line)
    return '\n'.join(lines) + '\n'


def strip_legion_inventories(raw):
    """Keep prose/renames; omit only pure nested API inventory lines."""
    return '\n'.join(line for line in raw.splitlines()
                     if not re.fullmatch(r'\*{2,3}\s*\{\{api\|[^{}]+\}\},?', line)) + '\n'


def render_patch_diff_reference(raw):
    """Keep transcluded historical diffs as explicit boundaries, not expanded evidence."""
    return re.sub(r'\{\{:Patch ([0-9.]+)/API changes/diff\}\}',
                  r'[Transcluded source: Patch \1/API changes/diff; not expanded]', raw)


def canonicalize_patch_navigation(raw):
    """Render the positional patch, even when named navigation parameters precede it."""
    def replace(match):
        patches = [part.strip() for part in match[1].split('|') if '=' not in part]
        if len(patches) != 1 or not re.fullmatch(r'\d+(?:\.\d+)+', patches[0]):
            raise ValueError('expected one positional patch in apichanges navigation')
        return f'Patch {patches[0]} API changes'
    return re.sub(r'\{\{apichanges\|([^{}]+)\}\}', replace, raw)


def strip_mists_automated_inventories(raw):
    """Keep 2013 prose and enum values; omit API tables already in the register."""
    raw = re.sub(r'^\{\|[^\n]*\n\|\+ (?:Global API|FrameXML|Widget API|Events) [\s\S]*?^\|\}\n?',
                 '', raw, flags=re.M)
    return '\n'.join(line for line in raw.splitlines()
                     if not line.startswith(('{|', '|+', '|-', '!', '| valign', '</div>', '|}'))) + '\n'


def normalize_lowercase_reflist(raw):
    """Retain the lowercase reference marker without expanding linked citations."""
    return raw.replace('{{reflist}}', '{{Reflist}}')


def render_mists_source_markup(raw):
    """Retain Elink fields and omit only the non-transcluded wrapper."""
    raw = raw.replace('<noinclude>{{Transclude|}}</noinclude>', '')
    return re.sub(r'\{\{Elink\|site=([^{}|]+)\|link=([^{}|]+)\|desc=([^{}|]+)\}\}',
                  r'[External link: \1; \2; \3]', raw)


def render_wrath_summary_markup(raw):
    """Keep summary attribution and literal inline XML without changing older extracts."""
    def navigation(match):
        parts = match[1].split('|')
        patch = next(part for part in parts if '=' not in part)
        misc = next((part[5:] for part in parts if part.startswith('misc=')), '')
        return f'Patch {patch} API changes\nSource attribution: {misc}'
    raw = re.sub(r'\{\{apichanges\|([^{}]+)\}\}', navigation, raw)
    return re.sub(r'<Button\s+[^>]+>',
                  lambda match: '`' + html.escape(match[0], quote=False) + '`', raw)


def render_retail_tbc_markup(raw):
    """Retain literal placeholder and nested 2008 citation fields without expansion."""
    raw = raw.replace('<CONSTANT_NAME>', '&lt;CONSTANT_NAME&gt;')
    raw = raw.replace('{{blizz}}', 'Blizzard ')
    return re.sub(r'\{\{Ref web\|([^{}]+)\}\}', r'[Reference: \1]', raw)


def extract_text(raw, *, preserve_examples=False, normalize_inventory_headings=False,
                 retain_reference_notes=False, legacy_api_tables=False,
                 legacy_api_bullets=False, legacy_cvar_tables=False,
                 bfa_prepatch=False, legion_prepatch=False,
                 retain_patch_diff_reference=False, canonical_patch_navigation=False,
                 mists_automated_diff=False, lowercase_reflist=False,
                 mists_source_markup=False, cataclysm_labeled_inventory=False,
                 wrath_summary_markup=False, numbered_reflist=False,
                 retail_tbc_markup=False):
    if retail_tbc_markup:
        raw = render_retail_tbc_markup(raw)
    if cataclysm_labeled_inventory:
        raw = '\n'.join(line for line in raw.splitlines()
                        if not re.fullmatch(r': (NEW|REMOVED) \{\{api\|[^{}]+\}\}', line)) + '\n'
        raw = canonicalize_patch_navigation(raw)
    if wrath_summary_markup:
        raw = render_wrath_summary_markup(raw)
    if numbered_reflist:
        raw = re.sub(r'\{\{Reflist\|(\d+)\}\}',
                     r'[References list; columns=\1; not expanded]', raw)
    if mists_source_markup:
        raw = render_mists_source_markup(raw)
    if lowercase_reflist:
        raw = normalize_lowercase_reflist(raw)
    if canonical_patch_navigation:
        raw = canonicalize_patch_navigation(raw)
    if mists_automated_diff:
        raw = strip_mists_automated_inventories(raw)
    if retain_patch_diff_reference:
        raw = render_patch_diff_reference(raw)
    if legion_prepatch:
        raw = strip_legion_inventories(raw)
    if bfa_prepatch:
        raw = strip_bfa_inventories(raw)
        raw = re.sub(r'\{\{ref web\|([^{}]+)\}\}', r'[Reference: \1]', raw)
        # Preserve the event section's editorial qualifiers, not its API inventory.
        raw = raw.replace('==Events==', '== Events ==')
    if legacy_api_tables:
        raw = re.sub(r'^\{\|[^\n]*\n\|\+ (?:Global API|Widget API|Widget Handlers|Events|Console variables|Console commands) [\s\S]*?^\|\}\n?',
                     '', raw, flags=re.M)
        raw = re.sub(r'\{\{ref web\|([^{}]+)\}\}', r'[Reference: \1]', raw)
    if legacy_cvar_tables:
        raw = strip_legacy_cvar_tables(raw)
    if retain_reference_notes:
        raw = re.sub(r'<ref>\{\{ref web\|([^{}]+)\}\}</ref>',
                     r'[Reference: \1]', raw, flags=re.I)
    if normalize_inventory_headings:
        raw = re.sub(r'^==\s*(Global API|Widgets|Events|CVars)\s*==$',
                     r'==\1==', raw, flags=re.M)
    lines = []
    inventory = False
    in_example = False
    xml_example = False
    in_ambox = False
    unheaded = "===Global API===" not in raw.splitlines()
    if normalize_inventory_headings and '==Global API==' in raw.splitlines():
        unheaded = False
    level_two = any(heading in raw.splitlines() for heading in (
        "==Global API==", "==Events=="))
    for line in raw.splitlines():
        if legacy_api_bullets:
            if line.startswith('* ') and '{{api|' in line:
                continue
            # Bullet inventories are removed individually, not by section: retain
            # unknown '?' statements, namespace notes and source/build context.
            lines.append(render_line(re.sub(r'^(=+)\s*(.*?)\s*\1$', r'\1 \2 \1', line)))
            continue
        if line == '{{Ambox':
            in_ambox = True
            continue
        if in_ambox:
            if line == '}}':
                in_ambox = False
                continue
            if line.startswith('| type = '):
                lines.append(f'[Warning: {line.removeprefix("| type = ")}]')
                continue
            if line.startswith('|'):
                continue
        if preserve_examples:
            opening = re.fullmatch(r'<syntaxhighlight lang="(lua|xml)">', line)
            if opening:
                in_example = True
                lines.append(f'```{opening[1]}')
                continue
            if in_example:
                if line == '</syntaxhighlight>':
                    in_example = False
                    lines.append('```')
                else:
                    lines.append(line)
                continue
        if line in ("===Global API===", "==Global API==", "==Events==") or (unheaded and line.startswith('{| class="wikitable"')):
            inventory = True
        if level_two and line in ('==Widgets==', '==CVars=='):
            inventory = True
        if level_two and line == 'Structures':
            inventory = False
            line = '==Structures=='
        if level_two and line in ("==Enums==", "==Structures==", "==References=="):
            inventory = False
            if line == "==Enums==":
                line = "== Enumerations =="
        if line == "===Enums===" or (unheaded and line == "===Structures==="):
            inventory = False
            if line == "===Enums===":
                line = "=== Enumerations ==="
        if inventory:
            continue
        if line.strip() in ('{{#tag:syntaxhighlight|', '|lang="wowtoc"}}'):
            continue
        if line.strip() in ('{| class="darktable"', '|', '|}'):
            continue
        if line.startswith("=="):
            line = re.sub(r"^(=+)\s*(.*?)\s*\1$", r"\1 \2 \1", line)
        if line.startswith('<syntaxhighlight lang="xml"'):
            xml_example = True
            continue
        if xml_example and line == '</syntaxhighlight>':
            xml_example = False
            continue
        rendered = line if xml_example else render_line(line)
        if '{{' in rendered or '}}' in rendered:
            raise ValueError(f"unhandled template: {line}")
        lines.append(rendered)
    return '\n'.join(lines) + '\n'


def is_source_context(value, number, patch):
    if number == 1 or value.startswith(('* TOC:', '* Official', '* Diffs:', '* Deprecated')):
        return True
    if patch == '11.0.2':
        return value.startswith((': 11.0.0', '[Transcluded source:'))
    if patch == '11.0.5':
        return value.startswith(': 11.0.2')
    if patch == '11.0.7':
        return value.startswith(': 11.0.5')
    if patch == '11.1.0':
        return value.startswith(': 11.0.7')
    if patch == '11.1.5':
        return value.startswith(': 11.1.0')
    if patch == '11.1.7':
        return value.startswith(': 11.1.5')
    if patch == '11.2.0':
        return value.startswith(('** Deprecated_', '** Blizzard_Deprecated', ': 11.1.7'))
    if patch == '11.2.5':
        return value.startswith(('** Deprecated_', ': 11.2.0'))
    if patch == '11.2.7':
        return value.startswith(': 11.2.5')
    if patch == '12.0.0':
        return value.startswith(('** ', '* Addon apocalypse.', ': 11.2.7',
                                 'API changes have been introduced',
                                 'The changes are not intended'))
    return value.startswith(('Patch 12.0.1 is ', 'Midnight 12.0.1 ',
                             'Hello again', 'DISCLAIMER:', 'NOTE:',
                             'Bonus notes that were forgotten:',
                             'Today we have an update', 'Over the course of Beta',
                             'While we had originally hoped', 'In the meantime',
                             'The full list of spells', ': 12.0.0'))


def seed_rows(text, patch='12.0.0'):
    rows = []
    section, parent, date = 'prose', '', 'undated'
    for number, line in enumerate(text.splitlines(), 1):
        value = line.strip()
        if not value:
            continue
        heading = re.fullmatch(r"=+\s*(.*?)\s*=+", value)
        metadata = bool(heading)
        reason = 'Editorial heading; no runtime credit.'
        if heading:
            title = heading[1].strip()
            if title in ('Enumerations', 'Structures'):
                section, parent = title.lower(), ''
            elif title.startswith('Deprecated'):
                section, parent = 'deprecated api', ''
            else:
                section, parent = 'prose', ''
            if re.fullmatch(r'\d{4}-\d{2}-\d{2}', title):
                date = title
        elif section in ('enumerations', 'structures'):
            if not value.startswith(('+', '-', '#')):
                parent = value.replace('.', '-')
            # Parent rows are audit targets too (the 12.1.0 convention).
        elif section == 'deprecated api':
            if value.startswith(': '):
                parent = value[2:].split()[0].replace('.', '-')
            elif value.startswith('Deprecated_'):
                metadata = True
                reason = 'Deprecated-source heading/context; no runtime credit.'
            else:
                parent = 'removal-summary'
        elif is_source_context(value, number, patch):
            metadata = True
            reason = 'Editorial/source/build context or external resource link; no runtime credit.'
        prefix = 'source-context' if metadata else (
            f'{section}-{parent}' if section != 'prose' else f'prose-{date}')
        rows.append({
            'source_id': f'{prefix}-{number:03}',
            'capabilities': [],
            'status': 'metadata-only' if metadata else 'audit-pending',
            'note': reason if metadata else (
                'Non-inventory statement requires behavioral audit; see '
                f'p{patch.replace(".", "")}-extract-scout.md.'),
        })
    return rows


def check_examples():
    """Behavior fixtures kept here: task forbids changing test files."""
    raw = ('==Summary==\n* [[COMBAT_LOG_EVENT]] will error.\n'
           '===Global API===\n{| class="wikitable"\n: {{api|IgnoreMe}}\n|}\n'
           '===Enums===\n Enum.X\n   + <font color="green">New</font>\n'
           '==Deprecated 11.x API==\n{| class="darktable"\n|\n'
           ': {{api|Old}} → {{api|C_New.Call}}\n|}\n')
    expected = ('== Summary ==\n* COMBAT_LOG_EVENT will error.\n'
                '=== Enumerations ===\n Enum.X\n   + New\n'
                '== Deprecated 11.x API ==\n: Old → C_New.Call\n')
    assert extract_text(raw) == expected
    rows = seed_rows(expected)
    assert [row['source_id'] for row in rows] == [
        'source-context-001', 'prose-undated-002', 'source-context-003',
        'enumerations-Enum-X-004', 'enumerations-Enum-X-005',
        'source-context-006', 'deprecated api-Old-007']
    assert render_line('{{ambox|border=purple|image=|type=Important<br>[[Secret Values|Secret values]] restrict Lua.|format=tiny}}') == 'Secret values restrict Lua.'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    parser.add_argument('--self-test', action='store_true')
    parser.add_argument('--patch', default='12.0.0')
    parser.add_argument('--preserve-examples', action='store_true',
                        help='Retain Lua/XML syntaxhighlight contents verbatim in fenced blocks')
    parser.add_argument('--normalize-inventory-headings', action='store_true',
                        help='Recognize spaced inventory headings without swallowing earlier deprecated tables')
    parser.add_argument('--retain-reference-notes', action='store_true',
                        help='Retain inline ref web citation fields alongside publication prose')
    parser.add_argument('--legacy-api-tables', action='store_true',
                        help='Strip older caption inventories while retaining API prose and references')
    parser.add_argument('--legacy-api-bullets', action='store_true',
                        help='Strip legacy API/event bullets without losing prose or unknown markers')
    parser.add_argument('--text-only', action='store_true',
                        help='Write/check plaintext only; never read or modify a coverage ledger')
    parser.add_argument('--legacy-cvar-tables', action='store_true',
                        help='Strip CVar inventory tables, retaining captions/citations; opt-in')
    parser.add_argument('--bfa-prepatch', action='store_true',
                        help='Strip 8.0.1 nested additions/events, retaining prose and reference fields; opt-in')
    parser.add_argument('--legion-prepatch', action='store_true',
                        help='Strip pure nested Legion inventories, retaining summaries and renames; opt-in')
    parser.add_argument('--retain-patch-diff-reference', action='store_true',
                        help='Retain transcluded patch diffs as unexpanded source references; opt-in')
    parser.add_argument('--canonical-patch-navigation', action='store_true',
                        help='Render positional apichanges patch after named parameters; opt-in')
    parser.add_argument('--mists-automated-diff', action='store_true',
                        help='Strip 2013 Mists API tables while retaining prose and enums; opt-in')
    parser.add_argument('--lowercase-reflist', action='store_true',
                        help='Retain lowercase reflist as an unexpanded reference marker; opt-in')
    parser.add_argument('--cataclysm-labeled-inventory', action='store_true',
                        help='Strip explicit NEW/REMOVED rows, retain breaking prose/build context; opt-in')
    parser.add_argument('--mists-source-markup', action='store_true',
                        help='Retain Mists external-link fields and omit noinclude wrappers; opt-in')
    parser.add_argument('--wrath-summary-markup', action='store_true',
                        help='Keep 2009 summary attribution and inline Button XML; opt-in')
    parser.add_argument('--numbered-reflist', action='store_true',
                        help='Retain numbered Reflist markers without expanding citations; opt-in')
    parser.add_argument('--retail-tbc-markup', action='store_true',
                        help='Keep 2008 retail placeholders/nested citation fields; opt-in')
    args = parser.parse_args()
    if args.self_test:
        check_examples()
        print('PASS: extraction and row-ID behavioral fixtures')
        return
    base = ROOT / 'data/patch-api/sources'
    raw_path = base / f'{args.patch}-api-changes.wikitext'
    text_path = base / f'{args.patch}-api-changes.txt'
    coverage_path = base / f'{args.patch}-page-coverage.json'
    text = extract_text(raw_path.read_text(), preserve_examples=args.preserve_examples,
                        normalize_inventory_headings=args.normalize_inventory_headings,
                        retain_reference_notes=args.retain_reference_notes,
                        legacy_api_tables=args.legacy_api_tables,
                        legacy_api_bullets=args.legacy_api_bullets,
                        legacy_cvar_tables=args.legacy_cvar_tables,
                        bfa_prepatch=args.bfa_prepatch,
                        legion_prepatch=args.legion_prepatch,
                        retain_patch_diff_reference=args.retain_patch_diff_reference,
                        canonical_patch_navigation=args.canonical_patch_navigation,
                        mists_automated_diff=args.mists_automated_diff,
                        lowercase_reflist=args.lowercase_reflist,
                        mists_source_markup=args.mists_source_markup,
                        cataclysm_labeled_inventory=args.cataclysm_labeled_inventory,
                        wrath_summary_markup=args.wrath_summary_markup,
                        numbered_reflist=args.numbered_reflist,
                        retail_tbc_markup=args.retail_tbc_markup)
    rows = seed_rows(text, args.patch)
    if args.text_only:
        if args.check:
            assert text_path.read_text() == text, 'extract differs'
        else:
            text_path.write_text(text)
        print(json.dumps({'patch': args.patch, 'rows': len(rows),
                          'statuses': dict(Counter(row['status'] for row in rows))}))
        return
    if args.patch != '12.0.0':
        raise ValueError('Use --text-only for other patches; coverage credit is audit-owned')
    coverage = json.loads(coverage_path.read_text())
    existing = coverage['source_rows']
    inventory_rows = [row for row in existing if row['source_id'].startswith('wt-')]
    audited = [row for row in existing if not row['source_id'].startswith('wt-')
               and (row['capabilities'] or row['status'] not in ('audit-pending', 'metadata-only'))]
    if audited and not args.check:
        raise ValueError('Supplemental rows already audited; refusing to reset proof')
    if args.check:
        assert text_path.read_text() == text, 'extract differs'
        assert [row for row in existing if not row['source_id'].startswith('wt-')] == rows, 'rows differ'
    else:
        text_path.write_text(text)
        coverage['source_rows'] = inventory_rows + rows
        coverage['non_inventory_source'] = {
            'path': str(text_path.relative_to(ROOT)),
            'sha256': hashlib.sha256(text.encode()).hexdigest(),
            'wikitext_revid': 6747189,
            'wikitext_sha256': hashlib.sha256(raw_path.read_bytes()).hexdigest(),
            'extract_tool': 'tools/extract_patch_non_inventory.py',
        }
        coverage['proof_policy'] = (
            'Development tests are not independent final acceptance. Inventory rows retain '
            'their original scope; non-inventory statements are audit-pending except editorial '
            'context. Linked external pages are not expanded. The occurrence-level audit '
            'in data/patch-api/12.0.0.json remains separate evidence.')
        coverage_path.write_text(json.dumps(coverage, indent=2, ensure_ascii=False) + '\n')
    ids = [row['source_id'] for row in coverage['source_rows']]
    assert len(ids) == len(set(ids)), 'duplicate source IDs'
    print(json.dumps({'new_rows': len(rows), 'new_statuses': dict(Counter(row['status'] for row in rows)),
                      'total_statuses': dict(Counter(row['status'] for row in coverage['source_rows'])),
                      'plaintext_sha256': hashlib.sha256(text.encode()).hexdigest()}, indent=2))


if __name__ == '__main__':
    main()
