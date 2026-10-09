#!/usr/bin/env python3
"""Reproduce archived parser defaults and recorded outputs; no runtime claims."""
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parent / 'archives/original'


def load(name):
    spec = importlib.util.spec_from_file_location(name, ROOT / 'tools' / name)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def digest(data):
    return hashlib.sha256(data).hexdigest()


def generate(tool, patch, raw, revision, flags):
    with tempfile.TemporaryDirectory() as temporary:
        output = Path(temporary) / 'register.json'
        result = subprocess.run([sys.executable, '-B', str(ROOT / 'tools' / tool),
                                 patch, str(raw), str(revision), str(output), *flags],
                                cwd=ROOT, capture_output=True)
        if result.returncode:
            return {'exit': result.returncode,
                    'error': result.stderr.decode().splitlines()[-1]}, None
        data = output.read_bytes()
        return {'exit': 0, 'sha256': digest(data)}, data


def extract(module, raw, flags):
    try:
        data = module.extract_text(raw, **{f.removeprefix('--').replace('-', '_'): True
                                          for f in flags}).encode()
        return {'exit': 0, 'sha256': digest(data)}, data
    except (ValueError, TypeError) as error:
        return {'exit': 1, 'error': f'{type(error).__name__}: {error}'}, None


def replay():
    base = load('base-extract_patch_non_inventory.py')
    current = load('extract_patch_non_inventory.py')
    defaults = []
    paths = sorted(set(ROOT.glob('data/patch-api/sources/*.wikitext')) |
                   set(ROOT.glob('data/patch-api/source-cache/legacy-2026-10-09/*-wikitext.txt')))
    for raw in paths:
        patch = raw.name.split('-')[0]
        old, _ = generate('base-gen_patch_wikitext_register.py', patch, raw, 0, [])
        new, _ = generate('gen_patch_wikitext_register.py', patch, raw, 0, [])
        assert old == new, f'default register changed: {raw}'
        old_extract, _ = extract(base, raw.read_text(), [])
        new_extract, _ = extract(current, raw.read_text(), [])
        assert old_extract == new_extract, f'default extract changed: {raw}'
        defaults.append({'path': str(raw.relative_to(ROOT)), 'register': new,
                         'extract': new_extract, 'base_equal': True})
    recorded = []
    for path in sorted(ROOT.glob('data/patch-api/sources/*-wikitext-register.json')):
        register = json.loads(path.read_text())
        patch = register['patch']
        provenance_path = ROOT / f'data/patch-api/sources/{patch}-api-changes.provenance.json'
        provenance = json.loads(provenance_path.read_text()) if provenance_path.exists() else {}
        raw = ROOT / register['source']['path']
        flags = provenance.get('generator_flags', [])
        result, data = generate('gen_patch_wikitext_register.py', patch, raw,
                                register['source']['revid'], flags)
        if patch != '2.4.0':
            old, old_data = generate('base-gen_patch_wikitext_register.py', patch, raw,
                                     register['source']['revid'], flags)
            assert old == result and old_data == data, f'recorded register behavior changed: {path}'
        item = {'patch': patch, 'generator_flags': flags, 'register_result': result,
                'register_matches_recorded': data == path.read_bytes(),
                'base_equal': patch != '2.4.0'}
        text_path = ROOT / f'data/patch-api/sources/{patch}-api-changes.txt'
        if text_path.exists():
            flags = provenance.get('extractor_flags', [])
            result, data = extract(current, raw.read_text(), flags)
            if patch != '2.4.0':
                old, old_data = extract(base, raw.read_text(), flags)
                assert old == result and old_data == data, f'recorded extract behavior changed: {path}'
            item.update(extractor_flags=flags, extract_result=result,
                        extract_matches_recorded=data == text_path.read_bytes())
        recorded.append(item)
    return {'defaults': defaults, 'recorded': recorded,
            'proof_boundary': 'parser replay only; inherited mismatches/errors retained'}


if __name__ == '__main__':
    print(json.dumps(replay(), indent=2, ensure_ascii=False))
