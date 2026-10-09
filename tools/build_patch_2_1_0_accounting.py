"""Pure, bounded SOURCE accounting for the frozen historical retail 2.1.0 page."""
from collections import Counter
import hashlib
import re

LIMITS = {
    'Highlights': 'Summary only; no measured timing/memory baseline or expanded directive contract.',
    'Template Handlers': 'Shared closure identity/setfenv warning; no measured closure, XML lifetime or environment behavior.',
    'Memory Profiling': 'Per-addon cached KB/one-byte precision and update boundary; no allocation attribution, cache snapshot or GC measurement.',
    'CPU Profiling': 'Disabled-by-default persisted scriptProfile/reload activation; seconds/about-microsecond precision, counts, optional subtree/event aggregation and reset require real profiling model; no invented default/coercion/measurement.',
    'Frames': 'Inherited alpha clamps per object; secure hook equivalence and effective-alpha expression unmeasured, not all-widget/security/native parity.',
    'GameTooltip': 'Second unitid return and index 1/2 comparison; wildcard removal remains unexpanded, linked helper does not enumerate removed methods.',
    'API Functions': 'Literal arguments/returns/prose only; taint source, merchant cursor, login timing, group feign death, whisper channel, castable aura ordering/time and gem/spam context unmeasured.',
    'Observed additions': 'Names only; linked API pages unexpanded, arguments/returns and behavioral contract unspecified.',
    'Observed removals': 'Literal removal occurrence retained including duplicate IsFeignDeath; likely replacement/rename is not an alias or native retirement proof.',
    'Secure Templates': 'Item identifiers, attribute-driven feedback, saved-state stack, state-driver macro example, template derivation/show attributes remain unmeasured; linked header usage unexpanded.',
    'Macros': 'Flyability/focus substitutions, dynamic conditional/slot feedback, bag-slot commands, item IDs and OR clauses remain unmeasured; example is not a complete grammar.',
    'Raid Support Commands': 'Role set/clear/current-target commands, protected unnamed API and secure type/action/unit attributes unmeasured; no inferred API name or toggle default.',
    'AddOn Features': 'LoadManagers ordered first-success/load-on-demand rule; no loader execution or implicit fallback model added.',
    'Default UI Code': 'Vendor call-site changes, reagent count, top-three memory, scriptErrors/error sink and hash_SlashCmdList cache are source claims, not vendor/runtime measurements.',
    'UIPanel Frame Management': 'UIParent defaults 384/-104/0/80 belong to attributes, not CVars; UIPanelWindows initialization, area/width/push/dead/combat layout remain unmeasured.',
    'Spam Reduction': 'Whisper throttle explicitly unspecified/undetermined; no guessed rate/window/channel budget or bypass model.',
    'Protected Actions': 'GM-ticket submit/edit protection; no secure-context, taint or native enforcement probe.',
    'Bug Fixes': 'Raid-position persistence, Cooldown alpha, item self-cast, this typo, auction sort tokens, BG event ordering and ScrollChild parent assignment remain unmeasured.',
    'Other Observed Changes': 'Eleven named GetWorldStateUIInfo returns with uiType first; no values/types/coercion or prior/current tuple parity inferred.',
}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def literal_calls(raw):
    """Retain exact call-like fragments, including repeated examples and wildcards.

    These are source spans, not a Lua parser or a claim of complete signatures.
    Parentheses inside quotes do not end the span; nesting is preserved.
    """
    rows = []
    for number, line in enumerate(raw.splitlines(), 1):
        for match in re.finditer(r'\b([A-Za-z_][\w:*]*)\(', line):
            depth, quote, escaped = 0, None, False
            end = None
            for position in range(match.end() - 1, len(line)):
                character = line[position]
                if escaped:
                    escaped = False
                elif character == '\\':
                    escaped = True
                elif quote:
                    if character == quote:
                        quote = None
                elif character in ('"', "'"):
                    quote = character
                elif character == '(':
                    depth += 1
                elif character == ')':
                    depth -= 1
                    if depth == 0:
                        end = position + 1
                        break
            if end is None:
                raise ValueError(f'unterminated call-like fragment at line {number}')
            rows.append({'source_id': f'call-2.1.0-{number:03}-{match.start()}',
                         'wikitext_line': number, 'start': match.start(), 'end': end,
                         'symbol': match[1], 'fragment': line[match.start():end],
                         'status': 'UNPROVEN', 'capabilities': [],
                         'note': 'UNPROVEN: literal call-like span; prose/example syntax is not full historical argument acceptance or return behavior.'})
    return rows


