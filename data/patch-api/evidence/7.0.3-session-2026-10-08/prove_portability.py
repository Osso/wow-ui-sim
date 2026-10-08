"""Validate an archived checkout, expansion by later audits and input tampering."""
import hashlib
import io
import json
from pathlib import Path
import subprocess
import sys
import tarfile

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
DESTINATION = Path('/home/osso/.cache/wow-ui-sim-targets/p703-page/validator-relocated')


def run_validator(root):
    validator = root / 'data/patch-api/evidence/7.0.3-session-2026-10-08/validate.py'
    result = subprocess.run([sys.executable, '-B', str(validator)], cwd=ROOT,
                            capture_output=True, text=True)
    return {'exit': result.returncode, 'stdout': result.stdout, 'stderr': result.stderr}


def main():
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    archive = subprocess.check_output(['git', 'archive', revision], cwd=ROOT)
    DESTINATION.mkdir(parents=True, exist_ok=True)
    with tarfile.open(fileobj=io.BytesIO(archive)) as handle:
        handle.extractall(DESTINATION, filter='data')
    # Git-object reads use the full existing history; no worktree registration or mutation.
    gitdir = subprocess.check_output(['git', 'rev-parse', '--absolute-git-dir'], cwd=ROOT, text=True).strip()
    gitfile = DESTINATION / '.git'
    if gitfile.exists():
        previous_gitfile = gitfile.read_text()
    gitfile.write_text('gitdir: ' + gitdir + '\n')
    before = run_validator(DESTINATION)
    assert before['exit'] == 0, before
    register = DESTINATION / 'data/patch-api/sources/6.0.0-portability-only-wikitext-register.json'
    sweep = DESTINATION / 'tests/patch_6_0_0_portability_only_publication_sweep.rs'
    register.write_text('{"schema":"patch-api-wikitext-register/v1","patch":"6.0.0-portability-only","entries":[]}\n')
    sweep.write_text('// Hypothetical later audit: historical proof must not grow with checkout contents.\n')
    expanded = run_validator(DESTINATION)
    assert expanded['exit'] == 0, expanded
    assert json.loads(before['stdout']) == json.loads(expanded['stdout'])
    ledger = DESTINATION / 'data/patch-api/sources/7.0.3-page-coverage.json'
    original = ledger.read_text()
    ledger.write_text(original + '\n')
    try:
        tampered = run_validator(DESTINATION)
        assert tampered['exit'] != 0 and '7.0.3-page-coverage.json' in tampered['stderr'], tampered
    finally:
        ledger.write_text(original)
    restored = run_validator(DESTINATION)
    assert restored['exit'] == 0, restored
    evidence = HERE / 'p703-validator-portability.json'
    if evidence.exists():
        previous_evidence = evidence.read_text()
    evidence.write_text(json.dumps({
        'revision': revision, 'relocated_root': str(DESTINATION), 'before': before,
        'extra_audit_inputs': [str(register.relative_to(DESTINATION)), str(sweep.relative_to(DESTINATION))],
        'expanded': expanded, 'tampered': tampered, 'restored': restored,
        'restored_ledger_sha256': hashlib.sha256(ledger.read_bytes()).hexdigest(),
        'read_only_validator': 'Only this fixture driver writes; validate.py performs Git/file reads only.',
    }, indent=2) + '\n')
    print('PASS: relocated checkout, historical scope with extra audit files, tamper rejection and restoration')


if __name__ == '__main__':
    main()
