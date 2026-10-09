"""Validate sealed historical redirect evidence, independent of later audits."""
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
PREFIX = 'data/patch-api/sources/5.0.1-'


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT).strip()


def blob(revision, path):
    return subprocess.check_output(['git', 'show', revision + ':' + path], cwd=ROOT)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def read_json(path):
    return json.loads(path.read_text())


def historical_json(revision, path):
    return json.loads(blob(revision, path))


def names(revision, path):
    return git('ls-tree', '-r', '--name-only', revision, path).decode().splitlines()


def check_seals(context):
    assert digest(Path(__file__).read_bytes()) == context['validator_sha256'], 'validator seal'
    for name, expected in context['session_sha256'].items():
        assert digest((HERE / name).read_bytes()) == expected, 'session seal: ' + name
    revision = context['source_revision']
    for path, expected in context['identities'].items():
        assert git('rev-parse', revision + ':' + path).decode() == expected, path
    tracked = names(context['evidence_revision'], HERE.relative_to(ROOT).as_posix())
    for name in context['session_sha256']:
        assert (HERE / name).relative_to(ROOT).as_posix() in tracked, name
    assert set(tracked) == {(HERE / name).relative_to(ROOT).as_posix()
                            for name in context['session_sha256']} | {
                                (HERE / 'validate.py').relative_to(ROOT).as_posix()}
    assert all((HERE / name).stat().st_size < 5_000_000 for name in context['session_sha256'])


def check_source(context):
    revision = context['source_revision']
    provenance = historical_json(revision, PREFIX + 'api-changes.provenance.json')
    response = read_json(HERE / 'source-response.json')['query']['pages']['554410']
    fetched = response['revisions'][0]
    raw = blob(revision, PREFIX + 'api-changes.wikitext')
    assert response['title'] == provenance['title'] == 'Patch 5.0.1/API changes'
    assert response['pageid'] == provenance['pageid'] == 554410
    assert fetched['revid'] == provenance['revid'] == 5344081
    assert fetched['timestamp'] == provenance['timestamp'] == '2012-08-06T00:56:48Z'
    assert raw.decode() == fetched['slots']['main']['*'] == '#REDIRECT [[Patch 5.0.4/API changes]]'
    assert digest(raw) == provenance['sha256'] == read_json(HERE / 'source-pin.json')['wikitext_sha256']
    assert provenance['classification'] == 'redirect-only' and provenance['redirect'] is True
    assert provenance['redirect_target'] == 'Patch 5.0.4/API changes'
    assert provenance['generator_flags'] == provenance['extractor_flags'] == []
    register = historical_json(revision, PREFIX + 'wikitext-register.json')
    assert register['source']['revid'] == 5344081 and register['source']['sha256'] == digest(raw)
    assert register['entries'] == register['header_counts'] == []
    text = blob(revision, PREFIX + 'api-changes.txt')
    assert text == b'#REDIRECT Patch 5.0.4/API changes\n'
    ledger = historical_json(revision, PREFIX + 'page-coverage.json')
    assert ledger['source_sha256'] == digest(blob(revision, PREFIX + 'wikitext-register.json'))
    assert ledger['non_inventory_source'] == {'path': PREFIX + 'api-changes.txt', 'sha256': digest(text)}
    assert len(ledger['source_rows']) == 1
    row = ledger['source_rows'][0]
    assert row['source_id'] == 'source-context-001' and row['status'] == 'metadata-only'
    assert row['capabilities'] == []
    assert historical_json(revision, 'tests/data/patch_5_0_1_sweep_known_gaps.json') == []
    accounting = read_json(HERE / 'p501-accounting.json')
    assert accounting == {'inventory': 0, 'metadata': 1, 'gaps': [], 'problematic_contracts': [],
                          'retirements': [], 'member_scans': [], 'positive_api_proof': False}


def check_reproduction(context):
    revision = context['source_revision']
    expected = {Path(name).name.removesuffix('-wikitext-register.json')
                for name in names(revision, 'data/patch-api/sources')
                if name.endswith('-wikitext-register.json')}
    registers = read_json(HERE / 'p501-register-reproduction.json')
    extracts = read_json(HERE / 'p501-saved-extract-reproduction.json')
    assert {row['patch'] for row in registers} == {row['patch'] for row in extracts} == expected
    inherited = historical_json(context['base_revision'],
        'data/patch-api/evidence/5.1.0-session-2026-10-08/integrated/p510-saved-extract-reproduction.json')
    failures = {row['patch']: row['error'] for row in inherited if not row['byte_identical']}
    assert set(failures) == {'12.0.5', '12.0.7', '12.1.0'}
    for row in registers:
        assert row['exit'] == 0 and row['byte_identical'], row['patch']
        assert row['revision'] == revision
        assert row['sha256'] == digest(blob(revision, 'data/patch-api/sources/' + row['patch'] + '-wikitext-register.json'))
    for row in extracts:
        assert row['revision'] == revision
        assert row['saved_sha256'] == digest(blob(revision, 'data/patch-api/sources/' + row['patch'] + '-api-changes.txt'))
        if row['patch'] in failures:
            assert not row['byte_identical'] and row['error'] == failures[row['patch']]
        else:
            assert row['byte_identical'] and row['error'] is None, row['patch']
    supplemental = read_json(HERE / 'supplemental-extract-reproduction.json')
    assert supplemental['byte_identical'] and supplemental['revision'] == revision
    assert supplemental['sha256'] == digest(blob(revision, supplemental['path']))
    return len(registers), sum(row['byte_identical'] for row in extracts)


