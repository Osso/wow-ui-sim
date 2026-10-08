"""Ordered targeted proof only; launched once without synchronous polling."""
import json
import os
from pathlib import Path
import sys

sys.dont_write_bytecode = True
from run_proof import EVIDENCE, ROOT, run_proof


def main():
    results = {}
    for path in sorted((ROOT / 'tests').glob('patch_*_publication_sweep.rs')):
        patch = path.stem.removeprefix('patch_').removesuffix('_publication_sweep')
        os.environ[f'P{patch.replace("_", "")}_SWEEP_OUT'] = str(EVIDENCE / f'{path.stem}-results.json')
    commands = [
        ('p725-all-sweeps', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'publication_sweep']),
        ('p725-bare-trees', ['cargo', 'test', '--test', 'integration', 'patch_7_2_5']),
        ('p725-garrison-callers', ['cargo', 'test', '--test', 'integration', 'garrison']),
        ('p725-cached-garrison', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'garrison']),
        ('p725-cached-anima', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'animadiversion']),
        ('p725-generator-fixtures', ['python3', '-B', 'tools/test_gen_patch_wikitext_register.py']),
        ('p725-extractor-fixtures', ['python3', '-B', 'tools/test_extract_patch_non_inventory.py']),
        ('p725-validator-fixtures', ['python3', '-B', 'tools/test_patch_audit_validation.py']),
        ('p725-source-reproduction', ['python3', '-B', str(EVIDENCE / 'reproduce_sources.py')]),
        ('p725-format', ['cargo', 'fmt', '--check']),
        ('p725-mists-check', ['cargo', 'check', '--no-default-features', '--features', 'sound,gui,casc,client-mists', '--tests']),
        ('p725-retail-build', ['cargo', 'build', '--bin', 'wow-sim']),
        ('p725-startup', ['timeout', '90', '/home/osso/.cache/wow-ui-sim-targets/p725-page/debug/wow-sim', '--no-addons', '--no-saved-vars', 'lua-errors']),
    ]
    for name, command in commands:
        results[name] = run_proof(name, command)
    (EVIDENCE / 'p725-verification-results.json').write_text(json.dumps(results, indent=2) + '\n')
    return 0 if all(exit == 0 for exit in results.values()) else 1


if __name__ == '__main__':
    sys.exit(main())
