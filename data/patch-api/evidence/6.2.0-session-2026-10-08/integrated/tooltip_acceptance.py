"""Prove requested spell-line modules and compare the extra layout failure on master."""
import importlib.util
import io
import os
from pathlib import Path
import select
import subprocess
import sys
import tarfile
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]


def main():
    # Event-driven dependency on the first worker; no polling or synchronous TUI wait.
    try:
        descriptor = os.pidfd_open(int(sys.argv[1]))
    except ProcessLookupError:
        descriptor = None
    if descriptor is not None:
        select.select([descriptor], [], [])
        os.close(descriptor)
    assert 'FINISHED' in (HERE / 'launcher.txt').read_text()
    spec = importlib.util.spec_from_file_location('checks', HERE / 'run_checks.py')
    checks = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(checks)
    revision = checks.git('rev-parse', 'HEAD')
    env = checks.environment(HERE, sorted((ROOT / 'tests').glob('patch_*_publication_sweep.rs')))
    modules = ('tooltip_item_spell', 'tooltip_basic', 'tooltip_spell_mount_identifiers',
               'tooltip_gc_rooting', 'tooltip_item_sources', 'tooltip_talent')
    for module in modules:
        checks.proof(module, ['cargo', 'test', '--test', 'integration', module + '::'], env, revision)
    with tempfile.TemporaryDirectory(prefix='p620-master-tooltip-') as directory:
        snapshot = Path(directory)
        archive = subprocess.check_output(['git', 'archive', checks.MASTER], cwd=ROOT)
        with tarfile.open(fileobj=io.BytesIO(archive)) as package:
            package.extractall(snapshot, filter='data')
        command = ['cargo', 'test', '--manifest-path', str(snapshot / 'Cargo.toml'),
                   '--test', 'integration', 'tooltip_text_layout::test_tooltip_layout_is_clamped_to_viewport_edges',
                   '--', '--exact']
        checks.proof('master-tooltip-clamp', command, env, checks.git('rev-parse', checks.MASTER))
    print('FINISHED', flush=True)


if __name__ == '__main__':
    main()
