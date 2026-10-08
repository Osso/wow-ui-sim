"""Dynamically validate retained 8.1.5 audit evidence; no fixed row/test counts."""
from collections import Counter
import json
from pathlib import Path
import re
import subprocess
import sys

sys.dont_write_bytecode = True
from build_accounting import ROOT, EVIDENCE, SOURCES, PATCH, read, sha, load_extractor

sys.path.insert(0, str(ROOT / 'tools'))
from patch_audit_validation import historical_registers, preserved_input_matches

# First committed integration of the real 8.2.0 register; already contains the
# complete register set refreshed by this evidence.
AUDIT_REVISION = 'ee23154261a5cf08f253713ddc0f1d2c14cf251f'


def audit_patches():
    return {path.name.removesuffix('-wikitext-register.json')
            for path in historical_registers(ROOT, AUDIT_REVISION)}


def check_source():
    provenance = read(SOURCES / f'{PATCH}-api-changes.provenance.json')
    page = next(iter(read(EVIDENCE / 'p815-fetch.json')['query']['pages'].values()))
    revision = page['revisions'][0]
    path = SOURCES / f'{PATCH}-api-changes.wikitext'
    raw = path.read_text()
    register = read(SOURCES / f'{PATCH}-wikitext-register.json')
    assert revision['slots']['main']['*'] == raw
    assert page['pageid'] == provenance['pageid'] and page['title'] == provenance['title']
    assert revision['revid'] == provenance['revid'] == register['source']['revid']
    assert revision['timestamp'] == provenance['timestamp']
    assert sha(path) == provenance['wikitext_sha256'] == register['source']['sha256']
    assert read(EVIDENCE / 'p815-fetch-attempts.json')[-1]['status'] == 200
    listed = read(SOURCES / 'api-change-pages-remaining.json')['pages']
    assert any(row['pageid'] == page['pageid'] for row in listed)
    literals = set()
    for number, line in enumerate(raw.splitlines(), 1):
        for body in re.findall(r'\{\{api\|([^{}]+)\}\}', line):
            symbol = [part for part in body.split('|') if '=' not in part][-1]
            literals.add((number, symbol))
    entries = {(row['wikitext_line'], row['symbol']) for row in register['entries']}
    assert literals == entries and len(entries) == len(register['entries'])
    assert not register['header_counts'], 'This source has no numerical inventory headers'
    extractor = load_extractor()
    options = {flag.removeprefix('--').replace('-', '_'): True for flag in provenance['extractor_flags']}
    text = (SOURCES / f'{PATCH}-api-changes.txt').read_text()
    assert extractor.extract_text(raw, **options) == text
    return register, raw.splitlines(), text, extractor


def check_accounting(register, raw, text, extractor):
    results = read(EVIDENCE / 'patch_8_1_5_publication_sweep-results.json')
    known = set(read(ROOT / 'tests/data/patch_8_1_5_sweep_known_gaps.json'))
    inventory = {row['id'] for row in register['entries']}
    assert set(results) == inventory
    assert {key for key, result in results.items() if not result['ok']} == known
    reviews = read(EVIDENCE / 'p815-gap-review.json')
    assert {row['source_id'] for row in reviews} == known
    for row in reviews:
        assert row['literal'] == raw[row['wikitext_line'] - 1]
        assert row['observation'] == results[row['source_id']]['observed']
        assert row['expectation'] == results[row['source_id']]['expected'] and row['reason']
    ledger = read(SOURCES / f'{PATCH}-page-coverage.json')
    indexed = {row['source_id']: row for row in ledger['source_rows']}
    scout = read(EVIDENCE / 'p815-extract-scout.json')
    extract_ids = {row['source_id'] for row in extractor.seed_rows(text, PATCH)}
    assert len(indexed) == len(ledger['source_rows'])
    assert set(indexed) == inventory | extract_ids
    assert {row['source_id'] for row in scout} == extract_ids
    for key, result in results.items():
        row = indexed[key]
        assert (row['status'] == 'audit-pending') == (key in known)
        assert bool(row['capabilities']) == result['ok']
        if result['ok'] and result['expected']['publication'] == 'absent':
            assert row['status'] == 'bounded-coverage'
    for row in scout:
        assert row['literal'] == text.splitlines()[row['extract_line'] - 1]
        assert row['raw_literal'] == raw[row['wikitext_line'] - 1]
        assert indexed[row['source_id']]['note'] == row['reason']
        assert indexed[row['source_id']]['status'] == row['status'] == 'metadata-only'
        assert not indexed[row['source_id']]['capabilities']
    raw_accounted = {row['wikitext_line'] for row in register['entries']} | {row['wikitext_line'] for row in scout}
    assert raw_accounted == {number for number, line in enumerate(raw, 1) if line.strip()}
    assert ledger['source_sha256'] == sha(SOURCES / f'{PATCH}-wikitext-register.json')
    assert ledger['non_inventory_source']['sha256'] == sha(SOURCES / f'{PATCH}-api-changes.txt')
    return {'inventory_rows': len(inventory), 'extract_rows': len(extract_ids), 'ledger_rows': len(indexed),
            'gaps': len(known), 'publication_ok': len(inventory) - len(known),
            'statuses': dict(Counter(row['status'] for row in indexed.values()))}


