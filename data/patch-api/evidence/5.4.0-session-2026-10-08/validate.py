"""Portable historical 5.4.0 proof: own seals, pinned shared inputs, derived counts."""
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

SEAL_REVISION = 'P540_SEAL_REVISION'
SESSION = HERE.relative_to(ROOT).as_posix()


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def pinned(path, revision):
    return git('show', f'{revision}:{path}')


def read(name):
    return json.loads((HERE / name).read_text())


def assert_receipt(name, expected_exit=0):
    receipt = read(name + '.proof.json')
    if expected_exit is None:
        assert receipt['exit'] != 0, (name, 'negative control unexpectedly passed')
    else:
        assert receipt['exit'] == expected_exit, (name, receipt['exit'])
    assert not receipt['invalidated'], name
    assert receipt['log_sha256'] == digest((HERE / receipt['log']).read_bytes()), name
    git('rev-parse', '--verify', receipt['revision'] + '^{commit}')
    return receipt


def check_sources(revision):
    register = json.loads(pinned('data/patch-api/sources/5.4.0-wikitext-register.json', revision))
    provenance = json.loads(pinned('data/patch-api/sources/5.4.0-api-changes.provenance.json', revision))
    for fetch in read('p540-fetch-receipts.json'):
        assert digest((HERE / fetch['file']).read_bytes()) == fetch['sha256']
    response = read('p540-fetch.json')['query']['pages']['414879']
    source = response['revisions'][0]
    assert source['revid'] == register['source']['revid'] == provenance['revid']
    assert source['slots']['main']['*'].encode() == pinned(register['source']['path'], revision)
    assert digest(source['slots']['main']['*'].encode()) == register['source']['sha256']
    parent = next(iter(read('p540-parent-fetch.json')['query']['pages'].values()))['revisions'][0]
    parent_text = parent['slots']['main']['*']
    assert parent['revid'] == provenance['parent_revid']
    assert re.search(r'\|toc\s*=\s*50400\b', parent_text)
    assert re.search(r'\|Version\s*=\s*17345\b', parent_text)
    assert 'September 10, 2013' in parent_text
    diff = next(iter(read('p540-diff-fetch.json')['query']['pages'].values()))['revisions'][0]
    assert diff['revid'] == provenance['diff_source']['revid']
    assert diff['slots']['main']['*'].encode() == pinned('data/patch-api/sources/5.4.0-api-changes-diff.wikitext', revision)
    assert digest(diff['slots']['main']['*'].encode()) == provenance['diff_source']['sha256']
    assert len({row['id'] for row in register['entries']}) == len(register['entries'])
    assert all(row['header_count'] == row['parsed_count'] for row in register['header_counts'])
    return register


def check_reproduction(revision):
    paths = historical_registers(ROOT, revision)
    rows = read('p540-register-reproduction.json')
    assert {row['patch'] for row in rows} == {p.name.removesuffix('-wikitext-register.json') for p in paths}
    for row in rows:
        assert row['byte_identical'] and row['exit'] == 0, row
        path = 'data/patch-api/sources/' + row['patch'] + '-wikitext-register.json'
        assert digest(pinned(path, row['revision'])) == row['sha256'], path
    preservation = read('p540-input-preservation.json')
    names = git('ls-tree', '-r', '--name-only', preservation['base_revision'], 'data/patch-api/sources').decode().splitlines()
    assert {row['path'] for row in preservation['rows']} == set(names)
    for row in preservation['rows']:
        assert row['before_sha256'] == row['after_sha256'] == digest(pinned(row['path'], preservation['base_revision']))
        assert row['after_sha256'] == digest(pinned(row['path'], preservation['audit_revision']))
    assert all(row['before'] == row['after'] for row in read('p540-extract-preservation.json'))
    extracts = read('p540-saved-extract-reproduction.json')
    assert {row['patch'] for row in extracts} == {row['patch'] for row in rows}
    inherited = json.loads(pinned('data/patch-api/evidence/5.4.8-session-2026-10-08/integrated/p548-saved-extract-reproduction.json', preservation['base_revision']))
    prior = {row['patch']: row for row in inherited}
    for row in extracts:
        if row['patch'] in prior:
            assert (row['byte_identical'], row['error']) == (prior[row['patch']]['byte_identical'], prior[row['patch']]['error'])
        else:
            assert row['byte_identical'], row
    assert read('p540-diff-extract-reproduction.json')['byte_identical']
    return len(rows), sum(row['byte_identical'] for row in extracts)


