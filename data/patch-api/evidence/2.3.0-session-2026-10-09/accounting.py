"""Page-owned frozen 2.3.0 SOURCE parser; shared defaults never change.

Only explicit labels and consolidated links form inventory. A listed identity
is not an inferred addition. Literal call fragments are not complete signatures.
"""
from collections import Counter
import hashlib
import importlib.util
import json
from pathlib import Path
import re

EVIDENCE = Path(__file__).resolve().parent
ROOT = EVIDENCE.parents[3]
SOURCE = ROOT / 'data/patch-api/sources'
CALL = re.compile(
    r'((?:[A-Z]\w*:[A-Z]\w*|(?:Get|Set|Is|Can|Query|Click|Take|Sort|Pickup)[A-Z]\w*|message))'
    r'\s*\(([^)]*)\)')
LABEL = re.compile(r'^\* (NEW|UPDATED|UDPATED|REMOVED|RENAMED|NOTE|ADDED)\s+(?:-\s*)?(.*)$')
HEADING = re.compile(r'==\s*([^=]+?)\s*==')
LINK = re.compile(r'\[\[([^]|]+)(?:\|([^]]+))?\]\]')
LIMITS = {
    'Important Changes': ('casting-latency-and-macro-fallthrough',
        'Casting latency, /stopcasting workarounds and instant-macro fallthrough changes have no complete timing, transition or dispatch contract.'),
    'Macro Commands': ('macro-dispatch-and-conditions',
        'Slash dispatch, pet toggle state, target exact/last identity, instant cancelform/dismount timing and five conditional aliases/SELFCAST behavior are not executed.'),
    'Frame methods': ('widget-input-output-and-state',
        'Event registration, formatted text/garbage allocation, string metrics, explicitOnly defaults, cursor position and tooltip tracking rename need actual widget state and native behavior.'),
    'Frame Support': ('registered-frame-enumeration',
        'Registered-frame varargs order, duplicate handling and event input validation are unspecified and unexecuted.'),
    'API Methods': ('item-spell-macro-and-message-state',
        'Message display with Lua errors disabled, item pickup/count/use flags, spell book/slot selection, action equivalence, helpful/harmful exceptions and macro item/spell/rank state remain unexecuted.'),
    'Garbage Collection': ('gc-combat-timing-and-allocation',
        'Out-of-combat collection and direct-return/caller-table advice supply no collector schedule, allocation budget or native measurement.'),
    'Auction API': ('auction-query-throttle-and-sort-state',
        'getAll full versus 50-item queries/15-minute throttle, boolean query/reversal outputs, ordered sorts, nil missing sort, table/type/index domains, prepend-without-apply and quantity sorting require backing state; partial additions do not establish full signatures.'),
    'Mail API': ('mail-attachment-index-state',
        'Compose slots versus received attachment indices, multi-attachment price, hasItem count and attachment retrieval/removal state are unexecuted; source attachSlot/attachIndex wording mismatch retained.'),
    'Talent API': ('inspected-unit-talent-state-and-event',
        'Optional truthy inspect selection and INSPECT_TALENT_READY availability/update trigger lack complete payload, inspection lifecycle and server state; source UDPATED typo retained.'),
    'Bug Fixes': ('unitname-absent-unit-regression',
        'UnitName absent-unit regression fix supplies neither old nor corrected tuple/value details; no native behavior reconstructed.'),
    '2.3.0 consolidated API changes': ('linked-consolidated-inventory',
        'Explicit linked identities only; linked API pages and Special:Diff/4997529/string-dump comparison are unexpanded, no signature or native completeness inference.'),
}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n')


def limit(section, line):
    if 'ScrollFrame:UpdateScrollChildRect()' in line:
        return ('scroll-child-rect-and-event-lifecycle',
                'Automatic scroll rectangle recalculation and appropriate event dispatch after frame/texture/fontstring changes remain unexecuted; PTR test request is not proof.')
    return LIMITS[section]


def unproven(section, line):
    key, note = limit(section, line)
    return dict(status='UNPROVEN', capabilities=[], limit_key=key,
                note='UNPROVEN: ' + note)


