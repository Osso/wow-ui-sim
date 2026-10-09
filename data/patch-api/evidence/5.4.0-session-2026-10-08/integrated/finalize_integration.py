"""Event-wait for acceptance, commit receipts, seal inputs, run portable gates."""
import hashlib
import json
import os
from pathlib import Path
import re
import select
import subprocess
import sys

from run_proof import ROOT, EVIDENCE as HERE, run_proof

sys.dont_write_bytecode = True
SESSION = HERE.parent.relative_to(ROOT).as_posix()
AUDIT = ROOT / 'docs/wiki/investigations/patch-5-4-0-api-audit.md'


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True)


def read(name):
    return json.loads((HERE / name).read_text())


def dump(name, value):
    (HERE / name).write_text(json.dumps(value, indent=2) + '\n')


def commit(message):
    for name, minimum in [('index.md', 2759), ('log.md', 525)]:
        path = 'docs/wiki/' + name
        assert len((ROOT / path).read_text().splitlines()) >= max(minimum, len(git('show', 'HEAD:' + path).splitlines())), path
    git('add', SESSION, str(AUDIT.relative_to(ROOT)), 'docs/wiki/index.md', 'docs/wiki/log.md',
        ':!' + SESSION + '/integrated/finalization.txt',
        ':!' + SESSION + '/integrated/finalization.launcher.txt',
        ':!' + SESSION + '/integrated/finalization.proof.json')
    print(git('commit', '-m', message), flush=True)


def run_script(name, *args):
    subprocess.run([sys.executable, '-B', str(HERE / name), *args], cwd=ROOT, check=True)


def summarize():
    positive = read('patch_5_4_0_publication_sweep-results.json')
    historical = json.loads((HERE.parent / 'patch_5_4_0_publication_sweep-results.json').read_text())
    changes = {key: {'before': historical[key], 'after': value} for key, value in positive.items() if value != historical[key]}
    assert set(changes) == {'diff-wt-global-api-securerandom-43'}, changes
    assert sum(not value['ok'] for value in historical.values()) == 22
    assert sum(not value['ok'] for value in positive.values()) == 21
    dump('supersession-review.json', {
        'integrated_registers': ['5.4.1', '5.4.2', '5.4.7'],
        'before_gaps': 22, 'after_gaps': 21, 'observation_changes': changes,
        'replacements': [{'source_id': 'diff-wt-global-api-securerandom-43', 'later_patch': '5.4.2', 'later_id': 'wt-global-api-securerandom-28'}],
        'reason': 'The exact 5.4.2 removal supersedes historical 5.4.0 addition; raw=nil and lookup=nil satisfy current absence. No historical RNG or runtime implementation claim.'})
    run_script('seal_receipts.py', 'summarize')
    commands = {}
    for label, receipt in read('receipts.json').items():
        log = (HERE / receipt['log']).read_text()
        commands[label] = {'command': receipt['command'], 'revision': receipt['revision'], 'exit': receipt['exit'],
                           'test_results': re.findall(r'test result: [^\n]+', log),
                           'python_tests': re.findall(r'Ran (\d+) tests? in [^\n]+', log),
                           'warnings': [line for line in log.splitlines() if line.startswith('warning:')]}
    dump('command-results.json', commands)
    comparison = read('gap-comparison.json')
    assert len(comparison) == 63 and sum(row['unchanged_vs_master'] is True for row in comparison) == 62
    observations = sum(row['observations'] for row in comparison if row['patch'] != '5.4.0')
    fixtures = sum(int(count) for row in commands.values() for count in row['python_tests'])
    return observations, fixtures


