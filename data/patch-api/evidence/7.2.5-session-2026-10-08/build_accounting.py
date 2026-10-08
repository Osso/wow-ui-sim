"""Account for all raw template identities and every retained plaintext row."""
import hashlib
import importlib.util
import json
from pathlib import Path
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'


def read(path):
    return json.loads(path.read_text())


def dump(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')


def main():
    spec = importlib.util.spec_from_file_location('extractor', ROOT / 'tools/extract_patch_non_inventory.py')
    extractor = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(extractor)
    register_path = SOURCES / '7.2.5-wikitext-register.json'
    register = read(register_path)
    results = read(HERE / 'patch_7_2_5_publication_sweep-results.json')
    text_path = SOURCES / '7.2.5-api-changes.txt'
    text = text_path.read_text()
    raw = (SOURCES / '7.2.5-api-changes.wikitext').read_text().splitlines()
    reasons = {
        'C_UI.Reload': 'Raw-absent. Existing ReloadUI and GUI reload only dispatch notifications; no VM teardown/recreation or SavedVariables reload model. Event-only alias would disguise missing lifecycle; no shim added.',
        'C_Unit': 'Raw-absent namespace. Source links the unqualified UnitIsOwnerOrControllerOfUnit, already modeled/published, but names no C_Unit member. No native namespace capture; do not invent an empty table or qualified alias.',
        'ReloadUI': 'Historical rename suggests removal, but thirteen current retail consumer lines prohibit retirement. Existing notification behavior retained; no later master/p730 register re-addition, no historical absence claim.',
    }
    publication = {'kind': 'publication', 'proof': 'tests/patch_7_2_5_publication_sweep.rs'}
    modeled = {'kind': 'garrison-tree-context-and-catalog', 'proof': 'tests/patch_7_2_5_garrison_trees.rs'}
    rows, gaps = [], []
    for entry in register['entries']:
        is_gap = not results[entry['id']]['ok']
        note = reasons[entry['symbol']] if is_gap else 'Raw publication and lookup after unmodified cached retail startup only; not native signatures/outputs/security/historical parity.'
        capabilities = [] if is_gap else [publication]
        if entry['symbol'].startswith('C_Garrison.'):
            note = 'Real retail selected-tree context and type/class catalog query, detached output, nullable friendship faction and no-result missing pair; bounded bare/cached tests. Native catalog order, Legion data and secret policy remain unverified.'
            capabilities = [publication, modeled]
        row = {'source_id': entry['id'], 'source': entry['symbol'],
               'status': 'audit-pending' if is_gap else 'bounded-coverage',
               'capabilities': capabilities, 'note': note}
        rows.append(row)
        if is_gap:
            gaps.append({'source_id': entry['id'], 'symbol': entry['symbol'], 'reason': note,
                         'observation': results[entry['id']]})
    scout, pending = [], []
    for row in extractor.seed_rows(text, '7.2.5'):
        number = int(row['source_id'].rsplit('-', 1)[-1])
        literal = text.splitlines()[number - 1]
        if row['status'] == 'metadata-only':
            status, capabilities, reason = 'metadata-only', [], 'Source title/heading only; no runtime credit.'
        elif literal.startswith('* Events added'):
            status, capabilities, reason = 'metadata-only', [], 'Documentation editorial statement; no event name/payload list is supplied and linked documentation is not expanded.'
        elif 'various functions' in literal or 'several added functions' in literal:
            status, capabilities, reason = 'audit-pending', [], 'Namespace publication is separately bounded, but page enumerates no member names/signatures or behavior. Do not invent commentator or transmog additions from an unexpanded linked page.'
        elif literal.startswith('* C_Unit'):
            status, capabilities, reason = 'audit-pending', [], reasons['C_Unit']
        elif literal.startswith('* ReloadUI'):
            status, capabilities, reason = 'audit-pending', [], reasons['C_UI.Reload'] + ' ' + reasons['ReloadUI']
        elif literal.startswith('* C_Garrison'):
            status, capabilities, reason = 'bounded-coverage', [publication, modeled], 'Three named queries modeled with real catalog/context inputs; no native progression, secret-policy, catalog-order or historical Legion capture claim.'
        else:
            status, capabilities, reason = 'bounded-coverage', [publication], 'Named namespace/member publication is separately observed; summary supplies no additional behavior contract. Existing modeled chat/action/LFG surfaces remain bounded, not native parity.'
        row.update(status=status, capabilities=capabilities, note=reason)
        rows.append(row)
        scout.append({'source_id': row['source_id'], 'extract_line': number,
                      'literal': literal, 'wikitext_line': number, 'raw_literal': raw[number - 1],
                      'status': status, 'reason': reason})
        if status == 'audit-pending':
            pending.append({'source_id': row['source_id'], 'literal': literal, 'reason': reason})
    coverage = {'schema': 'patch-page-coverage/v1', 'patch': '7.2.5',
                'source_sha256': hashlib.sha256(register_path.read_bytes()).hexdigest(),
                'non_inventory_source': {'path': str(text_path.relative_to(ROOT)),
                                         'sha256': hashlib.sha256(text_path.read_bytes()).hexdigest()},
                'proof_policy': 'Current retail publication plus bounded server-input garrison tree queries, not historical/native parity.',
                'source_rows': rows}
    dump(SOURCES / '7.2.5-page-coverage.json', coverage)
    dump(HERE / 'p725-extract-scout.json', scout)
    dump(HERE / 'p725-gap-review.json', {
        'publication_gaps': gaps, 'unmodeled_source_contracts': pending,
        'remaining_native_boundaries': [
            'Tree catalog order and Legion/server contents not captured; injected fixtures are not native data.',
            'Garrison secret-argument, numeric coercion and protected-context policy not verified.',
            'Existing chat-bubble metadata tables are not native Frames or forbidden-filter behavior.',
            'Unnamed commentator/transmog members and event documentation cannot be reconstructed from this page.',
            'Reload lifecycle and ambiguous C_Unit namespace retained, no aliases/shims or vendor changes.',
        ]})
    summaries = []
    for path in sorted(HERE.glob('patch_*_publication_sweep-results.json')):
        observations = read(path)
        summaries.append({'test': path.name.removesuffix('-results.json'), 'file': path.name,
                          'rows': len(observations), 'ok': sum(row['ok'] for row in observations.values()),
                          'gaps': sum(not row['ok'] for row in observations.values())})
    dump(HERE / 'p725-sweep-summary.json', summaries)
    print(json.dumps({'inventory': len(register['entries']), 'extract': len(scout),
                      'ledger': len(rows), 'gaps': len(gaps), 'pending_prose': len(pending)}))


if __name__ == '__main__':
    main()
