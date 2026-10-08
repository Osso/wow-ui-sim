"""Read-only 7.0.3 historical evidence gate; checkout and later-page independent."""
from collections import Counter
from functools import cache
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


def read(path):
    return json.loads(path.read_text())


def sha(data):
    return hashlib.sha256(data).hexdigest()


@cache
def blob(revision, path):
    return subprocess.check_output(['git', 'show', f'{revision}:{path}'], cwd=ROOT)


def load_tool(name):
    spec = importlib.util.spec_from_file_location(name, ROOT / 'tools' / (name + '.py'))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def check_source():
    sources = ROOT / 'data/patch-api/sources'
    provenance = read(sources / '7.0.3-api-changes.provenance.json')
    page = read(HERE / 'p703-fetch.json')['query']['pages'][str(provenance['pageid'])]
    revision = page['revisions'][0]
    raw = (sources / '7.0.3-api-changes.wikitext').read_text()
    register = read(sources / '7.0.3-wikitext-register.json')
    assert provenance['pageid'] == 549091
    assert page['title'] == provenance['title'] == 'Patch 7.0.3/API changes'
    assert revision['revid'] == provenance['revid'] == register['source']['revid']
    assert revision['timestamp'] == provenance['timestamp']
    assert raw == revision['slots']['main']['*']
    assert sha(raw.encode()) == provenance['sha256'] == register['source']['sha256']
    generator = load_tool('gen_patch_wikitext_register')
    assert register['entries'] == generator.parse_legion_prepatch(raw)
    assert len(register['entries']) == len({e['id'] for e in register['entries']})
    extractor = load_tool('extract_patch_non_inventory')
    text = (sources / '7.0.3-api-changes.txt').read_text()
    assert extractor.extract_text(raw, legion_prepatch=True) == text
    return register, extractor, raw, text


def check_accounting(context, register, extractor, raw, text):
    ledger = json.loads(blob(context['accounting_revision'], 'data/patch-api/sources/7.0.3-page-coverage.json'))
    rows = {r['source_id']: r for r in ledger['source_rows']}
    assert len(rows) == len(ledger['source_rows'])
    entries = {r['id'] for r in register['entries']}
    results = read(HERE / 'patch_7_0_3_publication_sweep-results.json')
    known = set(json.loads(blob(context['retail_proof_revision'], 'tests/data/patch_7_0_3_sweep_known_gaps.json')))
    assert set(results) == entries
    assert {key for key, row in results.items() if not row['ok']} == known
    gaps = read(HERE / 'p703-gap-review.json')
    assert {r['source_id'] for r in gaps} == known
    for gap in gaps:
        assert gap['reason'] == rows[gap['source_id']]['note'] and gap['reason']
        assert gap['observed'] == results[gap['source_id']]['observed']
    for key in entries:
        assert bool(rows[key]['capabilities']) == results[key]['ok']
    scout = read(HERE / 'p703-extract-scout.json')
    seeded = {r['source_id'] for r in extractor.seed_rows(text, '7.0.3')}
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
    assert summary == read(HERE / 'p703-accounting-summary.json')
    assert read(HERE / 'p703-problematic-contracts.json') == [r for r in scout if r['status'] == 'audit-pending']
    return summary, known


def check_reproduction(context):
    expected = {p.name.removesuffix('-wikitext-register.json')
                for p in historical_registers(ROOT, context['retail_proof_revision'])}
    registers = read(HERE / 'p703-register-reproduction.json')
    extracts = read(HERE / 'p703-saved-extract-reproduction.json')
    assert {r['patch'] for r in registers} == {r['patch'] for r in extracts} == expected
    assert len(registers) == len(extracts) == len(expected)
    inherited = read(ROOT / 'data/patch-api/evidence/7.2.0-session-2026-10-08/p720-saved-extract-reproduction.json')
    failures = {r['patch']: (r['exit'], r['error']) for r in inherited if not r['byte_identical']}
    for row in registers:
        assert row['exit'] == 0 and row['byte_identical']
        path = ROOT / ('data/patch-api/sources/' + row['patch'] + '-wikitext-register.json')
        assert sha(path.read_bytes()) == row['sha256']
    for row in extracts:
        assert row['byte_identical'] or failures.get(row['patch']) == (row['exit'], row['error'])
        path = ROOT / ('data/patch-api/sources/' + row['patch'] + '-api-changes.txt')
        assert sha(path.read_bytes()) == row['sha256']
    for path, digest in read(HERE / 'p703-input-preservation.json')['sha256'].items():
        assert preserved_input_matches(ROOT, path, digest), path
    return len(expected)


