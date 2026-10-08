"""Read-only historical proof; portable across checkouts and later page merges."""
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
AUDIT_REVISION = 'c5f05d05d'
sys.path.insert(0, str(ROOT / 'tools'))
from patch_audit_validation import historical_registers, historical_sweep_tests, preserved_input_matches


def read(path):
    return json.loads(path.read_text())


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def blob(revision, path):
    return subprocess.check_output(['git', 'show', f'{revision}:{path}'], cwd=ROOT)


def check_source():
    raw_path = SOURCES / '7.3.2-api-changes.wikitext'
    raw = raw_path.read_text()
    provenance = read(SOURCES / '7.3.2-api-changes.provenance.json')
    register = read(SOURCES / '7.3.2-wikitext-register.json')
    page = read(EVIDENCE / 'p732-fetch.json')['query']['pages'][str(provenance['pageid'])]
    revision = page['revisions'][0]
    assert revision['slots']['main']['*'] == raw
    assert revision['revid'] == provenance['revid'] == register['source']['revid']
    assert revision['timestamp'] == provenance['timestamp'] and page['title'] == provenance['title']
    assert sha(raw_path) == provenance['wikitext_sha256'] == register['source']['sha256']
    assert sha(EVIDENCE / 'p732-fetch.json') == read(EVIDENCE / 'p732-fetch-receipt.json')['sha256']
    references = [(number, match[1]) for number, line in enumerate(raw.splitlines(), 1)
                  for match in re.finditer(r'\[\[API ([^|\]]+)(?:\|[^\]]+)?\]\]', line)]
    assert Counter(references) == Counter((r['wikitext_line'], r['symbol']) for r in register['entries'])
    assert all(r['direction'] == 'changed' and r['annotation'] == raw.splitlines()[r['wikitext_line'] - 1]
               for r in register['entries'])
    spec = importlib.util.spec_from_file_location('extractor', ROOT / 'tools/extract_patch_non_inventory.py')
    extractor = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(extractor)
    options = {flag.removeprefix('--').replace('-', '_'): True for flag in provenance['extractor_flags']}
    text = (SOURCES / '7.3.2-api-changes.txt').read_text()
    assert extractor.extract_text(raw, **options) == text
    return register, extractor, text


def check_accounting(register, extractor, text):
    results = read(EVIDENCE / 'patch_7_3_2_publication_sweep-results.json')
    inventory = {r['id']: r for r in register['entries']}
    assert set(results) == set(inventory)
    gaps = set(read(ROOT / 'tests/data/patch_7_3_2_sweep_known_gaps.json'))
    assert {key for key, result in results.items() if not result['ok']} == gaps
    protection = read(EVIDENCE / 'p732-protection-green.json')
    assert set(protection) == {row['symbol'] for row in inventory.values()}
    assert all(row['ok'] and not row['call_succeeded'] and row['state_unchanged']
               and row['taint'] == 'Patch732Addon' for row in protection.values())
    red = read(EVIDENCE / 'p732-protection-red.json')
    assert set(red) == set(protection) and all(not row['ok'] for row in red.values())
    coverage = read(SOURCES / '7.3.2-page-coverage.json')
    assert coverage['source_sha256'] == sha(SOURCES / '7.3.2-wikitext-register.json')
    assert coverage['non_inventory_source']['sha256'] == sha(SOURCES / '7.3.2-api-changes.txt')
    rows = {row['source_id']: row for row in coverage['source_rows']}
    extract = extractor.seed_rows(text, '7.3.2')
    assert len(rows) == len(coverage['source_rows'])
    assert set(rows) == set(inventory) | {r['source_id'] for r in extract}
    for key in inventory:
        assert rows[key]['status'] == 'bounded-coverage' and rows[key]['capabilities']
    scout = read(EVIDENCE / 'p732-extract-scout.json')
    assert {r['source_id'] for r in scout} == {r['source_id'] for r in extract}
    raw_lines = (SOURCES / '7.3.2-api-changes.wikitext').read_text().splitlines()
    for row in scout:
        assert row['literal'] == text.splitlines()[row['extract_line'] - 1]
        assert row['raw_literal'] == raw_lines[row['wikitext_line'] - 1]
        assert rows[row['source_id']]['status'] == row['status']
        assert rows[row['source_id']]['note'] == row['reason']
    review = read(EVIDENCE / 'p732-gap-review.json')
    assert {row['source_id'] for row in review['publication_gaps']} == gaps
    assert not review['unmodeled_source_contracts']
    assert review['remaining_native_boundaries']
    return {'inventory_rows': len(inventory), 'extract_rows': len(extract), 'ledger_rows': len(rows),
            'publication_gaps': len(gaps), 'statuses': dict(Counter(row['status'] for row in rows.values()))}


