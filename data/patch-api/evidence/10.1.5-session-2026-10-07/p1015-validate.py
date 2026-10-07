#!/usr/bin/env python3
"""Validate retained occurrence accounting and targeted proof; never run tests."""
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


def validate_source_accounting():
    register_path = SOURCES / '10.1.5-wikitext-register.json'
    register = read_json(register_path)
    ledger = read_json(SOURCES / '10.1.5-page-coverage.json')
    raw = SOURCES / '10.1.5-api-changes.wikitext'
    text = SOURCES / '10.1.5-api-changes.txt'
    assert register['source']['revid'] == 3807695
    assert register['source']['sha256'] == sha256(raw)
    assert ledger['source_sha256'] == sha256(register_path)
    assert ledger['non_inventory_source']['sha256'] == sha256(text)
    spec = importlib.util.spec_from_file_location(
        'extractor', ROOT / 'tools/extract_patch_non_inventory.py')
    extractor = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(extractor)
    assert extractor.extract_text(raw.read_text(), preserve_examples=True) == text.read_text()
    inventory_ids = {r['id'] for r in register['entries']}
    extract_ids = {r['source_id'] for r in extractor.seed_rows(text.read_text(), '10.1.5')}
    ids = [r['source_id'] for r in ledger['source_rows']]
    assert len(inventory_ids) == 101 and len(extract_ids) == 98
    assert len(ids) == len(set(ids)) == 199
    assert set(ids) == inventory_ids | extract_ids
    assert Counter(r['direction'] for r in register['entries']) == {
        'added': 74, 'removed': 15, 'changed': 12}
    mismatches = [(r['section'], r['direction'], r['header_count'], r['parsed_count'])
                  for r in register['header_counts'] if r['header_count'] != r['parsed_count']]
    assert mismatches == [('global-api', 'added', 36, 40), ('global-api', 'removed', 3, 8)]
    scout = read_json(EVIDENCE / 'p1015-extract-scout.json')
    assert len(scout) == 98 and {r['source_id'] for r in scout} == extract_ids
    assert Counter(r['status'] for r in scout) == {'audit-pending': 89, 'metadata-only': 9}
    assert Counter(r['status'] for r in ledger['source_rows']) == {
        'bounded-coverage': 18, 'partial-development-green': 50,
        'audit-pending': 122, 'metadata-only': 9}
    return register


def validate_sweeps(register):
    final = read_json(EVIDENCE / 'p1015-sweep-final-result.json')
    discovery = read_json(EVIDENCE / 'p1015-sweep-discovery-result.json')
    known = set(read_json(ROOT / 'tests/data/patch_10_1_5_sweep_known_gaps.json'))
    assert set(final) == {r['id'] for r in register['entries']}
    assert {k for k, v in final.items() if not v['ok']} == known and len(known) == 32
    closed = {k for k, v in discovery.items() if not v['ok']} - known
    assert closed == {'wt-global-api-C_CampaignInfo.UsesNormalQuestIcons-65',
                      'wt-global-api-RequestArtifactCompletionHistory-71'}
    review = read_json(EVIDENCE / 'p1015-gap-review.json')
    assert len(review['rows']) == 34
    assert {r['source_id'] for r in review['rows']} == known | closed
    assert all(r['reason'] for r in review['rows'])
    defaults = {k for k, v in final.items() if v['observed']['default_mismatch']}
    assert defaults == {'wt-cvars-wmoPortalInteriorFade-168'}
    paths = sorted(SOURCES.glob('*-wikitext-register.json'),
                   key=lambda p: tuple(map(int, p.name.split('-')[0].split('.'))))
    latest = {}
    for path in paths[1:]:
        for row in read_json(path)['entries']:
            if row['direction'] != 'changed':
                latest[row['symbol']] = row
    for row in register['entries']:
        absent = row['direction'] == 'removed'
        newer = latest.get(row['symbol'])
        superseded = newer and (newer['direction'] == 'removed') != absent
        assert final[row['id']]['expected']['publication'] == (
            'absent' if (not absent if superseded else absent) else 'published')
        assert final[row['id']]['expected']['superseded_by'] == (
            newer['id'] if superseded else None)
    table = []
    for path in paths:
        patch = path.name.split('-')[0]
        result = read_json(EVIDENCE / f'p{patch.replace(".", "")}-sweep-final-result.json')
        fixture = set(read_json(ROOT / f'tests/data/patch_{patch.replace(".", "_")}_sweep_known_gaps.json'))
        assert set(result) == {r['id'] for r in read_json(path)['entries']}
        assert {k for k, v in result.items() if not v['ok']} == fixture
        table.append({'patch': patch, 'rows': len(result), 'ok': len(result) - len(fixture),
                      'gaps': len(fixture)})
    assert len(table) == 20
    negative = read_json(EVIDENCE / 'p1015-negative-result.json')
    negative_gaps = {k for k, v in negative.items() if not v['ok']}
    assert negative_gaps - known == {'wt-widgets-Frame:AbortDrag-98'}
    assert not known - negative_gaps and len(negative_gaps) == 33
    return table


def validate_proof():
    proof = read_json(EVIDENCE / 'p1015-proof.json')
    for row in proof:
        assert row['cwd'] == str(ROOT)
        assert row['exit'] == row.get('expected_exit', 0)
        assert sha256(ROOT / row['log']) == row['sha256']
        assert not row['invalidated']
    assert len([r for r in proof if r['scope'].endswith('publication_sweep')]) == 20
    startup = next(r for r in proof if r['scope'] == 'startup')
    assert startup['exit'] == 0 and json.loads(startup['stdout']) == []
    warnings = [line for line in (EVIDENCE / 'p1015-mists-check.txt').read_text().splitlines()
                if line.startswith('warning:')]
    assert len(warnings) == 7
    assert all('iced-wgpu-patched/Cargo.toml' in line or
               '`iced_wgpu` (manifest) generated 6 warnings' in line for line in warnings)
    reproduction = read_json(EVIDENCE / 'p1015-register-reproduction.json')
    assert len(reproduction) == 20 and all(r['byte_identical'] for r in reproduction)
    extracts = read_json(EVIDENCE / 'p1015-extract-reproduction.json')
    assert len(extracts) == 20
    assert all(r['identical'] for r in extracts if r['patch'] not in ('12.0.5', '12.0.7', '12.1.0'))
    # These three are MediaWiki/crawler captures, not outputs of this extractor.
    preserved = read_json(EVIDENCE / 'p1015-preserved-inputs.json')
    assert len(preserved) == 122
    assert all(r['identical'] and sha256(ROOT / r['path']) == r['sha256'] for r in preserved)
    supersessions = read_json(EVIDENCE / 'p1015-possible-1017-supersessions.json')
    assert supersessions['publication_intersections'] == []
    assert supersessions['changed_symbol_intersections'] == []


def main():
    register = validate_source_accounting()
    table = validate_sweeps(register)
    validate_proof()
    print(json.dumps({'result': 'PASS', 'source_ids': 199, 'sweeps': table,
                      'register_reproductions': 20, 'generated_extract_reproductions': 17,
                      'preserved_inputs': 122}, indent=2))


if __name__ == '__main__':
    main()
