"""Read-only historical 7.2.0 audit validation; checkout paths are not contracts."""
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT / 'tools'))
from patch_audit_validation import historical_registers, historical_sweep_tests, preserved_input_matches
from collections import Counter


def read(path):
    return json.loads(path.read_text())


def digest(data):
    return hashlib.sha256(data).hexdigest()


def blob(revision, path):
    return subprocess.check_output(['git', 'show', revision + ':' + path], cwd=ROOT)


def load_tool(name):
    spec = importlib.util.spec_from_file_location(name, ROOT / ('tools/' + name + '.py'))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def check_source():
    sources = ROOT / 'data/patch-api/sources'
    provenance = read(sources / '7.2.0-api-changes.provenance.json')
    page = read(HERE / 'p720-fetch.json')['query']['pages'][str(provenance['pageid'])]
    revision = page['revisions'][0]
    raw = (sources / '7.2.0-api-changes.wikitext').read_text()
    register = read(sources / '7.2.0-wikitext-register.json')
    assert provenance['pageid'] == 412195
    assert page['title'] == provenance['title'] == 'Patch 7.2.0/API changes'
    assert revision['revid'] == provenance['revid'] == register['source']['revid']
    assert revision['timestamp'] == provenance['timestamp']
    assert raw == revision['slots']['main']['*']
    assert digest(raw.encode()) == provenance['sha256'] == register['source']['sha256']
    generator = load_tool('gen_patch_wikitext_register')
    expected = generator.parse_legacy_widget_summaries(raw) + generator.parse_prose_namespace_migrations(raw)
    assert register['entries'] == expected
    extractor = load_tool('extract_patch_non_inventory')
    text = (sources / '7.2.0-api-changes.txt').read_text()
    assert extractor.extract_text(raw) == text
    return register, extractor, raw, text


def check_accounting(context, register, extractor, raw, text):
    ledger = json.loads(blob(context['runtime_revision'], 'data/patch-api/sources/7.2.0-page-coverage.json'))
    rows = {r['source_id']: r for r in ledger['source_rows']}
    assert len(rows) == len(ledger['source_rows'])
    entries = {r['id'] for r in register['entries']}
    results = read(HERE / 'patch_7_2_0_publication_sweep-results.json')
    known = set(json.loads(blob(context['runtime_revision'], 'tests/data/patch_7_2_0_sweep_known_gaps.json')))
    assert set(results) == entries
    assert {key for key, row in results.items() if not row['ok']} == known
    assert {r['source_id'] for r in read(HERE / 'p720-gap-review.json')} == known
    for key in entries:
        assert bool(rows[key]['capabilities']) == results[key]['ok']
    scout = read(HERE / 'p720-extract-scout.json')
    seeded = {r['source_id'] for r in extractor.seed_rows(text, '7.2.0')}
    assert {r['source_id'] for r in scout} == seeded
    assert set(rows) == entries | seeded
    for row in scout:
        assert row['literal'] == text.splitlines()[row['extract_line'] - 1]
        assert row['raw_literal'] == raw.splitlines()[row['wikitext_line'] - 1]
        assert row['reason'] == rows[row['source_id']]['note'] and row['reason']
        assert row['status'] == rows[row['source_id']]['status']
    summary = {'inventory_rows': len(entries), 'inventory_ok': len(entries) - len(known),
               'inventory_gaps': len(known), 'extract_rows': len(scout), 'ledger_rows': len(rows),
               'ledger_statuses': dict(Counter(r['status'] for r in rows.values())),
               'pending_prose': sum(r['status'] == 'audit-pending' for r in scout)}
    assert summary == read(HERE / 'p720-accounting-summary.json')
    for row in read(HERE / 'p720-problematic-contracts.json'):
        assert row['source_id'] in rows and row['reason'] == rows[row['source_id']]['note']
    return summary


def check_reproduction(context):
    expected = {p.name.removesuffix('-wikitext-register.json')
                for p in historical_registers(ROOT, context['runtime_revision'])}
    registers = read(HERE / 'p720-register-reproduction.json')
    extracts = read(HERE / 'p720-saved-extract-reproduction.json')
    assert {r['patch'] for r in registers} == {r['patch'] for r in extracts} == expected
    assert len(registers) == len(extracts) == len(expected)
    inherited = read(ROOT / 'data/patch-api/evidence/8.0.1-session-2026-10-08/p801-saved-extract-reproduction.json')
    failures = {r['patch']: (r['exit'], r['error']) for r in inherited if not r['byte_identical']}
    for row in registers:
        assert row['exit'] == 0 and row['byte_identical']
        assert digest((ROOT / ('data/patch-api/sources/' + row['patch'] + '-wikitext-register.json')).read_bytes()) == row['sha256']
    for row in extracts:
        assert row['byte_identical'] or failures.get(row['patch']) == (row['exit'], row['error'])
        assert digest((ROOT / ('data/patch-api/sources/' + row['patch'] + '-api-changes.txt')).read_bytes()) == row['sha256']
    for path, sha in read(HERE / 'p720-input-preservation.json')['sha256'].items():
        assert preserved_input_matches(ROOT, path, sha), path
    return len(expected)


