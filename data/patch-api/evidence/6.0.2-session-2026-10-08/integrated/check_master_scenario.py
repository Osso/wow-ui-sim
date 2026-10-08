"""Pin master and reproduce the existing scenario boundary failure without changing it."""
import io
import os
from pathlib import Path
import subprocess
import tarfile
import tempfile

from run_proof import run_proof

ROOT = Path(__file__).resolve().parents[5]
MASTER = 'd0fabed03'


def main():
    os.environ['PATCH_PROOF_REVISION'] = MASTER
    with tempfile.TemporaryDirectory(prefix='p602-master-scenario-') as directory:
        snapshot = Path(directory)
        archive = subprocess.check_output(['git', 'archive', MASTER], cwd=ROOT)
        with tarfile.open(fileobj=io.BytesIO(archive)) as package:
            package.extractall(snapshot, filter='data')
        result = run_proof('master-scenario', ['cargo', 'test', '--manifest-path',
                          str(snapshot / 'Cargo.toml'), '--test', 'integration', '--',
                          'scenario', '--skip', 'c_scenario_info_probes'])
        if result != 101:
            raise RuntimeError('Expected the recorded master failure; exit=' + str(result))


if __name__ == '__main__':
    main()
