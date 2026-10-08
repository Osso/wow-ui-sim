"""Complete Classic publication proofs after the retail driver exits; no polling."""
import os
from pathlib import Path
import select
import subprocess
import sys

from run_proof import ROOT, EVIDENCE as HERE, run_proof

sys.dont_write_bytecode = True
MASTER = '896086537a2b3c1ead5886d5ae3e430d56e7ef20'
MASTER_ROOT = '/home/osso/.worktrees/wow-ui-sim-p542-master-proof'


def main():
    descriptor = os.pidfd_open(int(sys.argv[1]))
    try:
        select.select([descriptor], [], [])
    finally:
        os.close(descriptor)
    import json
    assert json.loads((HERE / 'checks.proof.json').read_text())['exit'] == 0
    command = ['cargo', 'test', '--no-default-features', '--features',
               'sound,gui,casc,client-mists', '--test', 'integration', '--', 'publication_sweep']
    for label, revision, checkout, directory in [
        ('mists-all-sweeps', subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(), str(ROOT), HERE),
        ('master-mists-all-sweeps', MASTER, MASTER_ROOT, HERE / 'master'),
    ]:
        os.environ['PATCH_PROOF_REVISION'] = revision
        os.environ['PATCH_RUN_ROOT'] = checkout
        for patch in ['5.5.3', '5.5.4']:
            os.environ['P' + patch.replace('.', '') + '_SWEEP_OUT'] = str(directory / ('patch_' + patch.replace('.', '_') + '_publication_sweep-results.json'))
        if run_proof(label, command):
            return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())
