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
    (HERE / 'command-results.json').write_text(json.dumps(results, indent=2) + '\n')
    return result


if __name__ == '__main__':
    results = {}
    os.environ['CARGO_TARGET_DIR'] = TARGET
    os.environ['PYTHONDONTWRITEBYTECODE'] = '1'
    os.environ.pop('WOW_SIM_NO_ADDONS', None)
    sweeps(ROOT, HERE)
    check('own-sweep', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'patch_5_4_8_publication_sweep'])
    check('all-sweeps', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'publication_sweep'])
    for target in ['integration', 'prefork_full_ui']:
        check(target + '-list', ['cargo', 'test', '--test', target, '--', '--list'])
        if target == 'integration':
            check(target + '-regressions', ['cargo', 'test', '--test', target, '--', *SELECTORS])
        else:
            for selector in SELECTORS:
                check('prefork-' + selector, ['cargo', 'test', '--test', target, '--', selector])
        check(target + '-patch_5_4_8', ['cargo', 'test', '--test', target, '--', 'patch_5_4_8'])
    check('retail-build', ['cargo', 'build', '--bin', 'wow-sim'])
    check('branch-startup', ['timeout', '90', TARGET + '/debug/wow-sim', '--no-saved-vars', 'lua-errors'])
    negative = json.loads((HERE.parent / 'p548-negative-register.json').read_text())
    (HERE / 'negative-register.json').write_text(json.dumps(negative, indent=2) + '\n')
    os.environ['P548_SWEEP_REGISTER'] = str(HERE / 'negative-register.json')
    os.environ['P548_SWEEP_OUT'] = str(HERE / 'negative-results.json')
    check('negative', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'patch_5_4_8_publication_sweep'])
    os.environ.pop('P548_SWEEP_REGISTER')
    os.environ.pop('P548_SWEEP_OUT')
    check('format', ['cargo', 'fmt', '--check'])
    for path in sorted((ROOT / 'tools').glob('test_*.py')):
        check(path.stem, ['python3', '-B', str(path)])
    mists = ['--no-default-features', '--features', 'sound,gui,casc,client-mists']
    check('mists-check', ['cargo', 'check', *mists, '--tests'])
    for target in ['integration']:
        check('mists-' + target + '-list', ['cargo', 'test', *mists, '--test', target, '--', '--list'])
        check('mists-' + target + '-regressions', ['cargo', 'test', *mists, '--test', target, '--', 'cvar', 'taint'])
    print('FINISHED', flush=True)
