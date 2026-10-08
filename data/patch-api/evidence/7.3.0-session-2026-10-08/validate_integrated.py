"""Validate fresh integrated receipts without replacing sealed historical evidence."""
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
FRESH = HERE / 'integrated'
sys.path.insert(0, str(ROOT / 'tools'))
from patch_audit_validation import historical_registers, historical_sweep_tests


def read(path):
    return json.loads(path.read_text())


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def errors(log):
    """Decode the Lua error JSON, ignoring interleaved startup diagnostics."""
    for line in re.finditer(r'^\[', log, re.M):
        try:
            value, _ = json.JSONDecoder().raw_decode(log[line.start():])
        except ValueError:
            continue
        if isinstance(value, list):
            return value
    raise AssertionError('no Lua error JSON in startup log')


def check_sources(revision):
    expected = {p.name.removesuffix('-wikitext-register.json')
                for p in historical_registers(ROOT, revision)}
    registers = read(FRESH / 'p730-register-reproduction.json')
    extracts = read(FRESH / 'p730-saved-extract-reproduction.json')
    assert {r['patch'] for r in registers} == {r['patch'] for r in extracts} == expected
    assert len(registers) == len(extracts) == len(expected)
    inherited = {r['patch']: r for r in read(HERE / 'p730-saved-extract-reproduction.json')}
    for row in registers:
        path = ROOT / 'data/patch-api/sources' / (row['patch'] + '-wikitext-register.json')
        assert row['exit'] == 0 and row['byte_identical'] and digest(path) == row['sha256']
        provenance = read(path.with_name(row['patch'] + '-api-changes.provenance.json'))
        if provenance.get('generator_flags') is not None:
            assert row['verified_flags'] == provenance['generator_flags']
    for row in extracts:
        path = ROOT / 'data/patch-api/sources' / (row['patch'] + '-api-changes.txt')
        assert digest(path) == row['sha256']
        provenance = read(path.with_name(row['patch'] + '-api-changes.provenance.json'))
        if provenance.get('extractor_flags') is not None:
            assert row['verified_flags'] == provenance['extractor_flags']
        if not row['byte_identical']:
            old = inherited[row['patch']]
            assert not old['byte_identical'] and (row['exit'], row['error']) == (old['exit'], old['error'])
    extension = read(FRESH / 'extension/p730-register-reproduction.json')
    assert {r['patch'] for r in extension} == expected
    assert all(r['byte_identical'] for r in extension)
    added = next(r for r in extension if r['patch'] == '7.3.2')
    assert added['verified_flags'] == ['--prose-api-links']
    return len(expected), sum(r['byte_identical'] for r in extracts)


def check_sweeps(revision):
    log = (FRESH / 'all-sweeps.log').read_text()
    passed = set(re.findall(r'^test (\S+) \.\.\. ok$', log, re.M))
    count = 0
    observations = 0
    for path in historical_sweep_tests(ROOT, revision):
        source = subprocess.check_output(['git', 'show', f'{revision}:{path.relative_to(ROOT)}'], cwd=ROOT, text=True)
        functions = re.findall(r'fn (patch_\w+_publication_sweep)\(', source)
        assert functions
        for name in functions:
            assert any(p.endswith('::' + name) for p in passed), name
            count += 1
        results = read(FRESH / (path.stem + '-results.json'))
        patch = path.stem.removeprefix('patch_').removesuffix('_publication_sweep').replace('_', '.')
        register = read(ROOT / 'data/patch-api/sources' / (patch + '-wikitext-register.json'))
        assert set(results) == {r['id'] for r in register['entries']}
        gap_path = 'tests/data/patch_' + patch.replace('.', '_') + '_sweep_known_gaps.json'
        known = json.loads(subprocess.check_output(['git', 'show', f'{revision}:{gap_path}'], cwd=ROOT))
        assert {k for k, v in results.items() if not v['ok']} == set(known), patch
        observations += len(results)
    summary = read(FRESH / 'extension/p730-sweep-summary.json')
    assert len(summary) == count
    assert next(r for r in summary if r['patch'] == '7.3.2') == {
        'patch': '7.3.2', 'rows': 2, 'ok': 2, 'gaps': 0, 'result': 'pass'}
    return count, len(passed), observations


def check_receipts(context):
    for label in context['passing_proofs']:
        receipt = read(FRESH / (label + '.proof.json'))
        log_path = FRESH / receipt['log']
        assert receipt['exit'] == 0 and digest(log_path) == receipt['log_sha256'], label
        log = log_path.read_text()
        if receipt['command'][:2] == ['cargo', 'test']:
            matches = re.findall(r'test result: ok\. (\d+) passed; 0 failed;', log)
            assert matches and any(int(n) > 0 for n in matches), 'empty test selection: ' + label
        if label.startswith('master-'):
            assert receipt['source_revision'] == context['master_revision']
        else:
            assert receipt['source_revision'] == receipt['revision']
            for name, expected in context['runtime_scope'].items():
                data = subprocess.check_output(['git', 'show', f"{receipt['source_revision']}:{name}"], cwd=ROOT)
                assert hashlib.sha256(data).hexdigest() == expected, (label, name)
        if label == 'mists-check':
            assert receipt['command'] == ['cargo', 'check', '--no-default-features', '--features',
                                          'sound,gui,casc,client-mists', '--tests']
            assert all('iced-wgpu-patched/Cargo.toml:' in line or '`iced_wgpu` (manifest)' in line
                       for line in log.splitlines() if line.startswith('warning:'))
    negative = read(FRESH / 'negative.proof.json')
    assert negative['exit'] == 1 and digest(FRESH / negative['log']) == negative['log_sha256']
    assert 'resolved/stale gaps: []' in (FRESH / negative['log']).read_text()
    changed = read(HERE / 'p730-negative-register.json')['entries'][0]['id']
    assert {k for k, v in read(FRESH / 'negative-results.json').items() if not v['ok']} == {changed}
    assert errors((FRESH / 'startup.log').read_text()) == []
    branch = errors((FRESH / 'startup-addons.log').read_text())
    master = errors((FRESH / 'master-startup-addons.log').read_text())
    assert branch == master, 'addon Lua errors differ from master'
    source = read(FRESH / 'master-source-verification.json')
    assert source['exit'] == 0 and source['source_revision'] == context['master_revision']
    comparison = read(FRESH / 'lua-errors-comparison.json')
    assert comparison['branch'] == branch and comparison['master'] == master
    assert comparison['master_revision'] == context['master_revision']
    return len(context['passing_proofs']), len(branch)


def main():
    for name, expected in read(FRESH / 'artifact-hashes.json').items():
        assert digest(ROOT / name) == expected, 'changed integrated artifact: ' + name
    context = read(FRESH / 'context.json')
    registers, extracts = check_sources(context['runtime_revision'])
    pages, cases, observations = check_sweeps(context['runtime_revision'])
    proofs, addon_errors = check_receipts(context)
    print(json.dumps({'status': 'PASS', 'registers': registers, 'extracts': extracts,
                      'pages': pages, 'cases': cases, 'observations': observations,
                      'passing_proofs': proofs, 'addon_errors': addon_errors}, sort_keys=True))


if __name__ == '__main__':
    main()
