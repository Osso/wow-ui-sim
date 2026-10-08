"""Fresh integrated proofs; argv-only commands, complete logs, no historical overwrite."""
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
TARGET = '/home/osso/.cache/wow-ui-sim-targets/p703-page'
MASTER = Path(TARGET) / 'master-25fbde058'
BASE = '25fbde058'


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def dump(name, value):
    (HERE / name).write_text(json.dumps(value, indent=2) + '\n')


def scope(root):
    names = ['Cargo.toml', 'Cargo.lock', 'build.rs']
    for folder in ('src', 'tests', 'tools', 'data/patch-api/sources'):
        names.extend(p.relative_to(root).as_posix() for p in (root / folder).rglob('*') if p.is_file() and '__pycache__' not in p.parts)
    return {name: digest(root / name) for name in sorted(names)}


def prove(label, command, source, source_scope, extra=None, master=False):
    env = dict(os.environ, PYTHONDONTWRITEBYTECODE='1', CARGO_TARGET_DIR=TARGET)
    destination = HERE / 'master' if master else HERE
    destination.mkdir(exist_ok=True)
    for path in (MASTER if master else ROOT).glob('tests/patch_*_publication_sweep.rs'):
        match = re.search(r'out_env: "([^"]+)"', path.read_text())
        if match:
            env[match[1]] = str(destination / (path.stem + '-results.json'))
    env.update(extra or {})
    log = HERE / (label + '.txt')
    with log.open('wb') as handle:
        result = subprocess.run(command, cwd=ROOT, env=env, stdout=handle, stderr=subprocess.STDOUT)
    dump(label + '.proof.json', {'command': command, 'revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(), 'source_revision': source, 'scope': source_scope, 'exit': result.returncode, 'log': log.name, 'log_sha256': digest(log)})
    print(label, result.returncode, flush=True)
    return result.returncode


def main():
    HERE.mkdir(exist_ok=True)
    if '--detach' in sys.argv:
        with (HERE / 'launcher.txt').open('wb') as handle:
            process = subprocess.Popen([sys.executable, '-B', str(Path(__file__).resolve())], cwd=ROOT, stdout=handle, stderr=subprocess.STDOUT, start_new_session=True)
        dump('job.json', {'pid': process.pid})
        print('Launched integrated proofs:', process.pid)
        return
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    source_scope = scope(ROOT)
    master_scope = scope(MASTER)
    dump('context.json', {'runtime_revision': revision, 'master_revision': BASE, 'runtime_scope': source_scope, 'master_scope': master_scope})
    results = {}
    def run(label, command, extra=None, master=False):
        results[label] = prove(label, command, BASE if master else revision, master_scope if master else source_scope, extra, master)
        dump('command-results.json', results)
    # The own sweep already passed on the unchanged runtime; retain its full log.
    dump('own-sweep.proof.json', {'command': ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'patch_7_0_3_publication_sweep'], 'revision': revision, 'source_revision': revision, 'scope': source_scope, 'exit': 0, 'log': 'own-sweep.txt', 'log_sha256': digest(HERE / 'own-sweep.txt')})
    results['own-sweep'] = 0
    run('all-sweeps', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'publication_sweep'])
    for target, filters in [('integration', ['patch_7_0_3', 'professions_api::', 'test_crafting::', 'trade_info::', 'c_collection_api::', 'c_function_diff_coverage::', 'test_showuipanel_professions_crafting::']), ('prefork_full_ui', ['patch_7_0_3', 'professions', 'crafting', 'trade_skill', 'mount'])]:
        for selected in filters:
            run(target + '-' + selected.rstrip(':').replace('_', '-'), ['cargo', 'test', '--test', target, '--', selected, '--nocapture'])
    run('negative', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'patch_7_0_3_publication_sweep'], {'P703_SWEEP_REGISTER': str(HERE.parent / 'p703-negative-register.json'), 'P703_SWEEP_OUT': str(HERE / 'negative-results.json')})
    run('build', ['cargo', 'build', '--bin', 'wow-sim'])
    if results['build'] == 0:
        run('startup', ['timeout', '90', TARGET + '/debug/wow-sim', '--no-saved-vars', 'lua-errors'])
    for tool in ('gen_patch_wikitext_register', 'extract_patch_non_inventory', 'patch_audit_validation'):
        run(tool + '-fixtures', ['python3', '-B', str(ROOT / 'tools' / ('test_' + tool + '.py'))])
    run('format', ['cargo', 'fmt', '--check'])
    extension = HERE / 'extension'
    extension.mkdir(exist_ok=True)
    for name in ('p703-register-reproduction.json', 'p703-sweep-summary.json'):
        (extension / name).write_bytes((HERE.parent / name).read_bytes())
    run('extend-receipts', ['python3', '-B', str(ROOT / 'tools/extend_patch_audit_receipts.py'), str(ROOT), str(extension), 'p703', 'merged 7.1.0; recorded provenance flags', '7.0.3'])
    manifest = str(MASTER / 'Cargo.toml')
    run('master-all-sweeps', ['cargo', 'test', '--manifest-path', manifest, '--test', 'prefork_full_ui', '--', 'publication_sweep'], master=True)
    run('master-build', ['cargo', 'build', '--manifest-path', manifest, '--bin', 'wow-sim'], master=True)
    if results['master-build'] == 0:
        run('master-startup', ['timeout', '90', TARGET + '/debug/wow-sim', '--no-saved-vars', 'lua-errors'], master=True)
    features = ['--no-default-features', '--features', 'sound,gui,casc,client-mists']
    run('mists-check', ['cargo', 'check', *features, '--tests'])
    for selected in ('patch_7_0_3_recipe_name_filter_changes_catalog_results', 'mists_trade_skill_api::', 'professions_api::', 'c_collection_api::test_mount_journal'):
        run('mists-' + selected.rstrip(':').replace('_', '-'), ['cargo', 'test', *features, '--test', 'integration', '--', selected, '--nocapture'])
    dump('finished.json', results)


if __name__ == '__main__':
    main()
