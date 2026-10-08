"""Validate committed 5.5.3 proof without live shared-file comparisons."""
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


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT)


def blob(revision, path):
    return git('show', revision + ':' + path)


def digest(content):
    return hashlib.sha256(content).hexdigest()


def read(name):
    return json.loads((HERE / name).read_text())


def pinned_json(revision, path):
    return json.loads(blob(revision, path))


def verify_accounting(revision):
    prefix = 'data/patch-api/sources/5.5.3-'
    provenance = pinned_json(revision, prefix + 'api-changes.provenance.json')
    raw = blob(revision, prefix + 'api-changes.wikitext')
    register = pinned_json(revision, prefix + 'wikitext-register.json')
    ledger = pinned_json(revision, prefix + 'page-coverage.json')
    assert digest(raw) == provenance['wikitext_sha256'] == register['source']['sha256']
    assert provenance['revid'] == register['source']['revid'] == 6778082
    assert provenance['client_line'] == register['client_line'] == ledger['client_line'] == 'mists-classic'
    assert provenance['generator_flags'] == ['--client-line', 'mists-classic']
    assert provenance['extractor_flags'] == ['--canonical-patch-navigation']
    assert register['entries'] == register['header_counts'] == []
    assert pinned_json(revision, 'tests/data/patch_5_5_3_sweep_known_gaps.json') == []
    extractor = {'__file__': str(ROOT / 'tools/extract_patch_non_inventory.py'), '__name__': 'pinned_extractor'}
    exec(compile(blob(revision, 'tools/extract_patch_non_inventory.py'), 'pinned_extractor', 'exec'), extractor)
    text = extractor['extract_text'](raw.decode(), canonical_patch_navigation=True)
    assert text.encode() == blob(revision, prefix + 'api-changes.txt')
    assert ledger['source_rows'] == extractor['seed_rows'](text, '5.5.3')
    assert all(row['status'] == 'metadata-only' and not row['capabilities'] for row in ledger['source_rows'])
    assert ledger['source_sha256'] == digest(blob(revision, prefix + 'wikitext-register.json'))
    assert ledger['non_inventory_source']['sha256'] == digest(text.encode())
    blocker = read('p553-profile-blocker.json')
    assert blocker['status'] == 'STOPPED' and blocker['source_toc'] == 50503
    assert blocker['resolution']['status'] == 'RESOLVED'
    assert blocker['resolution']['decision'] == 'TOC 50503, an ancestor build of the 50504 profile; same client line'
    decisions = read('retirement-decisions.json')
    assert decisions['removed_entries'] == decisions['new_retirements'] == []
    later = pinned_json(revision, 'data/patch-api/sources/5.5.4-wikitext-register.json')
    assert later['client_line'] == 'mists-classic'
    assert decisions['later_readditions'] == [row['id'] for row in later['entries'] if row['direction'] == 'added']
    for receipt in read('scan-receipts.json'):
        content = (HERE / receipt['log']).read_bytes()
        assert receipt['tool'] == receipt['command'][0] == '/usr/bin/grep'
        assert '-RInw' in receipt['command'] and receipt['untruncated']
        assert receipt['exit'] in (0, 1) and digest(content) == receipt['log_sha256']
        assert len(content.decode().splitlines()) == receipt['line_count']
    return {'inventory': len(register['entries']), 'metadata': len(ledger['source_rows']), 'gaps': 0}


def verify_reproduction(revision, base):
    report = read('p553-reproduction.json')
    assert report['historical_flags_revision'] == base
    assert not git('diff', report['revision'], revision, '--', 'data/patch-api/sources',
                   'tools/gen_patch_wikitext_register.py', 'tools/extract_patch_non_inventory.py')
    registers = historical_registers(ROOT, revision)
    expected = {path.name.removesuffix('-wikitext-register.json') for path in registers}
    assert {row['patch'] for row in report['records']} == expected
    prior = pinned_json(base, 'data/patch-api/evidence/5.5.4-session-2026-10-08/integrated/p554-reproduction.json')
    prior_rows = {row['patch']: row for row in prior['records']}
    for row in report['records']:
        patch = row['patch']
        prefix = f'data/patch-api/sources/{patch}-'
        assert row['register_exit'] == 0 and row['register_byte_identical'], patch
        assert row['register_sha256'] == digest(blob(revision, prefix + 'wikitext-register.json'))
        assert row['extract_sha256'] == digest(blob(revision, prefix + 'api-changes.txt'))
        if patch in prior_rows:
            before = prior_rows[patch]
            for key in ['generator_flags', 'extractor_flags', 'generated_extract_sha256', 'generated_extract_error', 'extract_byte_identical']:
                assert row[key] == before[key], (patch, key)
        else:
            assert row['extract_byte_identical'] and row['generated_extract_error'] is None
    return {'registers': len(expected), 'extracts': sum(row['extract_byte_identical'] for row in report['records']),
            'inherited_extract_failures': sorted(row['patch'] for row in report['records'] if not row['extract_byte_identical'])}