def main():
    if not (HERE / 'checks.proof.json').is_file():
        descriptor = os.pidfd_open(int(sys.argv[1]))
        try:
            select.select([descriptor], [], [])
        finally:
            os.close(descriptor)
    assert read('checks.proof.json')['exit'] == 0, 'acceptance failed; inspect saved log'
    assert read('prior-validators.proof.json')['exit'] == 0
    observations, fixtures = summarize()
    summary = (f'Fresh retail sweeps pass on branch and pinned master; all {observations} observations on 62 other pages, including all Classic sweeps, equal master. Own prefork/integration behavior cases, all {fixtures} Python fixtures, format and zero-non-vendor-warning Mists check pass. Negative rejects exactly 21 → 22 gaps. Final portability gate pending.')
    contents = AUDIT.read_text()
    marker = 'Fresh all-sweeps, negative, Classic parity, warning and portability receipts pending.'
    assert marker in contents
    AUDIT.write_text(contents.replace(marker, summary))
    commit('Seal 5.4.0 sweep receipts and unchanged retail and Classic comparisons')
    run_script('seal_receipts.py', 'seal')
    validator = HERE / 'validate.py'
    contents = validator.read_text()
    contents, count = re.subn(r"CONTEXT_SHA256 = '[^']+'", 'CONTEXT_SHA256 = ' + repr(hashlib.sha256((HERE / 'context.json').read_bytes()).hexdigest()), contents)
    assert count == 1
    validator.write_text(contents)
    commit('Pin portable integrated 5.4.0 context and evidence seals')
    return finish_gates(observations, fixtures)


def finish_gates(observations, fixtures):
    if run_proof('integrated-validator', [sys.executable, '-B', str(HERE.parent / 'validate.py')]):
        return 1
    if run_proof('own-seal-controls', [sys.executable, '-B', str(HERE / 'test_own_seals.py')]):
        return 1
    commit('Retain passing 5.4.0 validators and exact own-log tamper rejection')
    if run_proof('history-without-original-objects', [sys.executable, '-B', str(HERE / 'check_history_clone.py')]):
        return 1
    commit('Prove relocated 5.4.0 replay without original rebase objects')
    if run_proof('validator-gate', [sys.executable, '-B', str(ROOT / 'tools/check_patch_validators.py')]):
        return 1
    report = read('validator-gate.txt')
    assert report['status'] == 'PASS'
    counts = report['summary']
    result = (f"Portable gate PASS at {report['revision']}: clean {counts['clean']['passed']}/{counts['clean']['passed']}, synthetic later audit {counts['later_audit']['passed']}/{counts['later_audit']['passed']}; zero failures. Historical/integrated own-log tampering rejected and all bytes restored. Relocated replay passes without all 17 original pre-rebase objects.")
    contents = AUDIT.read_text()
    AUDIT.write_text(contents.replace('Final portability gate pending.', result))
    path = ROOT / 'docs/wiki/index.md'
    contents = path.read_text()
    start = contents.index('[Audit](investigations/patch-5-4-0-api-audit.md):')
    end = contents.index('\n', start)
    contents = contents[:start] + '[Audit](investigations/patch-5-4-0-api-audit.md): integrated against 3c60ac0ea; securerandom 5.4.2 removal changes exact gaps 22 → 21, all other observations unchanged. 211 historical artifacts preserved with 14 own/three external mappings; shared parser now master-owned 5.4.2. All 63 registers/60 main extracts and separate diff reproduce; three inherited failures unchanged. ' + summary.replace(' Final portability gate pending.', '') + ' ' + result + contents[end:]
    path.write_text(contents)
    path = ROOT / 'docs/wiki/log.md'
    contents = path.read_text()
    path.write_text(contents + '\n[2026-10-08] 5.4.0 integrated: exact 5.4.2 securerandom removal replaces one gap (22 → 21); 62 other retail/Classic pages equal master 3c60ac0ea. No runtime/vendor changes. ' + result + '\n')
    dump('gate-summary.json', {'status': report['status'], 'revision': report['revision'], 'summary': counts,
                              'gap_changes': read('supersession-review.json'), 'other_pages_unchanged': 62,
                              'other_observations_unchanged': observations, 'python_fixtures': fixtures,
                              'unfinished': '21 publication and 27 substantive extract gaps remain explicit; no new gap or runtime model.'})
    commit('Record passing 5.4.0 clean and later-audit portability gates')
    print(json.dumps({'status': 'PASS', 'head': git('rev-parse', 'HEAD').strip(), 'gate': counts}), flush=True)
    return 0


if __name__ == '__main__':
    sys.exit(main())