def inventory_for_line(number, line, section):
    linked = re.fullmatch(r': \[\[API ([^]|]+)\|([^]]+)\]\]', line)
    if linked:
        assert linked[1] == linked[2], 'consolidated link identity'
        items = [(linked[1], 'global-api', 'listed', 'consolidated-link')]
    else:
        labeled = LABEL.fullmatch(line)
        if not labeled:
            return []
        label, body = labeled.groups()
        direction = {'NEW': 'added', 'ADDED': 'added', 'UPDATED': 'changed',
                     'UDPATED': 'changed', 'REMOVED': 'removed',
                     'RENAMED': 'renamed', 'NOTE': 'noted'}[label]
        if section == 'Macro Commands':
            items = [(m[0], 'commands', direction, 'explicit-label')
                     for m in re.finditer(r'/[A-Za-z]+', body.split(' will ', 1)[0])]
        elif body.startswith('Event '):
            items = [(body.split()[1], 'events', direction, 'explicit-label')]
        else:
            calls = list(CALL.finditer(body.split('--', 1)[0]))
            assert calls, f'unaccounted label at {number}'
            if label == 'RENAMED':
                assert len(calls) == 2, 'rename pair'
                items = [(calls[0][1], 'widgets', 'removed', 'rename-predecessor'),
                         (calls[1][1], 'widgets', 'added', 'rename-successor')]
            else:
                symbol = calls[0][1]
                items = [(symbol, 'widgets' if ':' in symbol else 'global-api',
                          direction, 'explicit-label')]
    return [dict(id=f'wt-{kind}-{symbol}-{number}-{ordinal}', section=kind,
                 source_section=section, symbol=symbol, direction=direction,
                 origin=origin, wikitext_line=number, source_text=line,
                 **unproven(section, line))
            for ordinal, (symbol, kind, direction, origin) in enumerate(items, 1)]


def signatures_for_line(number, line, section, inventory):
    labeled = LABEL.fullmatch(line)
    body = labeled[2] if labeled else line
    calls = list(CALL.finditer(body))
    signatures = []
    for ordinal, call in enumerate(calls, 1):
        prefix = body[:call.start()]
        signatures.append(dict(
            source_id=f'signature-{number:03}-{ordinal}', symbol=call[1],
            wikitext_line=number, source_text=line, fragment=call[0],
            arguments_literal=call[2],
            return_prefix=prefix if ordinal == 1 and '=' in prefix else None,
            role=('labeled-call' if labeled and ordinal == 1 else 'reference-or-example'),
            complete_signature=False, returns=None, **unproven(section, line)))
    for row in inventory:
        if row['origin'] == 'consolidated-link' or row['section'] == 'commands':
            signatures.append(dict(
                source_id=f"signature-limit-{row['id']}", symbol=row['symbol'],
                wikitext_line=number, source_text=line, fragment=None,
                arguments_literal=None, return_prefix=None,
                role='linked-identity-only' if row['origin'] == 'consolidated-link'
                     else 'literal-command-line-not-reconstructed',
                command_syntax_literal=body if row['section'] == 'commands' else None,
                complete_signature=False, returns=None, **unproven(section, line)))
    return signatures


def derive(raw):
    inventory, signatures, prose, source_rows, headers, links = [], [], [], [], [], []
    section = ''
    for number, line in enumerate(raw.splitlines(), 1):
        if not line.strip():
            continue
        heading = HEADING.fullmatch(line)
        metadata = bool(number == 1 or heading or line.startswith('<!--'))
        if heading:
            section = heading[1]
            assert section in LIMITS, 'unknown source header'
            headers.append(dict(wikitext_line=number, source_text=line, title=section))
        source_rows.append(dict(source_id=f'raw-line-{number:03}',
            wikitext_line=number, source_text=line, source_section=section,
            status='metadata-only' if metadata else 'UNPROVEN', capabilities=[],
            note=('Literal navigation/header/editorial context; template and diff/string-dump comparison unexpanded, not native completeness proof.'
                  if metadata else unproven(section, line)['note'])))
        if metadata:
            continue
        rows = inventory_for_line(number, line, section)
        inventory.extend(rows)
        signatures.extend(signatures_for_line(number, line, section, rows))
        if not line.startswith(': [['):
            prose.append(dict(source_id=f'prose-{number:03}', wikitext_line=number,
                              source_text=line, source_section=section,
                              **unproven(section, line)))
        for ordinal, match in enumerate(LINK.finditer(line), 1):
            links.append(dict(source_id=f'link-{number:03}-{ordinal}',
                              wikitext_line=number, source_text=line,
                              target=match[1], display=match[2], expanded=False,
                              **unproven(section, line)))
    return dict(schema='patch-source-accounting/v1', patch='2.3.0',
                client_line='historical-retail', scope='SOURCE-only',
                runtime_observations=0, native_observations=0, model_credit=0,
                later_registers=[], successor_queue=['2.4.0', '2.4.2', '3.0.2', '3.0.3', '3.0.8'],
                inventory_rows=inventory, signature_ledger=signatures,
                prose_ledger=prose, source_rows=source_rows, headers=headers,
                header_counts=[], links=links,
                navigation=dict(next='2.4.0', prev='2.2.0', expanded=False))


