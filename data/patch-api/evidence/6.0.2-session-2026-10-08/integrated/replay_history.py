"""Replay every original validator invariant using preserved bytes and mapped Git pins."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent

sys.path.insert(0, str(ROOT / 'tools'))
assert hashlib.sha256((ROOT / 'tools/patch_audit_pin_trees.py').read_bytes()).hexdigest() == 'd0405c9e3fb49ced8bfe73ce0f42a566e1f1a9d21ec74a12e26d24bcd2b62e6a', 'compact helper tamper'
from patch_audit_pin_trees import ProofGit, expand_pinned_inputs
PROOF_GIT = ProofGit(ROOT, HERE, json.loads((HERE / 'rebase-mapping.json').read_text()))
HISTORY = HERE.parent


def read(path):
    return json.loads(path.read_text())


def digest(value):
    return hashlib.sha256(value).hexdigest()


def git(*args, input=None):
    return PROOF_GIT(*args, input=input)


def main():
    mapping = read(HERE / 'rebase-mapping.json')
    mapping['pinned_inputs'] = expand_pinned_inputs(ROOT, HERE, mapping['pinned_inputs'])
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
    source = source.replace('from patch_audit_validation import historical_registers, historical_sweep_tests', '')
    namespace = {'__name__': 'historical_602_validator', '__file__': str(HISTORY / 'validate.py')}
    exec(compile(source, 'historical-validator.py.txt', 'exec'), namespace)
    namespace.update(git=historical_git, blob=historical_blob,
                     historical_registers=lambda root, rev: [root / p for p in historical_names(rev, 'data/patch-api/sources') if p.endswith('-wikitext-register.json')],
                     historical_sweep_tests=lambda root, rev: [root / p for p in historical_names(rev, 'tests') if p.endswith('_publication_sweep.rs')])
    context = read(HISTORY / 'p602-context.json')
    for name, expected in read(HISTORY / 'p602-artifact-hashes.json')['sha256'].items():
        path = HERE / 'historical-validator.py.txt' if name == 'validate.py' else HISTORY / name
        assert digest(path.read_bytes()) == expected, name
    register, extractor, text, diff = namespace['check_sources'](context['runtime_revision'])
    summary = namespace['check_accounting'](context['accounting_revision'], register, extractor, text, diff)
    registers, failures = namespace['check_reproduction'](context)
    scans = namespace['check_scans']()
    cases, prior = namespace['check_receipts'](context)
    print(json.dumps({'status': 'PASS', **summary, 'registers': registers,
                      'inherited_extract_failures': sorted(failures), 'grep_scans': scans,
                      'sweep_cases': cases, 'historical_prior_validators': prior}, sort_keys=True))


if __name__ == '__main__':
    main()
