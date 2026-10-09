#!/usr/bin/env python3
"""Run unchanged shared clean/later gate using disposable clones, not worktrees."""
import importlib.util
import json
from pathlib import Path
import shutil
import sys

ROOT = Path(__file__).resolve().parents[4]
spec = importlib.util.spec_from_file_location('patch_gate', ROOT / 'tools/check_patch_validators.py')
gate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gate)
original_git = gate.git


def clone_checkout_git(root, *args):
    if args[:3] == ('worktree', 'add', '--detach'):
        destination, revision = args[3:]
        original_git(root, 'clone', '--shared', '--no-checkout', str(root), destination)
        return original_git(Path(destination), 'checkout', '--detach', revision)
    if args[:3] == ('worktree', 'remove', '--force'):
        shutil.rmtree(args[3])
        return ''
    return original_git(root, *args)


gate.git = clone_checkout_git
report = gate.check_revision(ROOT, sys.argv[1] if len(sys.argv) > 1 else 'HEAD')
report['checkout_transport'] = 'disposable shared clone; no git worktree operations'
print(json.dumps(report, indent=2, sort_keys=True))
sys.exit(report['status'] != 'PASS')
