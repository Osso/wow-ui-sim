"""Run only this page's scoped proof, sequentially with durable logs."""
import json
from pathlib import Path
from run_proof import HERE, ROOT, TARGET, run_worker


def main():
    commands = [
        ('p620-green', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'patch_6_2_0']),
        ('p620-all-sweeps', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'publication_sweep']),
        ('p620-bare', ['cargo', 'test', '--test', 'integration', 'patch_6_2_0', '--', '--nocapture']),
        ('p620-tooltip-item-spell', ['cargo', 'test', '--test', 'integration', 'tooltip_item_spell::', '--', '--nocapture']),
        ('p620-tooltip-basic', ['cargo', 'test', '--test', 'integration', 'tooltip_basic::', '--', '--nocapture']),
        ('p620-tooltip-identifiers', ['cargo', 'test', '--test', 'integration', 'tooltip_spell_mount_identifiers::', '--', '--nocapture']),
        ('p620-branch-build', ['cargo', 'build', '--bin', 'wow-sim']),
        ('p620-branch-startup', ['timeout', '90', TARGET + '/debug/wow-sim', '--no-saved-vars', 'lua-errors']),
        ('p620-mists-check', ['cargo', 'check', '--no-default-features', '--features', 'sound,gui,casc,client-mists', '--tests']),
        ('p620-format', ['cargo', 'fmt', '--check']),
        ('p620-master-build', ['cargo', 'build', '--manifest-path', TARGET + '/master-source/Cargo.toml', '--bin', 'wow-sim']),
        ('p620-master-startup', ['timeout', '90', TARGET + '/debug/wow-sim', '--no-saved-vars', 'lua-errors']),
    ]
    for label, command in commands:
        run_worker(label, command)
        receipt = json.loads((HERE / (label + '.proof.json')).read_text())
        if receipt['exit']:
            raise SystemExit(f'{label}: exit {receipt["exit"]}; inspect saved log')


if __name__ == '__main__':
    main()
