"""Validate retained 9.1.5 accounting without rerunning runtime gates."""
from collections import Counter
import hashlib
import importlib.util
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / 'tools'))
from patch_audit_validation import preserved_input_matches

AUDIT_REVISION = 'cf5bd7b47887625d2846be400e47e559bd0a4ddb'
EVIDENCE = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'


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
    raw = SOURCES / '9.1.5-api-changes.wikitext'
    text = SOURCES / '9.1.5-api-changes.txt'
    register = read_json(SOURCES / '9.1.5-wikitext-register.json')
    provenance = read_json(SOURCES / '9.1.5-api-changes.provenance.json')
    fetch = read_json(EVIDENCE / 'p915-fetch.json')['query']['pages'][0]
    revision = fetch['revisions'][0]
    assert fetch['pageid'] == provenance['pageid'] == 219137
    assert revision['revid'] == provenance['revid'] == register['source']['revid'] == 5920444
    assert revision['slots']['main']['content'] == raw.read_text()
    assert provenance['wikitext_sha256'] == register['source']['sha256'] == sha256(raw)
    assert provenance['generator_flags'] == [
        '--expand-shared-changes', '--capture-span-defaults']
    generator = load_tool('gen_patch_wikitext_register')
    buckets = generator.split_sections(raw.read_text())
    generated, counts = [], []
    for section in generator.SECTIONS.values():
        entries, headers = generator.parse_section(
            section, buckets.get(section, []), expand_shared_changes=True,
            capture_span_defaults=True)
        generated.extend(entries)
        counts.extend(headers)
    assert generated == register['entries']
    assert counts == register['header_counts']
    assert all(row['header_count'] == row['parsed_count'] for row in counts)
    assert Counter(row['direction'] for row in generated) == {'added': 109, 'removed': 60}
    assert len(generated) == 169
    extractor = load_tool('extract_patch_non_inventory')
    for preserve in (False, True):
        assert extractor.extract_text(raw.read_text(), preserve_examples=preserve) == text.read_text()
    return register, extractor


def check_accounting(register, extractor):
    ledger = read_json(SOURCES / '9.1.5-page-coverage.json')
    text = SOURCES / '9.1.5-api-changes.txt'
    raw = SOURCES / '9.1.5-api-changes.wikitext'
    scout = read_json(EVIDENCE / 'p915-extract-scout.json')
    context = read_json(EVIDENCE / 'p915-inventory-context.json')
    inventory_ids = {row['id'] for row in register['entries']}
    extract_ids = {row['source_id'] for row in extractor.seed_rows(text.read_text(), '9.1.5')}
    expected = inventory_ids | extract_ids | {row['source_id'] for row in context}
    actual = [row['source_id'] for row in ledger['source_rows']]
    assert len(actual) == len(set(actual)) == 188
    assert set(actual) == expected
    assert len(scout) == len(extract_ids) == 18
    assert {row['source_id'] for row in scout} == extract_ids
    for row in scout:
        assert row['literal'] == text.read_text().splitlines()[row['extract_line'] - 1]
        assert row['raw_literal'] == raw.read_text().splitlines()[row['wikitext_line'] - 1]
        assert row['reason']
    assert len(context) == 1
    assert context[0]['literal'].startswith('|+ 9.1.0')
    assert context[0]['literal'] == raw.read_text().splitlines()[context[0]['wikitext_line'] - 1]
    assert ledger['source_sha256'] == sha256(SOURCES / '9.1.5-wikitext-register.json')
    assert ledger['non_inventory_source']['sha256'] == sha256(text)
    counts = Counter(row['status'] for row in ledger['source_rows'])
    assert counts == {'bounded-coverage': 111, 'partial-development-green': 11,
                      'audit-pending': 55, 'metadata-only': 11}
    for row in ledger['source_rows']:
        assert row['note']
        if row['status'] in ('metadata-only', 'audit-pending'):
            assert row['capabilities'] == []
    return counts


