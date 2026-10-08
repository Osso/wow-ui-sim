"""Validate retained 10.0.2 accounting artifacts; no runtime reruns."""
import gzip
import hashlib
import importlib.util
import json
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]

import sys
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / 'tools'))
from patch_audit_validation import historical_json, historical_registers, preserved_input_matches

AUDIT_REVISION = '4c3d0ffa215352b5ba72eacbc82ca749ca076fea'
EVIDENCE = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'


def read_json(path):
    relative = path.relative_to(ROOT).as_posix()
    if relative in {
        'tests/data/patch_9_2_5_sweep_known_gaps.json',
        'data/patch-api/sources/9.2.5-page-coverage.json',
    }:
        return historical_json(ROOT, relative, AUDIT_REVISION)
    return json.loads(path.read_text())


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def validate_sources():
    register = read_json(SOURCES / '10.0.2-wikitext-register.json')
    coverage = read_json(SOURCES / '10.0.2-page-coverage.json')
    provenance = read_json(SOURCES / '10.0.2-api-changes.provenance.json')
    raw = SOURCES / '10.0.2-api-changes.wikitext'
    text = SOURCES / '10.0.2-api-changes.txt'
    assert provenance['pageid'] == 303146
    assert provenance['wikitext']['revid'] == register['source']['revid'] == 2926754
    assert provenance['wikitext']['sha256'] == register['source']['sha256'] == sha256(raw)
    assert coverage['schema'] == 'patch-page-coverage/v1'
    assert coverage['source_sha256'] == sha256(SOURCES / '10.0.2-wikitext-register.json')
    assert coverage['non_inventory_source']['sha256'] == sha256(text)
    assert coverage['non_inventory_source']['wikitext_sha256'] == sha256(raw)
    assert len(register['entries']) == 416
    assert all(row['header_count'] == row['parsed_count'] for row in register['header_counts'])
    spec = importlib.util.spec_from_file_location('extractor', ROOT / 'tools/extract_patch_non_inventory.py')
    extractor = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(extractor)
    assert extractor.extract_text(raw.read_text(), preserve_examples=True) == text.read_text()
    supplemental = extractor.seed_rows(text.read_text(), '10.0.2')
    ids = [row['source_id'] for row in coverage['source_rows']]
    assert len(supplemental) == 167
    assert len(ids) == len(set(ids)) == 583
    assert set(ids) == {row['id'] for row in register['entries']} | {row['source_id'] for row in supplemental}
    assert Counter(row['status'] for row in coverage['source_rows']) == {
        'bounded-coverage': 137, 'partial-development-green': 130,
        'audit-pending': 305, 'metadata-only': 11}
    scout = read_json(EVIDENCE / 'p1002-extract-scout.json')
    assert {row['source_id'] for row in scout} == {row['source_id'] for row in supplemental}
    lines = text.read_text().splitlines()
    for row in scout:
        assert lines[row['line'] - 1] == row['literal']
        assert row['proof_boundary']
        assert row['status'] in ('audit-pending', 'metadata-only')
    return register, coverage


def validate_publication(register, coverage):
    final = read_json(EVIDENCE / 'p1002-sweep.json')
    discovery = read_json(EVIDENCE / 'p1002-discovery.json')
    gaps = {key for key, value in final.items() if not value['ok']}
    initial = {key for key, value in discovery.items() if not value['ok']}
    assert set(final) == {row['id'] for row in register['entries']}
    assert len(gaps) == 149 and len(initial) == 169
    assert gaps == set(read_json(ROOT / 'tests/data/patch_10_0_2_sweep_known_gaps.json'))
    review = read_json(EVIDENCE / 'p1002-gap-review.json')
    assert len(review) == len({row['source_id'] for row in review}) == 169
    assert {row['source_id'] for row in review} == initial
    retired = {row['source_id'] for row in review if row['disposition'] == 'closed-retirement'}
    assert retired == initial - gaps and len(retired) == 20
    assert all(row['reason'] for row in review)
    for row in coverage['source_rows'][:416]:
        assert (row['status'] == 'audit-pending') == (row['source_id'] in gaps)
    later = {}
    paths = sorted((path for path in historical_registers(ROOT, AUDIT_REVISION)
                    if not path.name.startswith('10.0.2-')),
                   key=lambda path: tuple(map(int, path.name.split('-')[0].split('.'))))
    assert len(paths) == 24
    for path in paths:
        for entry in read_json(path)['entries']:
            if entry['direction'] != 'changed':
                later[entry['symbol']] = entry
    for entry in register['entries']:
        removed = entry['direction'] == 'removed'
        newer = later.get(entry['symbol'])
        superseded = newer is not None and (newer['direction'] == 'removed') != removed
        expected = final[entry['id']]['expected']
        assert expected['publication'] == ('absent' if removed != superseded else 'published')
        assert expected['superseded_by'] == (newer['id'] if superseded else None)
    negative = read_json(EVIDENCE / 'p1002-negative-observed.json')
    negative_gaps = {key for key, value in negative.items() if not value['ok']}
    assert negative_gaps - gaps == {'wt-global-api-UnitTokenFromGUID-315'}
    assert not gaps - negative_gaps
    for row in read_json(EVIDENCE / 'p1002-exact-removal-consumers.json'):
        if row['id'] in retired:
            for mode in ('qualified', 'bare'):
                assert row[mode]['exit'] == 1
                assert not row[mode]['stdout'] and not row[mode]['stderr']
    assert (EVIDENCE / 'p1002-retirement-whole-caller-scan.txt').read_text() == ''
    for line in (EVIDENCE / 'p1002-final-retirement-whole-caller-scan.txt').read_text().splitlines():
        assert line.split(':', 1)[0] in {
            'src/c_api/patch_retired_members.rs', 'tests/patch_10_0_2_publication_fixes.rs',
            'tests/patch_10_0_2_classic_surfaces.rs'}


