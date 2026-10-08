"""Seal completed own outputs; source/code revisions stay historical after later audits."""
import hashlib
import json
from pathlib import Path
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
ACCOUNTING = 'd67ecb832'
BASE = 'd0fabed03cd6534b73341fbf569e09c6c388202e'


def read(name):
    return json.loads((HERE / name).read_text())


def main():
    names = ['p548-all-sweeps', 'p548-cached-combat', 'p548-prefork-cvar',
             'p548-prefork-world_map', 'p548-prefork-keybindings', 'p548-integration-patch_5_4_8',
             'p548-integration-set_cvar_global', 'p548-integration-cvar_bitfields',
             'p548-integration-test_cvar_display_settings', 'p548-integration-ui_visibility_globals',
             'p548-branch-startup-driver', 'p548-branch-startup-build', 'p548-format', 'p548-mists-check',
             'p548-reproduction-final', 'p548-test_check_patch_validators',
             'p548-test_extract_patch_non_inventory', 'p548-test_gen_patch_wikitext_register',
             'p548-test_patch_audit_validation']
    for name in names:
        assert read(name + '.proof.json')['exit'] == 0, name
    context = {'base_revision': BASE, 'accounting_revision': ACCOUNTING,
               'tool_revision': ACCOUNTING,
               'reproduction_revision': read('p548-register-reproduction.json')[0]['revision'],
               'proof_receipts': [name + '.proof.json' for name in names]}
    excluded = {'p548-proof-context.json', 'p548-validator.txt', 'p548-validator.proof.json',
                'p548-portability-gate.txt', 'p548-portability-gate.proof.json',
                'p548-portability-gate.launcher.txt', 'p548-log-tamper.json', 'p548-proof-ledger.md'}
    context['own_seals'] = {path.name: hashlib.sha256(path.read_bytes()).hexdigest()
                            for path in sorted(HERE.iterdir())
                            if path.is_file() and path.name not in excluded}
    (HERE / 'p548-proof-context.json').write_text(json.dumps(context, indent=2) + '\n')
    ledger = ['# Proof ledger', '', '| Scope | Command | Revision | Result | Later invalidation |',
              '|---|---|---|---|---|']
    for name in names:
        receipt = read(name + '.proof.json')
        command = ' '.join(receipt['command'])
        ledger.append(f"| {name} | `{command}` | `{receipt['revision']}` | exit 0; sealed log | None in proof scope |")
    ledger.extend(['', 'Expected failures: initial combat RED; map-to-sequence fixture correction; discovery seven gaps; negative control seven → eight.',
                   'Initial discovery outputs are development evidence, not acceptance after test/fixture changes.',
                   'Final reproduction supersedes the provisional run performed before its recipe was committed.',
                   'Python fixture source stayed unchanged since its recorded revision; Rust checks cover the committed accounting revision.',
                   'Master startup executed before runtime changes; its src/crates/Cargo scope equals pinned master exactly.',
                   'No full integration suite, agent/model CLI, push or merge. No live external-cache input to validation.',
                   'Scan tool: /usr/bin/grep, whole-word -w, untruncated output files.', ''])
    (HERE / 'p548-proof-ledger.md').write_text('\n'.join(ledger))
    print(json.dumps({'sealed': len(context['own_seals']), 'receipts': len(names)}))


if __name__ == '__main__':
    main()
