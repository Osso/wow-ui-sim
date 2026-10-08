"""Read-only validation of separately sealed post-rebase 7.2.5 evidence."""
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
from patch_audit_validation import historical_registers, historical_sweep_tests, historical_json


def read(path):
    return json.loads(path.read_text())


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def blob(revision, name):
    return subprocess.check_output(['git', 'show', f'{revision}:{name}'], cwd=ROOT)


def check_sources(revision):
    paths = historical_registers(ROOT, revision)
    expected = {path.name.removesuffix('-wikitext-register.json') for path in paths}
    registers = read(FRESH / 'p725-register-reproduction.json')
    extracts = read(FRESH / 'p725-saved-extract-reproduction.json')
    extension = read(FRESH / 'extension/p725-register-reproduction.json')
    for rows in (registers, extracts, extension):
        assert len(rows) == len(expected) and {row['patch'] for row in rows} == expected
    inherited = {row['patch']: row for row in read(HERE / 'p725-saved-extract-reproduction.json')}
    for row in registers + extension:
        path = ROOT / 'data/patch-api/sources' / (row['patch'] + '-wikitext-register.json')
        assert row['exit'] == 0 and row['byte_identical'] and row['sha256'] == digest(path)
        provenance = read(path.with_name(row['patch'] + '-api-changes.provenance.json'))
        if provenance.get('generator_flags') is not None:
            assert row['verified_flags'] == provenance['generator_flags']
    added = next(row for row in extension if row['patch'] == '7.3.0')
    assert added['verified_flags'] == ['--legacy-summary-tables']
    assert added['revision'] == read(FRESH / 'extend-receipts.proof.json')['revision']
    for row in extracts:
        path = ROOT / 'data/patch-api/sources' / (row['patch'] + '-api-changes.txt')
        assert digest(path) == row['saved_sha256']
        provenance = read(path.with_name(row['patch'] + '-api-changes.provenance.json'))
        if provenance.get('extractor_flags') is not None:
            assert row['verified_flags'] == provenance['extractor_flags']
        if row['patch'] in inherited:
            before = inherited[row['patch']]
            assert (row['byte_identical'], row['error']) == (before['byte_identical'], before['error'])
        else:
            assert row['byte_identical'] and row['error'] is None
    return len(expected), sum(row['byte_identical'] for row in extracts)


def passed_cases(log):
    return set(re.findall(r'^test (\S+) \.\.\. ok$', log, re.M))


def failed_cases(log):
    return set(re.findall(r'^test (\S+) \.\.\. FAILED', log, re.M))


def check_sweeps(revision):
    passed = passed_cases((FRESH / 'all-sweeps.txt').read_text())
    summaries = {row['patch']: row for row in read(FRESH / 'extension/p725-sweep-summary.json')}
    observations = 0
    changes = []
    paths = historical_sweep_tests(ROOT, revision)
    for path in paths:
        functions = re.findall(r'fn (patch_\w+_publication_sweep)\(', blob(revision, path.relative_to(ROOT)).decode())
        assert functions and all(any(case.endswith('::' + name) for case in passed) for name in functions)
        results = read(FRESH / (path.stem + '-results.json'))
        patch = path.stem.removeprefix('patch_').removesuffix('_publication_sweep').replace('_', '.')
        register = historical_json(ROOT, 'data/patch-api/sources/' + patch + '-wikitext-register.json', revision)
        assert set(results) == {row['id'] for row in register['entries']}
        known = historical_json(ROOT, 'tests/data/patch_' + patch.replace('.', '_') + '_sweep_known_gaps.json', revision)
        gaps = {key for key, row in results.items() if not row['ok']}
        assert gaps == set(known), patch
        assert summaries[patch] == {'patch': patch, 'rows': len(results), 'ok': len(results) - len(gaps),
                                    'gaps': len(gaps), 'result': 'pass'}
        old_path = HERE / (path.stem + '-results.json')
        if old_path.exists():
            previous = {key for key, row in read(old_path).items() if not row['ok']}
            if gaps != previous:
                changes.append({'patch': patch, 'resolved': sorted(previous - gaps), 'new': sorted(gaps - previous)})
        observations += len(results)
    comparison = read(FRESH / 'gap-comparison.json')
    assert comparison['changes'] == changes == []
    assert len(summaries) == len(paths)
    return len(paths), len(passed), observations


def errors(log):
    for match in re.finditer(r'^\[', log, re.M):
        try:
            value, _ = json.JSONDecoder().raw_decode(log[match.start():])
        except ValueError:
            continue
        if isinstance(value, list):
            return value
    raise AssertionError('missing Lua error JSON')