def check_receipts(context):
    for label, command in context['passing_proofs'].items():
        receipt = read(HERE / (label + '.proof.json'))
        assert receipt['command'] == command and receipt['exit'] == 0, label
        log = (HERE / receipt['log']).read_bytes()
        assert digest(log) == receipt['log_sha256'], label
        subprocess.check_output(['git', 'rev-parse', '--verify', receipt['revision']], cwd=ROOT)
        if command[:2] == ['cargo', 'test']:
            matches = re.findall(r'test result: ok\. (\d+) passed; 0 failed;', log.decode())
            assert matches and sum(map(int, matches)) > 0, label
        if label in context['runtime_proofs']:
            for path in context['runtime_scope']:
                assert receipt['scope'][path] == digest(blob(context['runtime_revision'], path)), (label, path)
        if label == 'p720-mists-check':
            warnings = [line for line in log.decode().splitlines() if line.startswith('warning:')]
            assert all('iced-wgpu-patched/Cargo.toml:' in line or '`iced_wgpu` (manifest)' in line for line in warnings), warnings
    log = (HERE / 'p720-all-sweeps.log').read_text()
    passed = set(re.findall(r'^test (\S+) \.\.\. ok$', log, re.M))
    for path in historical_sweep_tests(ROOT, context['runtime_revision']):
        source = blob(context['runtime_revision'], str(path.relative_to(ROOT))).decode()
        for function in re.findall(r'fn (patch_\w+_publication_sweep)\(', source):
            assert any(name.endswith('::' + function) for name in passed), function
        patch = path.stem.removeprefix('patch_').removesuffix('_publication_sweep').replace('_', '.')
        register = read(ROOT / ('data/patch-api/sources/' + patch + '-wikitext-register.json'))
        results = read(HERE / (path.stem + '-results.json'))
        assert set(results) == {r['id'] for r in register['entries']}
        known = json.loads(blob(context['runtime_revision'], 'tests/data/patch_' + patch.replace('.', '_') + '_sweep_known_gaps.json'))
        assert {key for key, row in results.items() if not row['ok']} == set(known)
    return len(passed)


def check_scans_and_negative(register):
    for row in read(HERE / 'p720-scans.json'):
        assert row['scanner'] == 'grep' and row['argv'][0] == '/usr/bin/grep'
        assert '-w' in row['argv'] and row['exit'] in (0, 1)
        assert digest((HERE / row['output']).read_bytes()) == row['sha256']
    retirements = read(HERE / 'p720-retirement-decisions.json')
    assert retirements['removed_inventory_members'] == retirements['new_retirements'] == []
    assert not any(row['direction'] == 'removed' for row in register['entries'])
    names = {row['symbol'] for row in register['entries']}
    for row in read(HERE / 'p720-later-register-scan.json'):
        later = json.loads(blob(row['revision'], row['path']))
        assert row['entries'] == [entry for entry in later['entries'] if entry['symbol'] in names]
    negative = read(HERE / 'p720-negative-register.json')
    changes = [(a, b) for a, b in zip(register['entries'], negative['entries']) if a != b]
    assert len(changes) == 1 and len(register['entries']) == len(negative['entries'])
    before, after = changes[0]
    assert after == dict(before, symbol='P720_NONEXISTENT_NAMESPACE')
    result = read(HERE / 'p720-negative-results.json')
    assert {key for key, row in result.items() if not row['ok']} == {before['id']}
    proof = read(HERE / 'p720-negative.proof.json')
    assert proof['exit'] != 0 and digest((HERE / proof['log']).read_bytes()) == proof['log_sha256']
    assert 'resolved/stale gaps: []' in (HERE / proof['log']).read_text()


def main():
    context = read(HERE / 'p720-context.json')
    for path, sha in read(HERE / 'p720-artifact-hashes.json').items():
        assert digest((ROOT / path).read_bytes()) == sha, path
    register, extractor, raw, text = check_source()
    summary = check_accounting(context, register, extractor, raw, text)
    count = check_reproduction(context)
    sweeps = check_receipts(context)
    check_scans_and_negative(register)
    paths = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only',
                                     context['base_revision'], 'data/patch-api/evidence'],
                                    cwd=ROOT, text=True).splitlines()
    historical = {path for path in paths if path.endswith('/validate.py')}
    assert historical == set(context['prior_validators'])
    matrix = read(HERE / 'p720-prior-validator-matrix.json')
    assert set(matrix) == historical
    for path, result in context['prior_validators'].items():
        assert result == matrix[path]['exit'] == 0, path
    startup = read(HERE / 'p720-startup-comparison.json')
    assert startup['branch_errors'] == startup['master_errors'] == []
    for key in ('branch', 'master'):
        assert digest((ROOT / startup[key + '_log']).read_bytes()) == startup[key + '_log_sha256']
    unchanged = subprocess.check_output(['git', 'diff', '--name-only', context['base_revision'],
                                         context['runtime_revision'], '--', 'src', 'Cargo.toml',
                                         'Cargo.lock', 'build.rs', 'Interface', 'data/blizzard-ui-files'], cwd=ROOT)
    assert unchanged == b'', unchanged
    print(json.dumps({'status': 'PASS', 'registers': count, 'sweeps_with_factory': sweeps, **summary}, indent=2))


if __name__ == '__main__':
    main()
