"""Event-wait for logged checks, seal committed receipts, then run portability gates."""
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
AUDIT = ROOT / 'docs/wiki/investigations/patch-5-4-1-api-audit.md'


def git(*arguments):
    return subprocess.check_output(['git', *arguments], cwd=ROOT, text=True)


def commit(message):
    for name, minimum in [('docs/wiki/index.md', 2751), ('docs/wiki/log.md', 508)]:
        current = (ROOT / name).read_text()
        previous = git('show', 'HEAD:' + name)
        assert len(current.splitlines()) >= max(minimum, len(previous.splitlines())), name
    git('add', SESSION, 'docs/wiki/investigations/patch-5-4-1-api-audit.md',
        'docs/wiki/index.md', 'docs/wiki/log.md',
        ':!' + SESSION + '/integrated/finalization.txt',
        ':!' + SESSION + '/integrated/finalization.launcher.txt',
        ':!' + SESSION + '/integrated/finalization.proof.json')
    print(git('commit', '-m', message), flush=True)


def read(name):
    return json.loads((HERE / name).read_text())


def replace(path, before, after):
    contents = path.read_text()
    assert before in contents, (path, before)
    path.write_text(contents.replace(before, after))


def summary():
    commands = {}
    for receipt in read('receipts.json').values():
        contents = (HERE / receipt['log']).read_text()
        commands[receipt['scope']] = {
            'command': receipt['command'], 'revision': receipt['revision'],
            'exit': receipt['exit'],
            'test_results': re.findall(r'test result: [^\n]+', contents),
            'python_tests': re.findall(r'Ran (\d+) tests? in [^\n]+', contents),
            'warnings': [line for line in contents.splitlines() if line.startswith('warning:')],
        }
    (HERE / 'command-results.json').write_text(json.dumps(commands, indent=2) + '\n')
    return commands