def check_retirements():
    before = read(EVIDENCE / 'p815-removal-consumers.json')
    after = read(EVIDENCE / 'p815-whole-callers-after.json')
    assert before['cache_exists'] and after['cache_exists']
    initial = {row['symbol']: row for row in before['rows']}
    final = {row['symbol']: row for row in after['rows']}
    register = read(SOURCES / f'{PATCH}-wikitext-register.json')
    removals = {row['symbol'] for row in register['entries'] if row['direction'] == 'removed'}
    assert set(initial) == set(final) == removals
    retired = {key for key, row in initial.items() if row['decision'] == 'retire'}
    block = (ROOT / 'src/c_api/patch_retired_members.rs').read_text().split('const RETIRED_8_1_5_MEMBERS:', 1)[1].split('\n];', 1)[0]
    actual = set()
    for namespace, members in re.findall(r'"(C_[^"]+)"\s*,\s*&\[(.*?)\]', block, re.S):
        actual.update(namespace + '.' + member for member in re.findall(r'"([^"]+)"', members))
    assert actual == retired
    for stage in (initial, final):
        for symbol, row in stage.items():
            assert row['reason']
            for kind, scan in row['scans'].items():
                assert scan['exit'] in (0, 1) and not scan['stderr']
                assert scan['command'][0] == '/usr/bin/grep'
                assert '\\b' in ' '.join(scan['command'])
                assert '--exclude-dir=*Documentation*' in scan['command']
                if symbol in retired and kind in ('qualified', 'bare'):
                    assert scan['exit'] == 1 and not scan['stdout']
    for symbol in retired:
        assert initial[symbol]['scans']['callers']['exit'] == 1
    discovery = read(EVIDENCE / 'p815-discovery-results.json')
    final_results = read(EVIDENCE / 'patch_8_1_5_publication_sweep-results.json')
    closed = {row['expected']['symbol'] for key, row in discovery.items() if not row['ok'] and final_results[key]['ok']}
    assert closed == retired | {'C_ToyBoxInfo.NeedsFanfare'}
    assert read(EVIDENCE / 'p815-later-gap-closures.json') == []
    return {'retired_members': len(retired), 'removal_scan_candidates': len(removals),
            'behaviorally_modeled_missing_members': len(closed - retired)}


def check_negative(register):
    baseline = read(EVIDENCE / 'patch_8_1_5_publication_sweep-results.json')
    negative = read(EVIDENCE / 'p815-negative-observation.json')
    mutated = read(EVIDENCE / 'p815-negative-register.json')
    receipt = read(EVIDENCE / 'p815-negative-result.json')
    differences = [(a, b) for a, b in zip(register['entries'], mutated['entries']) if a != b]
    assert len(differences) == 1
    before, after = differences[0]
    assert before['id'] == receipt['mutation'] and before['direction'] == 'added'
    assert after == dict(before, direction='removed')
    assert set(negative) == set(baseline)
    original = {key for key, row in baseline.items() if not row['ok']}
    failures = {key for key, row in negative.items() if not row['ok']}
    assert sorted(failures - original) == receipt['new_gaps'] == [receipt['mutation']]
    assert sorted(original - failures) == receipt['resolved_gaps'] == []
    assert (len(original), len(failures)) == (receipt['baseline_gaps'], receipt['negative_gaps'])
    assert receipt['exit'] == read(EVIDENCE / 'p815-negative.proof.json')['exit'] != 0
    return {'negative_baseline_gaps': len(original), 'negative_gaps': len(failures)}