def check_receipts(context):
    for label, policy in context['passing_proofs'].items():
        receipt = read(HERE / (label + '.proof.json'))
        assert receipt['command'] == policy['command'] and receipt['exit'] == 0, label
        log = (HERE / receipt['log']).read_bytes()
        assert sha(log) == receipt['log_sha256'], label
        subprocess.check_output(['git', 'rev-parse', '--verify', receipt['revision']], cwd=ROOT)
        if policy['command'][:2] == ['cargo', 'test']:
            matches = re.findall(r'test result: ok\. (\d+) passed; 0 failed;', log.decode())
            assert matches and sum(map(int, matches)) > 0, label
        if policy.get('runtime_revision'):
            for path in context['runtime_scope']:
                assert receipt['scope'][path] == sha(blob(policy['runtime_revision'], path)), (label, path)
        if label == 'p703-mists-fixed':
            warnings = [line for line in log.decode().splitlines() if line.startswith('warning:')]
            assert all('iced-wgpu-patched/Cargo.toml:' in line or '`iced_wgpu` (manifest)' in line for line in warnings), warnings
    old = blob(context['retail_proof_revision'], 'src/c_api/mod.rs').decode()
    new = blob(context['final_runtime_revision'], 'src/c_api/mod.rs').decode()
    before = '#[cfg(feature = "retail-12-0-0")]\npub(crate) mod c_trade_skill_filter;\npub mod c_trade_skill_quality;'
    after = 'pub(crate) mod c_trade_skill_filter;\n#[cfg(feature = "retail-12-0-0")]\npub mod c_trade_skill_quality;'
    assert old.count(before) == 1 and old.replace(before, after) == new
    for path in context['runtime_scope']:
        if path != 'src/c_api/mod.rs':
            assert blob(context['retail_proof_revision'], path) == blob(context['final_runtime_revision'], path), path
    # Both modules compile under default retail before/after the sole cfg repair.
    assert before.replace('#[cfg(feature = "retail-12-0-0")]\n', '') == after.replace('#[cfg(feature = "retail-12-0-0")]\n', '')
    log = (HERE / 'p703-all-sweeps.log').read_text()
    passed = set(re.findall(r'^test (\S+) \.\.\. ok$', log, re.M))
    for path in historical_sweep_tests(ROOT, context['retail_proof_revision']):
        source = blob(context['retail_proof_revision'], str(path.relative_to(ROOT))).decode()
        for function in re.findall(r'fn (patch_\w+_publication_sweep)\(', source):
            assert any(name.endswith('::' + function) for name in passed), function
        patch = path.stem.removeprefix('patch_').removesuffix('_publication_sweep').replace('_', '.')
        register = read(ROOT / ('data/patch-api/sources/' + patch + '-wikitext-register.json'))
        results = read(HERE / ('p703-all-sweeps-results.json' if patch == '7.0.3' else path.stem + '-results.json'))
        assert set(results) == {r['id'] for r in register['entries']}
        known = json.loads(blob(context['retail_proof_revision'], 'tests/data/patch_' + patch.replace('.', '_') + '_sweep_known_gaps.json'))
        assert {key for key, row in results.items() if not row['ok']} == set(known)
    return len(passed)


