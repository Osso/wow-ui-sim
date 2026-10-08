"""Read-only integrated 6.2.0 gate; shared inputs are pinned Git objects."""
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


def digest(data):
    return hashlib.sha256(data).hexdigest()


def git(*args, input=None):
    return subprocess.check_output(['git', *args], cwd=ROOT, input=input)


def blob(revision, name):
    return git('show', revision + ':' + name)


def pinned(revision, name):
    return json.loads(blob(revision, name))


def names(revision, directory):
    return git('ls-tree', '-r', '--name-only', revision, directory).decode().splitlines()


def check_mapping():
    mapping = read(HERE / 'rebase-mapping.json')
    assert mapping['rebased_base'].startswith('8dd11c1b9')
    assert len(mapping['commits']) == 8
    for row in mapping['commits']:
        revision = row.get('rebased_revision', row.get('superseded_by'))
        if 'rebased_revision' in row:
            assert git('show', '-s', '--format=%s', revision).decode().strip() == row['subject']
            patch = git('show', '--format=', '--binary', revision)
            patch_id = git('patch-id', '--stable', input=patch).decode().split()[0]
            assert patch_id == row['rebased_patch_id']
            assert row['patch_identical'] == (row['recorded_patch_id'] == patch_id)
            assert git('rev-parse', revision + '^{tree}').decode().strip() == row['rebased_tree']
            for entry in row['changed_blobs']:
                assert git('rev-parse', revision + ':' + entry['path']).decode().strip() == entry['after']
        else:
            assert row['subject'] == 'Use master 7.0.3 validator portability repair'
            for path, expected in row['upstream_blobs'].items():
                assert git('rev-parse', revision + ':' + path).decode().strip() == expected
        for directory in ('src', 'tests'):
            assert git('rev-parse', revision + ':' + directory).decode().strip() == row['rebased_' + directory + '_tree']
        # The rebase adds only the already-merged 6.2.2/6.2.4 inventory files.
        after = set(names(revision, 'data/patch-api/sources'))
        before = set(row['recorded_registers'])
        assert before <= after
        assert all(Path(path).name.startswith(('6.2.2-', '6.2.4-')) for path in after - before)
        after = {path for path in names(revision, 'tests') if path.endswith('_publication_sweep.rs')}
        before = set(row['recorded_sweeps'])
        assert before <= after
        assert after - before <= {'tests/patch_6_2_2_publication_sweep.rs', 'tests/patch_6_2_4_publication_sweep.rs'}


def check_preservation(context):
    for path, expected in read(HERE / 'historical-preservation.json').items():
        original = blob(context['runtime_revision'], path)
        assert digest(original) == expected
        current = (ROOT / path).read_bytes() if path.startswith(HISTORY.relative_to(ROOT).as_posix() + '/') else original
        if path.endswith('/validate.py'):
            current = (HERE / 'historical-validator.py.txt').read_bytes()
        assert digest(current) == expected, path
    for scope, expected in context['input_trees'].items():
        assert git('rev-parse', context['runtime_revision'] + ':' + scope).decode().strip() == expected
    preservation = read(HERE / 'p620-input-preservation.json')
    assert preservation['base_revision'].startswith('8dd11c1b9')
    assert {row['path'] for row in preservation['rows']} == set(names(context['master_revision'], 'data/patch-api/sources'))
    for row in preservation['rows']:
        before = digest(blob(context['master_revision'], row['path']))
        after = digest(blob(context['runtime_revision'], row['path']))
        assert before == after == row['before_sha256'] == row['after_sha256'], row['path']
    assert all(row['before'] == row['after'] for row in read(HERE / 'p620-extract-preservation.json'))


def check_reproduction(context):
    revision = context['runtime_revision']
    patches = {Path(path).name.removesuffix('-wikitext-register.json')
               for path in names(revision, 'data/patch-api/sources') if path.endswith('-wikitext-register.json')}
    registers = read(HERE / 'p620-register-reproduction.json')
    extracts = read(HERE / 'p620-saved-extract-reproduction.json')
    prior_dir = 'data/patch-api/evidence/6.2.4-session-2026-10-08/integrated/'
    prior_registers = {row['patch']: row for row in pinned(context['master_revision'], prior_dir + 'p624-register-reproduction.json')}
    prior_extracts = {row['patch']: row for row in pinned(context['master_revision'], prior_dir + 'p624-saved-extract-reproduction.json')}
    assert {row['patch'] for row in registers} == {row['patch'] for row in extracts} == patches
    assert len(registers) == len(extracts) == len(patches)
    for row in registers:
        patch = row['patch']
        provenance = pinned(revision, f'data/patch-api/sources/{patch}-api-changes.provenance.json')
        flags = provenance.get('generator_flags', prior_registers.get(patch, {}).get('verified_flags', []))
        assert row['verified_flags'] == flags
        assert row['exit'] == 0 and row['byte_identical']
        assert row['sha256'] == digest(blob(revision, f'data/patch-api/sources/{patch}-wikitext-register.json'))
    for row in extracts:
        patch = row['patch']
        provenance = pinned(revision, f'data/patch-api/sources/{patch}-api-changes.provenance.json')
        flags = provenance.get('extractor_flags', prior_extracts.get(patch, {}).get('verified_flags', []))
        assert row['verified_flags'] == flags
        assert row['saved_sha256'] == digest(blob(revision, f'data/patch-api/sources/{patch}-api-changes.txt'))
        if patch in prior_extracts:
            assert all(row[key] == prior_extracts[patch][key] for key in ('byte_identical', 'error', 'sha256', 'saved_sha256'))
        else:
            assert row['byte_identical'] and row['error'] is None
    assert sorted(row['patch'] for row in extracts if not row['byte_identical']) == ['12.0.5', '12.0.7', '12.1.0']
    return len(patches), sum(row['byte_identical'] for row in extracts)


