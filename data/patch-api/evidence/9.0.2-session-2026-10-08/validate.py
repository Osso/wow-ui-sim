"""Validate 9.0.2 occurrence accounting; integration-sensitive totals derive from inputs."""
from collections import Counter
import hashlib
import importlib.util
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'
PATCH = '9.0.2'


def read_json(path):
    return json.loads(path.read_text())


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load_tool(name):
    spec = importlib.util.spec_from_file_location(name, ROOT / f'tools/{name}.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def check_sources():
    raw_path = SOURCES / f'{PATCH}-api-changes.wikitext'
    text_path = SOURCES / f'{PATCH}-api-changes.txt'
    register = read_json(SOURCES / f'{PATCH}-wikitext-register.json')
    provenance = read_json(SOURCES / f'{PATCH}-api-changes.provenance.json')
    page = next(iter(read_json(EVIDENCE / 'p902-fetch.json')['query']['pages'].values()))
    revision = page['revisions'][0]
    assert page['pageid'] == provenance['pageid']
    assert revision['revid'] == provenance['revid'] == register['source']['revid']
    assert revision['timestamp'] == provenance['timestamp']
    assert revision['slots']['main']['*'] == raw_path.read_text()
    assert provenance['wikitext_sha256'] == register['source']['sha256'] == sha256(raw_path)
    extractor = load_tool('extract_patch_non_inventory')
    for preserve in (False, True):
        assert extractor.extract_text(raw_path.read_text(), preserve_examples=preserve) == text_path.read_text()
    raw_lines = raw_path.read_text().splitlines()
    headers = read_json(EVIDENCE / 'p902-header-accounting.json')
    expected_header_lines = {number for number, line in enumerate(raw_lines, 1)
                             if re.search(r'\| \d+ (new|removed)', line)}
    assert {row['wikitext_line'] for row in headers} == expected_header_lines
    for row in headers:
        assert row['literal'] == raw_lines[row['wikitext_line'] - 1]
        parsed = sum(entry['section'] == row['section'] and entry['direction'] == row['direction']
                     for entry in register['entries'])
        assert parsed == row['parsed_count'] == row['header_count']
    return register, extractor, raw_lines


def check_accounting(register, extractor, raw_lines):
    observed = read_json(EVIDENCE / 'patch_9_0_2_publication_sweep-results.json')
    known = set(read_json(ROOT / 'tests/data/patch_9_0_2_sweep_known_gaps.json'))
    inventory_ids = {row['id'] for row in register['entries']}
    assert set(observed) == inventory_ids
    assert {key for key, row in observed.items() if not row['ok']} == known
    reviews = read_json(EVIDENCE / 'p902-gap-review.json')
    assert {row['source_id'] for row in reviews} == known
    for row in reviews:
        assert row['reason']
        assert row['literal'] == raw_lines[row['wikitext_line'] - 1]
        assert row['observation'] == observed[row['source_id']]['observed']
    ledger = read_json(SOURCES / f'{PATCH}-page-coverage.json')
    text_path = SOURCES / f'{PATCH}-api-changes.txt'
    extract_rows = extractor.seed_rows(text_path.read_text(), PATCH)
    extract_ids = {row['source_id'] for row in extract_rows}
    context_ids = {f'source-context-wikitext-{number:03}'
                   for number, line in enumerate(raw_lines, 1) if line.startswith('|+')}
    rows = ledger['source_rows']
    indexed = {row['source_id']: row for row in rows}
    assert len(indexed) == len(rows)
    assert set(indexed) == inventory_ids | extract_ids | context_ids
    assert ledger['source_sha256'] == sha256(SOURCES / f'{PATCH}-wikitext-register.json')
    assert ledger['non_inventory_source']['sha256'] == sha256(text_path)
    for source_id, result in observed.items():
        row = indexed[source_id]
        expected_status = ('audit-pending' if source_id in known else
                           'bounded-coverage' if result['expected']['publication'] == 'absent'
                           else 'partial-development-green')
        assert row['status'] == expected_status
        assert row['note']
        assert bool(row['capabilities']) == result['ok']
    scout = read_json(EVIDENCE / 'p902-extract-scout.json')
    assert {row['source_id'] for row in scout} == extract_ids
    for row in scout:
        assert row['literal'] == text_path.read_text().splitlines()[row['extract_line'] - 1]
        assert row['raw_literal'] == raw_lines[row['wikitext_line'] - 1]
        assert row['reason'] == indexed[row['source_id']]['note']
        assert row['status'] == indexed[row['source_id']]['status']
        assert not indexed[row['source_id']]['capabilities']
    for source_id in context_ids:
        assert indexed[source_id]['status'] == 'metadata-only'
        assert not indexed[source_id]['capabilities']
    negative = read_json(EVIDENCE / 'p902-negative-observation.json')
    receipt = read_json(EVIDENCE / 'p902-negative-result.json')
    failures = {key for key, row in negative.items() if not row['ok']}
    assert set(negative) == inventory_ids
    assert failures - known == {receipt['mutation']}
    assert not known - failures
    assert receipt['baseline_gaps'] == len(known)
    assert receipt['negative_gaps'] == len(failures)
    initial = read_json(EVIDENCE / 'p902-initial-observation.json')
    closed = {key for key, row in initial.items() if not row['ok'] and observed[key]['ok']}
    scans = {row['source_id']: row for row in read_json(EVIDENCE / 'p902-removal-consumers.json')}
    for source_id in closed:
        assert observed[source_id]['expected']['publication'] == 'absent'
        assert all(scans[source_id][kind]['exit'] == 1 and not scans[source_id][kind]['stdout']
                   for kind in ('qualified', 'bare', 'callers'))
    return dict(Counter(row['status'] for row in rows)), len(known), len(closed)


def check_preservation():
    hashes = read_json(EVIDENCE / 'p902-input-hashes-before.json')
    for path, digest in hashes.items():
        assert sha256(ROOT / path) == digest, path
    before = read_json(EVIDENCE / 'p902-extract-before.json')
    after = read_json(EVIDENCE / 'p902-extract-after.json')
    indexed = {(row['patch'], row['preserve_examples']): row for row in after}
    for row in before:
        current = indexed[(row['patch'], row['preserve_examples'])]
        assert (row['exit'], row['stdout']) == (current['exit'], current['stdout'])
    assert all(row['exit'] == 0 for row in after if row['patch'] == PATCH)
    reproduced = read_json(EVIDENCE / 'p902-register-reproduction.json')
    assert {row['patch'] for row in reproduced} == {
        path.name.removesuffix('-wikitext-register.json')
        for path in SOURCES.glob('*-wikitext-register.json')}
    for row in reproduced:
        assert row['byte_identical'] and row['exit'] == 0
        assert sha256(SOURCES / f"{row['patch']}-wikitext-register.json") == row['sha256']
        provenance = SOURCES / f"{row['patch']}-api-changes.provenance.json"
        recorded = read_json(provenance).get('generator_flags') if provenance.exists() else None
        assert recorded == row['recorded_flags']
        if recorded is not None:
            assert recorded == row['verified_flags']
    return len(hashes), len(before), len(reproduced)


def check_receipts():
    for row in read_json(EVIDENCE / 'p902-proof.json'):
        assert row['cwd'] == str(ROOT)
        assert row['target'] == str(ROOT / 'target')
        if row['invalidated']:
            assert row['reason']
        else:
            assert row['exit'] == row.get('expected_exit', 0), row['scope']
        for path, digest in row['output_sha256'].items():
            assert sha256(EVIDENCE / path) == digest
    assert (EVIDENCE / 'p902-startup.stdout').read_text().strip() == '[]'
    warnings = [line for line in (EVIDENCE / 'p902-mists-check.log').read_text().splitlines()
                if line.startswith('warning:')]
    assert all('iced-wgpu-patched/Cargo.toml' in line or '`iced_wgpu` (manifest)' in line
               for line in warnings)
    summary = read_json(EVIDENCE / 'p902-sweep-summary.json')
    registers = {path.name.removesuffix('-wikitext-register.json')
                 for path in SOURCES.glob('*-wikitext-register.json')}
    assert {row['patch'] for row in summary} == registers
    for row in summary:
        stem = 'patch_' + row['patch'].replace('.', '_')
        results = read_json(EVIDENCE / f'{stem}_publication_sweep-results.json')
        known = set(read_json(ROOT / f'tests/data/{stem}_sweep_known_gaps.json'))
        assert row['rows'] == len(results)
        assert row['gaps'] == len(known)
        assert {key for key, result in results.items() if not result['ok']} == known
        assert row['ok'] + row['gaps'] == row['rows']
    return len(summary)


def main():
    register, extractor, raw_lines = check_sources()
    statuses, gaps, closed = check_accounting(register, extractor, raw_lines)
    preserved, mode_outcomes, reproduced = check_preservation()
    sweeps = check_receipts()
    print(json.dumps({'result': 'pass', 'inventory': len(register['entries']),
                      'ledger_statuses': statuses, 'gaps': gaps, 'closed': closed,
                      'preserved_inputs': preserved, 'preserved_mode_outcomes': mode_outcomes,
                      'reproduced_registers': reproduced, 'publication_sweeps': sweeps}, indent=2))


if __name__ == '__main__':
    main()
