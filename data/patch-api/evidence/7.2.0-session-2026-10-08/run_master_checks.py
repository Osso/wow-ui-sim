"""Build the fixed master Git-archive snapshot; compare addons-enabled startup."""
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
SNAPSHOT = Path('/home/osso/.cache/wow-ui-sim-targets/p720-page/master-source-1ade15b52')
COMMANDS = [
    ('p720-master-build', ['cargo', 'build', '--manifest-path', str(SNAPSHOT / 'Cargo.toml'), '--bin', 'wow-sim']),
    ('p720-master-startup', ['timeout', '90', '/home/osso/.cache/wow-ui-sim-targets/p720-page/debug/wow-sim', '--no-saved-vars', 'lua-errors']),
]


def main():
    for label, command in COMMANDS:
        code = subprocess.call([sys.executable, '-B', str(HERE / 'run_proof.py'), label, *command], cwd=ROOT)
        if code:
            raise SystemExit(code)
    print('Master-source queue finished', flush=True)


if __name__ == '__main__':
    main()