def check_reproduction():
    patches = audit_patches()
    registers = read(EVIDENCE / 'p815-register-reproduction.json')
    extracts = read(EVIDENCE / 'p815-saved-extract-reproduction.json')
    assert {row['patch'] for row in registers} == {row['patch'] for row in extracts} == patches
    assert all(row['byte_identical'] and row['exit'] == 0 for row in registers)
    inherited = {row['patch']: row for row in read(ROOT / 'data/patch-api/evidence/8.2.5-session-2026-10-08/p825-saved-extract-reproduction.json')}
    for row in extracts:
        if row['patch'] in inherited:
            old = inherited[row['patch']]
            assert (row['byte_identical'], row['error']) == (old['byte_identical'], old['error'])
        else:
            assert row['byte_identical']
    preservation = read(EVIDENCE / 'p815-input-preservation.json')['rows']
    assert all(row['unchanged'] and row['before'] == row['after']
               and preserved_input_matches(ROOT, row['path'], row['before']) for row in preservation)
    outcomes = read(EVIDENCE / 'p815-extract-preservation.json')
    assert {row['patch'] for row in outcomes} == patches - {PATCH}
    assert all(row['unchanged'] and row['before'] == row['after'] for row in outcomes)
    return {'registers_reproduced': len(registers), 'saved_extracts_checked': len(extracts),
            'inherited_extract_failures': [row['patch'] for row in extracts if not row['byte_identical']],
            'prior_inputs_preserved': len(preservation), 'prior_mode_outcomes_preserved': len(outcomes)}


def check_proof():
    required = ['p815-all-sweeps', 'p815-bare-final', 'p815-retirement-cached-final',
                'p815-mists-check', 'p815-mists-behavior', 'p815-default-check', 'p815-format',
                'p815-extract_patch_non_inventory-fixtures', 'p815-gen_patch_wikitext_register-fixtures',
                'p815-source-reproduction', 'p815-extract-cli-check', 'p815-negative',
                'p815-retail-build', 'p815-startup']
    required += ['p815-regression-' + scope for scope in (
        'toy', 'c_area_poi_probes', 'calendar', 'c_club_probes', 'pvp_info',
        'report_system', 'c_social_probes', 'date_and_time_deterministic_defaults', 'communities')]
    proofs = {}
    for path in EVIDENCE.glob('*.proof.json'):
        receipt = read(path)
        assert receipt['log_sha256'] == sha(EVIDENCE / receipt['log'])
        proofs[path.name.removesuffix('.proof.json')] = receipt
    for name in required:
        receipt = proofs[name]
        assert receipt['exit'] == receipt['expected_exit'] == (1 if name == 'p815-negative' else 0), name
        assert not receipt['invalidated']
        if receipt['command'][0] == 'cargo':
            changed = subprocess.run(['git', 'diff', '--name-only', receipt['revision'], 'HEAD', '--',
                                      'src', 'tests', 'build', 'build.rs', 'Cargo.toml', 'Cargo.lock'],
                                     cwd=ROOT, capture_output=True, text=True)
            assert changed.returncode == 0 and not changed.stdout, (name, changed.stdout)
    warnings = {}
    for name in ('p815-default-check', 'p815-mists-check'):
        lines = (EVIDENCE / proofs[name]['log']).read_text().splitlines()
        messages = [line for line in lines if line.startswith('warning:')]
        non_vendor = [line for line in messages if 'iced-wgpu-patched/Cargo.toml' not in line and '`iced_wgpu` (manifest)' not in line]
        assert not non_vendor, (name, non_vendor)
        warnings[name] = {'vendor_warnings': len(messages), 'non_vendor_warnings': len(non_vendor)}
    assert read(EVIDENCE / 'p815-startup.stdout') == []
    summaries = read(EVIDENCE / 'p815-sweep-summary.json')
    patches = audit_patches()
    assert {row['patch'] for row in summaries} == patches
    for row in summaries:
        register = read(SOURCES / (row['patch'] + '-wikitext-register.json'))
        assert row['rows'] == len(register['entries']) and row['ok'] + row['gaps'] == row['rows']
        assert row['exact_gap_identity']
    log = (EVIDENCE / proofs['p815-all-sweeps']['log']).read_text()
    observed = {patch.replace('.', '_') for patch in patches}
    passing = set(re.findall(r'test patch_([\d_]+)_publication_sweep::patch_[\d_]+_publication_sweep \.\.\. ok', log))
    assert passing == observed
    assert '0 failed' in log
    return {'publication_sweeps': len(summaries), 'inventory_rows_all_sweeps': sum(row['rows'] for row in summaries),
            'required_proofs': len(required), 'warnings': warnings}


if __name__ == '__main__':
    register, raw, text, extractor = check_source()
    summary = check_accounting(register, raw, text, extractor)
    summary.update(check_retirements())
    summary.update(check_negative(register))
    summary.update(check_reproduction())
    summary.update(check_proof())
    summary['validator'] = 'PASS'
    print(json.dumps(summary, indent=2))
