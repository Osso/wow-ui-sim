"""Logged bounded branch/master regression checks; launch once, never poll-wait."""
import io
import json
import os
from pathlib import Path
import re
import subprocess
import tarfile
import tempfile
from run_proof import run_proof

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent
MASTER = 'a9d7c9566'
SELECTORS = ['settings', 'cvar', 'edit_mode', 'keybind', 'nameplate', 'action_bar', 'chat', 'combat', 'taint', 'secure', 'ui_visibility', 'screenshot', 'hide_ui', 'hide_user_interface', 'patch_5_4_8']
TARGET = '/home/osso/.cache/wow-ui-sim-targets/p548-page'


def sweeps(source, output):
    output.mkdir(exist_ok=True)
    for path in sorted((source / 'tests').glob('patch_*_publication_sweep.rs')):
        match = re.search(r'out_env: "([^"]+)"', path.read_text())
        if match:
            os.environ[match[1]] = str(output / (path.stem + '-results.json'))


def check(label, args):
    result = run_proof(label, args)
    results[label] = result
    (HERE / 'master-command-results.json').write_text(json.dumps(results, indent=2) + '\n')
    return result


if __name__ == '__main__':
    results = {}
    mists = ['--no-default-features', '--features', 'sound,gui,casc,client-mists']
    with tempfile.TemporaryDirectory(prefix='p548-master-') as directory:
        snapshot = Path(directory)
        archive = subprocess.check_output(['git', 'archive', MASTER], cwd=ROOT)
        with tarfile.open(fileobj=io.BytesIO(archive)) as package:
            package.extractall(snapshot, filter='data')
        os.environ['CARGO_TARGET_DIR'] = '/home/osso/.cache/wow-ui-sim-targets/master-ref'
        os.environ['PATCH_PROOF_REVISION'] = MASTER
        sweeps(snapshot, HERE / 'master')
        manifest = ['--manifest-path', str(snapshot / 'Cargo.toml')]
        check('master-all-sweeps', ['cargo', 'test', *manifest, '--test', 'prefork_full_ui', '--', 'publication_sweep'])
        for target in ['integration', 'prefork_full_ui']:
            check('master-' + target + '-list', ['cargo', 'test', *manifest, '--test', target, '--', '--list'])
            if target == 'integration':
                check('master-' + target + '-regressions', ['cargo', 'test', *manifest, '--test', target, '--', *SELECTORS])
            else:
                for selector in SELECTORS:
                    check('master-prefork-' + selector, ['cargo', 'test', *manifest, '--test', target, '--', selector])
        check('master-retail-build', ['cargo', 'build', *manifest, '--bin', 'wow-sim'])
        check('master-startup', ['timeout', '90', os.environ['CARGO_TARGET_DIR'] + '/debug/wow-sim', '--no-saved-vars', 'lua-errors'])
        for target in ['integration']:
            check('master-mists-' + target + '-regressions', ['cargo', 'test', *manifest, *mists, '--test', target, '--', 'cvar', 'taint'])
    print('FINISHED', flush=True)
