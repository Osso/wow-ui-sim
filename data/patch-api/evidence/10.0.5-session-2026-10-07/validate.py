"""Validate retained 10.0.5 source-accounting and proof artifacts; no runtime reruns."""
import gzip
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
    provenance = read_json(SOURCES / '10.0.5-api-changes.provenance.json')
    register = read_json(SOURCES / '10.0.5-wikitext-register.json')
    coverage = read_json(SOURCES / '10.0.5-page-coverage.json')
    raw_path = SOURCES / '10.0.5-api-changes.wikitext'
    text_path = SOURCES / '10.0.5-api-changes.txt'
    assert provenance['pageid'] == 75033
    assert provenance['wikitext']['revid'] == register['source']['revid'] == 742761
    assert provenance['wikitext']['sha256'] == register['source']['sha256'] == sha256(raw_path)
    assert coverage['source_sha256'] == sha256(SOURCES / '10.0.5-wikitext-register.json')
    assert coverage['schema'] == 'patch-page-coverage/v1'
    assert coverage['non_inventory_source']['sha256'] == sha256(text_path)
    assert coverage['non_inventory_source']['wikitext_sha256'] == sha256(raw_path)
    assert coverage['non_inventory_source']['wikitext_revid'] == 742761
    assert len(register['entries']) == 93
    assert all(row['header_count'] == row['parsed_count'] for row in register['header_counts'])
    extractor = load_extractor()
    for preserve in (False, True):
        assert extractor.extract_text(raw_path.read_text(), preserve_examples=preserve) == text_path.read_text()
    supplemental = extractor.seed_rows(text_path.read_text(), '10.0.5')
    inventory_ids = {row['id'] for row in register['entries']}
    supplemental_ids = {row['source_id'] for row in supplemental}
    ids = [row['source_id'] for row in coverage['source_rows']]
    assert len(supplemental) == 67
    assert len(ids) == len(set(ids)) == 160
    assert set(ids) == inventory_ids | supplemental_ids
    assert Counter(row['status'] for row in coverage['source_rows']) == {
        'bounded-coverage': 17, 'partial-development-green': 49,
        'audit-pending': 87, 'metadata-only': 7}
    scout = read_json(EVIDENCE / 'p1005-extract-scout.json')
    assert len(scout) == 67
    assert {row['source_id'] for row in scout} == supplemental_ids
    lines = text_path.read_text().splitlines()
    for row in scout:
        assert lines[row['line'] - 1] == row['literal']
        assert row['proof_boundary']
    return register, coverage


def validate_publication(register, coverage):
    results = read_json(EVIDENCE / 'p1005-sweep-10_0_5.json')
    discovery = read_json(EVIDENCE / 'p1005-discovery.json')
    gaps = {key for key, result in results.items() if not result['ok']}
    discovery_gaps = {key for key, result in discovery.items() if not result['ok']}
    known = read_json(ROOT / 'tests/data/patch_10_0_5_sweep_known_gaps.json')
    assert len(known) == len(set(known)) == len(gaps) == 27
    assert set(known) == gaps
    assert len(discovery_gaps) == 30
    review = read_json(EVIDENCE / 'p1005-gap-review.json')
    assert len(review) == len({row['source_id'] for row in review}) == 30
    assert {row['source_id'] for row in review} == discovery_gaps
    closed = {row['source_id'] for row in review if row['disposition'] == 'closed-retirement'}
    assert len(closed) == 3
    assert closed == discovery_gaps - gaps
    assert all(row['reason'] for row in review)
    rows = {row['source_id']: row for row in coverage['source_rows']}
    for key, result in results.items():
        assert (rows[key]['status'] == 'audit-pending') == (not result['ok'])
    later = {}
    prior = sorted((p for p in SOURCES.glob('*-wikitext-register.json')
                    if not p.name.startswith('10.0.5-')),
                   key=lambda p: tuple(map(int, p.name.split('-')[0].split('.'))))
    assert len(prior) == 23
    for path in prior:
        for entry in read_json(path)['entries']:
            if entry['direction'] != 'changed':
                later[entry['symbol']] = entry
    for entry in register['entries']:
        own_removed = entry['direction'] == 'removed'
        newer = later.get(entry['symbol'])
        superseded = newer is not None and (newer['direction'] == 'removed') != own_removed
        expected = results[entry['id']]['expected']
        removed = not own_removed if superseded else own_removed
        assert expected['publication'] == ('absent' if removed else 'published')
        assert expected['superseded_by'] == (newer['id'] if superseded else None)
    negative = read_json(EVIDENCE / 'p1005-negative.json')
    negative_gaps = {key for key, result in negative.items() if not result['ok']}
    assert negative_gaps - gaps == {'wt-global-api-C_Mail.SetOpeningAll-22'}
    assert not gaps - negative_gaps
    assert len(negative_gaps) == 28
    return prior