def check_sweeps(context):
    revision = context['runtime_revision']
    pages = []
    passed = set(re.findall(r'^test (\S+) \.\.\. ok$', (HERE / 'all-sweeps.txt').read_text(), re.M))
    master_passed = set(re.findall(r'^test (\S+) \.\.\. ok$', (HERE / 'master-all-sweeps.txt').read_text(), re.M))
    for path in names(revision, 'tests'):
        if not path.endswith('_publication_sweep.rs'):
            continue
        stem = Path(path).stem
        patch = stem.removeprefix('patch_').removesuffix('_publication_sweep').replace('_', '.')
        functions = re.findall(r'fn (patch_\w+_publication_sweep)\(', blob(revision, path).decode())
        assert functions and all(any(case.endswith('::' + function) for case in passed) for function in functions)
        results = read(HERE / (stem + '-results.json'))
        register = pinned(revision, f'data/patch-api/sources/{patch}-wikitext-register.json')
        known = pinned(revision, f'tests/data/patch_{patch.replace(".", "_")}_sweep_known_gaps.json')
        gaps = sorted(key for key, value in results.items() if not value['ok'])
        assert gaps == sorted(known)
        assert set(results) == {row['id'] for row in register['entries']}
        if patch != '6.2.0':
            assert all(any(case.endswith('::' + function) for case in master_passed) for function in functions)
            master = read(HERE / 'master' / (stem + '-results.json'))
            assert results == master, patch
        pages.append({'patch': patch, 'observations': len(results), 'gaps': gaps,
                      'unchanged_vs_master': patch != '6.2.0'})
    assert read(HERE / 'gap-comparison.json') == pages
    assert read(HERE / 'patch_6_2_0_publication_sweep-results.json') == {}
    review = read(HERE / 'supersession-review.json')
    assert review == {'before_gaps': [], 'after_gaps': [], 'replacements': [],
                      'later_intersections': {'6.2.2': [], '6.2.4': []},
                      'pending_item_link_contracts': 2}
    negative = read(HERE / 'negative-results.json')
    assert set(negative) == {'p620-negative-global'} and not negative['p620-negative-global']['ok']
    assert 'P620MissingNegativeControl' in (HERE / 'negative-register.json').read_text()
    return len(pages), len(passed)


def check_commands(context):
    required = {'own-sweep', 'all-sweeps', 'integration-patch', 'prefork-patch',
                'integration-tooltip', 'prefork-tooltip', 'build-startup', 'startup',
                'format', 'mists-check', 'negative', 'master-all-sweeps',
                'test_check_patch_validators', 'test_extract_patch_non_inventory',
                'test_gen_patch_wikitext_register', 'test_patch_audit_validation'}
    assert required <= set(context['receipts'])
    for label, expected in context['receipts'].items():
        receipt = read(HERE / (label + '.proof.json'))
        assert receipt == expected
        assert digest((HERE / receipt['log']).read_bytes()) == receipt['log_sha256']
        assert receipt['exit'] != 0 if label == 'negative' else receipt['exit'] == 0
        if receipt['command'][:2] == ['cargo', 'test'] and label != 'negative':
            counts = re.findall(r'test result: ok\. (\d+) passed; 0 failed;', (HERE / receipt['log']).read_text())
            assert counts and sum(map(int, counts)) > 0
        for directory in ('src', 'tests', 'tools'):
            if label == 'master-all-sweeps':
                continue
            actual = git('rev-parse', receipt['revision'] + ':' + directory).decode().strip()
            assert actual == context['input_trees'][directory]
    log = (HERE / 'integration-tooltip.txt').read_text()
    for module in ('tooltip_item_spell::', 'tooltip_basic::', 'tooltip_spell_mount_identifiers::'):
        assert module in log and re.search(r'^test ' + re.escape(module) + r'\S+ \.\.\. ok$', log, re.M)
    assert (HERE / 'startup.txt').read_text().rstrip().endswith('[]')
    assert '--no-addons' not in context['receipts']['startup']['command']
    warnings = [line for line in (HERE / 'mists-check.txt').read_text().splitlines() if line.startswith('warning:')]
    assert all('iced-wgpu-patched/Cargo.toml:' in line or '`iced_wgpu` (manifest)' in line for line in warnings), warnings
    assert 'new gaps: ["p620-negative-global"]' in (HERE / 'negative.txt').read_text()
    matrix = read(HERE / 'prior-validator-matrix.json')
    paths = {path for path in names(context['master_revision'], 'data/patch-api/evidence')
             if Path(path).name in ('validate.py', 'validate_integrated.py')}
    assert {row['path'] for row in matrix} == paths
    assert all(row['exit'] == 0 and digest((HERE / row['log']).read_bytes()) == row['log_sha256'] for row in matrix)
    return len(matrix)


def main():
    context = read(HERE / 'context.json')
    for path, expected in read(HERE / 'artifact-hashes.json').items():
        assert digest((HISTORY / path).read_bytes()) == expected, path
    check_mapping()
    check_preservation(context)
    registers, extracts = check_reproduction(context)
    pages, cases = check_sweeps(context)
    validators = check_commands(context)
    print(json.dumps({'status': 'PASS', 'registers': registers, 'extracts': extracts,
                      'pages': pages, 'cases': cases, 'prior_validators': validators}, sort_keys=True))


if __name__ == '__main__':
    main()
