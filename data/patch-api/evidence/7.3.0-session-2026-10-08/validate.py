#!/usr/bin/env python3
"""Read-only 7.3.0 historical proof, independent of checkout path/later audits."""
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import subprocess
import sys
from collections import Counter

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
SOURCES = ROOT / 'data/patch-api/sources'
sys.path.insert(0, str(ROOT / 'tools'))
from patch_audit_validation import historical_registers, historical_sweep_tests, preserved_input_matches


def read(path):
    return json.loads(path.read_text())


def sha(data):
    return hashlib.sha256(data).hexdigest()


def blob(revision, path):
    return subprocess.check_output(['git', 'show', f'{revision}:{path}'], cwd=ROOT)


def load_tool(name):
    spec = importlib.util.spec_from_file_location(name, ROOT / 'tools' / (name + '.py'))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def check_artifacts():
    seal = read(HERE / 'p730-artifact-hashes.json')
    for name, digest in seal.items():
        assert sha((ROOT / name).read_bytes()) == digest, f'changed retained artifact: {name}'
    return len(seal)


def check_source():
    provenance = read(SOURCES / '7.3.0-api-changes.provenance.json')
    page = read(HERE / 'p730-fetch.json')['query']['pages'][str(provenance['pageid'])]
    revision = page['revisions'][0]
    raw = (SOURCES / '7.3.0-api-changes.wikitext').read_text()
    register = read(SOURCES / '7.3.0-wikitext-register.json')
    assert page['title'] == provenance['title'] == 'Patch 7.3.0/API changes'
    assert revision['revid'] == provenance['revid'] == register['source']['revid']
    assert revision['timestamp'] == provenance['timestamp']
    assert revision['slots']['main']['*'] == raw
    assert sha(raw.encode()) == provenance['sha256'] == register['source']['sha256']
    assert load_tool('gen_patch_wikitext_register').parse_legacy_summary_tables(raw) == register['entries']
    flags = {flag.removeprefix('--').replace('-', '_'): True for flag in provenance['extractor_flags']}
    extractor = load_tool('extract_patch_non_inventory')
    text = (SOURCES / '7.3.0-api-changes.txt').read_text()
    assert extractor.extract_text(raw, **flags) == text
    return register, extractor, raw, text


def check_accounting(context, register, extractor, raw, text):
    path = 'data/patch-api/sources/7.3.0-page-coverage.json'
    ledger_bytes = blob(context['accounting_revision'], path)
    assert sha(ledger_bytes) == context['accounting_sha256']
    ledger = json.loads(ledger_bytes)
    rows = {row['source_id']: row for row in ledger['source_rows']}
    ids = {row['id'] for row in register['entries']}
    assert len(ids) == len(register['entries']) and len(rows) == len(ledger['source_rows'])
    results = read(HERE / 'patch_7_3_0_publication_sweep-results.json')
    known = set(json.loads(blob(context['runtime_revision'], 'tests/data/patch_7_3_0_sweep_known_gaps.json')))
    assert set(results) == ids
    assert {key for key, value in results.items() if not value['ok']} == known
    assert {row['source_id'] for row in read(HERE / 'p730-gap-review.json')} == known
    for entry in register['entries']:
        row = rows[entry['id']]
        assert (row['status'] == 'audit-pending') == (entry['id'] in known)
        assert bool(row['capabilities']) == results[entry['id']]['ok']
    seeded = {row['source_id'] for row in extractor.seed_rows(text, '7.3.0')}
    scout = read(HERE / 'p730-extract-scout.json')
    assert {row['source_id'] for row in scout} == seeded
    assert set(rows) == ids | seeded
    assert ledger['source_sha256'] == sha((SOURCES / '7.3.0-wikitext-register.json').read_bytes())
    assert ledger['non_inventory_source']['sha256'] == sha(text.encode())
    for row in scout:
        assert row['literal'] == text.splitlines()[row['extract_line'] - 1]
        assert row['raw_literal'] == raw.splitlines()[row['wikitext_line'] - 1]
        assert row['status'] == rows[row['source_id']]['status']
        assert row['reason'] == rows[row['source_id']]['note'] and row['reason']
    summary = {'inventory_rows': len(ids), 'inventory_ok': len(ids) - len(known),
               'inventory_gaps': len(known), 'extract_rows': len(scout), 'ledger_rows': len(rows),
               'ledger_statuses': dict(Counter(row['status'] for row in rows.values())),
               'pending_prose': sum(row['status'] == 'audit-pending' for row in scout)}
    assert summary == read(HERE / 'p730-accounting-summary.json')
    for row in read(HERE / 'p730-problematic-contracts.json'):
        assert row['source_id'] in rows and row['reason'] == rows[row['source_id']]['note']
    return summary


def check_reproduction(context):
    expected = {path.name.removesuffix('-wikitext-register.json')
                for path in historical_registers(ROOT, context['runtime_revision'])}
    registers = read(HERE / 'p730-register-reproduction.json')
    extracts = read(HERE / 'p730-saved-extract-reproduction.json')
    assert {row['patch'] for row in registers} == {row['patch'] for row in extracts} == expected
    assert len(registers) == len(extracts) == len(expected)
    inherited = read(ROOT / 'data/patch-api/evidence/8.0.1-session-2026-10-08/p801-saved-extract-reproduction.json')
    failures = {row['patch']: (row['exit'], row['error']) for row in inherited if not row['byte_identical']}
    for row in registers:
        assert row['exit'] == 0 and row['byte_identical']
        assert sha((SOURCES / (row['patch'] + '-wikitext-register.json')).read_bytes()) == row['sha256']
    for row in extracts:
        assert sha((SOURCES / (row['patch'] + '-api-changes.txt')).read_bytes()) == row['sha256']
        assert row['byte_identical'] or failures.get(row['patch']) == (row['exit'], row['error'])
    preservation = read(HERE / 'p730-input-preservation.json')
    for name, digest in preservation['sha256'].items():
        assert preserved_input_matches(ROOT, name, digest), f'unrecorded earlier input drift: {name}'
    return len(expected), sum(row['byte_identical'] for row in extracts)


