"""Seal completed own artifacts and shared Git snapshots after targeted acceptance."""
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
BASE = '896086537a2b3c1ead5886d5ae3e430d56e7ef20'


def read(path):
    return json.loads(path.read_text())


def digest(data):
    return hashlib.sha256(data).hexdigest()


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT)


def write(name, value):
    (HERE / name).write_text(json.dumps(value, indent=2) + '\n')


def main():
    revision = git('rev-parse', 'HEAD').decode().strip()
    rust = ['p530-all-sweeps', 'p530-own-behavior', 'p530-bare-behavior',
            'p530-pvp-lib', 'p530-mists', 'p530-format', 'p530-negative']
    python = [path.stem.removesuffix('.proof') for path in sorted(HERE.glob('test_*.proof.json'))]
    proofs = {name: 1 if name == 'p530-negative' else 0
              for name in rust + python + ['p530-reproduction']}
    inputs, scopes = {}, {}
    runner = 'data/patch-api/evidence/5.3.0-session-2026-10-08/run_proof.py'
    for name, status in proofs.items():
        receipt = read(HERE / (name + '.proof.json'))
        assert receipt['exit'] == status and not receipt['invalidated'], name
        inputs[name] = {runner: digest(git('show', revision + ':' + runner))}
        scopes[name] = ['src', 'tests', 'build.rs', 'Cargo.toml', 'Cargo.lock'] if name in rust else ['tools']
    source_names = git('ls-tree', '-r', '--name-only', revision,
                       'data/patch-api/sources').decode().splitlines()
    reproduced_paths = [path for path in source_names if path.endswith(
        ('-wikitext-register.json', '-api-changes.wikitext', '-api-changes.diff.wikitext',
         '-api-changes.txt', '-api-changes.provenance.json'))]
    scopes['p530-reproduction'].extend(reproduced_paths)
    shared_names = reproduced_paths + [
        'data/patch-api/sources/5.3.0-page-coverage.json',
        'tests/data/patch_5_3_0_sweep_known_gaps.json',
        'tests/patch_5_3_0_publication_sweep.rs', 'tests/patch_5_3_0_behavior.rs',
        'tools/gen_patch_wikitext_register.py', 'tools/extract_patch_non_inventory.py',
        'docs/wiki/index.md', 'docs/wiki/log.md',
    ]
    shared = {path: digest(git('show', revision + ':' + path)) for path in shared_names}
    excluded_prefixes = ('p530-portability-gate', 'p530-local-validator',
                         'p530-tamper', 'p530-finalization')
    own = {path.name: digest(path.read_bytes()) for path in sorted(HERE.iterdir())
           if path.is_file() and path.name != 'p530-seal.json'
           and not path.name.startswith(excluded_prefixes)}
    write('p530-seal.json', {
        'audit_revision': revision, 'base_revision': BASE,
        'own_files': own, 'shared_files': shared, 'proofs': proofs,
        'proof_inputs': inputs, 'proof_scopes': scopes,
        'negative_id': read(HERE / 'p530-negative-id.json')['source_id'],
        'policy': 'Own immutable artifacts checked live; every shared input and scope checked at fixed revisions.',
    })
    print(json.dumps({'audit_revision': revision, 'own_seals': len(own),
                      'shared_snapshots': len(shared), 'proofs': len(proofs)}))


if __name__ == '__main__':
    main()
