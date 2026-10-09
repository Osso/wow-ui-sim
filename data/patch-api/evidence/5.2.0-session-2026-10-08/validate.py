"""Read-only historical 5.2.0 proof, portable to unrelated later audits."""
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT / 'tools'))
from patch_audit_validation import historical_registers


def read(name):
    return json.loads((HERE / name).read_text())


def digest(data):
    return hashlib.sha256(data).hexdigest()


def blob(revision, name):
    return subprocess.check_output(['git', 'show', f'{revision}:{name}'], cwd=ROOT)


def validate():
    final = read('finalization.json')
    for name, expected in final['own_artifacts'].items():
        assert digest((HERE / name).read_bytes()) == expected, 'own evidence drift: ' + name
    for name, expected in final['shared_inputs'].items():
        assert digest(blob(final['revision'], name)) == expected, 'pinned input drift: ' + name
    for name, expected in final['base_sources'].items():
        assert digest(blob(final['base_revision'], name)) == expected
        assert digest(blob(final['revision'], name)) == expected, 'earlier source changed: ' + name
    historical = historical_registers(ROOT, final['reproduction_revision'])
    reproduced = read('reproduction.json')
    assert {p.relative_to(ROOT).as_posix() for p in historical} == {r['path'] for r in reproduced}
    assert all(r['register_identical'] for r in reproduced)
    assert {r['patch'] for r in reproduced if not r['extract_identical']} == {'12.0.5', '12.0.7', '12.1.0'}
    for receipt_name, expected_exit in final['receipts'].items():
        proof = read(receipt_name)
        assert proof['exit'] == expected_exit and not proof['invalidated'], receipt_name
        assert digest((HERE / proof['log']).read_bytes()) == proof['log_sha256']
        subprocess.check_call(['git', 'cat-file', '-e', proof['revision'] + '^{commit}'], cwd=ROOT)
    register = json.loads(blob(final['revision'], 'data/patch-api/sources/5.2.0-wikitext-register.json'))
    ledger = json.loads(blob(final['revision'], 'data/patch-api/sources/5.2.0-page-coverage.json'))
    fixture = json.loads(blob(final['revision'], 'tests/data/patch_5_2_0_sweep_known_gaps.json'))
    observed = read('patch_5_2_0_publication_sweep-results.json')
    assert set(observed) == {r['id'] for r in register['entries']}
    assert {key for key, row in observed.items() if not row['ok']} == set(fixture)
    assert len({r['source_id'] for r in ledger['source_rows']}) == len(ledger['source_rows'])
    assert {r['source_id'] for r in ledger['source_rows'] if r['source_id'].startswith('diff-wt-')} == set(observed)
    assert all(r['note'] for r in ledger['source_rows'])
    assert all(c['header_count'] == c['parsed_count'] for c in register['header_counts'])
    negative = read('negative-results.json')
    assert {k for k, v in negative.items() if not v['ok']} == set(fixture) | {final['negative_id']}
    scans = read('retirement-scans.json')
    assert {r['source_id'] for r in scans} == {r['id'] for r in register['entries'] if r['direction'] == 'removed'}
    for scan in scans:
        assert '-w' in scan['tool'] and 'untruncated' in scan['tool']
        for label in ('cache', 'callers'):
            record = scan[label]
            data = (HERE / record['path']).read_bytes()
            assert digest(data) == record['sha256']
            assert len(data.decode().splitlines()) == record['matching_lines']
    later = read('later-register-scan.json')
    for row in later['registers']:
        assert digest(blob(row['revision'], row['register'])) == row['sha256']
        data = json.loads(blob(row['revision'], row['register']))
        assert data.get('client_line', 'retail') == 'retail'
        assert not row['readditions']
    baseline = read('wiki-baseline.json')
    for name, count in baseline.items():
        assert len(blob(final['revision'], name).decode().splitlines()) >= count
    mists = (HERE / 'mists.txt').read_text()
    warnings = re.findall(r'^warning: (.*)$', mists, flags=re.M)
    assert all(message.startswith(('iced-wgpu-patched/', '`iced_wgpu` (manifest)')) for message in warnings), warnings
    assert final['runtime_source_changed'] is False
    print(json.dumps({'status': 'PASS', 'inventory': len(observed), 'publication_gaps': len(fixture),
                      'accounted_ids': len(ledger['source_rows']), 'retirement_scans': len(scans),
                      'registers': len(reproduced), 'retirements': 0}))


if __name__ == '__main__':
    validate()