def check_receipts(context):
    for label in context['passing_proofs']:
        receipt = read(HERE / (label + '.proof.json'))
        log = HERE / receipt['log']
        assert receipt['exit'] == 0, label
        assert sha(log.read_bytes()) == receipt['log_sha256'], label
        assert receipt['command'], label
        subprocess.check_output(['git', 'rev-parse', '--verify', receipt['revision']], cwd=ROOT)
        text = log.read_text()
        if receipt['command'][0] == 'cargo' and receipt['command'][1] == 'test':
            match = re.search(r'test result: ok\. (\d+) passed; 0 failed;', text)
            assert match and int(match[1]) > 0, f'empty/unproven test selection: {label}'
        if label in context['runtime_proofs']:
            for name in context['runtime_scope']:
                assert receipt['scope'][name] == sha(blob(context['runtime_revision'], name)), (label, name)
        if label == 'p730-mists-final':
            assert receipt['command'] == ['cargo', 'check', '--no-default-features', '--features',
                                         'sound,gui,casc,client-mists', '--tests']
            assert all('iced-wgpu-patched/Cargo.toml:' in line or '`iced_wgpu` (manifest)' in line
                       for line in text.splitlines() if line.startswith('warning:')), 'non-vendor warning'
        if label == 'p730-startup':
            assert text.rstrip().endswith('[]') and 'Lua errors: 0 unique' in text
    all_sweeps = (HERE / 'p730-all-sweeps.log').read_text()
    passed = set(re.findall(r'^test (\S+) \.\.\. ok$', all_sweeps, re.M))
    sweep_count = 0
    for path in historical_sweep_tests(ROOT, context['runtime_revision']):
        relative = str(path.relative_to(ROOT))
        source = blob(context['runtime_revision'], relative).decode()
        functions = re.findall(r'fn (patch_\w+_publication_sweep)\(', source)
        assert functions, relative
        for function in functions:
            assert any(name.endswith('::' + function) for name in passed), function
            sweep_count += 1
        patch = path.stem.removeprefix('patch_').removesuffix('_publication_sweep').replace('_', '.')
        register = read(SOURCES / (patch + '-wikitext-register.json'))
        results = read(HERE / (path.stem + '-results.json'))
        assert set(results) == {row['id'] for row in register['entries']}
        gap_path = f'tests/data/patch_{patch.replace(".", "_")}_sweep_known_gaps.json'
        known = set(json.loads(blob(context['runtime_revision'], gap_path)))
        assert {key for key, row in results.items() if not row['ok']} == known, patch
    return sweep_count, len(passed)


def check_negative(register):
    modified = read(HERE / 'p730-negative-register.json')
    changed = [(before, after) for before, after in zip(register['entries'], modified['entries']) if before != after]
    assert len(register['entries']) == len(modified['entries']) and len(changed) == 1
    before, after = changed[0]
    assert after == dict(before, symbol='P730_NEGATIVE_NONEXISTENT_TABLE')
    results = read(HERE / 'p730-negative-results.json')
    assert {key for key, row in results.items() if not row['ok']} == {before['id']}
    receipt = read(HERE / 'p730-negative.proof.json')
    assert receipt['exit'] == 1 and sha((HERE / receipt['log']).read_bytes()) == receipt['log_sha256']
    assert 'resolved/stale gaps: []' in (HERE / receipt['log']).read_text()


def check_scans_and_prior(context):
    for row in read(HERE / 'p730-scans.json'):
        assert row['scanner'] == 'grep' and row['argv'][0] == '/usr/bin/grep'
        assert '-w' in row['argv'] and row['exit'] in (0, 1)
        assert sha((HERE / row['output']).read_bytes()) == row['sha256']
    decisions = read(HERE / 'p730-retirement-decisions.json')
    assert decisions['removed_inventory_members'] == decisions['new_retirements'] == []
    matrix = read(HERE / 'p730-prior-validator-matrix.json')
    paths = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', context['base_revision'],
                                     'data/patch-api/evidence'], cwd=ROOT, text=True).splitlines()
    expected = {path for path in paths if path.endswith('/validate.py')}
    assert {row['path'] for row in matrix['validators']} == expected
    assert all(row['exit'] == 0 for row in matrix['validators'])
    return len(expected)


def main():
    context = read(HERE / 'p730-context.json')
    sealed = check_artifacts()
    register, extractor, raw, text = check_source()
    summary = check_accounting(context, register, extractor, raw, text)
    registers, extracts = check_reproduction(context)
    sweeps, cases = check_receipts(context)
    check_negative(register)
    prior = check_scans_and_prior(context)
    print(json.dumps({'status': 'PASS', **summary, 'registers_reproduced': registers,
                      'extracts_reproduced': extracts, 'page_sweeps': sweeps,
                      'sweep_cases': cases, 'prior_validators': prior,
                      'sealed_artifacts': sealed}, sort_keys=True))


if __name__ == '__main__':
    main()
