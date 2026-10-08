"""Exercise the gate in an owned relocated snapshot, future additions, and tampering."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
SNAPSHOT = Path('/home/osso/.cache/wow-ui-sim-targets/p622-page/validator-relocation')


def copy_snapshot():
    if SNAPSHOT.exists():
        shutil.rmtree(SNAPSHOT)
    destination = SNAPSHOT / HERE.relative_to(ROOT)
    destination.parent.mkdir(parents=True)
    shutil.copytree(HERE, destination)
    shutil.copytree(ROOT / 'data/patch-api/sources', SNAPSHOT / 'data/patch-api/sources')
    for path in HERE.parent.iterdir():
        if path != HERE:
            (destination.parent / path.name).symlink_to(path, target_is_directory=path.is_dir())
    (SNAPSHOT / 'tools').symlink_to(ROOT / 'tools', target_is_directory=True)
    (SNAPSHOT / '.git').write_text((ROOT / '.git').read_text())
    tests = SNAPSHOT / 'tests'
    tests.mkdir()
    for path in (ROOT / 'tests').glob('patch_*_publication_sweep.rs'):
        shutil.copy2(path, tests / path.name)
    (tests / 'data').mkdir()
    shutil.copy2(ROOT / 'tests/data/patch_6_2_2_sweep_known_gaps.json',
                 tests / 'data/patch_6_2_2_sweep_known_gaps.json')
    return destination


def run_gate(destination, label):
    result = subprocess.run([sys.executable, '-B', str(destination / 'validate.py')],
                            cwd=ROOT, capture_output=True, text=True)
    log = HERE / (label + '.log')
    log.write_text(result.stdout + result.stderr)
    return {'label': label, 'exit': result.returncode, 'log': log.name,
            'log_sha256': hashlib.sha256(log.read_bytes()).hexdigest()}


def main():
    destination = copy_snapshot()
    outcomes = [run_gate(destination, 'p622-relocated')]
    (SNAPSHOT / 'data/patch-api/sources/0.0.1-wikitext-register.json').write_text('{}\n')
    (SNAPSHOT / 'tests/patch_0_0_1_publication_sweep.rs').write_text('// unrelated later audit\n')
    future_validator = destination.parent / 'future-audit/validate.py'
    future_validator.parent.mkdir()
    future_validator.write_text('raise RuntimeError("must not expand historical validator set")\n')
    outcomes.append(run_gate(destination, 'p622-relocated-future-audit'))
    raw = SNAPSHOT / 'data/patch-api/sources/6.2.2-api-changes.wikitext'
    original = raw.read_bytes()
    try:
        raw.write_bytes(original + b'\n')
        outcomes.append(run_gate(destination, 'p622-relocated-tamper'))
    finally:
        raw.write_bytes(original)
    outcomes.append(run_gate(destination, 'p622-relocated-restored'))
    assert [row['exit'] for row in outcomes] == [0, 0, 1, 0], outcomes
    (HERE / 'p622-validator-portability.json').write_text(json.dumps({
        'snapshot': str(SNAPSHOT), 'outcomes': outcomes,
        'original_restored': raw.read_bytes() == original,
        'scope_policy': 'Register, sweep and validator scopes come from recorded Git objects; future files do not expand them.',
    }, indent=2) + '\n')
    print(json.dumps({'portability': 'PASS', 'cases': len(outcomes)}))


if __name__ == '__main__':
    main()