def check_accounting(register, revision):
    results = read('patch_5_4_0_publication_sweep-results.json')
    ids = {row['id'] for row in register['entries']}
    assert set(results) == ids
    gaps = {key for key, value in results.items() if not value['ok']}
    known = set(json.loads(pinned('tests/data/patch_5_4_0_sweep_known_gaps.json', revision)))
    assert gaps == known == {row['source_id'] for row in read('p540-gap-review.json')}
    ledger = json.loads(pinned('data/patch-api/sources/5.4.0-page-coverage.json', revision))
    contracts = read('p540-contract-review.json')
    assert {row['source_id'] for row in ledger['source_rows']} == ids | {row['source_id'] for row in contracts}
    assert len(ledger['source_rows']) == len(ids) + len(contracts)
    rows = {row['source_id']: row for row in ledger['source_rows']}
    assert {key for key, row in rows.items() if key in ids and row['status'] == 'audit-pending'} == gaps
    namespace = {'__name__': 'pinned_extractor', '__file__': str(ROOT / 'tools/extract_patch_non_inventory.py')}
    exec(compile(pinned('tools/extract_patch_non_inventory.py', revision), 'pinned_extractor', 'exec'), namespace)
    seeded = set()
    for suffix in ('', '-diff'):
        path = f'data/patch-api/sources/5.4.0-api-changes{suffix}.txt'
        prefix = 'diff-' if suffix else ''
        seeded.update(prefix + row['source_id'] for row in namespace['seed_rows'](pinned(path, revision).decode(), '5.4.0'))
    assert seeded == {row['source_id'] for row in contracts}, 'extract occurrence omitted'
    for contract in contracts:
        path = contract['source_path']
        line = pinned(path, revision).decode().splitlines()[contract['source_line'] - 1]
        assert line == contract['statement']
        assert rows[contract['source_id']]['note']
    negative = read('p540-negative-results.json')
    assert {key for key, value in negative.items() if not value['ok']} == gaps | {'p540-intentional-missing'}
    summary = read('p540-sweep-summary.json')
    historical = historical_sweep_tests(ROOT, revision)
    assert {row['test'] for row in summary} == {p.relative_to(ROOT).as_posix() for p in historical}
    for row in summary:
        observations = read(row['results'])
        assert row['observations'] == len(observations)
        assert row['gaps'] == sum(not result['ok'] for result in observations.values())
    return len(ids), len(contracts), len(gaps)


def check_removals(register):
    scans = read('p540-retirement-scans.json')
    assert {row['symbol'] for row in scans} == {row['symbol'] for row in register['entries'] if row['direction'] == 'removed'}
    for row in scans:
        assert row['tool'] == '/usr/bin/grep' and row['whole_word']
        for scan in row['scans'].values():
            log = (HERE / scan['log']).read_bytes()
            assert scan['sha256'] == digest(log)
            assert scan['lines'] == len(log.decode().splitlines())
            assert scan['exit'] in (0, 1)
            assert '-RInw' in scan['command']
    for row in read('p540-later-register-check.json'):
        assert row['sha256'] == digest(pinned(row['path'], row['revision']))
        source = json.loads(pinned(row['path'], row['revision']))
        removed = {scan['symbol'] for scan in scans}
        assert row['matches'] == [entry for entry in source['entries'] if entry['symbol'] in removed]
    return len(scans)


def main():
    manifest_path = SESSION + '/p540-seal.json'
    sealed = pinned(manifest_path, SEAL_REVISION)
    assert (HERE / 'p540-seal.json').read_bytes() == sealed, 'own seal changed'
    manifest = json.loads(sealed)
    revision = manifest['code_revision']
    for row in manifest['own_files']:
        assert digest((HERE / row['path']).read_bytes()) == row['sha256'], row['path']
    for row in manifest['shared_files']:
        assert digest(pinned(row['path'], revision)) == row['sha256'], row['path']
    assert not git('diff', manifest['base_revision'], revision, '--', 'src'), 'unexpected runtime mutation'
    register = check_sources(revision)
    counts = check_accounting(register, revision)
    reproduction = check_reproduction(revision)
    removals = check_removals(register)
    for name in manifest['passing_receipts']:
        assert_receipt(name)
    assert_receipt('p540-negative', None)
    mists = (HERE / 'p540-mists-check.txt').read_text()
    assert not re.search(r'-->\s+(?:src|tests)/', mists), 'non-vendor diagnostic'
    assert 'Finished `dev` profile' in mists
    print(json.dumps({'status': 'PASS', 'inventory': counts[0], 'extract_rows': counts[1],
                      'retained_gaps': counts[2], 'registers_reproduced': reproduction[0],
                      'extracts_reproduced': reproduction[1], 'removal_scans': removals}))


if __name__ == '__main__':
    main()
