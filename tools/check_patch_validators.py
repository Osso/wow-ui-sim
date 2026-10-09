"""Run retained patch proofs in a clean revision and after unrelated later work."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def git(root, *args):
    return subprocess.check_output(['git', *args], cwd=root, stderr=subprocess.PIPE,
                                   text=True)


def oversized_evidence(root):
    """Limit every tracked evidence artifact, not just JSON or validator inputs."""
    paths = git(root, 'ls-files', '-z', '--', 'data/patch-api/evidence').split('\0')
    return [{'path': name, 'bytes': (root / name).stat().st_size}
            for name in paths if name and (root / name).stat().st_size > 5_000_000]


def run_validators(root):
    results = []
    for path in sorted((root / 'data/patch-api/evidence').rglob('validate.py')):
        result = subprocess.run([sys.executable, '-B', str(path)], cwd=root,
                                env=dict(os.environ, PYTHONDONTWRITEBYTECODE='1'),
                                capture_output=True, text=True)
        results.append({'validator': path.relative_to(root).as_posix(),
                        'exit': result.returncode, 'stdout': result.stdout,
                        'stderr': result.stderr})
    if not results:
        raise ValueError('no patch validators found')
    return results


def commit_later_audit(root):
    for name, comment in [('src/c_api/mod.rs', '// Synthetic unrelated later audit.'),
                          ('tools/gen_patch_wikitext_register.py', '# Synthetic unrelated later audit.'),
                          ('docs/wiki/log.md', 'Synthetic unrelated later audit.')]:
        path = root / name
        contents = path.read_text()
        path.write_text(contents + '\n' + comment + '\n')
    sources = root / 'data/patch-api/sources'
    register = json.loads(next(iter(sorted(sources.glob('*-wikitext-register.json')))).read_text())
    register['patch'] = '9.9.9'
    new_register = sources / '9.9.9-wikitext-register.json'
    evidence = root / 'data/patch-api/evidence/9.9.9-synthetic-later-audit/validate.py'
    if new_register.exists() or evidence.parent.exists():
        raise ValueError('synthetic audit paths already exist')
    new_register.write_text(json.dumps(register, indent=2) + '\n')
    evidence.parent.mkdir()
    evidence.write_text('"""Unrelated synthetic audit with no historical inputs."""\nprint("PASS: synthetic later audit")\n')
    git(root, 'add', 'src/c_api/mod.rs', 'tools/gen_patch_wikitext_register.py',
        'docs/wiki/log.md', str(new_register.relative_to(root)), str(evidence.relative_to(root)))
    git(root, '-c', 'user.name=Patch validator gate',
        '-c', 'user.email=patch-validator@example.invalid', '-c', 'commit.gpgsign=false',
        '-c', 'core.hooksPath=/dev/null', 'commit', '-qm', 'Synthetic unrelated later audit')
    return git(root, 'rev-parse', 'HEAD').strip()


def check_revision(root, revision):
    report = {'revision': revision, 'phases': {}, 'summary': {}, 'oversized_evidence': {}}
    with tempfile.TemporaryDirectory(prefix='patch-validator-gate-') as directory:
        checkout = Path(directory) / 'checkout'
        try:
            report['revision'] = git(root, 'rev-parse', '--verify', revision + '^{commit}').strip()
            git(root, 'worktree', 'add', '--detach', str(checkout), report['revision'])
            report['oversized_evidence']['clean'] = oversized_evidence(checkout)
            report['phases']['clean'] = run_validators(checkout)
            report['later_revision'] = commit_later_audit(checkout)
            report['oversized_evidence']['later_audit'] = oversized_evidence(checkout)
            report['phases']['later_audit'] = run_validators(checkout)
        except (subprocess.CalledProcessError, OSError, ValueError) as error:
            report['error'] = str(error)
            if isinstance(error, subprocess.CalledProcessError):
                report['error'] += ': ' + (error.stderr or '')
        finally:
            if checkout.exists():
                try:
                    git(root, 'worktree', 'remove', '--force', str(checkout))
                except subprocess.CalledProcessError as error:
                    report['error'] = 'worktree cleanup failed: ' + (error.stderr or str(error))
    for phase, results in report['phases'].items():
        report['summary'][phase] = {'passed': sum(row['exit'] == 0 for row in results),
                                    'failed': sum(row['exit'] != 0 for row in results)}
    report['status'] = 'FAIL' if 'error' in report or any(
        report['oversized_evidence'].values()) or any(
        counts['failed'] for counts in report['summary'].values()) else 'PASS'
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('revision', nargs='?', default='HEAD')
    report = check_revision(ROOT, parser.parse_args().revision)
    print(json.dumps(report, indent=2, sort_keys=True))
    return int(report['status'] != 'PASS')


if __name__ == '__main__':
    sys.exit(main())