def check_observations(register):
    observed = read_json(EVIDENCE / 'patch_9_1_5_publication_sweep-results.json')
    initial = read_json(EVIDENCE / 'p915-observation-initial.json')
    known = set(read_json(ROOT / 'tests/data/patch_9_1_5_sweep_known_gaps.json'))
    assert set(observed) == {row['id'] for row in register['entries']}
    assert {key for key, row in observed.items() if not row['ok']} == known
    assert len(known) == 47
    closed = {key for key, row in initial.items() if not row['ok']} - known
    assert len(closed) == 11
    reviews = read_json(EVIDENCE / 'p915-gap-review.json')
    assert {row['source_id'] for row in reviews} == known
    assert all(row['reason'] for row in reviews)
    scans = read_json(EVIDENCE / 'p915-removal-exact-consumers.json')
    assert closed <= {row['source_id'] for row in scans}
    for row in scans:
        if row['source_id'] in closed:
            assert row['qualified'] == []
            if row['symbol'] != 'C_LFGList.GetCategoryInfo':
                assert row['bare'] == []
    negative = read_json(EVIDENCE / 'p915-negative-observation.json')
    failures = {key for key, row in negative.items() if not row['ok']}
    assert set(negative) == set(observed)
    assert failures - known == {'wt-events-GAME_PAD_ACTIVE_CHANGED-183'}
    assert known - failures == set()
    assert len(failures) == 48
    assert read_json(EVIDENCE / 'p915-920-supersessions.json')['matches'] == []


def check_preservation():
    preserved = read_json(EVIDENCE / 'p915-input-hashes-before.json')
    for path, digest in preserved.items():
        assert preserved_input_matches(ROOT, path, digest), path
    assert len(preserved) == 148
    before = read_json(EVIDENCE / 'p915-extract-before.json')
    after = read_json(EVIDENCE / 'p915-extract-after.json')
    indexed = {(row['patch'], row['preserve_examples']): row for row in after}
    assert len(before) == 56
    for row in before:
        current = indexed[(row['patch'], row['preserve_examples'])]
        assert (row['exit'], row['stdout']) == (current['exit'], current['stdout'])
    assert sum(row['exit'] != 0 for row in before) == 16
    reproduced = read_json(EVIDENCE / 'p915-register-reproduction.json')
    assert len(reproduced) == 29
    for row in reproduced:
        assert row['byte_identical']
        assert sha256(SOURCES / f"{row['patch']}-wikitext-register.json") == row['sha256']
    return len(preserved)


def check_receipts():
    proof = read_json(EVIDENCE / 'p915-proof.json')
    for row in proof:
        assert not row['invalidated']
        assert row['exit'] == row.get('expected_exit', 0)
        for path, digest in row['output_sha256'].items():
            assert sha256(EVIDENCE / path) == digest
    assert (EVIDENCE / 'p915-startup.stdout').read_text().strip() == '[]'
    warnings = [line for line in (EVIDENCE / 'p915-mists-check.log').read_text().splitlines()
                if line.startswith('warning:')]
    assert len(warnings) == 7
    assert all('iced-wgpu-patched/Cargo.toml' in line or '`iced_wgpu` (manifest)' in line
               for line in warnings)
    summary = read_json(EVIDENCE / 'p915-sweep-summary.json')
    assert len(summary) == 29
    log = (EVIDENCE / 'p915-publication-sweeps.log').read_text()
    assert '30 passed; 0 failed; 30 total' in log
    for row in summary:
        name = 'patch_' + row['patch'].replace('.', '_') + '_publication_sweep'
        assert f'test {name}::{name} ... ok' in log
        assert row['rows'] == row['ok'] + row['gaps']
        observed = read_json(EVIDENCE / row['observation'])
        assert row['rows'] == len(observed)
        assert row['ok'] == sum(result['ok'] for result in observed.values())
    assert sum(row['rows'] for row in summary) == 6632
    return len(summary)


def main():
    register, extractor = check_sources()
    counts = check_accounting(register, extractor)
    check_observations(register)
    preserved = check_preservation()
    sweeps = check_receipts()
    print(json.dumps({'ledger_rows': 188, 'statuses': counts,
                      'preserved_inputs': preserved, 'preserved_mode_outcomes': 56,
                      'passing_publication_sweeps': sweeps}, indent=2))


if __name__ == '__main__':
    main()