def check_scans_and_negative(register, known):
    scans = read(HERE / 'p703-retirement-scans.json')
    removed = {row['symbol'] for row in register['entries'] if row['direction'] == 'removed'}
    assert {row['symbol'] for row in scans} == removed
    assert len(scans) == len(removed) * 4
    for row in scans:
        assert row['scanner'] == 'grep' and row['argv'][0] == '/usr/bin/grep'
        assert '-w' in row['argv'] and row['exit'] in (0, 1)
        if row['domain'] == 'cached':
            assert '--exclude=*Documentation*' in row['argv']
            assert '--exclude-dir=*Documentation*' in row['argv']
        assert sha((HERE / row['output']).read_bytes()) == row['sha256']
    decisions = read(HERE / 'p703-retirement-decisions.json')
    assert {row['symbol'] for row in decisions['members']} == removed
    for symbol in decisions['new_retirements']:
        matches = [row for row in scans if row['symbol'] == symbol]
        assert len(matches) == 4 and all(row['matches'] == 0 and row['exit'] == 1 for row in matches)
        decision = next(row for row in decisions['members'] if row['symbol'] == symbol)
        assert decision['action'] == 'new-retirement' and not decision['later_additions']
    for row in read(HERE / 'p703-later-register-scan.json'):
        later = json.loads(blob(row['revision'], row['path']))
        assert row['entries'] == [entry for entry in later['entries'] if entry['symbol'] in removed]
        assert not any(entry['direction'] == 'added' and entry['symbol'] in decisions['new_retirements'] for entry in row['entries'])
    integrated = read(HERE / 'p703-integrated-710-read-only.json')
    later = json.loads(blob(integrated['revision'], integrated['path']))
    names = {entry['symbol'] for entry in register['entries']}
    assert integrated['entries'] == later['entries']
    assert integrated['intersections'] == [entry for entry in later['entries'] if entry['symbol'] in names] == []
    negative = read(HERE / 'p703-negative-register.json')
    changes = [(a, b) for a, b in zip(register['entries'], negative['entries']) if a != b]
    assert len(changes) == 1 and len(register['entries']) == len(negative['entries'])
    before, after = changes[0]
    assert after == dict(before, symbol='P703_NONEXISTENT_NAMESPACE')
    result = read(HERE / 'p703-negative-results.json')
    assert {key for key, row in result.items() if not row['ok']} == known | {before['id']}
    proof = read(HERE / 'p703-negative.proof.json')
    assert proof['exit'] != 0 and sha((HERE / proof['log']).read_bytes()) == proof['log_sha256']
    assert 'resolved/stale gaps: []' in (HERE / proof['log']).read_text()


def check_startup_and_prior(context):
    comparison = read(HERE / 'p703-startup-comparison.json')
    assert comparison['branch_errors'] == comparison['master_errors'] == []
    for key in ('branch', 'master'):
        receipt = read(HERE / comparison[key + '_receipt'])
        assert receipt['exit'] == 0
        assert '--no-addons' not in receipt['command']
        assert sha((HERE / receipt['log']).read_bytes()) == receipt['log_sha256']
        assert 'Lua errors: 0 unique, 0 occurrence(s)' in (HERE / receipt['log']).read_text()
    diagnostic = read(HERE / 'p703-crafting-diagnostic-comparison.json')
    assert diagnostic['branch'] == diagnostic['master'] and diagnostic['unchanged']
    for key in ('branch', 'master'):
        receipt = read(HERE / diagnostic[key + '_receipt'])
        assert receipt['exit'] == 0
        log = (HERE / receipt['log']).read_bytes()
        assert sha(log) == receipt['log_sha256']
        observed = dict(Counter(re.findall(r'Lua Error: ([^\n]+)', log.decode())))
        assert observed == diagnostic[key]
    snapshot = read(HERE / diagnostic['master_snapshot'])
    for path, digest in snapshot['scope'].items():
        assert sha(blob(snapshot['revision'], path)) == digest, path
    for row in read(HERE / 'p703-prior-sweep-status-comparison.json'):
        assert row['same_id_set'] and not row['status_changes']
        old = read(ROOT / row['baseline'])
        current = read(HERE / (row['sweep'] + '.json'))
        assert set(old) == set(current) and len(current) == row['rows']
        assert all(old[key]['ok'] == current[key]['ok'] for key in old)
    paths = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only',
                                     context['base_revision'], 'data/patch-api/evidence'], cwd=ROOT, text=True).splitlines()
    expected = {path for path in paths if re.search(r'/validate[^/]*\.py$', path)}
    matrix = read(HERE / 'p703-prior-validator-matrix.json')
    assert set(matrix) == expected
    assert all(row['exit'] == 0 for row in matrix.values())
    return len(expected)


def main():
    context = read(HERE / 'p703-context.json')
    for path, digest in read(HERE / 'p703-artifact-hashes.json').items():
        assert sha((ROOT / path).read_bytes()) == digest, path
    register, extractor, raw, text = check_source()
    summary, known = check_accounting(context, register, extractor, raw, text)
    count = check_reproduction(context)
    sweeps = check_receipts(context)
    check_scans_and_negative(register, known)
    prior = check_startup_and_prior(context)
    print(json.dumps({'status': 'PASS', **summary, 'historical_registers': count,
                      'sweep_cases': sweeps, 'prior_validators': prior}, indent=2))


if __name__ == '__main__':
    main()
