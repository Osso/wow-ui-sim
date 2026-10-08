"""Finish targeted proof sequentially in the dedicated target; no polling."""
from pathlib import Path
import sys
from run_proof import run_proof

HERE = Path(__file__).resolve().parent


def main():
    commands = [
        ('p602-own-bare', ['cargo', 'test', '--test', 'integration', '--', 'patch_6_0_2']),
        ('p602-scenario-regressions', ['cargo', 'test', '--test', 'integration', '--', 'c_scenario_info_probes']),
        ('p602-branch-startup', ['timeout', '90', '/home/osso/.cache/wow-ui-sim-targets/p602-page/debug/wow-sim', '--no-saved-vars', 'lua-errors']),
        ('p602-format', ['cargo', 'fmt', '--check']),
        ('p602-mists', ['cargo', 'check', '--no-default-features', '--features', 'sound,gui,casc,client-mists', '--tests']),
    ]
    exits = [run_proof(label, command) for label, command in commands]
    return int(any(exits))


if __name__ == '__main__':
    sys.exit(main())