def validate_preservation():
    snapshot = read_json(EVIDENCE / 'p1002-preservation.json')
    assert len(snapshot) == 152
    assert all(preserved_input_matches(ROOT, path, digest) for path, digest in snapshot.items())
    before = read_json(EVIDENCE / 'p1002-extract-before.json')
    after = read_json(EVIDENCE / 'p1002-extract-after.json')
    assert len(before) == 48 and len(after) == 50
    for row in before:
        assert row == next(value for value in after if value['patch'] == row['patch']
                           and value['preserve_examples'] == row['preserve_examples'])
    own = [row for row in after if row['patch'].startswith('10.0.2-')]
    assert next(row for row in own if row['preserve_examples'])['exit'] == 0
    assert next(row for row in own if not row['preserve_examples'])['exit'] != 0
    reproduced = read_json(EVIDENCE / 'p1002-register-reproduction.json')
    assert len(reproduced) == 25 and all(row['reproducible'] for row in reproduced)


def validate_proof():
    proof = read_json(EVIDENCE / 'p1002-proof.json')
    sweeps = [row for row in proof if row['scope'].startswith('patch_')
              and row['scope'].endswith('_publication_sweep')]
    assert len(sweeps) == 25
    for row in proof:
        compressed = EVIDENCE / row['log']
        assert sha256(compressed) == row['compressed_sha256']
        data = gzip.decompress(compressed.read_bytes())
        assert hashlib.sha256(data).hexdigest() == row['log_sha256']
    for row in sweeps:
        assert row['exit'] == 0
        output = gzip.decompress((EVIDENCE / row['log']).read_bytes()).decode()
        assert 'test result: ok. 1 passed; 0 failed' in output
        patch = row['scope'].removeprefix('patch_').removesuffix('_publication_sweep')
        observed = read_json(EVIDENCE / ('p' + patch.replace('_', '') + '-sweep.json'))
        known = read_json(ROOT / f'tests/data/patch_{patch}_sweep_known_gaps.json')
        assert {key for key, value in observed.items() if not value['ok']} == set(known)
    assert read_json(EVIDENCE / 'p1002-startup.stdout.json') == []
    for scope in ('retirement-green', 'cached Game retirement lookup',
                  'Mists legacy namespace behavior', 'fmt-check', 'default-check',
                  'mists-check', 'retail-build', 'retail cached startup',
                  'tools/test_extract_patch_non_inventory.py',
                  'tools/test_gen_patch_wikitext_register.py'):
        assert next(row for row in proof if row['scope'] == scope)['exit'] == 0
    mists = next(row for row in proof if row['scope'] == 'mists-check')
    text = gzip.decompress((EVIDENCE / mists['log']).read_bytes()).decode()
    warnings = [line for line in text.splitlines() if line.startswith('warning:')]
    assert len(warnings) == 7
    assert all('iced-wgpu-patched/Cargo.toml:' in line or '`iced_wgpu` (manifest)' in line
               for line in warnings)


def main():
    register, coverage = validate_sources()
    validate_publication(register, coverage)
    validate_preservation()
    validate_proof()
    print(json.dumps({'result': 'PASS', 'inventory_rows': 416, 'extract_rows': 167,
                      'source_ids': 583, 'publication_ok': 267, 'publication_gaps': 149,
                      'closed_retirements': 20, 'isolated_sweeps': 25,
                      'preserved_inputs': 152}, indent=2))


if __name__ == '__main__':
    main()
