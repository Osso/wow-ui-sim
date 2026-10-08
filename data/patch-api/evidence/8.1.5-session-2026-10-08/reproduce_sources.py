"""Reproduce every saved source with recorded flags and prove prior inputs unchanged."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

from build_accounting import load_extractor, read, dump, ROOT, EVIDENCE, SOURCES

sys.dont_write_bytecode = True
BASE = '127aa3724'
TEMPLATE = ROOT / 'data/patch-api/evidence/8.2.5-session-2026-10-08'
OUTPUT = Path('/home/osso/.cache/wow-ui-sim-targets/p815-page/audit-reproduction')


def digest(data):
    return hashlib.sha256(data).hexdigest()


def extract_result(extract, raw, flags):
    options = {flag.removeprefix('--').replace('-', '_'): True for flag in flags}
    try:
        return {'sha256': digest(extract(raw, **options).encode()), 'error': None}
    except ValueError as error:
        return {'sha256': None, 'error': str(error)}


def reproduce():
    OUTPUT.mkdir(parents=True, exist_ok=True)
    recipes = {row['patch']: row for row in read(TEMPLATE / 'p825-register-reproduction.json')}
    extracts = {row['patch']: row for row in read(TEMPLATE / 'p825-saved-extract-reproduction.json')}
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    extractor = load_extractor()
    registers, saved = [], []
    for path in sorted(SOURCES.glob('*-wikitext-register.json')):
        patch = path.name.removesuffix('-wikitext-register.json')
        provenance = read(SOURCES / f'{patch}-api-changes.provenance.json')
        recorded = provenance.get('generator_flags')
        flags = recorded if recorded is not None else recipes[patch]['verified_flags']
        output = OUTPUT / path.name
        command = ['python3', 'tools/gen_patch_wikitext_register.py', patch,
                   str(SOURCES / f'{patch}-api-changes.wikitext'),
                   str(read(path)['source']['revid']), str(output), *flags]
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
        identical = result.returncode == 0 and output.read_bytes() == path.read_bytes()
        registers.append({'patch': patch, 'recorded_flags': recorded, 'verified_flags': flags,
                          'inferred_historical_flags': recorded is None, 'byte_identical': identical,
                          'sha256': digest(path.read_bytes()), 'command': command,
                          'cwd': str(ROOT), 'revision': revision, 'exit': result.returncode,
                          'stdout': result.stdout, 'stderr': result.stderr})
        assert identical, registers[-1]
        recorded = provenance.get('extractor_flags')
        flags = recorded if recorded is not None else extracts[patch]['verified_flags']
        raw = (SOURCES / f'{patch}-api-changes.wikitext').read_text()
        result = extract_result(extractor.extract_text, raw, flags)
        retained_sha = digest((SOURCES / f'{patch}-api-changes.txt').read_bytes())
        saved.append({'patch': patch, 'recorded_flags': recorded, 'verified_flags': flags,
                      'inferred_historical_flags': recorded is None,
                      'byte_identical': result['sha256'] == retained_sha,
                      'sha256': retained_sha, 'error': result['error']})
        if patch in extracts:
            assert saved[-1]['byte_identical'] == extracts[patch]['byte_identical'], saved[-1]
            assert saved[-1]['error'] == extracts[patch]['error'], saved[-1]
        else:
            assert saved[-1]['byte_identical'], saved[-1]
    dump(EVIDENCE / 'p815-register-reproduction.json', registers)
    dump(EVIDENCE / 'p815-saved-extract-reproduction.json', saved)
    return extractor, registers


def preserve(extractor, registers):
    before = read(EVIDENCE / 'p815-input-hashes-before.json')
    rows = []
    for path, old in before.items():
        current = digest((ROOT / path).read_bytes())
        assert old == current, path
        rows.append({'path': path, 'before': old, 'after': current, 'unchanged': True})
    dump(EVIDENCE / 'p815-input-preservation.json', {'base_revision': BASE, 'rows': rows})
    result = subprocess.run(['git', 'show', BASE + ':tools/extract_patch_non_inventory.py'],
                            cwd=ROOT, capture_output=True, text=True)
    assert result.returncode == 0, result.stderr
    namespace = {'__name__': 'before_extractor', '__file__': str(ROOT / 'tools/extract_patch_non_inventory.py')}
    exec(compile(result.stdout, 'before_extractor', 'exec'), namespace)
    outcomes = []
    for register in registers:
        patch = register['patch']
        if patch == '8.1.5':
            continue
        raw = (SOURCES / f'{patch}-api-changes.wikitext').read_text()
        for flags in ([], ['--preserve-examples']):
            before = extract_result(namespace['extract_text'], raw, flags)
            after = extract_result(extractor.extract_text, raw, flags)
            assert before == after, (patch, flags)
            outcomes.append({'patch': patch, 'flags': flags, 'before': before,
                             'after': after, 'unchanged': True})
    dump(EVIDENCE / 'p815-extract-preservation.json', outcomes)


if __name__ == '__main__':
    extractor, registers = reproduce()
    preserve(extractor, registers)
    print(json.dumps({'registers': len(registers), 'saved_extracts': len(registers),
                      'inherited_extract_failures': [row['patch'] for row in read(EVIDENCE / 'p815-saved-extract-reproduction.json') if not row['byte_identical']]}))
