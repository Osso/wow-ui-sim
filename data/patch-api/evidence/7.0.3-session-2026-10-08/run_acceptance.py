"""Run bounded final checks once; each command has an independent complete receipt."""
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
TARGET = '/home/osso/.cache/wow-ui-sim-targets/p703-page'
COMMANDS = [
    ('p703-all-sweeps', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'publication_sweep']),
    ('p703-profession-integration', ['cargo', 'test', '--test', 'integration', 'professions_api::', '--', '--nocapture']),
    ('p703-collection-integration', ['cargo', 'test', '--test', 'integration', 'c_collection_api::', '--', '--nocapture']),
    ('p703-function-diff-integration', ['cargo', 'test', '--test', 'integration', 'c_function_diff_coverage::', '--', '--nocapture']),
    ('p703-profession-prefork', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'professions']),
    ('p703-crafting-panel-prefork', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'test_showuipanel_professions_crafting']),
    ('p703-retail-build', ['cargo', 'build', '--bin', 'wow-sim']),
    ('p703-startup', ['timeout', '90', TARGET + '/debug/wow-sim', '--no-saved-vars', 'lua-errors']),
    ('p703-mists-check', ['cargo', 'check', '--no-default-features', '--features', 'sound,gui,casc,client-mists', '--tests']),
    ('p703-format-check', ['cargo', 'fmt', '--check']),
]


def main():
    env = dict(os.environ, PYTHONDONTWRITEBYTECODE='1', CARGO_BUILD_JOBS='4')
    outcomes = []
    for label, command in COMMANDS:
        result = subprocess.run([sys.executable, '-B', str(HERE / 'run_proof.py'), label, *command], cwd=ROOT, env=env)
        outcomes.append({'label': label, 'exit': result.returncode})
    (HERE / 'p703-acceptance-outcomes.json').write_text(json.dumps(outcomes, indent=2) + '\n')
    raise SystemExit(any(row['exit'] for row in outcomes))


if __name__ == '__main__':
    main()
