"""Validate pinned 8.3.7 accounting; derive integration-sensitive totals from inputs."""
from collections import Counter
import hashlib
import importlib.util
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[4]

import sys
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / 'tools'))
from patch_audit_validation import historical_json, historical_registers, preserved_input_matches

AUDIT_REVISION = '9bed9e12b5eb9598b072365ce8df0cd47c5a68fc'
EVIDENCE = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'
PATCH = '8.3.7'


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


def check_sources():
    provenance = read_json(SOURCES / f'{PATCH}-api-changes.provenance.json')
    register = read_json(SOURCES / f'{PATCH}-wikitext-register.json')
    page = next(iter(read_json(EVIDENCE / 'p837-fetch.json')['query']['pages'].values()))
    revision = page['revisions'][0]
    raw_path = SOURCES / f'{PATCH}-api-changes.wikitext'
    assert revision['slots']['main']['*'] == raw_path.read_text()
    assert page['pageid'] == provenance['pageid']
    assert page['title'] == provenance['title']
    assert revision['revid'] == provenance['revid'] == register['source']['revid']
    assert revision['timestamp'] == provenance['timestamp']
    assert sha256(raw_path) == provenance['wikitext_sha256'] == register['source']['sha256']
    lines = raw_path.read_text().splitlines()
    assert {row['wikitext_line'] for row in register['entries']} == {
        number for number, line in enumerate(lines, 1)
        if line.startswith('* ') and '{{api|' in line}
    spec = importlib.util.spec_from_file_location('extractor', ROOT / 'tools/extract_patch_non_inventory.py')
    extractor = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(extractor)
    text = (SOURCES / f'{PATCH}-api-changes.txt').read_text()
    for preserve in (False, True):
        assert extractor.extract_text(raw_path.read_text(), preserve_examples=preserve) == text
    return register, extractor, lines, text


def check_accounting(register, extractor, raw_lines, text):
    results = read_json(EVIDENCE / 'patch_8_3_7_publication_sweep-results.json')
    known = set(read_json(ROOT / 'tests/data/patch_8_3_7_sweep_known_gaps.json'))
    inventory_ids = {row['id'] for row in register['entries']}
    assert set(results) == inventory_ids
    assert {key for key, row in results.items() if not row['ok']} == known
    reviews = read_json(EVIDENCE / 'p837-gap-review.json')
    assert {row['source_id'] for row in reviews} == known
    for row in reviews:
        assert row['literal'] == raw_lines[row['wikitext_line'] - 1]
        assert row['observation'] == results[row['source_id']]['observed']
        assert row['reason']
    ledger = read_json(SOURCES / f'{PATCH}-page-coverage.json')
    indexed = {row['source_id']: row for row in ledger['source_rows']}
    extract_ids = {row['source_id'] for row in extractor.seed_rows(text, PATCH)}
    assert len(indexed) == len(ledger['source_rows'])
    assert set(indexed) == inventory_ids | extract_ids
    assert ledger['source_sha256'] == sha256(SOURCES / f'{PATCH}-wikitext-register.json')
    assert ledger['non_inventory_source']['sha256'] == sha256(SOURCES / f'{PATCH}-api-changes.txt')
    for key, result in results.items():
        status = ('audit-pending' if key in known else 'bounded-coverage'
                  if result['expected']['publication'] == 'absent' else 'partial-development-green')
        assert indexed[key]['status'] == status
        assert bool(indexed[key]['capabilities']) == result['ok']
    scout = read_json(EVIDENCE / 'p837-extract-scout.json')
    assert {row['source_id'] for row in scout} == extract_ids
    for row in scout:
        source = indexed[row['source_id']]
        assert row['literal'] == text.splitlines()[row['extract_line'] - 1]
        assert row['reason'] == source['note']
        assert row['status'] == source['status']
        assert not source['capabilities']
    negative = read_json(EVIDENCE / 'p837-negative-observation.json')
    receipt = read_json(EVIDENCE / 'p837-negative-result.json')
    failures = {key for key, row in negative.items() if not row['ok']}
    assert set(negative) == inventory_ids
    assert sorted(known - failures) == receipt['resolved_gaps'] == [receipt['mutation']]
    assert sorted(failures - known) == receipt['new_gaps'] == []
    assert len(known) == receipt['baseline_gaps']
    assert len(failures) == receipt['negative_gaps']
    assert receipt['exit'] == 1
    initial = read_json(EVIDENCE / 'p837-initial-observation.json')
    closed = sum(not row['ok'] and results[key]['ok'] for key, row in initial.items())
    return {'inventory_rows': len(inventory_ids), 'extract_rows': len(extract_ids),
            'ledger_statuses': dict(Counter(row['status'] for row in indexed.values())),
            'gaps_remaining': len(known), 'gaps_closed': closed}


