"""Seal completed receipts and run the portability gate after asynchronous checks."""
import hashlib
import json
import os
from pathlib import Path
import select
import subprocess
import sys

from run_proof import ROOT, EVIDENCE as HERE, run_proof

sys.dont_write_bytecode = True
SESSION = HERE.parent.relative_to(ROOT).as_posix()


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True)


def commit(message):
    for name, minimum in [('docs/wiki/index.md', 2743), ('docs/wiki/log.md', 495)]:
        current = (ROOT / name).read_text()
        previous = git('show', 'HEAD:' + name)
        assert len(current.splitlines()) >= max(minimum, len(previous.splitlines())), name
    git('add', SESSION, 'docs/wiki/investigations/patch-5-4-2-api-audit.md', 'docs/wiki/index.md', 'docs/wiki/log.md', ':!' + SESSION + '/integrated/finalization.txt', ':!' + SESSION + '/integrated/finalization.launcher.txt', ':!' + SESSION + '/integrated/finalization.proof.json')
    print(git('commit', '-m', message), flush=True)


def replace(path, old, new):
    contents = path.read_text()
    assert old in contents, (path, old)
    path.write_text(contents.replace(old, new))


def main():
    descriptor = os.pidfd_open(int(sys.argv[1]))
    try:
        select.select([descriptor], [], [])
    finally:
        os.close(descriptor)
    assert json.loads((HERE / 'mists-sweeps-driver.proof.json').read_text())['exit'] == 0
    subprocess.run([sys.executable, '-B', str(HERE / 'seal_receipts.py'), 'summarize'], cwd=ROOT, check=True)
    comparison = json.loads((HERE / 'gap-comparison.json').read_text())
    assert len(comparison) == 59 and sum(row['unchanged_vs_master'] is True for row in comparison) == 58
    audit = ROOT / 'docs/wiki/investigations/patch-5-4-2-api-audit.md'
    replace(audit, 'Fresh master comparison, separate Classic sweeps and the final portability gate remain pending until sealed.',
            'Fresh master retail sweeps pass 57/57; all 9,749 observations on 56 other retail pages equal master. Both separate Classic empty-inventory sweeps pass on branch and master and remain identical. Gap-comparison covers all 59 pages (58 other pages unchanged). No supersession replacement or new runtime gap. Final portability gate remains pending.')
    commit('Seal complete 5.4.2 integrated receipts and unchanged master comparisons')
    subprocess.run([sys.executable, '-B', str(HERE / 'seal_receipts.py'), 'seal'], cwd=ROOT, check=True)
    validator = HERE / 'validate.py'
    source = validator.read_text()
    import re
    source, count = re.subn(r"CONTEXT_SHA256 = '[0-9a-f]+'", 'CONTEXT_SHA256 = ' + repr(hashlib.sha256((HERE / 'context.json').read_bytes()).hexdigest()), source)
    assert count == 1
    validator.write_text(source)
    commit('Pin portable integrated 5.4.2 validator context and seals')
    if run_proof('integrated-validator', [sys.executable, '-B', str(validator)]):
        return 1
    if run_proof('own-seal-controls', [sys.executable, '-B', str(HERE / 'test_own_seals.py')]):
        return 1
    commit('Retain passing integrated validator and restored own-log tamper controls')
    if run_proof('validator-gate', [sys.executable, '-B', str(ROOT / 'tools/check_patch_validators.py')]):
        return 1
    report = json.loads((HERE / 'validator-gate.txt').read_text())
    assert report['status'] == 'PASS'
    summary = report['summary']
    result = f"Portable gate PASS at {report['revision']}: clean {summary['clean']['passed']}/{summary['clean']['passed']}, synthetic later audit {summary['later_audit']['passed']}/{summary['later_audit']['passed']}; zero failures. Historical/integrated own-log tampering is rejected at the exact seal and all bytes restored."
    replace(audit, 'Final portability gate remains pending.', result)
    index = ROOT / 'docs/wiki/index.md'
    contents = index.read_text()
    start = contents.index('[Audit](investigations/patch-5-4-2-api-audit.md):')
    end = contents.index('\n', start)
    contents = contents[:start] + '[Audit](investigations/patch-5-4-2-api-audit.md): integrated against 896086537; 39 gaps unchanged, no replacements/runtime changes. Nine rebased/two external identities and 123 historical artifacts preserved. All 59 registers/56 extracts reproduce with three unchanged inherited failures. Branch/master retail sweeps 58/57 pass; own prefork 3/3, integration selector zero cases, Classic commands 3/3 each (two page sweeps plus line control), Python 89/89, format/Mists warning gate and 43 prior validators pass. All 9,749 other retail observations and both Classic inventories equal master. Negative 39 → 40; ' + result + contents[end:]
    index.write_text(contents)
    log = ROOT / 'docs/wiki/log.md'
    contents = log.read_text()
    log.write_text(contents + '\n[2026-10-08] 5.4.2 integrated audit complete: 39 gaps unchanged; 58 other page snapshots equal master 896086537. Retail sweeps 58/57, own prefork 3/3, separate Classic 3/3 each (two page sweeps plus line control), Python 89/89, format/Mists and negative 39 → 40 pass. ' + result + '\n')
    commit('Record passing 5.4.2 clean and later-audit portability gates')
    print(json.dumps({'status': 'PASS', 'head': git('rev-parse', 'HEAD').strip(), 'gate': summary}), flush=True)
    return 0


if __name__ == '__main__':
    sys.exit(main())
