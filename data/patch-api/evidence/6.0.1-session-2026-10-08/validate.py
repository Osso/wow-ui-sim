"""Read-only redirect proof with historical shared inputs and sealed own evidence."""
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT / 'tools'))
from patch_audit_validation import historical_registers, historical_sweep_tests


def read_json(path):
    return json.loads(path.read_text())


def digest(data):
    return hashlib.sha256(data).hexdigest()


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT)


def blob(revision, path):
    return git('show', f'{revision}:{path}')


def historical_json(revision, path):
    return json.loads(blob(revision, path))


def check_source(context):
    revision = context['source_revision']
    prefix = 'data/patch-api/sources/6.0.1-'
    provenance = historical_json(revision, prefix + 'api-changes.provenance.json')
    page = read_json(HERE / 'p601-fetch.json')['query']['pages'][str(provenance['pageid'])]
    fetched = page['revisions'][0]
    raw = blob(revision, prefix + 'api-changes.wikitext')
    assert page['pageid'] == 3058
    assert page['title'] == provenance['title'] == 'Patch 6.0.1/API changes'
    assert fetched['revid'] == provenance['revid'] == 31159
    assert fetched['timestamp'] == provenance['timestamp']
    assert raw.decode() == fetched['slots']['main']['*'] == '#REDIRECT [[Patch 6.0.2/API changes]]'
    assert digest(raw) == provenance['sha256']
    assert provenance['classification'] == 'redirect-only' and provenance['redirect']
    assert provenance['generator_flags'] == provenance['extractor_flags'] == []
    register = historical_json(revision, prefix + 'wikitext-register.json')
    assert register['source']['revid'] == fetched['revid']
    assert register['source']['sha256'] == digest(raw)
    assert register['entries'] == register['header_counts'] == []
    text = blob(revision, prefix + 'api-changes.txt')
    assert text == b'#REDIRECT Patch 6.0.2/API changes\n'
    ledger = historical_json(revision, prefix + 'page-coverage.json')
    assert ledger['source_sha256'] == digest(blob(revision, prefix + 'wikitext-register.json'))
    assert ledger['non_inventory_source']['sha256'] == digest(text)
    assert {row['source_id'] for row in ledger['source_rows']} == {'source-context-001'}
    assert all(row['status'] == 'metadata-only' and row['capabilities'] == []
               for row in ledger['source_rows'])
    assert historical_json(revision, 'tests/data/patch_6_0_1_sweep_known_gaps.json') == {}
    gaps = read_json(HERE / 'p601-gap-review.json')
    problems = read_json(HERE / 'p601-problematic-contracts.json')
    decisions = read_json(HERE / 'p601-retirement-decisions.json')
    assert gaps == problems == decisions['removed_entries'] == decisions['new_retirements'] == decisions['scans'] == []
    assert read_json(HERE / 'p601-accounting-summary.json') == {
        'classification': provenance['classification'], 'inventory_rows': len(register['entries']),
        'inventory_gaps': len(gaps), 'modeled_gaps': 0, 'problematic_gaps': len(problems),
        'extract_rows': len(ledger['source_rows']), 'metadata_rows': len(ledger['source_rows']),
        'retirements': len(decisions['new_retirements']),
    }
    for snapshot in decisions['register_snapshots']:
        names = git('ls-tree', '-r', '--name-only', snapshot['revision'], 'data/patch-api/sources').decode()
        assert (HERE / snapshot['tree_log']).read_text() == names
        assert snapshot['registers'] == [name for name in names.splitlines()
                                         if name.endswith('-wikitext-register.json')]


def check_reproduction(context):
    revision = context['source_revision']
    expected = {path.name.removesuffix('-wikitext-register.json')
                for path in historical_registers(ROOT, revision)}
    registers = read_json(HERE / 'p601-register-reproduction.json')
    extracts = read_json(HERE / 'p601-saved-extract-reproduction.json')
    assert {row['patch'] for row in registers} == {row['patch'] for row in extracts} == expected
    assert len(registers) == len(extracts) == len(expected)
    inherited = historical_json(context['base_revision'],
        'data/patch-api/evidence/7.0.3-session-2026-10-08/p703-saved-extract-reproduction.json')
    failures = {row['patch']: (row['exit'], row['error']) for row in inherited if not row['byte_identical']}
    for row in registers:
        assert row['exit'] == 0 and row['byte_identical'], row['patch']
        path = f"data/patch-api/sources/{row['patch']}-wikitext-register.json"
        assert digest(blob(revision, path)) == row['sha256'], path
    for row in extracts:
        assert row['byte_identical'] or failures.get(row['patch']) == (row['exit'], row['error'])
        path = f"data/patch-api/sources/{row['patch']}-api-changes.txt"
        assert digest(blob(revision, path)) == row['sha256'], path
    return len(registers), sum(row['byte_identical'] for row in extracts)


