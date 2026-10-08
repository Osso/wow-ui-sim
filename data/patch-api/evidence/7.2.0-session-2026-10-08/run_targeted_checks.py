"""Run the bounded acceptance queue once, with per-command receipts."""
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
COMMANDS = [
    ('p720-all-sweeps', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'publication_sweep']),
    ('p720-texture-regression', ['cargo', 'test', '--test', 'integration', 'texture_methods_port::']),
    ('p720-mask-regression', ['cargo', 'test', '--test', 'integration', 'methods_texture::masks_and_misc::']),
    ('p720-equipment-regression', ['cargo', 'test', '--test', 'integration', 'equipment_set']),
    ('p720-equipment-lib', ['cargo', 'test', '--lib', 'wow_api_equipment_set::']),
    ('p720-mists-check', ['cargo', 'check', '--no-default-features', '--features', 'sound,gui,casc,client-mists', '--tests']),
    ('p720-format-check', ['cargo', 'fmt', '--check']),
    ('p720-retail-build', ['cargo', 'build', '--bin', 'wow-sim']),
    ('p720-startup', ['timeout', '90', '/home/osso/.cache/wow-ui-sim-targets/p720-page/debug/wow-sim', '--no-saved-vars', 'lua-errors']),
]


def main():
    for label, command in COMMANDS:
        code = subprocess.call([sys.executable, '-B', str(HERE / 'run_proof.py'), label, *command], cwd=ROOT)
        if code:
            raise SystemExit(code)
    print('Targeted queue finished', flush=True)


if __name__ == '__main__':
    main()
