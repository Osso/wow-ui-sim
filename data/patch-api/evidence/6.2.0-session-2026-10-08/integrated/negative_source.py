"""Negative provenance control for a zero-inventory parent page; no invariant bypass."""
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent
HISTORY = HERE.parent
PIN = 'ddd76addc3bd451894316ab8c3575ff9e8ec0f35'


def main():
    spec = importlib.util.spec_from_file_location('historical_gate', HISTORY / 'validate.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    fixture = json.loads((HERE / 'negative-fetch.json').read_text())
    original = json.loads((HISTORY / 'p620-fetch.json').read_text())
    assert fixture['query']['pages'][0]['title'] == original['query']['pages'][0]['title'] + ' tampered'
    with tempfile.TemporaryDirectory(prefix='p620-negative-source-') as directory:
        temporary = Path(directory)
        prefix = HISTORY.relative_to(ROOT).as_posix() + '/'
        names = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', PIN, prefix],
                                        cwd=ROOT, text=True).splitlines()
        for name in names:
            output = temporary / name.removeprefix(prefix)
            output.parent.mkdir(parents=True, exist_ok=True)
            contents = subprocess.check_output(['git', 'show', PIN + ':' + name], cwd=ROOT)
            output.write_bytes(contents)
        (temporary / 'p620-fetch.json').write_text(json.dumps(fixture, indent=2) + '\n')
        module.HERE = temporary
        module.main()


if __name__ == '__main__':
    main()
