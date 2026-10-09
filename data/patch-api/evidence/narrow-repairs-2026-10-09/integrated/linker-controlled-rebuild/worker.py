from pathlib import Path
import datetime
import hashlib
import json
import shutil
import threading

repo = Path('/home/osso/Projects/wow/wow-ui-sim')
root = Path('/home/osso/.local/state/wow-ui-sim/verification/linker-controlled-rebuild')
epoch = root / datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%SZ')
epoch.mkdir(exist_ok=False)

def now():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()

def write_json(name, value):
    (epoch / name).write_text(json.dumps(value, indent=2, sort_keys=True))

def hash_file(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def run_compile(argv):
    handle = cli.command(*argv).cwd(str(repo)).env('CARGO_BUILD_JOBS', '4').spawn()
    write_json('cargo-process.json', {'argv': argv, 'cwd': str(repo), 'pid': handle.pid, 'started': now(), 'environment_overrides': {'CARGO_BUILD_JOBS': '4'}})
    stream_errors = []
    def persist_stream(stream, name):
        try:
            with (epoch / name).open('w') as output:
                for line in stream:
                    output.write(line)
                    output.flush()
        except Exception as error:
            stream_errors.append({'stream': name, 'error': str(error)})
    stdout_thread = threading.Thread(target=persist_stream, args=(handle.process.stdout, 'compile.stdout'))
    stderr_thread = threading.Thread(target=persist_stream, args=(handle.process.stderr, 'compile.stderr'))
    stdout_thread.start()
    stderr_thread.start()
    exit_code = handle.process.wait()
    stdout_thread.join()
    stderr_thread.join()
    if stream_errors:
        write_json('cargo-stream-errors.json', stream_errors)
        raise RuntimeError('Cargo evidence stream persistence failed')
    write_json('cargo-result.json', {'exit': exit_code, 'ended': now(), 'argv': argv, 'cwd': str(repo), 'stream_encoding': 'CLI spawn text streams, separately persisted and flushed per line'})
    return exit_code

write_json('entry.json', {'started': now(), 'worker_sha256': hash_file(root / 'worker.py'), 'boot_id': Path('/proc/sys/kernel/random/boot_id').read_text().strip(), 'lock': '/home/osso/.worktrees/.builder.lock already held by parent build-lock.sh', 'purpose': 'controlled exact generated-artifact invalidation followed by same-source rebuild; no source/toolchain/linker changes'})
meminfo = {}
for line in Path('/proc/meminfo').read_text().splitlines():
    key, value = line.split(':', 1)
    meminfo[key] = int(value.strip().split()[0])
available_gib = meminfo['MemAvailable'] / (1024 * 1024)
load_one = float(Path('/proc/loadavg').read_text().split()[0])
write_json('acquired-lock-thresholds.json', {'observed': now(), 'available_gib': available_gib, 'load_one': load_one, 'required_available_gib': 6, 'required_load_below': 24})
if available_gib < 6 or load_one >= 24:
    write_json('outcome.json', {'status': 'resource-blocked', 'invalidation_performed': False, 'build_performed': False})
else:
    previous = Path('/home/osso/.local/state/wow-ui-sim/verification/fixture-and-callstack-proof/20261009T223142Z')
    expected = json.loads((previous / 'source-before.json').read_text())
    actual = {name: hash_file(repo / name) for name in expected}
    write_json('source-before.json', actual)
    if actual != expected:
        raise RuntimeError('Source scope changed while queued; no invalidation authorized')
    selected = json.loads(Path('/home/osso/.local/state/wow-ui-sim/verification/linker-package-invalidation/invalidation-before.json').read_text())
    paths = [Path(name) for name in selected['paths']]
    if len(paths) != 6 or not all(path.is_relative_to(repo / 'target/debug') and path.exists() for path in paths):
        raise RuntimeError('Exact six invalidation paths changed; no deletion authorized')
    protected = selected['protected_artifacts_before']
    if not all(hash_file(Path(name)) == digest for name, digest in protected.items()):
        raise RuntimeError('Protected artifact identity changed; no deletion authorized')
    write_json('invalidation-before.json', {'observed': now(), 'paths': [str(path) for path in paths], 'protected': protected, 'selection': selected['selection'], 'hypothesis': selected['hypothesis']})
    for path in paths:
        if path.is_dir():
            shutil.rmtree(path)
        else:
            path.unlink()
    if not all(not path.exists() for path in paths):
        raise RuntimeError('Exact generated artifacts not fully removed')
    if not all(hash_file(Path(name)) == digest for name, digest in protected.items()):
        raise RuntimeError('Protected artifact changed during invalidation')
    write_json('invalidation-after.json', {'observed': now(), 'removed_exact_six_paths': True, 'protected_artifacts_equal': True, 'dependencies_not_selected': True})
    head = cli.git('rev-parse', 'HEAD').cwd(str(repo)).capture().run()
    if head.exit_code != 0:
        raise RuntimeError('Could not identify source revision')
    write_json('source-revision.json', {'revision': head.stdout.strip(), 'scope_equal_to_failed_epoch': True, 'exclusions': ['external path dependencies', 'inherited environment', 'untracked .code-index.db']})
    argv = ['/usr/bin/cargo', 'test', '--offline', '--locked', '--lib', '--test', 'integration', '--no-run', '--message-format=json']
    code = run_compile(argv)
    after = {name: hash_file(repo / name) for name in expected}
    write_json('source-after.json', after)
    write_json('outcome.json', {'ended': now(), 'cargo_exit': code, 'source_scope_equal': actual == after, 'invalidation_performed': True, 'build_performed': True, 'boot_id_after': Path('/proc/sys/kernel/random/boot_id').read_text().strip()})
print(str(epoch))
