"""Replace only Classic receipts invalidated by page/control output collision."""
import os
import subprocess
import sys

from run_checks import MASTER, MASTER_ROOT, outputs
from run_proof import ROOT, EVIDENCE as HERE, run_proof

sys.dont_write_bytecode = True


def main():
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    command = ['cargo', 'test', '--no-default-features', '--features',
               'sound,gui,casc,client-mists', '--test', 'integration', '--', 'publication_sweep']
    for label, pin, checkout, directory in [
        ('mists-all-sweeps', revision, str(ROOT), HERE),
        ('master-mists-all-sweeps', MASTER, MASTER_ROOT, HERE / 'master'),
    ]:
        os.environ['PATCH_PROOF_REVISION'] = pin
        os.environ['PATCH_RUN_ROOT'] = checkout
        outputs(pin, directory)
        if run_proof(label, command):
            return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())
