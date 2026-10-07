#!/usr/bin/env python3
"""Validate retained 10.2.6 accounting and portable local proof, not native parity."""
import hashlib
import json
import runpy
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
SESSION = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'
AUDIT_CWD = '/home/osso/.worktrees/wow-ui-sim-p1026-page'
PATCHES = ('10.2.6', '10.2.7', '11.0.0', '11.0.2', '11.0.5', '11.0.7',
           '11.1.0', '11.1.5', '11.1.7', '11.2.0', '11.2.5', '11.2.7',
           '12.0.0', '12.0.1', '12.0.5', '12.0.7', '12.1.0')


def load(path):
    return json.loads(path.read_text())


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def validate_sources():
    register = load(SOURCES / '10.2.6-wikitext-register.json')
    provenance = load(SOURCES / '10.2.6-api-changes.provenance.json')
    coverage = load(SOURCES / '10.2.6-page-coverage.json')
    assert provenance['pageid'] == 582132
    assert provenance['wikitext']['revid'] == register['source']['revid'] == 6268742
    assert provenance['wikitext']['revision_timestamp'] == '2025-03-20T19:36:42Z'
    assert digest(SOURCES / '10.2.6-api-changes.wikitext') == provenance['wikitext']['sha256'] == register['source']['sha256']
    assert digest(SOURCES / '10.2.6-wikitext-register.json') == coverage['source_sha256']
    assert digest(SOURCES / '10.2.6-api-changes.txt') == coverage['non_inventory_source']['sha256']
    assert len(register['entries']) == 220
    assert len(register['header_counts']) == 6
    assert all(row['header_count'] == row['parsed_count'] for row in register['header_counts'])
    assert Counter(row['direction'] for row in register['entries']) == {'added': 136, 'removed': 84}
    extractor = runpy.run_path(str(ROOT / 'tools/extract_patch_non_inventory.py'))
    text = (SOURCES / '10.2.6-api-changes.txt').read_text()
    assert extractor['extract_text']((SOURCES / '10.2.6-api-changes.wikitext').read_text()) == text
    extract = extractor['seed_rows'](text, '10.2.6')
    scout = load(SESSION / 'p1026-extract-scout.json')
    assert len(extract) == len(scout) == 138
    ids = [row['source_id'] for row in coverage['source_rows']]
    assert len(ids) == len(set(ids)) == 358
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
        'partial-development-green': 102, 'bounded-coverage': 97,
        'audit-pending': 142, 'metadata-only': 17}
    reproduction = load(SESSION / 'p1026-register-reproduction.json')
    assert [row['patch'] for row in reproduction] == list(PATCHES)
    assert all(row['byte_identical'] and row['exit_code'] == 0 for row in reproduction)
    preserved = load(SESSION / 'p1026-preserved-inputs.json')
    assert len(preserved) == 104
    assert all(digest(ROOT / path) == sha for path, sha in preserved.items())
    return register, ledger


def validate_sweeps(register, ledger):
    results = load(SESSION / 'p1026-sweep-result.json')
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
    mismatches = {key for key, value in results.items() if value['observed']['default_mismatch']}
    assert mismatches == {'wt-cvars-gxMTDecals-249'}
    assert ledger['wt-cvars-gxMTDecals-249']['status'] == 'audit-pending'
    table = load(SESSION / 'p1026-sweep-table.json')
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
    negative = {key for key, value in load(SESSION / 'p1026-negative-result.json').items() if not value['ok']}
    assert len(baseline) == 20
    assert negative - baseline == {'wt-global-api-C_CurrencyInfo.GetCoinIcon-24'}
    assert not baseline - negative
    altered = load(SESSION / 'p1026-negative-register.json')
    assert len(register['entries']) == len(altered['entries']) == 220
    different = [(a, b) for a, b in zip(register['entries'], altered['entries']) if a != b]
    assert len(different) == 1
    original, changed = different[0]
    assert {**original, 'direction': 'removed'} == changed
    review = load(SESSION / 'p1026-gap-review.json')
    assert Counter(row['decision'] for row in review) == {'closed retirement': 3, 'retained gap': 20}
    discovery = load(SESSION / 'p1026-discovery-result.json')
    assert {row['source_id'] for row in review} == {key for key, value in discovery.items() if not value['ok']}
    assert {row['source_id'] for row in review if row['decision'] == 'retained gap'} == baseline
    for row in review:
        assert row['initial_observed'] == discovery[row['source_id']]['observed']
        assert row['final_observed'] == results[row['source_id']]['observed']
        assert row['reason']


def validate_proof():
    proof = load(SESSION / 'p1026-proof.json')
    successful = {row['scope']: row for row in proof if not row['invalidated'] and row['exit_code'] == 0}
    for patch in PATCHES:
        row = successful[f'isolated {patch} publication sweep']
        assert any('1 passed; 0 failed' in line for line in row['summary'])
    for scope in ('three retirements GREEN', 'cached Game UI retirement and profanityFilter retention',
                  'formatting final Rust scope', 'Mists all tests compile; classic gates preserved',
                  'separate current retail binary build', 'bounded current retail startup JSON clean',
                  'existing eighteen parser/extractor fixtures; PYTHONPATH corrected'):
        assert scope in successful, f'missing successful proof scope: {scope}'
    assert load(SESSION / 'p1026-startup-stdout.json') == []
    assert (SESSION / 'p1026-startup.log').read_text().splitlines()[0] == '[]'
    mists = successful['Mists all tests compile; classic gates preserved']
    warnings = [line for line in mists['summary'] if line.startswith('warning:')]
    assert len(warnings) == 7
    assert all('iced-wgpu-patched/Cargo.toml' in line or '`iced_wgpu` (manifest)' in line for line in warnings)
    negative = next(row for row in proof if row['scope'] == 'one-row negative control')
    assert negative['exit_code'] == 101 and not negative['invalidated']
    for row in proof:
        assert row['cwd'] == AUDIT_CWD
        assert row['environment']['CARGO_TARGET_DIR'] == AUDIT_CWD + '/target'
        assert row['revision']
        assert digest(ROOT / row['log']) == row['log_sha256']
    scopes = load(SESSION / 'p1026-proof-scopes.json')
    assert scopes['revision'] == '1ef2ed7b4'
    assert all(digest(ROOT / path) == sha for path, sha in scopes['paths'].items())


def main():
    register, ledger = validate_sources()
    validate_sweeps(register, ledger)
    validate_proof()
    print(json.dumps({'result': 'PASS', 'inventory_rows': 220, 'extract_rows': 138,
                      'unique_ids': 358, 'isolated_sweeps': 17, 'runtime_closures': 3,
                      'retained_publication_gaps': 20, 'default_mismatches': 1,
                      'preserved_inputs': 104, 'proof': 'local targeted only; no native parity'}))


if __name__ == '__main__':
    main()
