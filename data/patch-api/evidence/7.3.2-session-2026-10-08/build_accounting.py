"""Account for both API-link occurrences and every retained extract identity."""
import hashlib
import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'


def read(path):
    return json.loads(path.read_text())


def dump(path, value):
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n')


def main():
    spec = importlib.util.spec_from_file_location('extractor', ROOT / 'tools/extract_patch_non_inventory.py')
    extractor = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(extractor)
    register_path = SOURCES / '7.3.2-wikitext-register.json'
    register = read(register_path)
    results = read(EVIDENCE / 'patch_7_3_2_publication_sweep-results.json')
    protection = read(EVIDENCE / 'p732-protection-green.json')
    text_path = SOURCES / '7.3.2-api-changes.txt'
    text = text_path.read_text()
    raw = (SOURCES / '7.3.2-api-changes.wikitext').read_text().splitlines()
    note = ('Addon-tainted calls fail before mutating login/exit state; secure calls retain existing '
            'state transitions. Bare and unmodified cached UI proof. Native notification/error wording, '
            'hardware-event policy and logout countdown/cancellation remain unverified; no historical parity claim.')
    capabilities = [{'kind': 'publication', 'proof': 'tests/patch_7_3_2_publication_sweep.rs'},
                    {'kind': 'insecure-session-action-block', 'proof': 'tests/patch_7_3_2_session_protection.rs'}]
    rows = []
    for entry in register['entries']:
        assert results[entry['id']]['ok'] and protection[entry['symbol']]['ok']
        rows.append({'source_id': entry['id'], 'source': entry['symbol'],
                     'status': 'bounded-coverage', 'capabilities': capabilities, 'note': note})
    scout = []
    for row in extractor.seed_rows(text, '7.3.2'):
        number = int(row['source_id'].rsplit('-', 1)[-1])
        literal = text.splitlines()[number - 1]
        statement = literal.startswith('*Logout and Quit')
        reason = note if statement else 'Source/heading/unexpanded reference context only; no runtime credit.'
        row.update(status='bounded-coverage' if statement else 'metadata-only',
                   capabilities=capabilities if statement else [], note=reason)
        rows.append(row)
        scout.append({'source_id': row['source_id'], 'extract_line': number, 'literal': literal,
                      'wikitext_line': number, 'raw_literal': raw[number - 1],
                      'status': row['status'], 'reason': reason})
    coverage = {'schema': 'patch-page-coverage/v1', 'patch': '7.3.2',
                'source_sha256': hashlib.sha256(register_path.read_bytes()).hexdigest(),
                'non_inventory_source': {'path': str(text_path.relative_to(ROOT)),
                                         'sha256': hashlib.sha256(text_path.read_bytes()).hexdigest()},
                'proof_policy': 'Current retail publication plus bounded taint-driven session state protection; not native/historical parity.',
                'source_rows': rows}
    dump(SOURCES / '7.3.2-page-coverage.json', coverage)
    dump(EVIDENCE / 'p732-extract-scout.json', scout)
    dump(EVIDENCE / 'p732-gap-review.json', {
        'publication_gaps': [], 'unmodeled_source_contracts': [],
        'remaining_native_boundaries': [
            {'contract': 'Native block notification and error text',
             'reason': 'Pinned page says protected but specifies neither ADDON_ACTION_BLOCKED/ADDON_ACTION_FORBIDDEN payload nor error text; no native capture. Explicit simulator error is not native notification parity.'},
            {'contract': 'Hardware-event policy',
             'reason': 'No native hardware-event entitlement/gating capture. Stack taint establishes insecure-call rejection, not exact native input-event policy.'},
            {'contract': 'Logout countdown and cancellation',
             'reason': 'Existing is_logged_in transition has no pending deadline/rest-area policy. CancelLogout remains an existing temporary no-op; protection does not claim a countdown model.'},
            {'contract': 'Historical/secret/restricted execution parity',
             'reason': 'Current retail is the audit target. No 7.3.2 client or native secret/restricted-context capture; a bounded retail taint guard is not those contracts.'},
        ]})
    summaries = []
    for path in sorted(EVIDENCE.glob('patch_*_publication_sweep-results.json')):
        observations = read(path)
        summaries.append({'file': path.name, 'rows': len(observations),
                          'ok': sum(row['ok'] for row in observations.values()),
                          'gaps': sum(not row['ok'] for row in observations.values())})
    dump(EVIDENCE / 'p732-sweep-summary.json', summaries)
    print(json.dumps({'inventory': len(register['entries']), 'extract': len(scout), 'ledger': len(rows)}))


if __name__ == '__main__':
    main()
