"""Reproduce every saved register/extract using recorded or inherited flags."""
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'
OUTPUT = Path('/home/osso/.cache/wow-ui-sim-targets/p730-page/audit-reproduction')


def read(path):
    return json.loads(path.read_text())


def dump(path, value):
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n')


def main():
    OUTPUT.mkdir(parents=True, exist_ok=True)
    recipes = {row['patch']: row for row in read(ROOT / 'data/patch-api/evidence/8.0.1-session-2026-10-08/p801-register-reproduction.json')}
    extract_recipes = {row['patch']: row for row in read(ROOT / 'data/patch-api/evidence/8.0.1-session-2026-10-08/p801-saved-extract-reproduction.json')}
    inherited = ROOT / 'data/patch-api/evidence/8.1.0-session-2026-10-08'
    for row in read(inherited / 'p810-register-reproduction.json'):
        recipes.setdefault(row['patch'], row)
    for row in read(inherited / 'p810-saved-extract-reproduction.json'):
        extract_recipes.setdefault(row['patch'], row)
    spec = importlib.util.spec_from_file_location('extractor', ROOT / 'tools/extract_patch_non_inventory.py')
    extractor = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(extractor)
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    registers, extracts = [], []
    for path in sorted(SOURCES.glob('*-wikitext-register.json')):
        patch = path.name.removesuffix('-wikitext-register.json')
        provenance = read(SOURCES / f'{patch}-api-changes.provenance.json')
        flags = provenance.get('generator_flags')
        verified = flags if flags is not None else recipes[patch]['verified_flags']
        command = ['python3', '-B', str(ROOT / 'tools/gen_patch_wikitext_register.py'), patch,
                   str(SOURCES / f'{patch}-api-changes.wikitext'), str(read(path)['source']['revid']),
                   str(OUTPUT / path.name), *verified]
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
        identical = result.returncode == 0 and (OUTPUT / path.name).read_bytes() == path.read_bytes()
        registers.append({'patch': patch, 'recorded_flags': flags, 'verified_flags': verified,
                          'inferred_historical_flags': flags is None, 'command': command,
                          'revision': revision, 'exit': result.returncode, 'byte_identical': identical,
                          'stdout': result.stdout, 'stderr': result.stderr,
                          'sha256': hashlib.sha256(path.read_bytes()).hexdigest()})
        assert identical, registers[-1]
        flags = provenance.get('extractor_flags')
        verified = flags if flags is not None else extract_recipes[patch]['verified_flags']
        options = {flag.removeprefix('--').replace('-', '_'): True for flag in verified}
        text_path = SOURCES / f'{patch}-api-changes.txt'
        error = None
        try:
            generated = extractor.extract_text((SOURCES / f'{patch}-api-changes.wikitext').read_text(), **options)
            identical = generated.encode() == text_path.read_bytes()
        except ValueError as failure:
            identical, error = False, str(failure)
        extracts.append({'patch': patch, 'recorded_flags': flags, 'verified_flags': verified,
                         'inferred_historical_flags': flags is None, 'revision': revision,
                         'exit': 0 if error is None else 1, 'byte_identical': identical,
                         'error': error, 'sha256': hashlib.sha256(text_path.read_bytes()).hexdigest(),
                         'inherited_nonreproducible': patch in ('12.0.5', '12.0.7', '12.1.0')})
        assert identical or (patch in extract_recipes and
                            not extract_recipes[patch]['byte_identical']), extracts[-1]
    dump(HERE / 'p730-register-reproduction.json', registers)
    dump(HERE / 'p730-saved-extract-reproduction.json', extracts)
    print(json.dumps({'registers_reproduced': len(registers),
                      'extracts_reproduced': sum(row['byte_identical'] for row in extracts),
                      'inherited_extract_failures': [row['patch'] for row in extracts if not row['byte_identical']]}))


if __name__ == '__main__':
    main()
