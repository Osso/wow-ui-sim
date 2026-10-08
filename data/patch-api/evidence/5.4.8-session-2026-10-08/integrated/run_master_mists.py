"""Finish only the master Mists comparison omitted by the original driver error."""
import io
import os
from pathlib import Path
import subprocess
import tarfile
import tempfile
from run_proof import run_proof
from run_lib import run_lib_checks

ROOT = Path(__file__).resolve().parents[5]
MASTER = 'a9d7c9566'

if __name__ == '__main__':
    with tempfile.TemporaryDirectory(prefix='p548-master-mists-') as directory:
        snapshot = Path(directory)
        archive = subprocess.check_output(['git', 'archive', MASTER], cwd=ROOT)
        with tarfile.open(fileobj=io.BytesIO(archive)) as package:
            package.extractall(snapshot, filter='data')
        os.environ['CARGO_TARGET_DIR'] = '/home/osso/.cache/wow-ui-sim-targets/master-ref'
        os.environ['PATCH_PROOF_REVISION'] = MASTER
        manifest = ['--manifest-path', str(snapshot / 'Cargo.toml')]
        run_lib_checks(manifest, 'master-', 'retail')
        command = ['cargo', 'test', '--manifest-path', str(snapshot / 'Cargo.toml'),
                   '--no-default-features', '--features', 'sound,gui,casc,client-mists',
                   '--test', 'integration', '--', 'cvar', 'taint']
        run_proof('master-mists-integration-regressions', command)
        run_lib_checks(manifest, 'master-', 'mists')
    print('FINISHED', flush=True)
