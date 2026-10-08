"""Reproduce current saved artifacts and compare original inputs at pinned revisions."""
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'
BASE = '7fab847ebe3ec0ef4b4b6212482bd4e9b86cae9b'
TEMPLATE = ROOT / 'data/patch-api/evidence/8.2.5-session-2026-10-08'
OUTPUT = Path('/home/osso/.cache/wow-ui-sim-targets/p810-page/audit-reproduction')


def read(path):
    return json.loads(path.read_text())


def dump(path, value):
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n')


def digest(blob):
    return hashlib.sha256(blob).hexdigest()


def git_blob(ref, path):
    return subprocess.check_output(['git', 'show', f'{ref}:{path}'], cwd=ROOT)


def load_extractor():
    spec = importlib.util.spec_from_file_location('extractor', ROOT / 'tools/extract_patch_non_inventory.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def extract_result(function, raw, flags):
    options = {flag.removeprefix('--').replace('-', '_'): True for flag in flags}
    try:
        return {'sha256': digest(function(raw, **options).encode()), 'error': None}
    except ValueError as error:
        return {'sha256': None, 'error': str(error)}


def reproduce(extractor):
    OUTPUT.mkdir(parents=True, exist_ok=True)
    recipes = {row['patch']: row for row in read(TEMPLATE / 'p825-register-reproduction.json')}
    extract_recipes = {row['patch']: row for row in read(TEMPLATE / 'p825-saved-extract-reproduction.json')}
    registers, extracts = [], []
    for path in sorted(SOURCES.glob('*-wikitext-register.json')):
        patch = path.name.removesuffix('-wikitext-register.json')
        provenance = read(SOURCES / f'{patch}-api-changes.provenance.json')
        flags = provenance.get('generator_flags')
        inferred = flags is None
        if inferred:
            flags = recipes[patch]['verified_flags']
        command = ['python3', 'tools/gen_patch_wikitext_register.py', patch,
                   str(SOURCES / f'{patch}-api-changes.wikitext'), str(read(path)['source']['revid']),
                   str(OUTPUT / path.name), *flags]
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
        identical = result.returncode == 0 and (OUTPUT / path.name).read_bytes() == path.read_bytes()
        registers.append({'patch': patch, 'verified_flags': flags, 'inferred_historical_flags': inferred,
                          'exit': result.returncode, 'byte_identical': identical,
                          'command': command, 'stdout': result.stdout, 'stderr': result.stderr})
        assert identical, registers[-1]
        flags = provenance.get('extractor_flags')
        inferred = flags is None
        if inferred:
            flags = extract_recipes[patch]['verified_flags']
        outcome = extract_result(extractor.extract_text,
                                 (SOURCES / f'{patch}-api-changes.wikitext').read_text(), flags)
        identical = outcome['sha256'] == digest((SOURCES / f'{patch}-api-changes.txt').read_bytes())
        extracts.append({'patch': patch, 'verified_flags': flags, 'inferred_historical_flags': inferred,
                         'byte_identical': identical, **outcome})
        assert identical or patch in ('12.0.5', '12.0.7', '12.1.0'), extracts[-1]
    dump(EVIDENCE / 'p810-register-reproduction.json', registers)
    dump(EVIDENCE / 'p810-saved-extract-reproduction.json', extracts)
    return registers


def preserve_original_inputs(extractor):
    # Compare base and audit tree, not mutable post-merge files. Later page audits
    # legitimately update shared ledgers/provenance; those do not rewrite history.
    audit = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    paths = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', BASE,
                                     'data/patch-api/sources'], cwd=ROOT, text=True).splitlines()
    rows = []
    for path in paths:
        before, after = git_blob(BASE, path), git_blob(audit, path)
        assert before == after, path
        rows.append({'path': path, 'base_sha256': digest(before), 'audit_sha256': digest(after)})
    dump(EVIDENCE / 'p810-input-preservation.json',
         {'base_revision': BASE, 'audit_revision': audit, 'rows': rows})
    namespace = {'__name__': 'base_extractor', '__file__': str(ROOT / 'tools/extract_patch_non_inventory.py')}
    exec(compile(git_blob(BASE, 'tools/extract_patch_non_inventory.py'), 'base_extractor', 'exec'), namespace)
    outcomes = []
    for path in paths:
        if not path.endswith('-api-changes.wikitext'):
            continue
        raw = git_blob(BASE, path).decode()
        for flags in ([], ['--preserve-examples']):
            before = extract_result(namespace['extract_text'], raw, flags)
            after = extract_result(extractor.extract_text, raw, flags)
            assert before == after, (path, flags)
            outcomes.append({'path': path, 'flags': flags, 'before': before, 'after': after})
    dump(EVIDENCE / 'p810-extract-preservation.json', outcomes)


if __name__ == '__main__':
    extractor = load_extractor()
    registers = reproduce(extractor)
    preserve_original_inputs(extractor)
    print(json.dumps({'registers_reproduced': len(registers), 'input_preservation': 'pinned base/audit git trees'}))
