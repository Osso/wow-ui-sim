"""Replay unchanged historical invariants from preserved bytes and rebased Git pins."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent
HISTORY = HERE.parent


def git(*args, input=None):
    return subprocess.check_output(['git', *args], cwd=ROOT, input=input)


def main():
    mapping = json.loads((HERE / 'rebase-mapping.json').read_text())
    scopes = {row['recorded_revision']: row for row in mapping['pinned_inputs']}
    revisions = {row['recorded_revision']: row['rebased_revision']
                 for row in mapping['commits'] + mapping['external_commits']}

    def canonical(revision):
        matches = [pin for pin in revisions if pin.startswith(revision)]
        assert len(matches) <= 1, revision
        return matches[0] if matches else revision

    def historical_blob(revision, path):
        pin = canonical(revision)
        if pin not in revisions:
            return git('show', pin + ':' + path)
        row = scopes[pin]['inputs'][path]
        content = ((HERE / row['historical_file']).read_bytes() if 'historical_file' in row
                   else git('show', revisions[pin] + ':' + path))
        assert hashlib.sha256(content).hexdigest() == row['sha256'], (pin, path)
        assert git('hash-object', '--stdin', input=content).decode().strip() == row['recorded_blob']
        return content

    def historical_names(revision, directory):
        pin = canonical(revision)
        if pin in revisions:
            return scopes[pin]['inventories'][directory]
        return git('ls-tree', '-r', '--name-only', pin, directory).decode().splitlines()

    def entries(revision, directories):
        pin = canonical(revision)
        if pin in scopes:
            return {path: row['recorded_blob'] for path, row in scopes[pin]['inputs'].items()
                    if any(path == directory or path.startswith(directory + '/') for directory in directories)}
        return {line.split('\t', 1)[1]: line.split('\t', 1)[0].split()[2]
                for line in git('ls-tree', '-r', pin, *directories).decode().splitlines()}

    class HistoricalSubprocess:
        @staticmethod
        def check_output(command, **kwargs):
            if command[:3] == ['git', 'diff', '--name-only']:
                before, after = entries(command[3], command[6:]), entries(command[4], command[6:])
                changed = sorted(path for path in set(before) | set(after) if before.get(path) != after.get(path))
                result = '\n'.join(changed) + ('\n' if changed else '')
                return result if kwargs.get('text') else result.encode()
            return subprocess.check_output(command, **kwargs)

    source = (HERE / 'historical-validator.py.txt').read_text()
    namespace = {'__name__': 'historical_554_validator', '__file__': str(HERE / 'historical-validator.py.txt')}
    # Inventory helpers are supplied below from the recorded Git inventories,
    # not imported from whichever shared tool happens to be in the live tree.
    source = source.replace('from patch_audit_validation import historical_registers, historical_sweep_tests', '')
    exec(compile(source, 'historical-validator.py.txt', 'exec'), namespace)
    namespace.update(ROOT=ROOT, HERE=HISTORY, blob=historical_blob, subprocess=HistoricalSubprocess,
                     historical_registers=lambda root, rev: [Path(path) for path in historical_names(rev, 'data/patch-api/sources') if path.endswith('-wikitext-register.json')],
                     historical_sweep_tests=lambda root, rev: [Path(path) for path in historical_names(rev, 'tests') if path.endswith('_publication_sweep.rs')])
    namespace['main']()


if __name__ == '__main__':
    main()