def check_proofs(context):
    for label, policy in context['proofs'].items():
        receipt = read_json(HERE / (label + '.proof.json'))
        assert receipt['command'] == policy['command'] and receipt['exit'] == policy['exit'], label
        assert receipt['revision'] == policy['revision'], label
        for path, identity in receipt['identities'].items():
            assert git('rev-parse', receipt['revision'] + ':' + path).decode() == identity, label
        assert receipt['identities'] == context['identities'], label
        log = (HERE / receipt['log']).read_bytes()
        assert digest(log) == receipt['log_sha256'], label
        if 'test_' in label and receipt['exit'] == 0:
            assert re.search(r'Ran [1-9]\d* tests?', log.decode()) and '\nOK' in log.decode(), label
    negative = (HERE / 'p501-negative.log').read_text()
    assert 'register row count changed' in negative and 'left: 1' in negative and 'right: 0' in negative
    assert re.search(r'test result: FAILED\. 0 passed; 1 failed;', negative)
    fake = read_json(HERE / 'p501-fabricated-register.json')
    assert len(fake['entries']) == 1 and fake['entries'][0]['symbol'] == 'P501FabricatedMissingAPI'
    assert read_json(HERE / 'p501-own-results.json') == read_json(HERE / 'p501-all-sweeps-results.json') == {}
    own = (HERE / 'p501-own.log').read_text()
    assert re.search(r'test result: ok\. 1 passed; 0 failed;', own)
    mists = (HERE / 'p501-mists-check.log').read_text()
    warnings = [line for line in mists.splitlines() if line.startswith('warning:')]
    assert all('iced-wgpu-patched/Cargo.toml:' in line or '`iced_wgpu` (manifest)' in line
               for line in warnings), warnings
    assert 'Finished' in mists


def check_sweeps(context):
    revision = context['source_revision']
    log = (HERE / 'p501-all-sweeps.log').read_text()
    assert re.search(r'test result: ok\. [1-9]\d* passed; 0 failed;', log)
    passed = set(re.findall(r'^test (\S+) \.\.\. ok$', log, re.M))
    for path in names(revision, 'tests'):
        if not re.fullmatch(r'tests/patch_[\d_]+_publication_sweep.rs', path):
            continue
        patch = Path(path).stem.removeprefix('patch_').removesuffix('_publication_sweep').replace('_', '.')
        register = historical_json(revision, 'data/patch-api/sources/' + patch + '-wikitext-register.json')
        if register.get('client_line', 'retail') != 'retail':
            continue
        function = Path(path).stem
        assert any(name.endswith('::' + function) for name in passed), function
        name = 'p501-all-sweeps-results.json' if patch == '5.0.1' else function + '-results.json'
        results = read_json(HERE / name)
        assert set(results) == {row['id'] for row in register['entries']}, patch
        gaps = historical_json(revision, 'tests/data/' + function.removesuffix('_publication_sweep') + '_sweep_known_gaps.json')
        assert {key for key, row in results.items() if not row['ok']} == set(gaps), patch
    return len(passed)


def check_preservation(context):
    base, source = context['base_revision'], context['source_revision']
    assert not git('diff', '--name-only', base, source, '--', 'src', 'tools', 'Cargo.toml', 'Cargo.lock', 'build.rs')
    changed = git('diff', '--name-only', base, source, '--', 'data/patch-api/sources').decode().splitlines()
    assert all(name.startswith(PREFIX) for name in changed), changed
    for path in ['docs/wiki/index.md', 'docs/wiki/log.md']:
        assert blob(base, path) in blob(context['documentation_revision'], path), path
    prior = [name for name in names(base, 'data/patch-api/evidence') if name.endswith('/validate.py')]
    assert git('rev-parse', base + ':data/patch-api/evidence').decode() == context['base_evidence_tree']
    for path in prior:
        assert blob(base, path) == blob(context['evidence_revision'], path), path
    return len(prior)


def main():
    context = read_json(HERE / 'p501-context.json')
    check_seals(context)
    check_source(context)
    registers, extracts = check_reproduction(context)
    check_proofs(context)
    sweeps = check_sweeps(context)
    prior = check_preservation(context)
    print(json.dumps({'patch': '5.0.1', 'classification': 'redirect-only', 'inventory': 0,
                      'positive_api_proof': False, 'registers': registers, 'extracts': extracts,
                      'sweep_cases': sweeps, 'prior_validators': prior, 'result': 'PASS'}))


if __name__ == '__main__':
    main()