def failure_payload(log, case):
    marker = 'test ' + case + ' ... FAILED (panic: '
    assert log.count(marker) == 1, 'missing/duplicate failure payload: ' + case
    return log.split(marker, 1)[1].split('\n---- ', 1)[0]


def check_receipts(context):
    for label, expected_exit in context['expected_exits'].items():
        receipt = read(FRESH / (label + '.proof.json'))
        log = FRESH / receipt['log']
        assert receipt['exit'] == expected_exit and not receipt['invalidated'], label
        assert receipt['command'] and digest(log) == receipt['log_sha256'], label
        source = context['master_revision'] if label.startswith('master-') else context['runtime_revision']
        assert receipt['source_revision'] == source, label
        if not label.startswith('master-'):
            for name, expected in context['runtime_scope'].items():
                assert hashlib.sha256(blob(receipt['revision'], name)).hexdigest() == expected, (label, name)
        if receipt['command'][:2] == ['cargo', 'test'] and expected_exit == 0:
            counts = re.findall(r'test result: ok\. (\d+) passed; 0 failed;', log.read_text())
            assert counts, label
            if label == 'prefork-diversion':
                assert counts == ['0'], 'update explicitly documented empty diversion selection'
            else:
                assert any(int(count) > 0 for count in counts), 'empty selection: ' + label
    diversion = {case for case in passed_cases((FRESH / 'integration-anima.txt').read_text())
                 if 'blizzard_animadiversionui::' in case}
    assert len(diversion) == 42, 'AnimaDiversion integration coverage changed'
    # These diversion cases use #[test], not the cached prefork case macro.
    assert not any('blizzard_animadiversionui::' in case
                   for case in passed_cases((FRESH / 'prefork_full_ui-anima.txt').read_text()))
    warning_lines = [line for line in (FRESH / 'mists-check.txt').read_text().splitlines() if line.startswith('warning:')]
    assert all('iced-wgpu-patched/Cargo.toml' in line or '`iced_wgpu` (manifest)' in line for line in warning_lines)
    assert errors((FRESH / 'startup-addons.txt').read_text()) == errors((FRESH / 'master-startup-addons.txt').read_text()) == []
    source = read(FRESH / 'master-source-verification.json')
    assert source['source_revision'] == context['master_revision'] and source['exit'] == 0 and source['files_verified'] > 0
    master = (FRESH / 'master-garrison.txt').read_text()
    branch = (FRESH / 'prefork_full_ui-garrison.txt').read_text()
    expected_failure = {'blizzard_garrison_ui_loads::blizzard_garrison_ui_loads_explicitly_via_load_addon_without_errors'}
    assert failed_cases(branch) == failed_cases(master) == expected_failure
    diagnostic = "bad argument #1 to 'ipairs' (table expected, got nil) at ...ard_GarrisonUI/Mainline/Blizzard_AdventuresCombatLog.lua:90"
    case = next(iter(expected_failure))
    branch_failure = failure_payload(branch, case)
    master_failure = failure_payload(master, case)
    assert branch_failure == master_failure, 'cached garrison failure differs from exact master'
    assert diagnostic in branch_failure
    diagnostics = re.findall(r'(?:Failed to create frame|Lua Error:)[^\n]+', branch_failure)
    assert diagnostics and all(diagnostic in line for line in diagnostics), diagnostics
    for row in read(HERE / 'p725-garrison-baseline-context.json')['files']:
        assert blob(context['master_revision'], row['path']) == blob(context['runtime_revision'], row['path'])
    negative = read(FRESH / 'negative-results.json')
    baseline = read(FRESH / 'patch_7_2_5_publication_sweep-results.json')
    control = read(HERE / 'p725-negative-control.json')
    assert sum(not row['ok'] for row in baseline.values()) == control['before'] == 3
    assert sum(not row['ok'] for row in negative.values()) == control['after'] == 4
    assert set(negative) == set(baseline)
    assert {key for key in baseline if baseline[key]['ok'] != negative[key]['ok']} == {control['mutated_id']}
    assert 'resolved/stale gaps: []' in (FRESH / 'negative.txt').read_text()


def main():
    for name, expected in read(FRESH / 'artifact-hashes.json').items():
        assert digest(ROOT / name) == expected, 'changed integrated artifact: ' + name
    context = read(FRESH / 'context.json')
    registers, extracts = check_sources(context['runtime_revision'])
    pages, cases, observations = check_sweeps(context['runtime_revision'])
    check_receipts(context)
    print(json.dumps({'status': 'pass', 'registers': registers, 'extracts': extracts,
                      'pages': pages, 'cases': cases, 'observations': observations,
                      'gaps': 3, 'negative_gaps': 4, 'addon_errors': 0}, sort_keys=True))


if __name__ == '__main__':
    main()
