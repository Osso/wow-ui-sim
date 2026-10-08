"""Run only requested publication/affected-area checks; never a full suite."""
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
TARGET = Path('/home/osso/.cache/wow-ui-sim-targets/p730-page')
CHECKS = [
    ('p730-all-sweeps', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'publication_sweep']),
    ('p730-console', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'blizzard_console_']),
    ('p730-debug-tools', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'blizzard_debug_tools_']),
    ('p730-deprecated-sound', ['cargo', 'test', '--test', 'prefork_full_ui', '--', 'blizzard_deprecated_sound_']),
    ('p730-bare-model', ['cargo', 'test', '--test', 'integration', 'patch_7_3_0_sound_model::', '--', '--nocapture']),
    ('p730-utility-sound', ['cargo', 'test', '--test', 'integration', 'utility_api::test_sound_', '--', '--nocapture']),
    ('p730-soundkit', ['cargo', 'test', '--test', 'integration', 'soundkit_ig_inventory_rotate_character::', '--', '--nocapture']),
    ('p730-options', ['cargo', 'test', '--test', 'integration', 'patch_12_1_0_struct_shapes::patch_12_1_0_play_sound', '--', '--nocapture']),
    ('p730-sound-defaults', ['cargo', 'test', '--lib', 'sound_driver_defaults::tests::', '--', '--nocapture']),
    ('p730-retail-build', ['cargo', 'build', '--bin', 'wow-sim']),
    ('p730-startup', ['timeout', '90', str(TARGET / 'debug/wow-sim'), '--no-addons', '--no-saved-vars', 'lua-errors']),
]


def main():
    for label, command in CHECKS:
        result = subprocess.run([sys.executable, '-B', str(HERE / 'run_proof.py'), label,
                                 *command], cwd=ROOT)
        if result.returncode:
            raise RuntimeError(f'proof runner failed for {label}: {result.returncode}')
    print('Targeted command queue finished', flush=True)


if __name__ == '__main__':
    main()
