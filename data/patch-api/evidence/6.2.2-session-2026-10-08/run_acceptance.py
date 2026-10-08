"""Run bounded final checks sequentially with complete logs; no full suite."""
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent


def run_proof(label, command, extra_env=None):
    env = os.environ.copy()
    env['PYTHONDONTWRITEBYTECODE'] = '1'
    env.update(extra_env or {})
    result = subprocess.run(
        [sys.executable, '-B', str(HERE / 'run_proof.py'), label, *command],
        cwd=ROOT, env=env,
    )
    return {'label': label, 'exit': result.returncode}


def main():
    if sys.argv[1:] == ['--detach']:
        command = [sys.executable, '-B', str(Path(__file__).resolve())]
        with (HERE / 'p622-acceptance-launch.log').open('wb') as handle:
            process = subprocess.Popen(command, cwd=ROOT, stdout=handle,
                                       stderr=subprocess.STDOUT, start_new_session=True)
        (HERE / 'p622-acceptance-job.json').write_text(
            json.dumps({'pid': process.pid, 'command': command}) + '\n')
        print(f'Launched acceptance: PID {process.pid}')
        return
    outcomes = []
    outcomes.append(run_proof('p622-all-sweeps', [
        'cargo', 'test', '--test', 'prefork_full_ui', '--', 'publication_sweep']))
    outcomes.append(run_proof('p622-negative', [
        'cargo', 'test', '--test', 'prefork_full_ui', '--', 'patch_6_2_2'],
        {'P622_SWEEP_REGISTER': str(HERE / 'p622-negative-register.json')}))
    outcomes.append(run_proof('p622-mists-check', [
        'cargo', 'check', '--no-default-features', '--features',
        'sound,gui,casc,client-mists', '--tests']))
    outcomes.append(run_proof('p622-format-check', ['cargo', 'fmt', '--check']))
    (HERE / 'p622-acceptance-outcomes.json').write_text(json.dumps(outcomes, indent=2) + '\n')
    assert all(row['exit'] == (1 if row['label'] == 'p622-negative' else 0)
               for row in outcomes), outcomes


if __name__ == '__main__':
    main()
