"""Lossless bounded accounting of the frozen Forever page; no runtime policy."""
from collections import Counter
import hashlib
import importlib.util
import json
from pathlib import Path
import re

CACHE = 'data/patch-api/source-cache/legacy-2026-10-09'
SOURCE = CACHE + '/1.60.1-wikitext.txt'
CALL = re.compile(r'([A-Za-z_]\w*(?:[.:][A-Za-z_]\w*)*)\(')


def digest(data):
    return hashlib.sha256(data).hexdigest()


def json_bytes(value):
    return (json.dumps(value, indent=2, ensure_ascii=False) + '\n').encode()


def tool(evidence):
    path = evidence / 'historical-gen_patch_wikitext_register.py'
    spec = importlib.util.spec_from_file_location('frozen_forever_generator', path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def parse_register(evidence, raw):
    generator = tool(evidence)
    buckets = generator.split_sections(raw.decode())
    entries, headers = [], []
    for section in generator.SECTIONS.values():
        section_entries, counts = generator.parse_section(section, buckets.get(section, []))
        entries.extend(section_entries)
        headers.extend(counts)
    return dict(schema='patch-api-wikitext-register/v1', patch='1.60.1',
                source=dict(path=SOURCE, revid=6902509, sha256=digest(raw)),
                header_counts=headers, entries=entries)


def references(number, line):
    patterns = [('transclusion', r'\{\{:[^{}]+\}\}'),
                ('wiki-link', r'\[\[[^\]]+\]\]'),
                ('external-link', r'\[https?://[^\]]+\]')]
    matches = [(match.start(), kind, match.group()) for kind, pattern in patterns
               for match in re.finditer(pattern, line)]
    return [dict(id=f'reference-{number}-{ordinal}', wikitext_line=number, kind=kind,
                 literal_reference=value, expanded=False, status='UNPROVEN',
                 note='Linked content not fetched or expanded; local enclosing prose is accounted separately.')
            for ordinal, (_, kind, value) in enumerate(sorted(matches), 1)]


def calls(line):
    """Retain balanced literal call fragments, including nested example calls."""
    for match in CALL.finditer(line):
        start = match.end() - 1
        depth, quote, escaped = 0, None, False
        for end in range(start, len(line)):
            char = line[end]
            if quote:
                if escaped:
                    escaped = False
                elif char == '\\':
                    escaped = True
                elif char == quote:
                    quote = None
            elif char in ('\"', "'"):
                quote = char
            elif char == '(':
                depth += 1
            elif char == ')':
                depth -= 1
                if depth == 0:
                    yield match[1], line[match.start():end + 1], line[start + 1:end]
                    break
        else:
            yield match[1], line[match.start():], None


def contract_limit(context, section=None, symbol=''):
    if section == 'framexml':
        return 'Named FrameXML diff only, not native API removal/publication. Loaded vendor UI, argument/return/effect and input/dispatch behavior UNPROVEN; misspellings retained, no alias repair.'
    if section == 'events':
        return 'Event identity diff only; payload, producer/model cause, dispatch timing/order and native firing UNPROVEN.'
    if section == 'cvars':
        return 'Literal type/default/scope/category/description retained where present; absent fields unspecified. Setting state, persistence, described effects, native defaults and command execution UNPROVEN.'
    if section == 'widgets':
        return 'Widget identity only; receiver, arguments/returns, focus/mouse state, model display, soft cursor and native effects UNPROVEN. No 3D implementation implied.'
    if section == 'global-api':
        return 'Native namespace/global identity diff only; arguments/returns/coercion/defaults, state transitions and historical/native/security behavior UNPROVEN. Similar current names do not prove the contract.'
    if context == 'Notable changes':
        return 'Build-qualified literal correction/constant claim, not a measured bug fix. UnitName player before PLAYER_LOGIN, token consistency, SavedVariables and RestrictedEnvironment loading remain UNPROVEN; no returned corrected tuple or full token mapping specified.'
    if context == 'TOC format':
        return 'Literal camelot allow/exclude and title-order examples; unknown game-type handling, file/metadata gating and foreign-client behavior UNPROVEN. Do not derive runtime production policy.'
    if context == 'Game types':
        return 'Literal project constant/interface range/[Game] examples; source TOC 16001 is separate from configured code and native interfaces. GetBuildInfo tuple example is not a complete signature contract.'
    if context == 'Rulesets':
        return 'Literal rule checks and PvE else branch; no rule numeric values, precedence/mutation/default contract or native observation.'
    if context in ('2026-09-15', '2026-09-24', '2026-10-02') or context in ('Cooldown Manager', 'Gamepad Support', 'User Interface'):
        return 'Literal local blue-post prose retained; Mainline/shared-secret/aura claims do not supersede Retail/Era/TBC histories. Class/rank/cooldown/input/layout/art/loot/auction behavior and linked summaries remain UNPROVEN.'
    return 'Literal source claim/example retained without inferred parameters/returns/effects, expanded links, publication/native or security credit.'


def literal_fields(generator, line):
    match = generator.TEMPLATE.search(line)
    return dict(part.split('=', 1) for part in match[2].split('|') if '=' in part) if match else {}


def build_ledger(evidence):
    raw = (evidence / 'original/source.wikitext').read_bytes()
    text = raw.decode()
    toc = re.search(r'^\* TOC: <code>(\d+)</code>$', text, re.M)
    assert toc and 'Forever and modern WoW (Midnight) are two game types' in text
    assert 'Camelot (to be renamed before launch) and Standard' in text
    register = parse_register(evidence, raw)
    generator = tool(evidence)
    lines = text.splitlines()
    inventory, byline = [], {}
    for row in register['entries']:
        number = row['wikitext_line']
        enriched = dict(row, source_text=lines[number - 1],
                        literal_fields=literal_fields(generator, lines[number - 1]),
                        status='UNPROVEN', capabilities=[],
                        note=contract_limit('', row['section'], row['symbol']))
        inventory.append(enriched)
        byline.setdefault(number, []).append(enriched)
    headers, prose, signatures, sources, refs = [], [], [], [], []
    context, table = '', False
    for number, line in enumerate(lines, 1):
        if not line.strip():
            continue
        heading = re.fullmatch(r'(={2,3})([^=]+)\1', line)
        template_heading = re.fullmatch(r'\{\{apisummary.heading\|([^{}]+)\}\}', line)
        if heading or template_heading:
            context = heading[2].strip() if heading else template_heading[1]
            headers.append(dict(id=f'header-{number}', wikitext_line=number, source_text=line,
                                title=context, level=len(heading[1]) if heading else None,
                                literal_count=None, status='metadata-only'))
        count = re.search(r'<small>\((\d+)\)</small>', line)
        if count:
            headers.append(dict(id=f'header-{number}', wikitext_line=number, source_text=line,
                                title=context, literal_count=int(count[1]),
                                direction='added' if '>Added<' in line else 'removed', status='metadata-only'))
        if line.startswith('{|'):
            table = True
        rows = byline.get(number, [])
        for row in rows:
            if row['section'] in ('global-api', 'framexml', 'widgets'):
                signatures.append(dict(id='signature-' + row['id'], inventory_id=row['id'],
                                       symbol=row['symbol'], wikitext_line=number, source_text=line,
                                       role='identity-only; signature unspecified', literal_arguments=None,
                                       literal_returns=None, status='UNPROVEN', capabilities=[],
                                       note=contract_limit(context, row['section'], row['symbol'])))
        if not rows:
            for ordinal, (symbol, fragment, arguments) in enumerate(calls(line), 1):
                signatures.append(dict(id=f'signature-example-{number}-{ordinal}', inventory_id=None,
                                       symbol=symbol, fragment=fragment, wikitext_line=number,
                                       source_text=line, role='literal example/prose, not complete API definition',
                                       literal_arguments=arguments, literal_returns=None,
                                       status='UNPROVEN', capabilities=[], note=contract_limit(context)))
        line_refs = references(number, line)
        refs.extend(line_refs)
        wrapper = bool(heading or template_heading or table or line.startswith((
            '{{apichanges|', '{{#description2:', '<syntaxhighlight', '</syntaxhighlight>',
            ':<syntaxhighlight', '<blockquote', '</blockquote', '{{bluepost', '|poster=',
            '|date=', '|link=', '|title=', '|body=', '{{reflist')) or line == '}}')
        # CVar descriptions are separate prose effects; the inventory identity is not that behavior.
        description = rows[0]['literal_fields'].get('desc') if rows else None
        if (not wrapper and not rows) or description:
            prose.append(dict(id=f'prose-{number}', wikitext_line=number, source_text=line,
                              context=context, literal_description=description,
                              status='UNPROVEN', capabilities=[], note=contract_limit(
                                  context, rows[0]['section'] if rows else None)))
        category = 'inventory' if rows else 'metadata' if wrapper else 'prose'
        sources.append(dict(id=f'source-{number}', wikitext_line=number, source_text=line,
                            context=context, category=category,
                            inventory_ids=[row['id'] for row in rows],
                            reference_ids=[row['id'] for row in line_refs],
                            status='metadata-only' if category == 'metadata' else 'UNPROVEN', capabilities=[]))
        if line == '|}':
            table = False
    return dict(schema='patch-api-full-literal-source/v1', patch='1.60.1',
                client_line='wow-forever-camelot', source_interface=int(toc[1]),
                actual_successors=[], pending_source_navigation=['1.60.2'],
                proof_policy='SOURCE only; no native/historical/security/loaded-UI parity or publication credit.',
                inventory_rows=inventory, header_ledger=headers, signature_ledger=signatures,
                prose_ledger=prose, source_rows=sources, reference_ledger=refs)


def literal_extract(ledger):
    """Literal non-inventory mirror, not expanded/rendered MediaWiki output."""
    return ''.join(row['source_text'] + '\n' for row in ledger['source_rows'] if row['category'] != 'inventory')


def counts(ledger):
    occurrences = Counter((row['section'], row['direction']) for row in ledger['inventory_rows'])
    return dict(client_line=ledger['client_line'], source_interface=ledger['source_interface'],
                inventory_occurrences=len(ledger['inventory_rows']),
                inventory_counts={f'{section}/{direction}': count for (section, direction), count in sorted(occurrences.items())},
                nonblank_rows=len(ledger['source_rows']), headers=len(ledger['header_ledger']),
                signatures=len(ledger['signature_ledger']), prose_limits=len(ledger['prose_ledger']),
                references=len(ledger['reference_ledger']),
                metadata_rows=sum(row['category'] == 'metadata' for row in ledger['source_rows']),
                literal_command_occurrences=sum(row.get('kind') == 'command' for row in ledger['inventory_rows']),
                native_observations=0, runtime_publication_observations=0, meaningful_closures=0)
