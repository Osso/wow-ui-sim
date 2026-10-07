#!/usr/bin/env python3
"""Validate retained 10.2.7 source accounting and portable local proof, not native parity."""
import hashlib
import json
import runpy
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
SESSION = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'
AUDIT_CWD = '/home/osso/.worktrees/wow-ui-sim-p1027-page'
PATCHES = ('10.2.7', '11.0.0', '11.0.2', '11.0.5', '11.0.7', '11.1.0',
           '11.1.5', '11.1.7', '11.2.0', '11.2.5', '11.2.7', '12.0.0',
           '12.0.1', '12.0.5', '12.0.7', '12.1.0')


def load(path):
    return json.loads(path.read_text())


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def validate_sources():
    register = load(SOURCES / '10.2.7-wikitext-register.json')
    provenance = load(SOURCES / '10.2.7-api-changes.provenance.json')
    coverage = load(SOURCES / '10.2.7-page-coverage.json')
    assert provenance['pageid'] == 584467
    assert provenance['wikitext']['revid'] == register['source']['revid'] == 6268738
    assert provenance['wikitext']['revision_timestamp'] == '2025-03-20T19:32:28Z'
    assert digest(SOURCES / '10.2.7-api-changes.wikitext') == provenance['wikitext']['sha256'] == register['source']['sha256']
    assert digest(SOURCES / '10.2.7-wikitext-register.json') == coverage['source_sha256']
    assert digest(SOURCES / '10.2.7-api-changes.txt') == coverage['non_inventory_source']['sha256']
    assert len(register['entries']) == 104
    assert len(register['header_counts']) == 8
    assert all(row['header_count'] == row['parsed_count'] for row in register['header_counts'])
    assert Counter(row['direction'] for row in register['entries']) == {'added': 69, 'removed': 16, 'changed': 19}
    extractor = runpy.run_path(str(ROOT / 'tools/extract_patch_non_inventory.py'))
    text = (SOURCES / '10.2.7-api-changes.txt').read_text()
    assert extractor['extract_text']((SOURCES / '10.2.7-api-changes.wikitext').read_text()) == text
    extract = extractor['seed_rows'](text, '10.2.7')
    scout = load(SESSION / 'p1027-extract-scout.json')
    assert len(extract) == len(scout) == 83
    ids = [row['source_id'] for row in coverage['source_rows']]
    assert len(ids) == len(set(ids)) == 187
    assert set(ids) == {row['id'] for row in register['entries']} | {row['source_id'] for row in extract}
    assert {row['source_id'] for row in scout} == {row['source_id'] for row in extract}
    ledger = {row['source_id']: row for row in coverage['source_rows']}
    for row in scout:
        number = int(row['source_id'].rsplit('-', 1)[-1])
        assert row['statement'] == text.splitlines()[number - 1].strip()
        assert row['proof'] == ledger[row['source_id']]['status']
        assert row['decision'] == ledger[row['source_id']]['note']
        assert not ledger[row['source_id']]['capabilities']
    assert Counter(row['status'] for row in coverage['source_rows']) == {
        'partial-development-green': 49, 'bounded-coverage': 19,
        'audit-pending': 107, 'metadata-only': 12}
    reproduction = load(SESSION / 'p1027-register-reproduction.json')
    assert {row['patch'] for row in reproduction} == set(PATCHES)
    assert all(row['byte_identical'] and row['exit_code'] == 0 for row in reproduction)
    preserved = load(SESSION / 'p1027-preserved-inputs.json')
    assert len(preserved) == 98
    assert all(digest(ROOT / row['path']) == row['baseline_sha256'] for row in preserved)
    return register, ledger


