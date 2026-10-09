"""Read-only historical retail 5.1.0 proof; no live shared-file comparisons."""
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
SEAL_SHA256 = 'debef6fa802b74d3ccfdbaa9454830e2c47b3e00d2182268a3b21a8369a64deb'


def digest(data):
    return hashlib.sha256(data).hexdigest()


def read(name):
    return json.loads((HERE / name).read_text())


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT)


def blob(revision, path):
    return git('show', f'{revision}:{path}')


def shared_json(seal, path):
    return json.loads(blob(seal['accounting_revision'], path))


def check_seals():
    raw = (HERE / 'p510-seal.json').read_bytes()
    assert digest(raw) == SEAL_SHA256, 'session seal changed'
    seal = json.loads(raw)
    for name, expected in seal['own_inputs'].items():
        assert digest((HERE / name).read_bytes()) == expected, f'own input changed: {name}'
    for item in seal['shared_inputs']:
        actual = digest(blob(item['revision'], item['path']))
        assert actual == item['sha256'], f'pinned shared input changed: {item}'
    return seal


def check_source_and_accounting(seal):
    register = shared_json(seal, 'data/patch-api/sources/5.1.0-wikitext-register.json')
    ledger = shared_json(seal, 'data/patch-api/sources/5.1.0-page-coverage.json')
    fixture = shared_json(seal, 'tests/data/patch_5_1_0_sweep_known_gaps.json')
    observations = read('p510-own-results.json')
    source = blob(seal['proof_revision'], register['source']['path'])
    assert digest(source) == register['source']['sha256']
    page = read('p510-fetch.json')['query']['pages']['282785']
    assert page['revisions'][0]['revid'] == register['source']['revid']
    assert page['revisions'][0]['slots']['main']['*'].encode() == source
    parent = next(iter(read('p510-parent-fetch.json')['query']['pages'].values()))
    parent_text = parent['revisions'][0]['slots']['main']['*']
    assert re.search(r'^\|toc\s*=\s*50100$', parent_text, re.M)
    assert re.search(r'^\|Version\s*=\s*16309$', parent_text, re.M)
    assert '|Release = November 27, 2012' in parent_text
    provenance = shared_json(seal, 'data/patch-api/sources/5.1.0-api-changes.provenance.json')
    transclusion = provenance['transcluded_diff']
    diff = next(iter(read('p510-diff-fetch.json')['query']['pages'].values()))
    assert diff['pageid'] == transclusion['pageid']
    assert diff['revisions'][0]['revid'] == transclusion['revid']
    assert digest(diff['revisions'][0]['slots']['main']['*'].encode()) == transclusion['sha256']
    assert digest(blob(seal['proof_revision'], transclusion['path'])) == transclusion['sha256']
    assert register['client_line'] == provenance['client_line'] == 'retail'
    assert all(c['header_count'] == c['parsed_count'] for c in register['header_counts'])
    ids = {row['id'] for row in register['entries']}
    assert len(ids) == len(register['entries']) == len(observations)
    assert ids == set(observations)
    gaps = {key for key, value in observations.items() if not value['ok']}
    assert gaps == set(fixture) == {row['source_id'] for row in read('p510-gap-review.json')}
    inventory = {row['source_id']: row for row in ledger['source_rows'] if row['source_id'] in ids}
    assert set(inventory) == ids
    for source_id, observation in observations.items():
        expected = 'bounded-coverage' if observation['ok'] else 'audit-pending'
        assert inventory[source_id]['status'] == expected, source_id
        if not observation['ok']:
            assert not inventory[source_id]['capabilities']
    extractor = {'__name__': 'pinned_extractor', '__file__': str(ROOT / 'tools/extract_patch_non_inventory.py')}
    exec(compile(blob(seal['proof_revision'], 'tools/extract_patch_non_inventory.py'),
                 'pinned_extractor', 'exec'), extractor)
    text = blob(seal['proof_revision'], ledger['non_inventory_source']['path']).decode()
    assert digest(text.encode()) == ledger['non_inventory_source']['sha256']
    prose_ids = {row['source_id'] for row in extractor['seed_rows'](text, '5.1.0')}
    ledger_ids = [row['source_id'] for row in ledger['source_rows']]
    assert len(set(ledger_ids)) == len(ledger_ids)
    assert set(ledger_ids) == ids | prose_ids
    return {'inventory': len(ids), 'extract': len(prose_ids), 'api_gaps': len(gaps),
            'ledger': len(ledger_ids)}


