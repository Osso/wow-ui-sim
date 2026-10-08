"""Replay every original validator invariant using preserved bytes and mapped Git pins."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent
HISTORY = HERE.parent


def read(path):
    return json.loads(path.read_text())


def digest(value):
    return hashlib.sha256(value).hexdigest()


def git(*args, input=None):
    return subprocess.check_output(['git', *args], cwd=ROOT, input=input)


def main():
    mapping = read(HERE / 'rebase-mapping.json')
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
        if 'historical_file' in row:
            content = (HERE / row['historical_file']).read_bytes()
        else:
            content = git('show', revisions[pin] + ':' + path)
        assert digest(content) == row['sha256'], (pin, path)
        assert git('hash-object', '--stdin', input=content).decode().strip() == row['recorded_blob']
        return content

    def historical_names(revision, directory):
        pin = canonical(revision)
        if pin in revisions:
            return scopes[pin]['inventories'][directory]
        return git('ls-tree', '-r', '--name-only', pin, directory).decode().splitlines()

    def historical_git(*args):
        if args[0] == 'show' and len(args) == 2 and ':' in args[1]:
            return historical_blob(*args[1].split(':', 1))
        if args[:3] == ('ls-tree', '-r', '--name-only'):
            return ('\n'.join(historical_names(args[3], args[4])) + '\n').encode()
        if args[:2] == ('rev-parse', '--verify'):
            revision = args[2].removesuffix('^{commit}')
            pin = canonical(revision)
            return git('rev-parse', '--verify', revisions.get(pin, pin) + '^{commit}')
        return git(*args)

    source = (HERE / 'historical-validator.py.txt').read_text()
    namespace = {'__name__': 'historical_548_validator', '__file__': str(HISTORY / 'validate.py')}
    exec(compile(source, 'historical-validator.py.txt', 'exec'), namespace)
    native_git = historical_git
    def mapped_git(*args):
        if args[:2] == ('diff', '--name-only'):
            left, right = args[2:4]
            directories = args[5:]
            def entries(revision):
                pin = canonical(revision)
                if pin in scopes:
                    return {path: row['recorded_blob'] for path, row in scopes[pin]['inputs'].items()
                            if any(path == directory or path.startswith(directory + '/') for directory in directories)}
                return {line.split('\t', 1)[1]: line.split('\t', 1)[0].split()[2]
                        for line in git('ls-tree', '-r', pin, *directories).decode().splitlines()}
            before, after = entries(left), entries(right)
            changed = sorted(path for path in set(before) | set(after) if before.get(path) != after.get(path))
            return ('\n'.join(changed) + ('\n' if changed else '')).encode()
        return native_git(*args)
    namespace.update(git=mapped_git, blob=historical_blob, paths=historical_names,
                     historical_registers=lambda rev: [p for p in historical_names(rev, 'data/patch-api/sources') if p.endswith('-wikitext-register.json')])
    namespace['main']()


if __name__ == '__main__':
    main()
