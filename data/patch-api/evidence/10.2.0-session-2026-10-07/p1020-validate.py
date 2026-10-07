#!/usr/bin/env python3
"""Validate retained 10.2.0 source IDs, chronological sweeps and proof artifacts."""
import hashlib
import importlib.util
import json
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'


def read_json(path):
    return json.loads(path.read_text())


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load_extractor():
    spec = importlib.util.spec_from_file_location(
        'extractor', ROOT / 'tools/extract_patch_non_inventory.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def validate_sources():
    register_path = SOURCES / '10.2.0-wikitext-register.json'
    register = read_json(register_path)
    provenance = read_json(SOURCES / '10.2.0-api-changes.provenance.json')
    ledger = read_json(SOURCES / '10.2.0-page-coverage.json')
    raw = SOURCES / '10.2.0-api-changes.wikitext'
    text_path = SOURCES / '10.2.0-api-changes.txt'
    assert provenance['pageid'] == 12983
    assert provenance['wikitext']['revid'] == 6473470
    assert provenance['wikitext']['sha256'] == register['source']['sha256'] == sha256(raw)
    assert ledger['source_sha256'] == sha256(register_path)
    assert ledger['non_inventory_source']['sha256'] == sha256(text_path)
    extractor = load_extractor()
    assert extractor.extract_text(raw.read_text()) == text_path.read_text()
    extract_ids = {row['source_id'] for row in extractor.seed_rows(text_path.read_text(), '10.2.0')}
    inventory_ids = {entry['id'] for entry in register['entries']}
    ids = [row['source_id'] for row in ledger['source_rows']]
    assert len(register['entries']) == len(inventory_ids) == 150
    assert len(extract_ids) == 134
    assert len(ids) == len(set(ids)) == 284
    assert set(ids) == inventory_ids | extract_ids
    assert all(c['header_count'] == c['parsed_count'] for c in register['header_counts'])
    assert Counter(e['direction'] for e in register['entries']) == {'added': 79, 'removed': 64, 'changed': 7}
    scout = read_json(EVIDENCE / 'p1020-extract-scout.json')
    assert len(scout) == 134 and {r['source_id'] for r in scout} == extract_ids
    assert Counter(r['proof'] for r in scout) == {'audit-pending': 116, 'metadata-only': 18}
    ledger_rows = {r['source_id']: r for r in ledger['source_rows']}
    for row in scout:
        assert ledger_rows[row['source_id']]['status'] == row['proof']
        assert not ledger_rows[row['source_id']]['capabilities']
        number = int(row['source_id'].rsplit('-', 1)[1])
        assert row['statement'] == text_path.read_text().splitlines()[number - 1].strip()
    assert Counter(r['status'] for r in ledger['source_rows']) == {
        'audit-pending': 151, 'bounded-coverage': 64,
        'partial-development-green': 51, 'metadata-only': 18}
    return register


def validate_sweeps():
    table = read_json(EVIDENCE / 'p1020-sweep-table.json')
    patches = sorted(table, key=lambda r: tuple(map(int, r['patch'].split('.'))))
    assert len(patches) == 18
    for index, row in enumerate(patches):
        patch = row['patch']
        tag = 'p' + patch.replace('.', '')
        register = read_json(SOURCES / f'{patch}-wikitext-register.json')
        results = read_json(EVIDENCE / f'{tag}-sweep-final-result.json')
        assert set(results) == {e['id'] for e in register['entries']}
        gaps = {key for key, value in results.items() if not value['ok']}
        fixture = read_json(ROOT / 'tests/data' / f'patch_{patch.replace(".", "_")}_sweep_known_gaps.json')
        assert gaps == set(fixture)
        assert row['rows'] == len(results) and row['gaps'] == len(gaps)
        assert row['ok'] == len(results) - len(gaps) and row['exit_code'] == 0
        latest = {}
        for later in patches[index + 1:]:
            entries = read_json(SOURCES / f'{later["patch"]}-wikitext-register.json')['entries']
            latest.update({e['symbol']: e for e in entries if e['direction'] != 'changed'})
        for entry in register['entries']:
            newer = latest.get(entry['symbol'])
            removed = entry['direction'] == 'removed'
            superseded = None
            if newer and (newer['direction'] == 'removed') != removed:
                removed = not removed
                superseded = newer['id']
            expected = results[entry['id']]['expected']
            assert expected['publication'] == ('absent' if removed else 'published')
            assert expected['superseded_by'] == superseded
    final = read_json(EVIDENCE / 'p1020-sweep-final-result.json')
    initial = read_json(EVIDENCE / 'p1020-discovery-result.json')
    review = read_json(EVIDENCE / 'p1020-gap-review.json')
    assert {r['source_id'] for r in review} == {k for k, v in initial.items() if not v['ok']}
    assert Counter(r['decision'] for r in review) == {'closed bounded gap': 10, 'retained gap': 30}
    for row in review:
        assert row['reason']
        assert row['final_observed'] == final[row['source_id']]['observed']
        assert (row['decision'] == 'closed bounded gap') == final[row['source_id']]['ok']
    mismatches = [v for v in final.values() if v['observed']['default_mismatch']]
    assert len(mismatches) == 5
    assert all(float(v['expected']['page_default']) == float(v['observed']['default']) for v in mismatches)
    negative = read_json(EVIDENCE / 'p1020-negative-final-control.json')
    assert negative['before'] == 30 and negative['after'] == 31
    assert negative['new'] == ['wt-widgets-TextureBase:GetTextureSliceMargins-182']
    assert not negative['resolved'] and negative['exit_code'] == 101


def validate_artifacts():
    for path, digest in read_json(EVIDENCE / 'p1020-preserved-inputs.json').items():
        assert sha256(ROOT / path) == digest, path
    reproduced = {r['patch']: r for r in read_json(EVIDENCE / 'p1020-register-reproduction.json')}
    assert len(reproduced) == 18
    assert all(r['byte_identical'] and r['exit_code'] == 0 for r in reproduced.values())
    proof = read_json(EVIDENCE / 'p1020-proof.json')
    for row in proof:
        assert sha256(EVIDENCE / row['log']) == row['log_sha256']
        assert row['cwd'] == '/home/osso/.worktrees/wow-ui-sim-p1020-page'
    assert read_json(EVIDENCE / 'p1020-startup-final-stdout.json') == []
    warning_lines = [line for line in (EVIDENCE / 'p1020-mists-check-final.log').read_text().splitlines()
                     if line.startswith('warning:')]
    assert len(warning_lines) == 7
    assert all('iced-wgpu-patched/Cargo.toml' in line or '`iced_wgpu` (manifest)' in line
               for line in warning_lines)


def main():
    validate_sources()
    validate_sweeps()
    validate_artifacts()
    print('PASS: 284 source IDs, 18 exact chronological sweeps, 10 closures, 30 gaps, '
          '5 default-format mismatches, negative control and preserved inputs')


if __name__ == '__main__':
    main()
