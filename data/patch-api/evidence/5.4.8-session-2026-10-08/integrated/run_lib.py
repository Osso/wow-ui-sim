"""Source-unit regression proof for every touched area, separately per client profile."""
import os
from run_proof import run_proof

LIB_SELECTORS = ['cvar', 'set_cvar', 'ui_visibility', 'taint', 'secure', 'settings']
MISTS = ['--no-default-features', '--features', 'sound,gui,casc,client-mists']


def run_lib_checks(manifest, prefix, profile):
    features = MISTS if profile == 'mists' else []
    base = ['cargo', 'test', *manifest, *features, '--lib', '--']
    run_proof(prefix + 'lib-' + profile + '-list', [*base, '--list'])
    run_proof(prefix + 'lib-' + profile + '-regressions', [*base, *LIB_SELECTORS])


if __name__ == '__main__':
    os.environ['CARGO_TARGET_DIR'] = '/home/osso/.cache/wow-ui-sim-targets/p548-page'
    for profile in ['retail', 'mists']:
        run_lib_checks([], '', profile)
    print('FINISHED', flush=True)
