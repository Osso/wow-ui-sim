"""Read-only integrated 6.0.2 proof: own receipts sealed; shared scope pinned."""
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent
HISTORY = HERE.parent


def read(path):
    return json.loads(path.read_text())


def digest(value):
    return hashlib.sha256(value).hexdigest()


def git(*args, input=None):
    return subprocess.check_output(['git', *args], cwd=ROOT, input=input)


def blob(revision, path):
    return git('show', revision + ':' + path)


def names(revision, directory):
    return git('ls-tree', '-r', '--name-only', revision, directory).decode().splitlines()


def pinned(revision, path):
    return json.loads(blob(revision, path))


def check_mapping(context):
    mapping = read(HERE / 'rebase-mapping.json')
    assert mapping['rebased_base'] == context['master_revision']
    revisions = {}
    for row in mapping['commits'] + mapping['external_commits']:
        revision = row['rebased_revision']
        revisions[row['recorded_revision']] = revision
        assert git('show', '-s', '--format=%s', revision).decode().strip() == row['subject']
        assert git('rev-parse', revision + '^{tree}').decode().strip() == row['rebased_tree']
        patch = git('show', '--format=', '--binary', revision)
        assert git('patch-id', '--stable', input=patch).decode().split()[0] == row['rebased_patch_id']
    for scope in mapping['pinned_inputs']:
        revision = scope['rebased_revision']
        assert revisions[scope['recorded_revision']] == revision
        for path, expected in scope['inputs'].items():
            assert git('rev-parse', revision + ':' + path).decode().strip() == expected['rebased_blob']
            content = (HERE / expected['historical_file']).read_bytes() if 'historical_file' in expected else blob(revision, path)
            assert digest(content) == expected['sha256']
            assert git('hash-object', '--stdin', input=content).decode().strip() == expected['recorded_blob']
    return len(mapping['commits'])


def check_preservation(context):
    for path, expected in read(HERE / 'historical-preservation.json').items():
        current = HERE / 'historical-validator.py.txt' if path.endswith('/validate.py') else ROOT / path
        assert current.is_relative_to(HISTORY)
        assert digest(current.read_bytes()) == expected, path
    preserved = read(HERE / 'p602-input-preservation.json')
    assert {row['path'] for row in preserved['rows']} == set(names(context['master_revision'], 'data/patch-api/sources'))
    for row in preserved['rows']:
        assert digest(blob(context['master_revision'], row['path'])) == row['before_sha256']
        assert digest(blob(context['runtime_revision'], row['path'])) == row['after_sha256'] == row['before_sha256']
    assert all(row['before'] == row['after'] for row in read(HERE / 'p602-extract-preservation.json'))
    changed = set(git('diff', '--name-only', context['master_revision'], context['runtime_revision'], '--', 'src').decode().splitlines())
    assert changed == {'src/c_api/c_scenario_bonus.rs', 'src/c_api/mod.rs',
                       'src/c_api/registration.rs', 'src/c_api/patch_retired_members.rs'}
    assert not git('diff', context['master_revision'], context['runtime_revision'], '--', 'Interface')