def validate_sweeps(register, ledger):
    results = load(SESSION / 'p1027-sweep-result.json')
    assert set(results) == {row['id'] for row in register['entries']}
    latest = {}
    for patch in PATCHES[1:]:
        for row in load(SOURCES / f'{patch}-wikitext-register.json')['entries']:
            if row['direction'] != 'changed':
                latest[row['symbol']] = row
    for row in register['entries']:
        expected = results[row['id']]['expected']
        newer = latest.get(row['symbol'])
        removed = (newer or row)['direction'] == 'removed'
        assert expected['publication'] == ('absent' if removed else 'published')
        reversed_direction = newer and removed != (row['direction'] == 'removed')
        assert expected['superseded_by'] == (newer['id'] if reversed_direction else None)
        assert bool(ledger[row['id']]['capabilities']) == results[row['id']]['ok']
    table = load(SESSION / 'p1027-sweep-table.json')
    assert [row['patch'] for row in table] == list(PATCHES)
    for patch, row in zip(PATCHES, table):
        result = load(SESSION / ('p' + patch.replace('.', '') + '-sweep-result.json'))
        known = load(ROOT / 'tests/data' / ('patch_' + patch.replace('.', '_') + '_sweep_known_gaps.json'))
        assert {key for key, value in result.items() if not value['ok']} == set(known)
        assert len(known) == len(set(known)) == row['gaps']
        assert len(result) == row['rows']
        assert sum(value['ok'] for value in result.values()) == row['ok']
        assert row['exit_code'] == 0
    baseline = {key for key, value in results.items() if not value['ok']}
    negative = {key for key, value in load(SESSION / 'p1027-negative-result.json').items() if not value['ok']}
    assert len(baseline) == 36
    assert negative - baseline == {'wt-global-api-C_StableInfo.ClosePetStables-77'}
    assert not baseline - negative
    altered = load(SESSION / 'p1027-negative-register.json')
    assert len(register['entries']) == len(altered['entries']) == 104
    different = [(a, b) for a, b in zip(register['entries'], altered['entries']) if a != b]
    assert len(different) == 1
    original, changed = different[0]
    assert {**original, 'direction': 'removed'} == changed
    review = load(SESSION / 'p1027-gap-review.json')
    assert Counter(row['decision'] for row in review) == {'closed': 3, 'probe corrected': 4, 'retained gap': 36}
    discovery = load(SESSION / 'p1027-discovery-result.json')
    assert {row['source_id'] for row in review} == {key for key, value in discovery.items() if not value['ok']}
    assert {row['source_id'] for row in review if row['decision'] == 'retained gap'} == baseline


def validate_proof():
    proof = load(SESSION / 'p1027-proof.json')
    successful = {row['scope']: row for row in proof if not row['invalidated'] and row['exit_code'] == 0}
    for patch in PATCHES:
        row = successful[f'isolated publication sweep {patch}']
        assert any('1 passed; 0 failed' in line for line in row['summary'])
    for scope in ('fixes-green', 'prefork-stable', 'pvp-successor', 'parser/extractor fixtures; unchanged tooling',
                  'fmt-check', 'mists-check', 'retail-build', 'startup'):
        assert scope in successful, f'missing successful proof scope: {scope}'
    assert successful['startup']['stdout'].strip() == '[]'
    assert load(SESSION / 'p1027-startup-stdout.json') == []
    mists = successful['mists-check']
    warnings = [line for line in mists['summary'] if line.startswith('warning:')]
    assert len(warnings) == 7
    assert all('iced-wgpu-patched/Cargo.toml' in line or '`iced_wgpu` (manifest)' in line for line in warnings)
    negative = next(row for row in proof if row['scope'] == 'negative')
    assert negative['exit_code'] == 101 and not negative['invalidated']
    for row in proof:
        assert row['cwd'] == AUDIT_CWD
        if row.get('environment', {}).get('CARGO_TARGET_DIR'):
            assert row['environment']['CARGO_TARGET_DIR'] == AUDIT_CWD + '/target'
        assert row['revision']


def main():
    register, ledger = validate_sources()
    validate_sweeps(register, ledger)
    validate_proof()
    print(json.dumps({'result': 'PASS', 'inventory_rows': 104, 'extract_rows': 83,
                      'unique_ids': 187, 'isolated_sweeps': 16, 'runtime_closures': 3,
                      'probe_corrections': 4, 'retained_gaps': 36,
                      'preserved_inputs': 98, 'proof': 'local targeted only; no native parity'}))


if __name__ == '__main__':
    main()
