"""Build an immutable master snapshot under this audit's own cache; no worktree edits."""
import hashlib
import io
import json
import os
import shutil
from pathlib import Path
import subprocess
import tarfile

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
TARGET = Path('/home/osso/.cache/wow-ui-sim-targets/p703-page')
REVISION = 'aa57dd8f8'


def main():
    revision = subprocess.check_output(['git', 'rev-parse', REVISION], cwd=ROOT, text=True).strip()
    snapshot = TARGET / 'master-source'
    snapshot.mkdir(parents=True, exist_ok=True)
    archive = subprocess.check_output(['git', 'archive', revision], cwd=ROOT)
    with tarfile.open(fileobj=io.BytesIO(archive)) as handle:
        handle.extractall(snapshot, filter='data')
    env = dict(os.environ, CARGO_TARGET_DIR=str(TARGET),
               CARGO_BUILD_JOBS='4', PYTHONDONTWRITEBYTECODE='1')
    command = ['cargo', 'build', '--manifest-path', str(snapshot / 'Cargo.toml'), '--bin', 'wow-sim']
    with (HERE / 'p703-master-build.log').open('wb') as log:
        result = subprocess.run(command, cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT)
    if result.returncode:
        raise SystemExit(result.returncode)
    executable = TARGET / 'master-wow-sim'
    shutil.copy2(TARGET / 'debug/wow-sim', executable)
    command = ['timeout', '90', str(executable), '--no-saved-vars', 'lua-errors']
    with (HERE / 'p703-master-startup.log').open('wb') as log:
        result = subprocess.run(command, cwd=snapshot, env=env, stdout=log, stderr=subprocess.STDOUT)
    log = HERE / 'p703-master-startup.log'
    (HERE / 'p703-master-startup.proof.json').write_text(json.dumps({
        'revision': revision, 'command': command, 'exit': result.returncode,
        'addons_enabled': True, 'snapshot': str(snapshot), 'log': log.name,
        'log_sha256': hashlib.sha256(log.read_bytes()).hexdigest(),
    }, indent=2) + '\n')
    raise SystemExit(result.returncode)


if __name__ == '__main__':
    main()