def check_preservation():
    preservation = read(EVIDENCE / 'p732-input-preservation.json')
    expected_paths = subprocess.check_output(
        ['git', 'ls-tree', '-r', '--name-only', preservation['base_revision'], 'data/patch-api/sources'],
        cwd=ROOT, text=True).splitlines()
    assert {row['path'] for row in preservation['rows']} == set(expected_paths)
    for row in preservation['rows']:
        original = hashlib.sha256(blob(preservation['base_revision'], row['path'])).hexdigest()
        audited = hashlib.sha256(blob(preservation['audit_revision'], row['path'])).hexdigest()
        assert original == row['base_sha256'] == row['audit_sha256'] == audited
        assert preserved_input_matches(ROOT, row['path'], original), row['path']
    modes = read(EVIDENCE / 'p732-extract-preservation.json')
    assert all(row['before'] == row['after'] for row in modes)
    patches = {p.name.removesuffix('-wikitext-register.json') for p in historical_registers(ROOT, AUDIT_REVISION)}
    registers = read(EVIDENCE / 'p732-register-reproduction.json')
    assert {r['patch'] for r in registers} == patches
    assert all(r['exit'] == 0 and r['byte_identical'] for r in registers)
    extracts = read(EVIDENCE / 'p732-saved-extract-reproduction.json')
    assert {r['patch'] for r in extracts} == patches
    inherited = {r['patch']: r for r in read(ROOT / 'data/patch-api/evidence/8.1.0-session-2026-10-08/p810-saved-extract-reproduction.json')}
    for row in extracts:
        if row['patch'] in inherited:
            previous = inherited[row['patch']]
            assert (row['byte_identical'], row['error']) == (previous['byte_identical'], previous['error'])
        else:
            assert row['byte_identical']
    return {'preserved_inputs': len(expected_paths), 'preserved_extract_modes': len(modes),
            'reproduced_registers': len(registers),
            'inherited_extract_failures': [r['patch'] for r in extracts if not r['byte_identical']]}


def check_scans(register):
    scans = read(EVIDENCE / 'p732-removal-consumers.json')
    candidates = {r['symbol'] for r in register['entries'] if r['direction'] == 'removed'}
    assert candidates == set(scans['candidates']) == set(scans['retired'])
    assert scans['cache_exists']
    assert sha(EVIDENCE / '8.0.1-later-register.json') == scans['p801_register_sha256']
    assert {row['symbol'] for row in scans['rows']} == {row['symbol'] for row in register['entries']}
    for row in scans['rows']:
        assert not row['later_additions']
        for scan in row['scans'].values():
            assert scan['command'][0] == '/usr/bin/grep' and '\\b' in ' '.join(scan['command'])
            assert '--exclude-dir=*Documentation*' in scan['command']
            assert scan['exit'] in (0, 1) and not scan['stderr']
            assert bool(scan['stdout']) == (scan['exit'] == 0)
    return {'retired': sorted(candidates)}


def check_negative(register):
    baseline = read(EVIDENCE / 'patch_7_3_2_publication_sweep-results.json')
    negative = read(EVIDENCE / 'p732-negative-observation.json')
    modified = read(EVIDENCE / 'p732-negative-register.json')
    differences = [(a, b) for a, b in zip(register['entries'], modified['entries']) if a != b]
    assert len(differences) == 1
    before, after = differences[0]
    assert after == dict(before, direction='removed') and before['direction'] == 'changed'
    assert set(negative) == set(baseline)
    old = {key for key, row in baseline.items() if not row['ok']}
    new = {key for key, row in negative.items() if not row['ok']}
    assert new - old == {before['id']} and not old - new
    assert read(EVIDENCE / 'p732-negative.proof.json')['exit'] == 1
    return {'negative_baseline': len(old), 'negative_gaps': len(new)}


def check_proof():
    proof = read(EVIDENCE / 'p732-proof.json')
    for requirement in proof['required']:
        receipt = read(EVIDENCE / requirement['receipt'])
        assert receipt['exit'] == requirement['expected_exit'] and not receipt['invalidated']
        assert sha(EVIDENCE / receipt['log']) == receipt['log_sha256']
        for path in requirement.get('source_scope', []):
            assert blob(receipt['revision'], path) == blob(proof['code_revision'], path), path
    mists = read(EVIDENCE / 'p732-mists-check.proof.json')
    warnings = [line for line in (EVIDENCE / mists['log']).read_text().splitlines()
                if line.startswith('warning:') and not line.startswith(
                    ('warning: iced-wgpu-patched/', 'warning: `iced_wgpu`'))]
    assert not warnings, warnings
    expected = {p.stem + '-results.json' for p in historical_sweep_tests(ROOT, AUDIT_REVISION)}
    summaries = read(EVIDENCE / 'p732-sweep-summary.json')
    assert {row['file'] for row in summaries} == expected
    for row in summaries:
        results = read(EVIDENCE / row['file'])
        assert row['rows'] == len(results)
        assert row['gaps'] == sum(not result['ok'] for result in results.values())
        patch = row['file'].removeprefix('patch_').removesuffix('_publication_sweep-results.json')
        fixture = ROOT / f'tests/data/patch_{patch}_sweep_known_gaps.json'
        assert {key for key, result in results.items() if not result['ok']} == set(read(fixture))
    receipt = read(EVIDENCE / 'p732-all-sweeps.proof.json')
    log = (EVIDENCE / receipt['log']).read_text()
    passing = set(re.findall(r'test patch_([\d_]+)_publication_sweep::patch_[\d_]+_publication_sweep \.\.\. ok', log))
    assert passing == {p.name.removeprefix('patch_').removesuffix('_publication_sweep.rs')
                       for p in historical_sweep_tests(ROOT, AUDIT_REVISION)}
    assert '0 failed' in log
    return {'required_receipts': len(proof['required']), 'publication_sweeps': len(summaries),
            'non_vendor_mists_warnings': len(warnings)}


if __name__ == '__main__':
    register, extractor, text = check_source()
    summary = {**check_accounting(register, extractor, text), **check_preservation(),
               **check_scans(register), **check_negative(register), **check_proof()}
    print(json.dumps({'status': 'PASS', **summary}, indent=2))