def check_reproduction(context):
    revision = context['runtime_revision']
    patches = {Path(path).name.removesuffix('-wikitext-register.json')
               for path in names(revision, 'data/patch-api/sources') if path.endswith('-wikitext-register.json')}
    registers = read(HERE / 'p602-register-reproduction.json')
    extracts = read(HERE / 'p602-saved-extract-reproduction.json')
    previous = 'data/patch-api/evidence/6.1.0-session-2026-10-08/integrated/'
    prior_registers = {row['patch']: row for row in pinned(context['master_revision'], previous + 'p610-register-reproduction.json')}
    prior_extracts = {row['patch']: row for row in pinned(context['master_revision'], previous + 'p610-saved-extract-reproduction.json')}
    assert {row['patch'] for row in registers} == {row['patch'] for row in extracts} == patches
    for row in registers:
        patch = row['patch']
        provenance = pinned(revision, f'data/patch-api/sources/{patch}-api-changes.provenance.json')
        assert row['verified_flags'] == provenance.get('generator_flags', prior_registers.get(patch, {}).get('verified_flags', []))
        assert row['exit'] == 0 and row['byte_identical']
        assert row['sha256'] == digest(blob(revision, f'data/patch-api/sources/{patch}-wikitext-register.json'))
    provenance = pinned(revision, 'data/patch-api/sources/6.0.2-api-changes.provenance.json')
    diff = provenance['transcluded_diff']
    assert digest(blob(revision, diff['path'])) == diff['sha256']
    assert provenance['generator_flags'] == ['--warlords-prepatch', '--warlords-diff', diff['path']]
    for row in extracts:
        patch = row['patch']
        provenance = pinned(revision, f'data/patch-api/sources/{patch}-api-changes.provenance.json')
        assert row['verified_flags'] == provenance.get('extractor_flags', prior_extracts.get(patch, {}).get('verified_flags', []))
        assert row['saved_sha256'] == digest(blob(revision, f'data/patch-api/sources/{patch}-api-changes.txt'))
        if patch in prior_extracts:
            assert all(row[key] == prior_extracts[patch][key] for key in ('byte_identical', 'error', 'sha256', 'saved_sha256'))
        else:
            assert row['byte_identical'] and row['error'] is None
    assert sorted(row['patch'] for row in extracts if not row['byte_identical']) == ['12.0.5', '12.0.7', '12.1.0']
    return len(registers), sum(row['byte_identical'] for row in extracts)


def check_sweeps(context):
    revision = context['runtime_revision']
    passed = set(re.findall(r'^test (\S+) \.\.\. ok$', (HERE / 'all-sweeps.txt').read_text(), re.M))
    master_passed = set(re.findall(r'^test (\S+) \.\.\. ok$', (HERE / 'master-all-sweeps.txt').read_text(), re.M))
    pages = []
    for path in names(revision, 'tests'):
        if not path.endswith('_publication_sweep.rs'):
            continue
        stem = Path(path).stem
        patch = stem.removeprefix('patch_').removesuffix('_publication_sweep').replace('_', '.')
        functions = re.findall(r'fn (patch_\w+_publication_sweep)\(', blob(revision, path).decode())
        assert functions and all(any(case.endswith('::' + fn) for case in passed) for fn in functions)
        results = read(HERE / (stem + '-results.json'))
        register = pinned(revision, f'data/patch-api/sources/{patch}-wikitext-register.json')
        gaps = sorted(key for key, value in results.items() if not value['ok'])
        assert gaps == sorted(pinned(revision, f'tests/data/patch_{patch.replace(".", "_")}_sweep_known_gaps.json'))
        assert set(results) == {row['id'] for row in register['entries']}
        if patch != '6.0.2':
            assert all(any(case.endswith('::' + fn) for case in master_passed) for fn in functions)
            assert results == read(HERE / 'master' / (stem + '-results.json')), patch
        pages.append({'patch': patch, 'observations': len(results), 'gaps': gaps,
                      'unchanged_vs_master': patch != '6.0.2'})
    assert read(HERE / 'gap-comparison.json') == pages
    positive = read(HERE / 'patch_6_0_2_publication_sweep-results.json')
    historical = read(HISTORY / 'patch_6_0_2_publication_sweep-results.json')
    assert positive == historical
    assert read(HERE / 'supersession-review.json') == {
        'before_gaps': 275, 'after_gaps': 275, 'observation_changes': {}, 'replacements': [],
        'integrated_registers': ['6.1.0', '6.2.0', '6.2.2'], 'pending_prose': 95,
        'pending_enum_members': 79}
    negative = read(HERE / 'negative-results.json')
    negative_register = read(HERE / 'negative-register.json')
    changed = [entry for entry in negative_register['entries'] if entry['symbol'] == 'P602NegativeMissingAPI']
    assert len(changed) == 1
    control = changed[0]['id']
    assert {key for key, value in negative.items() if not value['ok']} == {key for key, value in positive.items() if not value['ok']} | {control}
    assert {key: value for key, value in negative.items() if key != control} == {key: value for key, value in positive.items() if key != control}
    return len(pages), len(passed)