def check_proofs(context):
    for label, policy in context['proofs'].items():
        receipt = read_json(HERE / (label + '.proof.json'))
        assert receipt['command'] == policy['command'] and receipt['exit'] == policy['exit'], label
        log = (HERE / receipt['log']).read_bytes()
        assert digest(log) == receipt['log_sha256'], label
        assert not git('diff', '--name-only', context['source_revision'], receipt['revision'], '--',
                       'src', 'tests', 'tools', 'Cargo.toml', 'Cargo.lock', 'build.rs',
                       'data/patch-api/sources'), label
        for name in context['proof_scope']:
            assert receipt['scope'][name] == digest(blob(receipt['revision'], name)), (label, name)
        if label in ('p601-discovery', 'p601-all-sweeps'):
            assert re.search(r'test result: ok\. [1-9]\d* passed; 0 failed;', log.decode()), label
        if label == 'p601-mists-check':
            warnings = [line for line in log.decode().splitlines() if line.startswith('warning:')]
            assert all('iced-wgpu-patched/Cargo.toml:' in line or '`iced_wgpu` (manifest)' in line
                       for line in warnings), warnings
    assert read_json(HERE / 'p601-discovery-results.json') == {}
    assert read_json(HERE / 'p601-all-sweeps-results.json') == {}
    log = (HERE / 'p601-all-sweeps.log').read_text()
    passed = set(re.findall(r'^test (\S+) \.\.\. ok$', log, re.M))
    revision = read_json(HERE / 'p601-all-sweeps.proof.json')['revision']
    for path in historical_sweep_tests(ROOT, revision):
        source = blob(revision, path.relative_to(ROOT).as_posix()).decode()
        for function in re.findall(r'fn (patch_\w+_publication_sweep)\(', source):
            assert any(name.endswith('::' + function) for name in passed), function
        patch = path.stem.removeprefix('patch_').removesuffix('_publication_sweep').replace('_', '.')
        register = historical_json(revision, f'data/patch-api/sources/{patch}-wikitext-register.json')
        result_path = 'p601-all-sweeps-results.json' if patch == '6.0.1' else path.stem + '-results.json'
        results = read_json(HERE / result_path)
        assert set(results) == {row['id'] for row in register['entries']}, patch
        known = historical_json(revision, f'tests/data/patch_{patch.replace(".", "_")}_sweep_known_gaps.json')
        assert {key for key, row in results.items() if not row['ok']} == set(known), patch
    return len(passed)


def check_preservation(context):
    baseline = read_json(HERE / 'p601-wiki-baseline.json')
    for path, entry in baseline.items():
        original = blob(context['base_revision'], path)
        updated = blob(context['documentation_revision'], path)
        assert digest(original) == entry['sha256']
        assert len(original.decode().splitlines()) == entry['lines']
        assert len(updated.decode().splitlines()) >= entry['lines']
        assert original in updated, path
    names = git('ls-tree', '-r', '--name-only', context['base_revision'],
                'data/patch-api/evidence').decode().splitlines()
    prior = [name for name in names if name.endswith('/validate.py')]
    assert prior == context['prior_validators']
    for path in prior:
        assert blob(context['base_revision'], path) == blob(context['source_revision'], path), path
    assert not git('diff', '--name-only', context['base_revision'], context['source_revision'], '--',
                   'src', 'tools', 'Cargo.toml', 'Cargo.lock', 'build.rs')
    return len(prior)


def main():
    context = read_json(HERE / 'p601-context.json')
    for name, expected in context['session_sha256'].items():
        path = HERE / name
        assert path.is_file() and digest(path.read_bytes()) == expected, name
    check_source(context)
    registers, extracts = check_reproduction(context)
    sweeps = check_proofs(context)
    prior = check_preservation(context)
    print(json.dumps({'patch': '6.0.1', 'classification': 'redirect-only',
                      'registers': registers, 'extracts': extracts, 'sweep_cases': sweeps,
                      'prior_validators': prior, 'result': 'PASS'}))


if __name__ == '__main__':
    main()
