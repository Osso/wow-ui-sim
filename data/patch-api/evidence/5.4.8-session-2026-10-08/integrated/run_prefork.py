"""Prefork accepts one positional filter; retain each exact selector separately."""
import io
import os
from pathlib import Path
import sys
import tarfile
import tempfile
import subprocess
from run_proof import run_proof
from run_checks import SELECTORS

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent


def checks(manifest, prefix):
    for selector in SELECTORS:
        run_proof(prefix + 'prefork-' + selector,
                  ['cargo', 'test', *manifest, '--test', 'prefork_full_ui', '--', selector])


if __name__ == '__main__':
    if len(sys.argv) > 1 and sys.argv[1] == 'master':
        with tempfile.TemporaryDirectory(prefix='p548-master-prefork-') as directory:
            snapshot = Path(directory)
            archive = subprocess.check_output(['git', 'archive', 'a9d7c9566'], cwd=ROOT)
            with tarfile.open(fileobj=io.BytesIO(archive)) as package:
                package.extractall(snapshot, filter='data')
            os.environ['CARGO_TARGET_DIR'] = '/home/osso/.cache/wow-ui-sim-targets/master-ref'
            os.environ['PATCH_PROOF_REVISION'] = 'a9d7c9566'
            checks(['--manifest-path', str(snapshot / 'Cargo.toml')], 'master-')
    else:
        os.environ['CARGO_TARGET_DIR'] = '/home/osso/.cache/wow-ui-sim-targets/p548-page'
        checks([], '')
    print('FINISHED', flush=True)