def failures(log):
    return re.findall(r'^---- (\S+) stdout ----$', log, re.M)


def check_commands(context):
    required = {'own-sweep', 'all-sweeps', 'negative', 'format', 'mists-check',
                'branch-startup', 'master-startup', 'master-all-sweeps', 'master-scenario'}
    required |= {Path(path).stem for path in names(context['runtime_revision'], 'tools')
                 if Path(path).parent == Path('tools') and Path(path).name.startswith('test_') and path.endswith('.py')}
    for target in ('integration', 'prefork_full_ui'):
        required |= {target + '-' + selector for selector in ('patch_6_0_2', 'scenario', 'objective_tracker',
                     'blizzard_shared_map_data_providers_loads', 'blizzard_flight_map_loads',
                     'blizzard_quest_navigation_loads', 'blizzard_poi_button_loads')}
    required.add('integration-c_scenario_info_probes')
    assert required <= set(context['receipts'])
    for label, expected in context['receipts'].items():
        receipt = read(HERE / (label + '.proof.json'))
        assert receipt == expected
        log = (HERE / receipt['log']).read_bytes()
        assert digest(log) == receipt['log_sha256']
        expected_exit = 1 if label == 'negative' else 101 if label in ('integration-scenario', 'master-scenario') else 0
        assert receipt['exit'] == expected_exit, label
        revision = context['master_revision'] if label.startswith('master-') else context['runtime_revision']
        for directory in ('src', 'tests', 'tools'):
            assert git('rev-parse', receipt['revision'] + ':' + directory) == git('rev-parse', revision + ':' + directory)
        if receipt['command'][:2] == ['cargo', 'test'] and expected_exit == 0:
            counts = re.findall(r'test result: ok\. (\d+) passed; 0 failed;', log.decode())
            assert counts, label
            if label not in ('prefork_full_ui-scenario', 'prefork_full_ui-blizzard_shared_map_data_providers_loads'):
                assert sum(map(int, counts)) > 0, label
    scenario = (HERE / 'integration-scenario.txt').read_text()
    master = (HERE / 'master-scenario.txt').read_text()
    assert failures(scenario) == failures(master) == ['c_api_surface::scenario_defaults_are_not_c_api_temporary_shims']
    assert '9 passed; 1 failed;' in scenario and '9 passed; 1 failed;' in master
    for label in ('branch-startup', 'master-startup'):
        assert (HERE / (label + '.txt')).read_text().rstrip().endswith('[]')
        assert '--no-addons' not in context['receipts'][label]['command']
        assert 'WOW_SIM_NO_ADDONS' not in context['receipts'][label]['environment']
    warnings = [line for line in (HERE / 'mists-check.txt').read_text().splitlines() if line.startswith('warning:')]
    assert all('iced-wgpu-patched/Cargo.toml:' in line or '`iced_wgpu` (manifest)' in line for line in warnings), warnings
    matrix = read(HERE / 'prior-validator-matrix.json')
    paths = {path for path in names(context['master_revision'], 'data/patch-api/evidence')
             if Path(path).name in ('validate.py', 'validate_integrated.py')}
    assert {row['path'] for row in matrix} == paths
    for row in matrix:
        assert row['exit'] == 0
        assert digest((HERE / row['log']).read_bytes()) == row['log_sha256']
        assert digest(blob(context['master_revision'], row['path'])) == row['code_sha256']
    return len(matrix)


def main():
    context = read(HERE / 'context.json')
    for path, expected in read(HERE / 'artifact-hashes.json').items():
        assert digest((HERE / path).read_bytes()) == expected, path
    mapped = check_mapping(context)
    check_preservation(context)
    registers, extracts = check_reproduction(context)
    pages, cases = check_sweeps(context)
    prior = check_commands(context)
    print(json.dumps({'status': 'PASS', 'registers': registers, 'extracts': extracts,
                      'pages': pages, 'cases': cases, 'prior_validators': prior,
                      'mapped_commits': mapped, 'unchanged_master_scenario_failure': True}, sort_keys=True))


if __name__ == '__main__':
    main()
