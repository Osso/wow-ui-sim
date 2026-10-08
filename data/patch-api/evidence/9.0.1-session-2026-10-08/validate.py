"""Validate 9.0.1 accounting without freezing integration-sensitive totals."""
from collections import Counter
import hashlib
import importlib.util
import json
import sys
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[4]

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / 'tools'))
from patch_audit_validation import (
    historical_registers,
    historical_sweep_tests,
    preserved_input_matches,
    read_audit_json,
)

AUDIT_REVISION = '56a1b8e6cacc1acc115956a1fac2497600178762'
EVIDENCE = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'
PATCH = '9.0.1'


def read_json(path):
    return read_audit_json(ROOT, path, AUDIT_REVISION)


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load_tool(name):
    spec = importlib.util.spec_from_file_location(name, ROOT / f'tools/{name}.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def reproduce_register(register, flags, generator):
    options = {flag.removeprefix('--').replace('-', '_'): True for flag in flags}
    buckets = generator.split_sections(
        (ROOT / register['source']['path']).read_text(),
        separate_inline_structures=options.get('separate_inline_structures', False))
    entries, counts = [], []
    section_options = {key: options.get(key, False) for key in (
        'expand_shared_changes', 'capture_span_defaults', 'skip_plain_scripts_label')}
    for section in generator.SECTIONS.values():
        entry_section = 'cvars' if section == 'commands' else section
        rows, headers = generator.parse_section(
            entry_section, buckets.get(section, []), **section_options)
        if section == 'commands':
            for header in headers:
                header['section'] = 'commands'
        entries.extend(rows)
        counts.extend(headers)
    if options.get('inventory_only'):
        for row in entries:
            for key in ('kind', 'page_default', 'test_inline'):
                row.pop(key, None)
    reproduced = {**register, 'entries': entries, 'header_counts': counts}
    return (json.dumps(reproduced, indent=2, ensure_ascii=False) + '\n').encode()


def check_sources():
    provenance = read_json(SOURCES / f'{PATCH}-api-changes.provenance.json')
    register_path = SOURCES / f'{PATCH}-wikitext-register.json'
    register = read_json(register_path)
    raw_path = SOURCES / f'{PATCH}-api-changes.wikitext'
    page = next(iter(read_json(EVIDENCE / 'p901-fetch.json')['query']['pages'].values()))
    revision = page['revisions'][0]
    assert page['pageid'] == provenance['pageid']
    assert revision['revid'] == provenance['revid'] == register['source']['revid']
    assert revision['timestamp'] == provenance['timestamp']
    assert revision['slots']['main']['*'] == raw_path.read_text()
    assert sha256(raw_path) == provenance['wikitext_sha256'] == register['source']['sha256']
    assert reproduce_register(register, provenance['generator_flags'],
                              load_tool('gen_patch_wikitext_register')) == register_path.read_bytes()
    extractor = load_tool('extract_patch_non_inventory')
    flags = {flag.removeprefix('--').replace('-', '_'): True
             for flag in provenance['extractor_flags']}
    text = (SOURCES / f'{PATCH}-api-changes.txt').read_text()
    assert extractor.extract_text(raw_path.read_text(), **flags) == text
    headers = read_json(EVIDENCE / 'p901-header-accounting.json')
    raw_lines = raw_path.read_text().splitlines()
    current_section, directions = None, {}
    for number, literal in enumerate(raw_lines, 1):
        heading = re.fullmatch(r'==\s*(Global API|Widgets|Events|CVars)\s*==', literal)
        if heading:
            current_section = {'Global API': 'global-api', 'Widgets': 'widgets',
                               'Events': 'events', 'CVars': 'cvars'}[heading[1]]
        if literal.startswith('! style='):
            header = next(row for row in headers['headers'] if row['wikitext_line'] == number)
            assert header['literal'] == literal
            direction = 'added' if 'new ' in literal else 'removed'
            total = sum(map(int, re.findall(r'\b(\d+) (?:new|removed)', literal)))
            directions[(current_section, direction)] = total
    assert directions == Counter((row['section'], row['direction']) for row in register['entries'])
    return register, extractor, text, raw_lines


def check_accounting(register, extractor, text, raw_lines):
    results = read_json(EVIDENCE / 'patch_9_0_1_publication_sweep-results.json')
    known = set(read_json(ROOT / 'tests/data/patch_9_0_1_sweep_known_gaps.json'))
    assert {key for key, row in results.items() if not row['ok']} == known
    inventory_ids = {row['id'] for row in register['entries']}
    assert set(results) == inventory_ids
    reviews = read_json(EVIDENCE / 'p901-gap-review.json')
    assert {row['source_id'] for row in reviews} == known
    for row in reviews:
        assert row['literal'] == raw_lines[row['wikitext_line'] - 1]
        assert row['observation'] == results[row['source_id']]['observed']
        assert row['reason']
    scout = read_json(EVIDENCE / 'p901-extract-scout.json')
    extract_ids = {row['source_id'] for row in extractor.seed_rows(text, PATCH)}
    assert {row['source_id'] for row in scout} == extract_ids
    for row in scout:
        assert row['literal'] == text.splitlines()[row['extract_line'] - 1]
        assert row['raw_literal'] == raw_lines[row['wikitext_line'] - 1]
        assert row['reason']
    captions = read_json(EVIDENCE / 'p901-build-context.json')
    assert {row['wikitext_line'] for row in captions} == {
        number for number, line in enumerate(raw_lines, 1) if line.startswith('|+')}
    for row in captions:
        assert row['literal'] == raw_lines[row['wikitext_line'] - 1]
    ledger = read_json(SOURCES / f'{PATCH}-page-coverage.json')
    assert ledger['source_sha256'] == sha256(SOURCES / f'{PATCH}-wikitext-register.json')
    assert ledger['non_inventory_source']['sha256'] == sha256(SOURCES / f'{PATCH}-api-changes.txt')
    rows = ledger['source_rows']
    ids = [row['source_id'] for row in rows]
    assert len(ids) == len(set(ids))
    assert set(ids) == inventory_ids | extract_ids | {row['source_id'] for row in captions}
    indexed = {row['source_id']: row for row in rows}
    for key, result in results.items():
        expected_status = ('audit-pending' if key in known else 'bounded-coverage'
                           if result['expected']['publication'] == 'absent'
                           else 'partial-development-green')
        assert indexed[key]['status'] == expected_status
        assert bool(indexed[key]['capabilities']) == result['ok']
    for row in scout:
        assert indexed[row['source_id']]['status'] == row['status']
        assert not indexed[row['source_id']]['capabilities']
    negative = read_json(EVIDENCE / 'p901-negative-observation.json')
    receipt = read_json(EVIDENCE / 'p901-negative-result.json')
    failures = {key for key, row in negative.items() if not row['ok']}
    assert set(negative) == inventory_ids
    assert failures - known == {receipt['mutation']} and not known - failures
    assert receipt['baseline_gaps'] == len(known)
    assert receipt['negative_gaps'] == len(failures)
    return len(known), dict(Counter(row['status'] for row in rows))


def check_preservation():
    hashes = read_json(EVIDENCE / 'p901-input-hashes-before.json')
    for path, digest in hashes.items():
        assert preserved_input_matches(ROOT, path, digest), path
    before = read_json(EVIDENCE / 'p901-extract-before.json')
    after = read_json(EVIDENCE / 'p901-extract-after.json')
    indexed = {(row['patch'], row['preserve_examples']): row for row in after}
    for row in before:
        assert indexed[(row['patch'], row['preserve_examples'])] == row
    reproduced = read_json(EVIDENCE / 'p901-register-reproduction.json')
    assert {row['patch'] for row in reproduced} == {
        path.name.removesuffix('-wikitext-register.json')
        for path in historical_registers(ROOT, AUDIT_REVISION)}
    generator = load_tool('gen_patch_wikitext_register')
    for row in reproduced:
        path = SOURCES / f"{row['patch']}-wikitext-register.json"
        assert row['byte_identical'] and sha256(path) == row['sha256']
        assert reproduce_register(read_json(path), row['verified_flags'], generator) == path.read_bytes()
        provenance = SOURCES / f"{row['patch']}-api-changes.provenance.json"
        recorded = read_json(provenance).get('generator_flags') if provenance.exists() else None
        assert row['recorded_flags'] == recorded
        if recorded is not None:
            assert row['verified_flags'] == recorded
    return len(hashes), len(before), len(reproduced)


def check_proofs():
    for row in read_json(EVIDENCE / 'p901-proof.json'):
        if not row['invalidated']:
            assert row['exit'] == row.get('expected_exit', 0), row['scope']
        for path, digest in row['output_sha256'].items():
            assert sha256(EVIDENCE / path) == digest
    assert (EVIDENCE / 'p901-startup.stdout').read_text().strip() == '[]'
    warnings = [line for line in (EVIDENCE / 'p901-mists-check.log').read_text().splitlines()
                if line.startswith('warning:')]
    assert all('iced-wgpu-patched/Cargo.toml' in line or '`iced_wgpu` (manifest)' in line
               for line in warnings)
    summary = read_json(EVIDENCE / 'p901-sweep-summary.json')
    # Source files at the audit revision define the complete historical sweep set.
    source_tests = set()
    for path in historical_sweep_tests(ROOT, AUDIT_REVISION):
        if re.search(r'fn patch_[0-9_]+_publication_sweep\(', path.read_text()):
            source_tests.add(path.stem.removeprefix('patch_').removesuffix('_publication_sweep').replace('_', '.'))
    assert {row['patch'] for row in summary} == source_tests
    for row in summary:
        stem = 'patch_' + row['patch'].replace('.', '_')
        results = read_json(EVIDENCE / f'{stem}_publication_sweep-results.json')
        known = set(read_json(ROOT / f'tests/data/{stem}_sweep_known_gaps.json'))
        assert {key for key, value in results.items() if not value['ok']} == known
        assert row['rows'] == len(results) and row['gaps'] == len(known)
        assert row['ok'] + row['gaps'] == row['rows']
    return len(summary)


def main():
    register, extractor, text, raw_lines = check_sources()
    gaps, statuses = check_accounting(register, extractor, text, raw_lines)
    preserved, outcomes, registers = check_preservation()
    sweeps = check_proofs()
    print(json.dumps({'result': 'pass', 'inventory': len(register['entries']),
                      'ledger_statuses': statuses, 'gaps': gaps,
                      'preserved_inputs': preserved, 'extract_outcomes': outcomes,
                      'reproduced_registers': registers, 'publication_sweeps': sweeps}, indent=2))


if __name__ == '__main__':
    main()