def validate_preservation():
    snapshot = read_json(EVIDENCE / 'p1005-preservation-before.json')
    assert len(snapshot) == 146
    assert all(sha256(ROOT / path) == digest for path, digest in snapshot.items())
    before = read_json(EVIDENCE / 'p1005-extract-before.json')
    after = read_json(EVIDENCE / 'p1005-extract-after.json')
    assert len(before) == 46 and len(after) == 48
    for row in before:
        saved = next(item for item in after if item['patch'] == row['patch']
                     and item['preserve_examples'] == row['preserve_examples'])
        assert row['exit'] == saved['exit']
        assert row['stdout'] == saved['stdout']
        assert row['stderr'] == saved['stderr']
    for patch in ('12.0.5', '12.0.7', '12.1.0'):
        assert all(row['exit'] != 0 for row in after if row['patch'] == patch)
    assert all(row['exit'] == 0 for row in after if row['patch'] == '10.0.5')
    reproduction = read_json(EVIDENCE / 'p1005-register-reproduction.json')
    assert len(reproduction) == 24
    assert all(row['reproducible'] for row in reproduction)


def validate_proof():
    proof = read_json(EVIDENCE / 'p1005-proof.json')
    assert len(proof) == 43
    for row in proof:
        assert row['cwd'] == str(ROOT)
        if row['command'][0] == 'cargo' and 'fmt' not in row['command']:
            assert row['target'] == str(ROOT / 'target')
        compressed = EVIDENCE / row['log']
        assert sha256(compressed) == row['compressed_sha256']
        data = gzip.decompress(compressed.read_bytes())
        assert hashlib.sha256(data).hexdigest() == row['log_sha256']
    sweeps = [row for row in proof if row['scope'].startswith('sweep-')]
    assert len(sweeps) == 24
    for row in sweeps:
        assert row['exit'] == 0
        output = gzip.decompress((EVIDENCE / row['log']).read_bytes()).decode()
        assert 'test result: ok. 1 passed; 0 failed' in output
        patch = row['scope'].removeprefix('sweep-')
        results = read_json(EVIDENCE / f'p1005-sweep-{patch}.json')
        register = read_json(SOURCES / f'{patch.replace("_", ".")}-wikitext-register.json')
        known = read_json(ROOT / f'tests/data/patch_{patch}_sweep_known_gaps.json')
        assert set(results) == {row['id'] for row in register['entries']}
        assert {key for key, result in results.items() if not result['ok']} == set(known)
    assert read_json(EVIDENCE / 'p1005-startup.stdout.json') == []
    startup = next(row for row in proof if row['scope'].startswith('retail cached startup'))
    assert startup['exit'] == 0
    mists = next(row for row in proof if row['scope'] == 'mists-check')
    assert mists['exit'] == 0
    output = gzip.decompress((EVIDENCE / mists['log']).read_bytes()).decode()
    warnings = [line for line in output.splitlines() if line.startswith('warning:')]
    assert len(warnings) == 7
    assert all('iced-wgpu-patched/Cargo.toml:' in line or '`iced_wgpu` (manifest)' in line
               for line in warnings)
    for scope in ('retirement-green', 'retirement-prefork', 'mists-preservation',
                  'fmt-check', 'default-check', 'retail-build', 'extractor-tests', 'parser-tests'):
        assert next(row for row in proof if row['scope'] == scope)['exit'] == 0
    consumers = read_json(EVIDENCE / 'p1005-removal-consumers.json')
    for row in consumers:
        assert row['qualified']['exit'] == row['bare']['exit'] == 1
        assert not row['qualified']['stdout'] and not row['bare']['stdout']
        assert not row['qualified']['stderr'] and not row['bare']['stderr']
    assert (EVIDENCE / 'p1005-whole-caller-scan.txt').read_text() == ''
    for line in (EVIDENCE / 'p1005-final-whole-caller-scan.txt').read_text().splitlines():
        assert line.split(':', 1)[0] in {
            'src/c_api/patch_retired_members.rs', 'tests/patch_10_0_5_publication_fixes.rs',
            'tests/patch_10_0_5_classic_surfaces.rs'}


def main():
    register, coverage = validate_sources()
    validate_publication(register, coverage)
    validate_preservation()
    validate_proof()
    print(json.dumps({'result': 'PASS', 'source_ids': 160, 'inventory_rows': 93,
                      'extract_rows': 67, 'publication_ok': 66, 'publication_gaps': 27,
                      'isolated_sweeps': 24, 'preserved_inputs': 146, 'hashed_logs': 43}, indent=2))


if __name__ == '__main__':
    main()
