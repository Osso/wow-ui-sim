"""Validate retained 9.0.5 artifacts; counts derive from observations and fixtures."""
from collections import Counter
import hashlib
import importlib.util
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / 'tools'))
from patch_audit_validation import read_audit_json, historical_registers, preserved_input_matches

AUDIT_REVISION = 'e648db9f99bb32dd4ac87b8d8eb9e2cf2518c0fb'
EVIDENCE = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'


def read_json(path):
    return read_audit_json(ROOT, path, AUDIT_REVISION)


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load_tool(name):
    spec = importlib.util.spec_from_file_location(name, ROOT / f'tools/{name}.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def check_sources():
    raw = SOURCES / '9.0.5-api-changes.wikitext'
    text = SOURCES / '9.0.5-api-changes.txt'
    register = read_json(SOURCES / '9.0.5-wikitext-register.json')
    provenance = read_json(SOURCES / '9.0.5-api-changes.provenance.json')
    page = next(iter(read_json(EVIDENCE / 'p905-fetch.json')['query']['pages'].values()))
    revision = page['revisions'][0]
    assert page['pageid'] == provenance['pageid']
    assert revision['revid'] == provenance['revid'] == register['source']['revid']
    assert revision['timestamp'] == provenance['timestamp']
    assert revision['slots']['main']['*'] == raw.read_text()
    assert provenance['wikitext_sha256'] == register['source']['sha256'] == sha256(raw)
    assert provenance['generator_flags'] == []
    generator = load_tool('gen_patch_wikitext_register')
    buckets = generator.split_sections(raw.read_text())
    entries, counts = [], []
    for section in generator.SECTIONS.values():
        rows, headers = generator.parse_section(section, buckets.get(section, []))
        entries.extend(rows)
        counts.extend(headers)
    assert entries == register['entries']
    assert counts == register['header_counts']
    for header in read_json(EVIDENCE / 'p905-header-accounting.json'):
        assert header['literal'] == raw.read_text().splitlines()[header['wikitext_line'] - 1]
        section = {'Global API': 'global-api', 'CVars': 'cvars'}[header['section']]
        parsed = sum(row['section'] == section and row['direction'] == header['direction']
                     for row in entries)
        assert header['header_count'] == header['parsed_count'] == parsed
    extractor = load_tool('extract_patch_non_inventory')
    for preserve in (False, True):
        assert extractor.extract_text(raw.read_text(), preserve_examples=preserve) == text.read_text()
    return register, extractor


def check_accounting(register, extractor):
    observed = read_json(EVIDENCE / 'patch_9_0_5_publication_sweep-results.json')
    known = set(read_json(ROOT / 'tests/data/patch_9_0_5_sweep_known_gaps.json'))
    inventory_ids = {row['id'] for row in register['entries']}
    assert set(observed) == inventory_ids
    assert {key for key, row in observed.items() if not row['ok']} == known
    reviews = read_json(EVIDENCE / 'p905-gap-review.json')
    assert {row['source_id'] for row in reviews} == known
    raw_lines = (SOURCES / '9.0.5-api-changes.wikitext').read_text().splitlines()
    for row in reviews:
        assert row['reason']
        assert row['literal'] == raw_lines[row['wikitext_line'] - 1]
        assert row['observation'] == observed[row['source_id']]['observed']
    ledger = read_json(SOURCES / '9.0.5-page-coverage.json')
    text = SOURCES / '9.0.5-api-changes.txt'
    extract_rows = extractor.seed_rows(text.read_text(), '9.0.5')
    expected_ids = inventory_ids | {row['source_id'] for row in extract_rows}
    rows = ledger['source_rows']
    ids = [row['source_id'] for row in rows]
    assert len(ids) == len(set(ids))
    assert set(ids) == expected_ids
    assert ledger['source_sha256'] == sha256(SOURCES / '9.0.5-wikitext-register.json')
    assert ledger['non_inventory_source']['sha256'] == sha256(text)
    indexed = {row['source_id']: row for row in rows}
    for source_id, result in observed.items():
        row = indexed[source_id]
        expected_status = ('audit-pending' if source_id in known else
                           'bounded-coverage' if result['expected']['publication'] == 'absent'
                           else 'partial-development-green')
        assert row['status'] == expected_status
        assert row['note']
        assert bool(row['capabilities']) == result['ok']
    assert [row for row in rows if row['source_id'] not in inventory_ids] == extract_rows
    scout = read_json(EVIDENCE / 'p905-extract-scout.json')
    assert {row['source_id'] for row in scout} == {row['source_id'] for row in extract_rows}
    for row in scout:
        assert row['literal'] == text.read_text().splitlines()[row['extract_line'] - 1]
        assert row['raw_literal'] == raw_lines[row['wikitext_line'] - 1]
        assert row['reason']
    negative = read_json(EVIDENCE / 'p905-negative-observation.json')
    receipt = read_json(EVIDENCE / 'p905-negative-result.json')
    failures = {key for key, row in negative.items() if not row['ok']}
    assert set(negative) == set(observed)
    assert failures - known == {receipt['mutation']}
    assert not known - failures
    assert receipt['baseline_gaps'] == len(known)
    assert receipt['negative_gaps'] == len(failures)
    scans = read_json(EVIDENCE / 'p905-removal-consumers.json')
    retired = next(row for row in scans if row['symbol'] == 'C_Soulbinds.GetConduitItemLevel')
    assert all(retired[k]['exit'] == 1 and not retired[k]['stdout']
               for k in ('qualified', 'bare', 'callers'))
    return dict(Counter(row['status'] for row in rows)), len(known)


def check_preservation():
    hashes = read_json(EVIDENCE / 'p905-input-hashes-before.json')
    for path, digest in hashes.items():
        assert preserved_input_matches(ROOT, path, digest), path
    before = read_json(EVIDENCE / 'p905-extract-before.json')
    after = read_json(EVIDENCE / 'p905-extract-after.json')
    indexed = {(row['patch'], row['preserve_examples']): row for row in after}
    for row in before:
        current = indexed[(row['patch'], row['preserve_examples'])]
        assert (row['exit'], row['stdout']) == (current['exit'], current['stdout'])
    assert all(row['exit'] == 0 for row in after if row['patch'] == '9.0.5')
    reproduced = read_json(EVIDENCE / 'p905-register-reproduction.json')
    assert {row['patch'] for row in reproduced} == {
        path.name.removesuffix('-wikitext-register.json')
        for path in historical_registers(ROOT, AUDIT_REVISION)}
    for row in reproduced:
        assert row['byte_identical']
        assert sha256(SOURCES / f"{row['patch']}-wikitext-register.json") == row['sha256']
        provenance = SOURCES / f"{row['patch']}-api-changes.provenance.json"
        if provenance.exists():
            recorded = read_json(provenance).get('generator_flags')
            assert recorded == row['recorded_flags']
            if recorded is not None:
                assert recorded == row['verified_flags']
    return len(hashes), len(reproduced)


def check_receipts():
    for row in read_json(EVIDENCE / 'p905-proof.json'):
        if row['invalidated']:
            assert row['reason']
        else:
            assert row['exit'] == row.get('expected_exit', 0), row['scope']
        for path, digest in row['output_sha256'].items():
            assert sha256(EVIDENCE / path) == digest
    assert (EVIDENCE / 'p905-startup.stdout').read_text().strip() == '[]'
    warnings = [line for line in (EVIDENCE / 'p905-mists-check.log').read_text().splitlines()
                if line.startswith('warning:')]
    assert all('iced-wgpu-patched/Cargo.toml' in line or '`iced_wgpu` (manifest)' in line
               for line in warnings)
    summary = read_json(EVIDENCE / 'p905-sweep-summary.json')
    for row in summary:
        stem = 'patch_' + row['patch'].replace('.', '_') + '_publication_sweep'
        results = read_json(EVIDENCE / f'{stem}-results.json')
        known = set(read_json(ROOT / f'tests/data/{stem.removesuffix("_publication_sweep")}_sweep_known_gaps.json'))
        assert row['rows'] == len(results)
        assert row['gaps'] == len(known)
        assert {key for key, result in results.items() if not result['ok']} == known
        assert row['ok'] + row['gaps'] == row['rows']
    return len(summary)


def main():
    register, extractor = check_sources()
    statuses, gaps = check_accounting(register, extractor)
    preserved, reproduced = check_preservation()
    sweeps = check_receipts()
    print(json.dumps({'result': 'pass', 'inventory': len(register['entries']),
                      'ledger_statuses': statuses, 'gaps': gaps,
                      'preserved_inputs': preserved, 'reproduced_registers': reproduced,
                      'publication_sweeps': sweeps}, indent=2))


if __name__ == '__main__':
    main()
