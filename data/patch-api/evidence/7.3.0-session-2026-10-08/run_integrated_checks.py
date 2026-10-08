"""Run the requested integrated proof once, retaining complete logs and receipts."""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent / 'integrated'
TARGET = '/home/osso/.cache/wow-ui-sim-targets/p730-page'
MASTER_TARGET = '/home/osso/.cache/wow-ui-sim-targets/master-ref'
MASTER = '/home/osso/Projects/wow/wow-ui-sim'


def dump(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')


def run(label, command, overrides=None):
    env = dict(os.environ, PYTHONDONTWRITEBYTECODE='1', CARGO_TARGET_DIR=TARGET)
    for path in (ROOT / 'tests').glob('patch_*_publication_sweep.rs'):
        match = re.search(r'out_env: "([^"]+)"', path.read_text())
        if match:
            env[match[1]] = str(HERE / (path.stem + '-results.json'))
    env.update(overrides or {})
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    source_root = Path(MASTER) if label.startswith('master-') else ROOT
    source_revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=source_root, text=True).strip()
    log = HERE / (label + '.log')
    with log.open('wb') as output:
        result = subprocess.run(command, cwd=ROOT, env=env, stdout=output, stderr=subprocess.STDOUT)
    dump(HERE / (label + '.proof.json'), {
        'command': command, 'revision': revision, 'source_revision': source_revision,
        'source_root': str(source_root), 'environment': overrides or {},
        'exit': result.returncode, 'log': log.name,
        'log_sha256': hashlib.sha256(log.read_bytes()).hexdigest(),
    })
    print(label, result.returncode, flush=True)
    return result.returncode


def main():
    HERE.mkdir(exist_ok=True)
    if sys.argv[1:] == ['--detach']:
        with (HERE / 'launch.log').open('wb') as output:
            child = subprocess.Popen([sys.executable, '-B', str(Path(__file__).resolve())],
                                     cwd=ROOT, stdout=output, stderr=subprocess.STDOUT,
                                     start_new_session=True)
        print('Launched integrated proof:', child.pid)
        return
    selections = [
        ('integration-p730', ['--test', 'integration', '--', 'patch_7_3_0']),
        ('prefork-p730', ['--test', 'prefork_full_ui', '--', 'patch_7_3_0']),
        ('integration-sound', ['--test', 'integration', '--', 'sound']),
        ('prefork-sound', ['--test', 'prefork_full_ui', '--', 'sound']),
        ('lib-sound', ['--lib', '--', 'sound']),
    ]
    for name in ['blizzard_static_popup', 'game_menu', 'ui_panel']:
        for target in ['integration', 'prefork_full_ui']:
            selections.append((target + '-' + name, ['--test', target, '--', name]))
    selections.append(('all-sweeps', ['--test', 'prefork_full_ui', '--', 'publication_sweep']))
    for label, arguments in selections:
        run(label, ['cargo', 'test', *arguments])
    run('negative', ['cargo', 'test', '--test', 'prefork_full_ui', '--',
                     'patch_7_3_0_publication_sweep'], {
        'P730_SWEEP_REGISTER': str(HERE.parent / 'p730-negative-register.json'),
        'P730_SWEEP_OUT': str(HERE / 'negative-results.json'),
    })
    for tool in ['gen_patch_wikitext_register', 'extract_patch_non_inventory', 'patch_audit_validation']:
        run(tool + '-fixtures', ['python3', '-B', str(ROOT / 'tools' / ('test_' + tool + '.py'))])
    run('format', ['cargo', 'fmt', '--check'])
    run('mists-check', ['cargo', 'check', '--no-default-features', '--features',
                        'sound,gui,casc,client-mists', '--tests'])
    run('retail-build', ['cargo', 'build', '--bin', 'wow-sim'])
    run('startup', ['timeout', '90', TARGET + '/debug/wow-sim', '--no-addons', '--no-saved-vars', 'lua-errors'])
    run('startup-addons', ['timeout', '90', TARGET + '/debug/wow-sim', '--no-saved-vars', 'lua-errors'])
    run('master-build', ['cargo', 'build', '--manifest-path', MASTER + '/Cargo.toml', '--bin', 'wow-sim'],
        {'CARGO_TARGET_DIR': MASTER_TARGET})
    run('master-startup-addons', ['timeout', '90', MASTER_TARGET + '/debug/wow-sim', '--no-saved-vars', 'lua-errors'])


if __name__ == '__main__':
    main()