def check_proofs(seal):
    for name in seal['successful_proofs']:
        proof = read(name + '.proof.json')
        assert proof['exit'] == 0, name
        assert digest((HERE / proof['log']).read_bytes()) == proof['log_sha256'], name
        git('rev-parse', '--verify', proof['revision'] + '^{commit}')
    for area in ['pet-info', 'pet-stats', 'namespace', 'lib-namespace', 'lib-retired']:
        branch = (HERE / ('p510-' + area + '.log')).read_text()
        master = (HERE / ('p510-master-' + area + '.log')).read_text()
        pattern = r'test (\S+) \.\.\. (ok|FAILED)'
        assert sorted(re.findall(pattern, branch)) == sorted(re.findall(pattern, master)), area
        if area != 'lib-retired':
            assert re.findall(pattern, branch), f'empty caller proof: {area}'
    # This module has no unit cases; namespace caller units and loaded/bare APIs
    # supply positive behavioral coverage. Never count a zero-case filter as proof.
    assert 'running 0 tests' in (HERE / 'p510-lib-retired.log').read_text()
    master_cases = set(re.findall(r'test (\S+) \.\.\. ok', (HERE / 'p510-master-prefork-pet.log').read_text()))
    branch_cases = set(re.findall(r'test (\S+) \.\.\. ok', (HERE / 'p510-prefork-pet.log').read_text()))
    assert master_cases <= branch_cases
    branch_errors = read('p510-startup-frozen-repeat-errors.json')
    master_errors = read('p510-master-startup-same-root-errors.json')
    assert branch_errors == master_errors, 'addons-enabled startup differs from pinned master'
    assert branch_errors == read('p510-startup-frozen-errors.json'), 'same-source repeat drift'
    for name in ['p510-startup-frozen-repeat', 'p510-master-startup-same-root']:
        proof = read(name + '.proof.json')
        assert proof['exit'] == seal['startup_exit']
        assert proof['env']['WOW_SIM_ADDONS_PATH'].endswith('/addon-snapshot')
        assert proof['env']['WOW_SIM_DISABLE_BYTECODE_CACHE'] == '1'
    mists = (HERE / 'p510-mists.log').read_text()
    warnings = [line for line in mists.splitlines() if line.startswith('warning:')]
    assert all('iced-wgpu-patched/Cargo.toml:' in line or '`iced_wgpu` (manifest)' in line
               for line in warnings), warnings
    negative = read('p510-negative-committed-results.json')
    original = read('p510-own-results.json')
    old_gaps = {key for key, value in original.items() if not value['ok']}
    new_gaps = {key for key, value in negative.items() if not value['ok']}
    assert old_gaps < new_gaps and len(new_gaps - old_gaps) == 1
    assert len(negative) == len(original)
    assert read('p510-negative-committed.proof.json')['exit'] != 0
    for row in read('p510-unchanged-page-observations.json'):
        assert read(row['path']) == read('master-results/' + row['path'])
    assert all(row['exit'] == 0 for row in read('p510-python-fixtures.json'))
    changed = git('diff', '--name-only', seal['master_revision'], seal['proof_revision'], '--',
                  'src', 'Interface').decode().splitlines()
    assert changed == ['src/c_api/patch_retired_members.rs'], changed


def check_historical_reproduction(seal):
    helper = {'__name__': 'pinned_helper'}
    exec(compile(blob(seal['proof_revision'], 'tools/patch_audit_validation.py'),
                 'pinned_helper', 'exec'), helper)
    paths = helper['historical_registers'](ROOT, seal['proof_revision'])
    reproduced = read('p510-register-reproduction.json')
    expected = {path.name.removesuffix('-wikitext-register.json') for path in paths}
    assert expected == {row['patch'] for row in reproduced}
    assert all(row['byte_identical'] and row['exit'] == 0 for row in reproduced)
    extracts = read('p510-saved-extract-reproduction.json')
    assert expected == {row['patch'] for row in extracts}
    assert {row['patch'] for row in extracts if not row['byte_identical']} == {
        '12.0.5', '12.0.7', '12.1.0'}
    preservation = read('p510-input-preservation.json')
    for row in preservation['rows']:
        assert row['before_sha256'] == row['after_sha256']
        assert digest(blob(preservation['base_revision'], row['path'])) == row['before_sha256']
        assert digest(blob(seal['proof_revision'], row['path'])) == row['after_sha256']
    for row in read('p510-extract-preservation.json'):
        assert row['before'] == row['after']
    return {'registers': len(reproduced), 'matching_extracts': sum(row['byte_identical'] for row in extracts)}


def check_retirements_and_wiki(seal):
    scans = read('p510-scans.json')
    for scan in scans:
        assert scan['tool'] == '/usr/bin/grep' and '-w' in scan['argv']
        assert digest((HERE / scan['path']).read_bytes()) == scan['sha256']
        assert len((HERE / scan['path']).read_text().splitlines()) == scan['matches']
        assert scan['exit'] in [0, 1]
    for decision in read('p510-retirement-decisions.json'):
        if decision['decision'] == 'retired':
            assert decision['scans'] == {'retail': 0, 'callers': 0}
            assert not decision['later_readds'] and decision['observation']['ok']
    for branch in read('p510-retirement-later-registers.json'):
        names = git('ls-tree', '-r', '--name-only', branch['revision'], 'data/patch-api/sources').decode()
        for register in branch['matching_registers']:
            assert register['path'] in names.splitlines()
            assert digest(blob(branch['revision'], register['path'])) == register['sha256']
        retired = {row['member'] for row in read('p510-retirement-decisions.json')
                   if row['decision'] == 'retired'}
        for path in names.splitlines():
            if not path.endswith('-wikitext-register.json'):
                continue
            register = json.loads(blob(branch['revision'], path))
            if register.get('client_line', 'retail') != 'retail':
                continue
            assert not any(row['symbol'] in retired and row['direction'] == 'added'
                           for row in register['entries']), f're-added retirement: {path}'
    for path, count in read('p510-wiki-baseline.json').items():
        # Validate the documented audit snapshot, not a future live wiki.
        assert len(blob(seal['documentation_revision'], path).decode().splitlines()) >= count


def main():
    seal = check_seals()
    result = check_source_and_accounting(seal)
    check_proofs(seal)
    result.update(check_historical_reproduction(seal))
    check_retirements_and_wiki(seal)
    print(json.dumps({'status': 'PASS', **result}, sort_keys=True))


if __name__ == '__main__':
    main()