def account(register, raw, text):
    rows, headers = [], []
    heading = ''
    rendered = text.splitlines()
    raw_lines = raw.splitlines()
    assert len(raw_lines) == len(rendered), 'default rendering changed line count'
    for number, line in enumerate(raw_lines, 1):
        if not line.strip():
            continue
        is_header = line.startswith('==')
        if is_header:
            heading = line.strip('= ')
            headers.append({'source_id': f'raw-2.1.0-{number:03}', 'wikitext_line': number,
                            'literal': line, 'heading': heading, 'inventory_count': None})
        metadata = is_header or number == 1 or line in ('UIParent Attributes:', 'UI Panel Attributes (initial values come from the UIPanelWindows table):')
        note = ('Metadata/source attribution only; external links/navigation are not expanded.' if metadata else
                'UNPROVEN: ' + LIMITS[heading])
        rows.append({'source_id': f'raw-2.1.0-{number:03}', 'wikitext_line': number,
                     'literal': line, 'rendered': rendered[number - 1], 'heading': heading,
                     'status': 'metadata-only' if metadata else 'UNPROVEN',
                     'capabilities': [], 'note': note})
    for entry in register['entries']:
        rows.append({'source_id': entry['id'], 'wikitext_line': entry['wikitext_line'],
                     'symbol': entry['symbol'], 'literal': entry['annotation'],
                     'status': 'UNPROVEN', 'capabilities': [],
                     'note': 'UNPROVEN: literal source occurrence; no runtime publication, absence, retirement, alias or behavioral observation.'})
    for number, line in enumerate(rendered, 1):
        if line.strip():
            rows.append({'source_id': f'extract-2.1.0-{number:03}', 'extract_line': number,
                         'literal': line, 'status': 'cross-reference-only', 'capabilities': [],
                         'note': 'Rendered view only; authoritative raw line retains markup, attribution and command placeholders stripped by default rendering.'})
    signatures = literal_calls(raw)
    by_line = {}
    for call in signatures:
        by_line.setdefault(call['wikitext_line'], []).append(call)
    for entry in register['entries']:
        if entry['section'] not in ('global-api', 'widgets', 'commands'):
            continue
        matches = [call for call in by_line.get(entry['wikitext_line'], []) if call['symbol'] == entry['symbol']]
        line = entry['annotation']
        label = re.match(r'^\* (NEW|UPDATED|REMOVED)\s*[-:]\s*(.*)', line)
        returns = None
        if matches and label:
            prefix = line[label.start(2):matches[0]['start']].strip()
            if prefix.endswith('='):
                returns = prefix[:-1].strip()
        signatures.append({'source_id': 'signature-' + entry['id'], 'inventory_id': entry['id'],
                           'wikitext_line': entry['wikitext_line'], 'symbol': entry['symbol'],
                           'fragment': matches[0]['fragment'] if matches else None,
                           'returns': returns, 'literal': line, 'status': 'UNPROVEN',
                           'capabilities': [], 'note': 'UNPROVEN: literal partial signature/command syntax only; unspecified arguments and return types/values stay unknown.'})
    gaps = [{'source_id': row['source_id'], 'contract': 'source-semantics', 'reason': row['note']}
            for row in [*rows, *signatures] if row['status'] == 'UNPROVEN']
    ledger = {'schema': 'patch-source-accounting/v1', 'patch': '2.1.0', 'client_line': 'retail',
              'source_sha256': digest(raw.encode()),
              'non_inventory_source': {'sha256': digest(text.encode()), 'extractor_flags': []},
              'source_rows': rows, 'signature_rows': signatures, 'headers': headers,
              'header_counts': register['header_counts'], 'meaningful_closures': 0,
              'runtime_observations': [], 'native_observations': []}
    return ledger, gaps


def counts(register, ledger, gaps):
    rows = ledger['source_rows']
    signatures = ledger['signature_rows']
    return {'inventory_rows': len(register['entries']),
            'inventory_sections': dict(Counter(row['section'] for row in register['entries'])),
            'inventory_directions': dict(Counter(row['direction'] for row in register['entries'])),
            'raw_nonblank_rows': sum(row['source_id'].startswith('raw-') for row in rows),
            'extract_rows': sum(row['source_id'].startswith('extract-') for row in rows),
            'ledger_rows': len(rows), 'ledger_statuses': dict(Counter(row['status'] for row in rows)),
            'named_headers': len(ledger['headers']), 'numerical_inventory_headers': 0,
            'signature_rows': len(signatures),
            'literal_call_spans': sum(row['source_id'].startswith('call-') for row in signatures),
            'unspecified_signature_records': sum(row['source_id'].startswith('signature-') and row['fragment'] is None for row in signatures),
            'contract_gap_records': len(gaps), 'meaningful_closures': 0,
            'runtime_observations': 0, 'native_observations': 0}
