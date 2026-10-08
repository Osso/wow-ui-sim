"""Validate retained audit evidence after merge; no absolute checkout assertions."""
from collections import Counter
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'
PATCH = '8.1.0'
sys.path.insert(0, str(ROOT / 'tools'))
from patch_audit_validation import (
    historical_registers, historical_sweep_tests, preserved_input_matches,
)

# Integrated 8.1.5/8.2.0 registers and the complete 8.1.0 sweep source set.
AUDIT_REVISION = 'ba7e46ad6'


def audit_patches():
    return {path.name.removesuffix('-wikitext-register.json')
            for path in historical_registers(ROOT, AUDIT_REVISION)}


def read(path):
    return json.loads(path.read_text())


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git_blob(ref, path):
    return subprocess.check_output(['git', 'show', f'{ref}:{path}'], cwd=ROOT)


def load_extractor():
    spec = importlib.util.spec_from_file_location('extractor', ROOT / 'tools/extract_patch_non_inventory.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def check_source():
    raw_path = SOURCES / f'{PATCH}-api-changes.wikitext'
    raw = raw_path.read_text()
    provenance = read(SOURCES / f'{PATCH}-api-changes.provenance.json')
    register = read(SOURCES / f'{PATCH}-wikitext-register.json')
    page = read(EVIDENCE / 'p810-fetch.json')['query']['pages'][str(provenance['pageid'])]
    revision = page['revisions'][0]
    assert revision['slots']['main']['*'] == raw
    assert revision['revid'] == provenance['revid'] == register['source']['revid']
    assert revision['timestamp'] == provenance['timestamp'] and page['title'] == provenance['title']
    assert sha(raw_path) == provenance['wikitext_sha256'] == register['source']['sha256']
    assert sha(EVIDENCE / 'p810-fetch.json') == read(EVIDENCE / 'p810-fetch-receipt.json')['sha256']
    # Independently count EVERY raw API occurrence, including both rename sides.
    occurrences = []
    for number, line in enumerate(raw.splitlines(), 1):
        for body in re.findall(r'\{\{api\|([^{}]+)\}\}', line):
            symbol = [part for part in body.split('|') if '=' not in part][-1]
            occurrences.append((number, symbol))
    assert Counter(occurrences) == Counter((r['wikitext_line'], r['symbol']) for r in register['entries'])
    assert all(row['parsed_count'] == row['header_count'] for row in register['header_counts'])
    extractor = load_extractor()
    options = {flag.removeprefix('--').replace('-', '_'): True for flag in provenance['extractor_flags']}
    text = (SOURCES / f'{PATCH}-api-changes.txt').read_text()
    assert extractor.extract_text(raw, **options) == text
    return register, extractor, raw.splitlines(), text


def check_accounting(register, extractor, raw_lines, text):
    results = read(EVIDENCE / 'patch_8_1_0_publication_sweep-results.json')
    inventory = {row['id']: row for row in register['entries']}
    known = set(read(ROOT / 'tests/data/patch_8_1_0_sweep_known_gaps.json'))
    assert set(results) == set(inventory)
    assert {key for key, row in results.items() if not row['ok']} == known
    review = read(EVIDENCE / 'p810-gap-review.json')
    assert {row['source_id'] for row in review} == known
    for row in review:
        assert row['literal'] == raw_lines[row['wikitext_line'] - 1]
        assert row['observation'] == results[row['source_id']]['observed']
        assert row['expectation'] == results[row['source_id']]['expected'] and row['reason']
    ledger = read(SOURCES / f'{PATCH}-page-coverage.json')
    indexed = {row['source_id']: row for row in ledger['source_rows']}
    extract = extractor.seed_rows(text, PATCH)
    assert set(indexed) == set(inventory) | {row['source_id'] for row in extract}
    assert len(indexed) == len(ledger['source_rows'])
    assert ledger['source_sha256'] == sha(SOURCES / f'{PATCH}-wikitext-register.json')
    assert ledger['non_inventory_source']['sha256'] == sha(SOURCES / f'{PATCH}-api-changes.txt')
    for key, result in results.items():
        source = indexed[key]
        assert (source['status'] == 'audit-pending') == (key in known)
        assert bool(source['capabilities']) == result['ok']
    scout = read(EVIDENCE / 'p810-extract-scout.json')
    assert {row['source_id'] for row in scout} == {row['source_id'] for row in extract}
    for row in scout:
        assert row['literal'] == text.splitlines()[row['extract_line'] - 1]
        assert row['raw_literal'] == raw_lines[row['wikitext_line'] - 1]
        assert indexed[row['source_id']]['status'] == row['status']
        assert indexed[row['source_id']]['note'] == row['reason']
    return {'inventory_rows': len(inventory), 'extract_rows': len(extract),
            'ledger_rows': len(indexed), 'gaps': len(known),
            'statuses': dict(Counter(row['status'] for row in indexed.values()))}


def check_preservation():
    receipt = read(EVIDENCE / 'p810-input-preservation.json')
    for row in receipt['rows']:
        before = hashlib.sha256(git_blob(receipt['base_revision'], row['path'])).hexdigest()
        after = hashlib.sha256(git_blob(receipt['audit_revision'], row['path'])).hexdigest()
        assert before == row['base_sha256'] == after == row['audit_sha256']
        assert preserved_input_matches(ROOT, row['path'], row['base_sha256']), row['path']
    # Current replacements require exact attributable later-audit byte pairs.
    modes = read(EVIDENCE / 'p810-extract-preservation.json')
    assert all(row['before'] == row['after'] for row in modes)
    registers = read(EVIDENCE / 'p810-register-reproduction.json')
    assert {row['patch'] for row in registers} == audit_patches()
    assert all(row['byte_identical'] and row['exit'] == 0 for row in registers)
    extracts = read(EVIDENCE / 'p810-saved-extract-reproduction.json')
    assert {row['patch'] for row in extracts} == audit_patches()
    inherited = {row['patch']: row for row in read(
        ROOT / 'data/patch-api/evidence/8.1.5-session-2026-10-08/p815-saved-extract-reproduction.json')}
    for row in extracts:
        if row['patch'] in inherited:
            old = inherited[row['patch']]
            assert (row['byte_identical'], row['error']) == (old['byte_identical'], old['error'])
        else:
            assert row['byte_identical']
    return {'preserved_inputs': len(receipt['rows']), 'preserved_extract_modes': len(modes),
            'reproduced_registers': len(registers),
            'inherited_extract_failures': [row['patch'] for row in extracts if not row['byte_identical']]}


def check_scans(register):
    before = read(EVIDENCE / 'p810-removal-consumers.json')
    after = read(EVIDENCE / 'p810-whole-callers-after.json')
    candidates = {r['symbol'] for r in register['entries'] if r['direction'] == 'removed'}
    assert candidates == {r['symbol'] for r in before['rows']} == {r['symbol'] for r in after['rows']}
    retired = {r['symbol'] for r in before['rows'] if r['decision'] == 'retire'}
    for stage in (before, after):
        assert stage['cache_exists']
        for row in stage['rows']:
            for scan in row['scans'].values():
                assert scan['command'][0] == '/usr/bin/grep'
                assert '\\b' in ' '.join(scan['command'])
                assert scan['exit'] in (0, 1) and not scan['stderr']
                assert '--exclude-dir=*Documentation*' in scan['command']
            if row['symbol'] in retired:
                assert not row['later_readds']
                assert all(row['scans'][kind]['exit'] == 1 and not row['scans'][kind]['stdout']
                           for kind in ('qualified', 'bare'))
    return {'retirement_candidates': len(candidates), 'retired': sorted(retired)}


def check_negative(register):
    baseline = read(EVIDENCE / 'patch_8_1_0_publication_sweep-results.json')
    negative = read(EVIDENCE / 'p810-negative-observation.json')
    modified = read(EVIDENCE / 'p810-negative-register.json')
    changes = [(a, b) for a, b in zip(register['entries'], modified['entries']) if a != b]
    assert len(changes) == 1
    before, after = changes[0]
    assert after == dict(before, direction='removed') and before['direction'] == 'added'
    assert set(negative) == set(baseline)
    old = {key for key, row in baseline.items() if not row['ok']}
    new = {key for key, row in negative.items() if not row['ok']}
    assert new - old == {before['id']} and not old - new
    assert read(EVIDENCE / 'p810-integration-negative.proof.json')['exit'] == 1
    return {'negative_baseline': len(old), 'negative_gaps': len(new)}


def check_proof():
    proof = read(EVIDENCE / 'p810-proof.json')
    for row in proof['required']:
        receipt = read(EVIDENCE / row['receipt'])
        assert receipt['exit'] == row['expected_exit'] and not receipt['invalidated']
        assert sha(EVIDENCE / receipt['log']) == receipt['log_sha256']
        # Source scope, not HEAD equality: later docs/other audits do not invalidate proof.
        for scope in row.get('source_scope', []):
            assert git_blob(receipt['revision'], scope) == git_blob(proof['code_revision'], scope)
    check = read(EVIDENCE / 'p810-integration-mists-check.proof.json')
    lines = (EVIDENCE / check['log']).read_text().splitlines()
    non_vendor = [line for line in lines if line.startswith('warning:')
                  and not line.startswith(('warning: iced-wgpu-patched/', 'warning: `iced_wgpu`'))]
    assert not non_vendor, non_vendor
    summary = read(EVIDENCE / 'p810-sweep-summary.json')
    expected_files = {path.stem + '-results.json'
                      for path in historical_sweep_tests(ROOT, AUDIT_REVISION)}
    assert {row['file'] for row in summary} == expected_files
    for row in summary:
        results = read(EVIDENCE / row['file'])
        assert row['rows'] == len(results)
        assert row['gaps'] == sum(not value['ok'] for value in results.values())
        fixture = ROOT / f"tests/data/patch_{row['patch'].replace('.', '_')}_sweep_known_gaps.json"
        assert {key for key, value in results.items() if not value['ok']} == set(read(fixture))
    receipt = read(EVIDENCE / 'p810-integration-all-sweeps.proof.json')
    log = (EVIDENCE / receipt['log']).read_text()
    passing = set(re.findall(r'test patch_([\d_]+)_publication_sweep::patch_[\d_]+_publication_sweep \.\.\. ok', log))
    assert passing == {patch.replace('.', '_') for patch in audit_patches()}
    assert '0 failed' in log
    return {'required_receipts': len(proof['required']), 'publication_sweeps': len(summary),
            'non_vendor_mists_warnings': len(non_vendor)}


def check_supersessions():
    fixture = 'tests/data/patch_8_1_0_sweep_known_gaps.json'
    previous = set(json.loads(git_blob(AUDIT_REVISION, fixture)))
    current = set(read(ROOT / fixture))
    closures = read(EVIDENCE / 'p810-later-gap-closures.json')
    assert len(closures) == 1 and closures[0]['patch'] == PATCH
    assert set(closures[0]['resolved']) == previous - current
    assert not current - previous
    results = read(EVIDENCE / 'patch_8_1_0_publication_sweep-results.json')
    rows = {row['source_id']: row for row in read(SOURCES / f'{PATCH}-page-coverage.json')['source_rows']}
    for closure in closures[0]['closures']:
        identifier = closure['source_id']
        assert results[identifier]['ok']
        assert results[identifier]['expected']['superseded_by'] == closure['superseded_by']
        register = read(SOURCES / (closure['superseded_by_patch'] + '-wikitext-register.json'))
        assert any(row['id'] == closure['superseded_by'] and row['symbol'] == closure['symbol']
                   and row['direction'] == 'removed' for row in register['entries'])
        assert rows[identifier]['status'] == 'bounded-coverage'
        assert rows[identifier]['note'].startswith('Superseded by ' + closure['superseded_by_patch'] + ' removal')
    assert {row['source_id'] for row in closures[0]['closures']} == previous - current
    return {'integrated_gap_closures': len(previous - current)}


if __name__ == '__main__':
    register, extractor, lines, text = check_source()
    summary = {**check_accounting(register, extractor, lines, text),
               **check_preservation(), **check_scans(register),
               **check_negative(register), **check_proof(), **check_supersessions()}
    print(json.dumps({'status': 'PASS', **summary}, indent=2))