def validate_ledger(raw, ledger):
    expected = derive(raw)
    assert ledger == expected, 'literal source accounting or invented credit'
    return summary(ledger)


def summary(ledger):
    entries = ledger['inventory_rows']
    return dict(source_rows=len(ledger['source_rows']),
                statuses=dict(Counter(r['status'] for r in ledger['source_rows'])),
                inventory_occurrences=len(entries),
                kinds=dict(Counter(r['section'] for r in entries)),
                directions=dict(Counter(r['direction'] for r in entries)),
                signatures=len(ledger['signature_ledger']),
                literal_call_fragments=sum(r['fragment'] is not None
                                           for r in ledger['signature_ledger']),
                complete_signatures=0, prose_limits=len(ledger['prose_ledger']),
                headers=len(ledger['headers']), numerical_headers=0,
                unexpanded_links=len(ledger['links']), cvar_occurrences=0,
                runtime_observations=0, native_observations=0, model_credit=0)


def frozen_inputs():
    response = (EVIDENCE / 'source-response.json').read_bytes()
    raw = (SOURCE / '2.3.0-api-changes.wikitext').read_bytes()
    pin = json.loads((EVIDENCE / 'source-pin.json').read_bytes())
    manifest = json.loads((EVIDENCE / 'frozen-manifest.json').read_bytes())
    registry_bytes = (EVIDENCE / 'frozen-registry.json').read_bytes()
    registry = json.loads(registry_bytes)
    assert digest(registry_bytes) == manifest['registry_sha256'] == (
        'e357f60af2c745b7797ab8f9e7ac151345cddb6bf43de7786ee25785ee92e91c'), 'registry hash'
    assert len(registry['pages']) == 101 and registry['pages'][-1]['version'] == '1.0.0', 'registry endpoint'
    assert pin == next(p for p in manifest['pages'] if p['version'] == '2.3.0'), 'manifest pin'
    identity = dict(title='Patch 2.3.0/API changes', version='2.3.0', pageid=44655,
                    revid=6055877, timestamp='2024-06-04T05:03:50Z')
    assert {k: pin[k] for k in identity} == identity, 'frozen identity'
    assert next(p for p in registry['pages'] if p['version'] == '2.3.0') == identity, 'registry identity'
    page = json.loads(response)['query']['pages']['44655']
    revision, = page['revisions']
    assert page['pageid'] == pin['pageid'] and page['title'] == pin['title'], 'response page'
    assert revision['revid'] == pin['revid'] and revision['timestamp'] == pin['timestamp'], 'response revision'
    assert revision['slots']['main']['*'].encode() == raw, 'response body'
    assert digest(raw) == pin['wikitext_sha256'] == (
        '2d87459c71de044511f67565c26c981385f01477d8a4195eca593cf20b4cf283'), 'body hash'
    assert digest(response) == pin['response_sha256'] == (
        '7052cd94b09d2c1a2fca15d16973b2962f27218a5170c2759069bc38cd4c9dc8'), 'response hash'
    assert len(raw) == pin['wikitext_bytes'] == 12134, 'body bytes'
    return raw.decode()


def extracted_text(raw):
    spec = importlib.util.spec_from_file_location(
        'historical_extract', EVIDENCE / 'historical-extract_patch_non_inventory.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.extract_text(raw, canonical_patch_navigation=True)


def register_for(raw, ledger):
    return dict(schema='patch-api-wikitext-register/v1', patch='2.3.0',
                source=dict(path='data/patch-api/sources/2.3.0-api-changes.wikitext',
                            revid=6055877, sha256=digest(raw.encode())),
                client_line='retail', scope='SOURCE-only',
                entries=ledger['inventory_rows'], header_counts=ledger['header_counts'])


if __name__ == '__main__':
    raw = frozen_inputs()
    ledger = derive(raw)
    write_json(SOURCE / '2.3.0-page-coverage.json', ledger)
    write_json(SOURCE / '2.3.0-wikitext-register.json', register_for(raw, ledger))
    (SOURCE / '2.3.0-api-changes.txt').write_text(extracted_text(raw))
    print(json.dumps(summary(ledger), sort_keys=True))
