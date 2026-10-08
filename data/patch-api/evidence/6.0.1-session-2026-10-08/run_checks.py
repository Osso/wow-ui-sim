"""Execute the requested bounded proof commands sequentially with complete logs."""
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent


def main():
    commands = [
        ('discovery', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'patch_6_0_1']),
        ('all-sweeps', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'publication_sweep']),
        ('mists-check', ['cargo', 'check', '--no-default-features', '--features',
                         'sound,gui,casc,client-mists', '--tests']),
        ('format-check', ['cargo', 'fmt', '--check']),
        ('reproduction', ['python3', '-B', str(HERE / 'reproduce_sources.py')]),
    ]
    commands.extend((path.stem, ['python3', '-B', str(path)])
                    for path in sorted((ROOT / 'tools').glob('test_*.py')))
    failures = []
    for label, command in commands:
        result = subprocess.run([sys.executable, '-B', str(HERE / 'run_proof.py'),
                                 'p601-' + label, *command], cwd=ROOT)
        if result.returncode:
            failures.append(label)
    print('Completed requested checks; failures:', failures, flush=True)
    return bool(failures)


if __name__ == '__main__':
    sys.exit(main())
