#!/usr/bin/env python3
"""Separate sealed current literal refinements; original history remains immutable."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

HERE = Path(__file__).resolve().parent
MANIFEST_SHA256 = 'e225dc325f295c5900cedfb7a902d6aeb3e9e0d5188463a2ae69bb06c7ec1c60'


def digest(value):
    return hashlib.sha256(value).hexdigest()


def load_inputs(here):
    raw = (here / 'inputs.json').read_bytes()
    assert digest(raw) == MANIFEST_SHA256, 'current manifest seal'
    manifest = json.loads(raw)
    for name, seal in manifest['sealed_files'].items():
        value = (here / name).read_bytes()
        assert digest(value) == seal['sha256'], 'current sealed input: ' + name
        assert len(value) == seal['bytes'] < 5_000_000, 'current sealed size: ' + name
    for name, seal in manifest['original_outer_sha256'].items():
        assert digest((here.parent / name).read_bytes()) == seal, 'original outer seal: ' + name
    return manifest


def load_original(here):
    load_inputs(here)
    path = here.parent / 'validate.py'
    spec = importlib.util.spec_from_file_location('p201_original', path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def load_accounting(here):
    original = load_original(here)
    manifest, bundle = original.load_inputs(here.parent)
    pin, raw = original.verify_pin(here.parent, bundle)
    auditor = original.load_tool({'tools/audit_patch_2_0_1_source.py':
                                  (here / 'auditor.py').read_text()},
                                 'tools/audit_patch_2_0_1_source.py')
    expected = auditor.account(raw.decode())
    gaps = json.loads((here / 'known-gaps.json').read_bytes())
    return expected, gaps


def verify_current(ledger, gaps, expected):
    original = load_original(HERE)
    return original.verify_accounting(ledger, gaps, expected)


def replay(here):
    inputs = load_inputs(here)
    expected, gaps = load_accounting(here)
    ledger = json.loads((here / 'page-coverage.json').read_bytes())
    result = dict(verify_current(ledger, gaps, expected))
    assert (json.dumps(expected, indent=2, ensure_ascii=False) + '\n').encode() == (
        here / 'page-coverage.json').read_bytes(), 'current serialized ledger'
    with tempfile.TemporaryDirectory(prefix='p201-current-frozen-fixtures-') as directory:
        root = Path(directory)
        (root / 'tools').mkdir()
        (root / 'tools/audit_patch_2_0_1_source.py').write_bytes((here / 'auditor.py').read_bytes())
        (root / 'tools/test_patch_2_0_1_source.py').write_bytes((here / 'source-fixtures.py').read_bytes())
        source = root / 'data/patch-api/source-cache/legacy-2026-10-09/2.0.1-wikitext.txt'
        source.parent.mkdir(parents=True)
        source.write_bytes((here.parent / 'source.wikitext').read_bytes())
        proof = subprocess.run([sys.executable, '-B', 'tools/test_patch_2_0_1_source.py'],
                               cwd=root, capture_output=True, text=True,
                               env=dict(os.environ, PATH='/p201-no-git', PYTHONPATH=''))
        assert proof.returncode == 0 and 'Ran 6 tests' in proof.stderr, 'frozen current SOURCE fixtures'
        assert not (root / '.git').exists() and not (root / 'target').exists(), 'Git/target dependency'
    result['current_gap_records'] = len(gaps)
    result['archived_source_cases'] = 6
    result['current_sealed_files'] = len(inputs['sealed_files'])
    result['original_seals_unchanged'] = 18
    return result


if __name__ == '__main__':
    print(json.dumps(replay(HERE), sort_keys=True))