def check_preservation():
    hashes = read_json(EVIDENCE / 'p837-input-hashes-before.json')
    for path, digest in hashes.items():
        assert preserved_input_matches(ROOT, path, digest), path
    before = read_json(EVIDENCE / 'p837-extract-before.json')
    after = read_json(EVIDENCE / 'p837-extract-after.json')
    indexed = {(row['patch'], row['preserve_examples']): row for row in after}
    for row in before:
        current = indexed[(row['patch'], row['preserve_examples'])]
        assert (row['exit'], row['stdout']) == (current['exit'], current['stdout'])
    assert all(row['exit'] == 0 for row in after if row['patch'] == PATCH)
    registers = {path.name.removesuffix('-wikitext-register.json')
                 for path in historical_registers(ROOT, AUDIT_REVISION)}
    reproduced = read_json(EVIDENCE / 'p837-register-reproduction.json')
    assert {row['patch'] for row in reproduced} == registers
    for row in reproduced:
        assert row['exit'] == 0 and row['byte_identical']
        assert sha256(SOURCES / f"{row['patch']}-wikitext-register.json") == row['sha256']
        provenance = SOURCES / f"{row['patch']}-api-changes.provenance.json"
        flags = read_json(provenance).get('generator_flags') if provenance.exists() else None
        assert flags == row['recorded_flags']
        if flags is not None:
            assert flags == row['verified_flags']
    return {'preserved_inputs': len(hashes), 'preserved_extract_modes': len(before),
            'reproduced_registers': len(registers)}


def check_sweeps_and_proof():
    summary = []
    for register_path in historical_registers(ROOT, AUDIT_REVISION):
        patch = register_path.name.removesuffix('-wikitext-register.json')
        stem = 'patch_' + patch.replace('.', '_') + '_publication_sweep'
        results = read_json(EVIDENCE / f'{stem}-results.json')
        rows = read_json(register_path)['entries']
        assert set(results) == {row['id'] for row in rows}
        gaps = {key for key, row in results.items() if not row['ok']}
        assert gaps == set(read_json(ROOT / f'tests/data/patch_{patch.replace(".", "_")}_sweep_known_gaps.json'))
        summary.append({'patch': patch, 'rows': len(rows), 'ok': len(rows) - len(gaps),
                        'gaps': len(gaps), 'result': 'pass'})
    summary.sort(key=lambda row: tuple(map(int, row['patch'].split('.'))))
    for row in read_json(EVIDENCE / 'p837-proof.json'):
        if not row['invalidated']:
            assert row['exit'] == row['expected_exit'], row['scope']
        for path, digest in row.get('output_sha256', {}).items():
            assert sha256(EVIDENCE / path) == digest
    assert (EVIDENCE / 'p837-startup.stdout').read_text().strip() == '[]'
    warnings = [line for line in (EVIDENCE / 'p837-mists-check.log').read_text().splitlines()
                if line.startswith('warning:')]
    assert all('iced-wgpu-patched/Cargo.toml' in line or '`iced_wgpu` (manifest)' in line
               for line in warnings)
    return summary


def main():
    register, extractor, lines, text = check_sources()
    report = check_accounting(register, extractor, lines, text)
    report.update(check_preservation())
    report['sweeps'] = check_sweeps_and_proof()
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
