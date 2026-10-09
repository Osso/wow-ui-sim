"""Replay unchanged historical invariants without requiring pre-rebase commit objects."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import types

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent
HISTORY = HERE.parent


def read(path):
    return json.loads(path.read_text())


def main():
    mapping = read(HERE / 'rebase-mapping.json')
    rows = {row['recorded_revision']: row for row in mapping['commits'] + mapping['external_pins']}
    overrides = {(row['recorded_revision'], row['path']): row for row in mapping['historical_overrides']}
    seal = read(HISTORY / 'p510-seal.json')
    external = {row['revision']: row['branch'] for row in read(HISTORY / 'p510-retirement-later-registers.json')}
    native_check_output = subprocess.check_output
    verified = set()

    def git(*args, input=None):
        return native_check_output(['git', *args], cwd=ROOT, input=input)

    def canonical(revision):
        matches = [pin for pin in rows if pin.startswith(revision)]
        assert len(matches) <= 1, revision
        return matches[0] if matches else revision

    def historical_blob(revision, path):
        pin = canonical(revision)
        override = overrides.get((pin, path))
        if override:
            raw = (HERE / override['historical_file']).read_bytes()
            assert hashlib.sha256(raw).hexdigest() == override['sha256'], ('historical blob drift', pin, path)
            assert git('hash-object', '--stdin', input=raw).decode().strip() == override['recorded_blob']
            return raw
        mapped = rows[pin]['rebased_revision'] if pin in rows else pin
        return git('show', mapped + ':' + path)

    def historical_names(revision, directory):
        pin = canonical(revision)
        if pin not in rows:
            return git('ls-tree', '-r', '--name-only', pin, directory).decode().splitlines()
        assert directory == 'data/patch-api/sources', ('unrecorded inventory scope', directory)
        if pin in external:
            paths = (HISTORY / ('p510-register-tree-' + external[pin] + '.txt')).read_text().splitlines()
        else:
            paths = sorted(row['path'] for row in seal['shared_inputs'] if row['path'].startswith(directory + '/'))
        if pin not in verified:
            # Reconstruct the exact original source-directory tree from existing
            # historical inventories, not a newly copied per-file mapping.
            mapped = rows[pin]['rebased_revision']
            entries = dict(line.split('\t', 1)[::-1] for line in git('ls-tree', mapped + ':' + directory).decode().splitlines())
            selected = []
            for path in paths:
                name = Path(path).name
                assert path == directory + '/' + name
                selected.append(entries[name] + '\t' + name)
            tree = git('mktree', input=('\n'.join(sorted(selected)) + '\n').encode()).decode().strip()
            assert tree == rows[pin]['directories'][directory]['recorded_tree'], ('historical source tree drift', pin)
            verified.add(pin)
        return paths

    def historical_git(*args):
        if args[0] == 'show' and len(args) == 2 and ':' in args[1]:
            return historical_blob(*args[1].split(':', 1))
        if args[:3] == ('ls-tree', '-r', '--name-only'):
            return ('\n'.join(historical_names(args[3], args[4])) + '\n').encode()
        if args[0] == 'rev-parse' and args[-1].endswith('^{commit}'):
            pin = canonical(args[-1].removesuffix('^{commit}'))
            mapped = rows[pin]['rebased_revision'] if pin in rows else pin
            return git('rev-parse', '--verify', mapped + '^{commit}')
        if args[:2] == ('diff', '--name-only'):
            mapped = list(args)
            for index in (2, 3):
                pin = canonical(mapped[index])
                if pin in rows:
                    trees = rows[pin]['directories']['src']
                    assert trees['recorded_tree'] == trees['rebased_tree'], 'historical runtime diff requires identical mapped src tree'
                    mapped[index] = rows[pin]['rebased_revision']
            return git(*mapped)
        return git(*args)

    def historical_check_output(command, *args, **kwargs):
        if command[0] == 'git':
            result = historical_git(*command[1:])
            return result.decode() if kwargs.get('text') or kwargs.get('universal_newlines') else result
        return native_check_output(command, *args, **kwargs)

    source = (HERE / 'historical-validator.py.txt').read_bytes()
    expected = read(HERE / 'historical-preservation.json')['validate.py']
    assert hashlib.sha256(source).hexdigest() == expected, 'original validator drift'
    proxy = types.ModuleType('subprocess')
    proxy.__dict__.update(subprocess.__dict__)
    proxy.check_output = historical_check_output
    namespace = {'__name__': 'historical_510_validator', '__file__': str(HISTORY / 'validate.py')}
    sys.modules['subprocess'] = proxy
    try:
        exec(compile(source, 'historical-validator.py.txt', 'exec'), namespace)
        namespace['main']()
    finally:
        sys.modules['subprocess'] = subprocess


if __name__ == '__main__':
    main()
