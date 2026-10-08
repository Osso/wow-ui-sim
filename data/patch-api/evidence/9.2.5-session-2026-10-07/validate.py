"""Validate retained 9.2.5 accounting without rerunning runtime tests."""
from collections import Counter
import hashlib
import importlib.util
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / 'tools'))
from patch_audit_validation import read_audit_json, preserved_input_matches

AUDIT_REVISION = 'b3f5906c0dc1432c8e5123b6748bed10f086d271'
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
    raw = SOURCES / '9.2.5-api-changes.wikitext'
    text = SOURCES / '9.2.5-api-changes.txt'
    register = read_json(SOURCES / '9.2.5-wikitext-register.json')
    provenance = read_json(SOURCES / '9.2.5-api-changes.provenance.json')
    assert provenance['revid'] == register['source']['revid'] == 2301036
    assert provenance['wikitext_sha256'] == register['source']['sha256'] == sha256(raw)
    fetch = read_json(EVIDENCE / 'p925-fetch.json')
    page = fetch['query']['pages']['237255']
    revision = page['revisions'][0]
    assert revision['revid'] == 2301036
    assert revision['slots']['main']['*'] == raw.read_text()
    extractor = load_tool('extract_patch_non_inventory')
    assert extractor.extract_text(raw.read_text(), preserve_examples=True) == text.read_text()
    assert extractor.extract_text(raw.read_text()) != text.read_text()
    generator = load_tool('gen_patch_wikitext_register')
    sections = generator.split_sections(raw.read_text(), separate_inline_structures=True)
    generated = []
    for section in generator.SECTIONS.values():
        entries, _ = generator.parse_section(
            section, sections.get(section, []), expand_shared_changes=True,
            capture_span_defaults=True)
        generated.extend(entries)
    assert generated == register['entries']
    assert Counter(row['direction'] for row in generated) == {'added': 55, 'removed': 18, 'changed': 11}
    assert all(row['header_count'] == row['parsed_count'] for row in register['header_counts'])
    return register, extractor


def check_accounting(register, extractor):
    ledger = read_json(SOURCES / '9.2.5-page-coverage.json')
    text = SOURCES / '9.2.5-api-changes.txt'
    raw = SOURCES / '9.2.5-api-changes.wikitext'
    scout = read_json(EVIDENCE / 'p925-extract-scout.json')
    context = read_json(EVIDENCE / 'p925-inventory-context.json')
    inventory = {row['id'] for row in register['entries']}
    extract = {row['source_id'] for row in extractor.seed_rows(text.read_text(), '9.2.5')}
    expected = inventory | extract | {row['source_id'] for row in context}
    actual = [row['source_id'] for row in ledger['source_rows']]
    assert len(actual) == len(set(actual)) == 220
    assert set(actual) == expected
    assert len(scout) == len(extract) == 135
    assert {row['source_id'] for row in scout} == extract
    for row in scout:
        assert row['literal'] == text.read_text().splitlines()[row['extract_line'] - 1]
        assert row['raw_literal'] == raw.read_text().splitlines()[row['wikitext_line'] - 1]
        assert row['reason']
    assert ledger['source_sha256'] == sha256(SOURCES / '9.2.5-wikitext-register.json')
    assert ledger['non_inventory_source']['sha256'] == sha256(text)
    counts = Counter(row['status'] for row in ledger['source_rows'])
    assert counts == {'bounded-coverage': 33, 'partial-development-green': 17,
                      'audit-pending': 154, 'metadata-only': 16}
    for row in ledger['source_rows']:
        if row['status'] in ('metadata-only', 'audit-pending'):
            assert row['capabilities'] == []
    return counts


def check_observations(register):
    observed = read_json(EVIDENCE / 'patch_9_2_5_publication_sweep-results.json')
    initial = read_json(EVIDENCE / 'p925-observation-initial.json')
    known = set(read_json(ROOT / 'tests/data/patch_9_2_5_sweep_known_gaps.json'))
    assert set(observed) == {row['id'] for row in register['entries']}
    assert {key for key, row in observed.items() if not row['ok']} == known
    assert len(known) == 34
    closed = {key for key, row in initial.items() if not row['ok']} - known
    assert len(closed) == 9
    reviews = read_json(EVIDENCE / 'p925-gap-review.json')
    assert {row['source_id'] for row in reviews} == known
    assert all(row['reason'] for row in reviews)
    scans = read_json(EVIDENCE / 'p925-removal-consumers.json')
    for scan in scans:
        if scan['source_id'] in closed:
            assert scan['qualified'] == scan['bare'] == []
    assert closed <= {row['source_id'] for row in scans}
    negative = read_json(EVIDENCE / 'p925-negative-observation.json')
    failures = {key for key, row in negative.items() if not row['ok']}
    assert set(negative) == set(observed)
    assert failures - known == {'wt-events-GAME_PAD_POWER_CHANGED-224'}
    assert known - failures == set()
    assert len(failures) == 35


def check_preservation():
    preserved = read_json(EVIDENCE / 'p925-input-hashes-before.json')
    for path, digest in preserved.items():
        assert preserved_input_matches(ROOT, path, digest), path
    before = read_json(EVIDENCE / 'p925-extract-before.json')
    after = read_json(EVIDENCE / 'p925-extract-after.json')
    indexed = {(row['patch'], row['preserve_examples']): row for row in after}
    assert len(before) == 54
    for row in before:
        current = indexed[(row['patch'], row['preserve_examples'])]
        assert (row['exit'], row['stdout']) == (current['exit'], current['stdout'])
    assert sum(row['exit'] != 0 for row in before) == 15
    return len(preserved)


def check_receipts():
    proof = read_json(EVIDENCE / 'p925-proof.json')
    for row in proof:
        assert not row['invalidated']
        assert row['exit'] == row.get('expected_exit', 0)
        for path, digest in row.get('output_sha256', {}).items():
            assert sha256(EVIDENCE / path) == digest
    assert (EVIDENCE / 'p925-startup.stdout').read_text().strip() == '[]'
    warnings = [line for line in (EVIDENCE / 'p925-mists-check.log').read_text().splitlines()
                if line.startswith('warning:')]
    assert len(warnings) == 7
    assert all('iced-wgpu-patched/Cargo.toml' in line or '`iced_wgpu` (manifest)' in line
               for line in warnings)
    table = read_json(EVIDENCE / 'p925-sweep-summary.json')
    assert len(table) == 28
    log = (EVIDENCE / 'p925-publication-sweeps.log').read_text()
    assert '29 passed; 0 failed; 29 total' in log
    for row in table:
        name = 'patch_' + row['patch'].replace('.', '_') + '_publication_sweep'
        assert f'test {name}::{name} ... ok' in log
        assert row['rows'] == row['ok'] + row['gaps']
        observed = read_json(EVIDENCE / row['observation'])
        assert row['rows'] == len(observed)
        assert row['ok'] == sum(result['ok'] for result in observed.values())
    return len(table)


def main():
    register, extractor = check_sources()
    counts = check_accounting(register, extractor)
    check_observations(register)
    preserved = check_preservation()
    sweeps = check_receipts()
    print(json.dumps({'ledger_rows': 220, 'statuses': counts, 'preserved_inputs': preserved,
                      'preserved_mode_outcomes': 54, 'passing_publication_sweeps': sweeps}, indent=2))


if __name__ == '__main__':
    main()