def main():
    if not (HERE / 'mists-routing-correction.proof.json').is_file():
        descriptor = os.pidfd_open(int(sys.argv[1]))
        try:
            select.select([descriptor], [], [])
        finally:
            os.close(descriptor)
    assert read('checks.proof.json')['exit'] == 0
    assert read('mists-routing-correction.proof.json')['exit'] == 0
    assert read('prior-validators.proof.json')['exit'] == 0
    positive = read('patch_5_4_1_publication_sweep-results.json')
    historical = json.loads((HERE.parent / 'patch_5_4_1_publication_sweep-results.json').read_text())
    changes = {key: {'before': historical[key], 'after': positive[key]}
               for key in positive if historical[key] != positive[key]}
    assert not changes, changes
    review = {'integrated_registers': ['5.4.2', '5.4.7', '5.4.8'],
              'before_gaps': sum(not row['ok'] for row in historical.values()),
              'after_gaps': sum(not row['ok'] for row in positive.values()),
              'observation_changes': changes, 'replacements': [],
              'reason': 'No 5.4.1 symbol occurs in the three newly integrated later registers; neither existing backing-state gap is resolved. Historical gap fixtures and ledger remain unchanged.'}
    assert review['before_gaps'] == review['after_gaps'] == 2
    (HERE / 'supersession-review.json').write_text(json.dumps(review, indent=2) + '\n')
    subprocess.run([sys.executable, '-B', str(HERE / 'seal_receipts.py'), 'summarize'], cwd=ROOT, check=True)
    commands = summary()
    comparison = read('gap-comparison.json')
    assert len(comparison) == 61 and sum(row['unchanged_vs_master'] is True for row in comparison) == 60
    other_observations = sum(row['observations'] for row in comparison if row['patch'] != '5.4.1')
    python_tests = sum(int(count) for row in commands.values() for count in row['python_tests'])
    runtime_summary = ('Fresh retail publication sweeps pass on branch and pinned master; all '
                       + str(other_observations) + ' observations on 60 other pages equal master, including separate Classic 5.5.2/5.5.3/5.5.4 sweeps. Both own prefork cases pass; integration selector has zero cases and makes no behavior claim. Python fixtures '
                       + str(python_tests) + '/' + str(python_tests)
                       + ', format and Mists check pass with zero non-vendor warnings. Negative control changes gaps 2 → 3 and fails exactly as required. No supersession replacement, new gap or runtime change. Final portability gate remains pending.')
    contents = AUDIT.read_text()
    marker = 'Fresh sweep, negative, cross-page, warning and portability results pending.'
    if marker in contents:
        contents = contents.replace(marker, runtime_summary)
    else:
        start = contents.index('Fresh retail publication sweeps pass on branch and pinned master;')
        end = contents.index('Final portability gate remains pending.', start) + len('Final portability gate remains pending.')
        contents = contents[:start] + runtime_summary + contents[end:]
    AUDIT.write_text(contents)
    commit('Seal 5.4.1 sweep receipts and unchanged retail and Classic comparisons')
    subprocess.run([sys.executable, '-B', str(HERE / 'seal_receipts.py'), 'seal'], cwd=ROOT, check=True)
    validator = HERE / 'validate.py'
    source = validator.read_text()
    source, count = re.subn(r"CONTEXT_SHA256 = '[^']+'", 'CONTEXT_SHA256 = ' + repr(hashlib.sha256((HERE / 'context.json').read_bytes()).hexdigest()), source)
    assert count == 1
    validator.write_text(source)
    commit('Pin portable integrated 5.4.1 context and evidence seals')
    if run_proof('integrated-validator', [sys.executable, '-B', str(validator)]):
        return 1
    if run_proof('own-seal-controls', [sys.executable, '-B', str(HERE / 'test_own_seals.py')]):
        return 1
    commit('Retain passing 5.4.1 integrated validator and own-log tamper controls')
    if run_proof('validator-gate', [sys.executable, '-B', str(ROOT / 'tools/check_patch_validators.py')]):
        return 1
    report = read('validator-gate.txt')
    assert report['status'] == 'PASS'
    counts = report['summary']
    result = (f"Portable gate PASS at {report['revision']}: clean {counts['clean']['passed']}/{counts['clean']['passed']}, "
              f"synthetic later audit {counts['later_audit']['passed']}/{counts['later_audit']['passed']}; zero failures. Historical/integrated own-log tampering rejects the exact seal and restores all bytes. Relocated replay passes with all ten original pre-rebase objects absent.")
    replace(AUDIT, 'Final portability gate remains pending.', result)
    index = ROOT / 'docs/wiki/index.md'
    contents = index.read_text()
    start = contents.index('[Audit](investigations/patch-5-4-1-api-audit.md):')
    end = contents.index('\n', start)
    contents = contents[:start] + '[Audit](investigations/patch-5-4-1-api-audit.md): integrated against 279a38f3d; 15 observations, two publication/two prose gaps unchanged, no replacements/runtime edits. Seven rebased/three external pins and 205 historical artifacts preserved. Master parser origin pinned; only lowercase-reflist remains new. All 61 registers/58 extracts reproduce with three unchanged inherited failures. ' + runtime_summary.replace(' Final portability gate remains pending.', '') + ' ' + result + contents[end:]
    index.write_text(contents)
    log = ROOT / 'docs/wiki/log.md'
    contents = log.read_text()
    log.write_text(contents + '\n[2026-10-08] 5.4.1 integration complete: two gaps unchanged, no replacements/runtime edits; 60 other retail/Classic pages equal master 279a38f3d. All requested command results in integrated/command-results.json. ' + result + '\n')
    (HERE / 'gate-summary.json').write_text(json.dumps({
        'status': report['status'], 'revision': report['revision'],
        'summary': counts, 'gap_changes': review, 'other_pages_unchanged': 60,
        'other_observations_unchanged': other_observations,
        'python_fixtures': python_tests, 'unfinished': 'Two publication and two prose backing-state contracts remain explicitly pending; no new gap.'}, indent=2) + '\n')
    commit('Record passing 5.4.1 clean and later-audit portability gates')
    print(json.dumps({'status': 'PASS', 'head': git('rev-parse', 'HEAD').strip(), 'gate': counts}), flush=True)
    return 0


if __name__ == '__main__':
    sys.exit(main())
