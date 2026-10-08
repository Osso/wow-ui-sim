"""Refresh post-rebase receipts without rewriting sealed historical artifacts."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tarfile

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
FRESH = HERE / 'integrated'
TARGET = '/home/osso/.cache/wow-ui-sim-targets/p710-page'
MASTER_REVISION = 'aa57dd8f8'
MASTER = Path(TARGET) / 'master-aa57dd8f8'


def read(path):
    return json.loads(path.read_text())


def dump(path, data):
    path.write_text(json.dumps(data, indent=2) + '\n')


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()


def run(label, command, env, revision, scope):
    log = FRESH / (label + '.txt')
    with log.open('wb') as handle:
        result = subprocess.run(command, cwd=ROOT, env=env, stdout=handle, stderr=subprocess.STDOUT)
    dump(FRESH / (label + '.proof.json'), {
        'command': command, 'revision': git('rev-parse', 'HEAD'),
        'source_revision': revision, 'scope': scope, 'exit': result.returncode,
        'log': log.name, 'log_sha256': digest(log),
    })
    print(label, result.returncode, flush=True)
    return result.returncode


def sweep_env(base, source, output):
    env = dict(base)
    output.mkdir(exist_ok=True)
    for path in (source / 'tests').glob('patch_*_publication_sweep.rs'):
        name = re.search(r'out_env: "([A-Z0-9_]+)"', path.read_text()).group(1)
        env[name] = str(output / (path.stem + '-results.json'))
    return env


def main():
    FRESH.mkdir(exist_ok=True)
    revision = git('rev-parse', 'HEAD')
    master_revision = git('rev-parse', MASTER_REVISION)
    names = git('ls-files', 'src', 'tests', 'tools', 'Cargo.toml', 'Cargo.lock', 'build.rs',
                'data/patch-api/sources').splitlines()
    scope = {name: digest(ROOT / name) for name in names}
    env = dict(os.environ, CARGO_TARGET_DIR=TARGET, PYTHONDONTWRITEBYTECODE='1')
    branch_env = sweep_env(env, ROOT, FRESH)
    results = {}

    def execute(label, command, environment=branch_env, source_revision=revision, source_scope=scope):
        code = run(label, command, environment, source_revision, source_scope)
        results[label] = code
        if code != (1 if label == 'negative' else 0):
            dump(FRESH / 'command-results.json', results)
            raise SystemExit(f'{label} failed; inspect its retained log before continuing')

    execute('own-sweep', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'patch_7_1_0_publication_sweep'])
    spec = importlib.util.spec_from_file_location('reproduction', HERE / 'reproduce_sources.py')
    reproduction = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(reproduction)
    reproduction.HERE = FRESH
    extractor_spec = importlib.util.spec_from_file_location('extractor', ROOT / 'tools/extract_patch_non_inventory.py')
    extractor = importlib.util.module_from_spec(extractor_spec)
    extractor_spec.loader.exec_module(extractor)
    dump(FRESH / 'source-reproduction-summary.json', reproduction.reproduce(extractor, revision))

    extension = FRESH / 'extension'
    extension.mkdir(exist_ok=True)
    dump(extension / 'p710-register-reproduction.json', read(HERE / 'p710-register-reproduction.json'))
    summaries = []
    for path in sorted(HERE.glob('patch_*_publication_sweep-results.json')):
        data = read(path)
        patch = path.name.removeprefix('patch_').removesuffix('_publication_sweep-results.json').replace('_', '.')
        gaps = sum(not row['ok'] for row in data.values())
        summaries.append({'patch': patch, 'rows': len(data), 'ok': len(data) - gaps,
                          'gaps': gaps, 'result': 'pass'})
    dump(extension / 'p710-sweep-summary.json', summaries)
    execute('extend-receipts', ['python3', '-B', str(ROOT / 'tools/extend_patch_audit_receipts.py'),
                              str(ROOT), str(extension), 'p710',
                              'Merged 7.2.0 at ' + master_revision, '7.1.0'])
    rows = read(extension / 'p710-register-reproduction.json')
    added = next(row for row in rows if row['patch'] == '7.2.0')
    added['template_revision'] = added['revision']
    added['revision'] = read(FRESH / 'extend-receipts.proof.json')['revision']
    dump(extension / 'p710-register-reproduction.json', rows)

    probe = Path(TARGET) / 'intrinsic-probe'
    probe.mkdir(exist_ok=True)
    probe_archive = Path(TARGET) / 'intrinsic-probe.tar'
    with probe_archive.open('wb') as handle:
        subprocess.run(['git', 'archive', revision], cwd=ROOT, stdout=handle, check=True)
    with tarfile.open(probe_archive) as handle:
        handle.extractall(probe, filter='tar')
    original = (probe / 'tests/patch_7_1_0_behavior.rs').read_bytes()
    harness = (HERE / 'p710-intrinsic-discovery.rs').read_bytes()
    (probe / 'tests/patch_7_1_0_behavior.rs').write_bytes(harness)
    probe_scope = dict(scope)
    probe_scope['tests/patch_7_1_0_behavior.rs'] = hashlib.sha256(harness).hexdigest()
    code = run('intrinsic-discovery', ['cargo', 'test', '--manifest-path', str(probe / 'Cargo.toml'),
               '--test', 'prefork_full_ui', '--', 'patch_7_1_0_clip_children_state_and_intrinsic_xml'],
               env, revision, probe_scope)
    results['intrinsic-discovery'] = code
    dump(FRESH / 'intrinsic-discovery-context.json', {'revision': revision,
         'archived_harness': '../p710-intrinsic-discovery.rs', 'harness_sha256': hashlib.sha256(harness).hexdigest(),
         'original_sha256': hashlib.sha256(original).hexdigest(), 'exit': code})
    if code not in (0, 1):
        raise SystemExit('intrinsic discovery failed to execute; inspect log')

    # Exact master sources, never the moving canonical checkout or another worktree.
    archive = Path(TARGET) / 'master-aa57dd8f8.tar'
    MASTER.mkdir(exist_ok=True)
    with archive.open('wb') as handle:
        subprocess.run(['git', 'archive', master_revision], cwd=ROOT, stdout=handle, check=True)
    with tarfile.open(archive) as handle:
        handle.extractall(MASTER, filter='tar')
    master_scope = {}
    for name in git('ls-tree', '-r', '--name-only', master_revision, 'src', 'tests', 'tools',
                    'Cargo.toml', 'Cargo.lock', 'build.rs', 'data/patch-api/sources').splitlines():
        expected = subprocess.check_output(['git', 'show', f'{master_revision}:{name}'], cwd=ROOT)
        assert (MASTER / name).read_bytes() == expected, name
        master_scope[name] = digest(MASTER / name)
    master_output = FRESH / 'master'
    master_env = sweep_env(env, MASTER, master_output)
    execute('master-all-sweeps', ['cargo', 'test', '--manifest-path', str(MASTER / 'Cargo.toml'),
                                '--test', 'prefork_full_ui', '--', 'publication_sweep'],
            master_env, master_revision, master_scope)
    execute('all-sweeps', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'publication_sweep'])
    commands = [
        ('integration-p710', ['cargo', 'test', '--test', 'integration', '--', 'patch_7_1_0']),
        ('prefork-p710', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'patch_7_1_0']),
        ('screen-regression', ['cargo', 'test', '--test', 'integration', '--', 'screen_mode::']),
        ('items-regression', ['cargo', 'test', '--test', 'integration', '--', 'c_item_api::']),
        ('intrinsic-regression', ['cargo', 'test', '--test', 'integration', '--', 'intrinsic_types::']),
    ]
    for name in ('gen_patch_wikitext_register', 'extract_patch_non_inventory', 'patch_audit_validation'):
        commands.append((name + '-fixtures', ['python3', '-B', str(ROOT / 'tools' / ('test_' + name + '.py'))]))
    commands.extend([
        ('format', ['cargo', 'fmt', '--check']),
        ('mists-check', ['cargo', 'check', '--no-default-features', '--features', 'sound,gui,casc,client-mists', '--tests']),
    ])
    for label, command in commands:
        execute(label, command)
    negative_env = dict(branch_env, P720_SWEEP_REGISTER=str(HERE / 'p710-negative-register.json'),
                        P720_SWEEP_OUT=str(FRESH / 'negative-results.json'))
    execute('negative', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'patch_7_1_0_publication_sweep'], negative_env)
    dump(FRESH / 'command-results.json', results)
    dump(FRESH / 'context.json', {'runtime_revision': revision, 'master_revision': master_revision,
                                'historical_context': '../p710-context.json', 'runtime_scope': scope,
                                'master_scope': master_scope, 'expected_exits': results})
    print('Integrated command queue finished', flush=True)


if __name__ == '__main__':
    if '--detach' in sys.argv:
        FRESH.mkdir(exist_ok=True)
        with (FRESH / 'launcher.txt').open('wb') as handle:
            process = subprocess.Popen([sys.executable, '-B', str(Path(__file__).resolve())], cwd=ROOT,
                                       stdout=handle, stderr=subprocess.STDOUT, start_new_session=True)
        dump(FRESH / 'job.json', {'pid': process.pid})
        print('Launched integrated queue:', process.pid)
    else:
        main()