def verify_retail(revision, base):
    baseline = read('master-baseline.json')
    assert baseline['master_revision'] == base
    scope = ['src', 'tests', 'tools', 'Cargo.toml', 'Cargo.lock', 'build.rs']
    assert not git('diff', baseline['runtime_revision'], base, '--', *scope)
    changed = git('diff', '--name-only', base, revision, '--', *scope).decode().splitlines()
    assert sorted(changed) == ['tests/data/patch_5_5_3_sweep_known_gaps.json', 'tests/patch_5_5_3_publication_sweep.rs']
    assert not git('diff', base, revision, '--', 'Interface')
    expected = []
    for path in historical_sweep_tests(ROOT, base):
        patch = path.stem.removeprefix('patch_').removesuffix('_publication_sweep').replace('_', '.')
        register = pinned_json(base, f'data/patch-api/sources/{patch}-wikitext-register.json')
        if register.get('client_line', 'retail') == 'retail':
            expected.append(patch)
    assert sorted(row['patch'] for row in baseline['records']) == sorted(expected)
    comparison = []
    for row in baseline['records']:
        patch = row['patch']
        content = (HERE / row['path']).read_bytes()
        assert content == blob(base, row['source_path']) and digest(content) == row['sha256']
        before = json.loads(content)
        after = read('results/' + Path(row['path']).name)
        assert before == after, patch
        register = pinned_json(base, f'data/patch-api/sources/{patch}-wikitext-register.json')
        assert set(after) == {entry['id'] for entry in register['entries']}
        gaps = sorted(key for key, result in after.items() if not result['ok'])
        assert gaps == sorted(pinned_json(revision, f'tests/data/patch_{patch.replace(".", "_")}_sweep_known_gaps.json'))
        comparison.append({'patch': patch, 'observations': len(after), 'gaps': gaps, 'identical': True})
    assert read('gap-comparison.json') == comparison
    passed = set(re.findall(r'^test (\S+) \.\.\. ok$', (HERE / 'all-sweeps.txt').read_text(), re.M))
    for patch in expected:
        case = 'patch_' + patch.replace('.', '_') + '_publication_sweep'
        assert any(name.endswith('::' + case) for name in passed), case
    assert any(name.endswith('::patch_10_0_0_animation_probe_factories') for name in passed), passed
    for path in historical_registers(ROOT, base):
        relative = path.relative_to(ROOT).as_posix()
        assert blob(base, relative) == blob(revision, relative), relative
    for patch in ['5_5_3', '5_5_4']:
        assert read(f'results/patch_{patch}_publication_sweep-results.json') == {}
    return {'pages': len(comparison), 'observations': sum(row['observations'] for row in comparison), 'differences': 0}


def verify_receipts(context):
    for label, policy in context['proofs'].items():
        receipt = read(label + '.proof.json')
        assert receipt['command'] == policy['command']
        assert receipt['exit'] == receipt['expected_exit'] == policy['exit'], label
        scope = ['src', 'tests', 'tools', 'Cargo.toml', 'Cargo.lock', 'build.rs']
        if receipt['profile'] == 'retail' and not label.startswith('format'):
            # Retail compiles neither version of this Mists-only module.
            mists_test = 'tests/patch_5_5_3_publication_sweep.rs'
            for revision in [receipt['revision'], context['runtime_revision']]:
                assert '#![cfg(feature = "client-mists")]' in blob(revision, mists_test).decode()
            scope = [*scope, ':!tests/patch_5_5_3_publication_sweep.rs']
        assert not git('diff', receipt['revision'], context['runtime_revision'], '--', *scope), label
        content = (HERE / receipt['log']).read_bytes()
        assert digest(content) == receipt['log_sha256'], label
        if 'passed_cases' in policy:
            passed = set(re.findall(r'^test (\S+) \.\.\. ok$', content.decode(), re.M))
            assert passed == set(policy['passed_cases']), label
            counts = re.findall(r'test result: ok\. (\d+) passed; 0 failed;', content.decode())
            assert counts and int(counts[-1]) == len(passed), label
        if label.startswith('mists') or label == 'negative':
            assert receipt['profile'] == 'mists'
            assert '--no-default-features' in receipt['command']
            assert 'sound,gui,casc,client-mists' in receipt['command']
            assert receipt['target'].endswith('/p553-page-mists')
    check = (HERE / 'mists-check.txt').read_text()
    warnings = [line for line in check.splitlines() if line.startswith('warning:')]
    assert all('iced-wgpu-patched/Cargo.toml:' in line or '`iced_wgpu` (manifest)' in line for line in warnings), warnings
    tools = (HERE / 'tools-tests.txt').read_text()
    assert re.search(r'Ran \d+ tests', tools) and tools.rstrip().endswith('OK')
    negative = (HERE / 'negative.txt').read_text()
    assert 'register row count changed' in negative and 'left: 1' in negative and 'right: 0' in negative
    assert context['proofs']['negative']['exit'] == 101
    assert read('results/patch_5_5_4_publication_sweep-MISTS_LINE_CONTROL_OUT-results.json')['own']['ok'] is False


def main():
    context = read('context.json')
    for name, expected in context['session_sha256'].items():
        path = HERE / name
        assert '..' not in Path(name).parts and path.is_relative_to(HERE)
        assert digest(path.read_bytes()) == expected, name
    for path, expected in context['shared_sha256'].items():
        assert digest(blob(context['runtime_revision'], path)) == expected, path
    accounting = verify_accounting(context['runtime_revision'])
    reproduction = verify_reproduction(context['runtime_revision'], context['master_revision'])
    retail = verify_retail(context['runtime_revision'], context['master_revision'])
    verify_receipts(context)
    print(json.dumps({'status': 'PASS', 'accounting': accounting, 'reproduction': reproduction, 'retail': retail}, sort_keys=True))


if __name__ == '__main__':
    main()
