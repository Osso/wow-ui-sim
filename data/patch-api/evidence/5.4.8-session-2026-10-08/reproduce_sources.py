"""Reproduce recorded artifacts and preserve the complete pre-audit source set."""
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'
BASE = 'd0fabed03cd6534b73341fbf569e09c6c388202e'


def read(path):
    return json.loads(path.read_text())


def dump(name, value):
    (HERE / name).write_text(json.dumps(value, indent=2) + '\n')


def digest(value):
    return hashlib.sha256(value).hexdigest()


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT)


def outcome(extract, raw, flags):
    options = {flag.removeprefix('--').replace('-', '_'): True for flag in flags}
    try:
        return {'sha256': digest(extract(raw, **options).encode()), 'error': None}
    except ValueError as error:
        return {'sha256': None, 'error': str(error)}


def reproduce(extractor, revision):
    inherited = ROOT / 'data/patch-api/evidence/6.1.0-session-2026-10-08/integrated'
    recipes = {r['patch']: r for r in read(inherited / 'p610-register-reproduction.json')}
    extracts = {r['patch']: r for r in read(inherited / 'p610-saved-extract-reproduction.json')}
    registers, saved = [], []
    with tempfile.TemporaryDirectory(prefix='p548-reproduction-') as temporary:
        for path in sorted(SOURCES.glob('*-wikitext-register.json')):
            patch = path.name.removesuffix('-wikitext-register.json')
            provenance = read(SOURCES / f'{patch}-api-changes.provenance.json')
            flags = provenance.get('generator_flags')
            verified = flags if flags is not None else recipes[patch]['verified_flags']
            output = Path(temporary) / path.name
            command = ['python3', '-B', str(ROOT / 'tools/gen_patch_wikitext_register.py'), patch,
                       str(SOURCES / f'{patch}-api-changes.wikitext'), str(read(path)['source']['revid']),
                       str(output), *verified]
            result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
            identical = result.returncode == 0 and output.read_bytes() == path.read_bytes()
            registers.append({'patch': patch, 'verified_flags': verified, 'recorded_flags': flags,
                              'revision': revision, 'exit': result.returncode, 'byte_identical': identical,
                              'sha256': digest(path.read_bytes()), 'stderr': result.stderr})
            assert identical, registers[-1]
            flags = provenance.get('extractor_flags')
            verified = flags if flags is not None else extracts[patch]['verified_flags']
            generated = outcome(extractor.extract_text,
                                (SOURCES / f'{patch}-api-changes.wikitext').read_text(), verified)
            text_digest = digest((SOURCES / f'{patch}-api-changes.txt').read_bytes())
            identical = generated['sha256'] == text_digest
            saved.append({'patch': patch, 'verified_flags': verified, 'recorded_flags': flags,
                          'revision': revision, 'byte_identical': identical,
                          'sha256': text_digest, **generated, 'saved_sha256': text_digest})
            if patch in extracts:
                assert (identical, generated['error']) == (extracts[patch]['byte_identical'], extracts[patch]['error']), saved[-1]
            else:
                assert identical, saved[-1]
    dump('p548-register-reproduction.json', registers)
    dump('p548-saved-extract-reproduction.json', saved)
    return {'registers': len(registers), 'extracts': sum(r['byte_identical'] for r in saved),
            'inherited_extract_failures': [r['patch'] for r in saved if not r['byte_identical']]}


def preserve(extractor, revision):
    paths = git('ls-tree', '-r', '--name-only', BASE, 'data/patch-api/sources').decode().splitlines()
    rows, modes = [], []
    namespace = {'__name__': 'historical_extractor', '__file__': str(ROOT / 'tools/extract_patch_non_inventory.py')}
    exec(compile(git('show', f'{BASE}:tools/extract_patch_non_inventory.py'), 'historical_extractor', 'exec'), namespace)
    for path in paths:
        before = git('show', f'{BASE}:{path}')
        after = (ROOT / path).read_bytes()
        assert before == after, path
        rows.append({'path': path, 'before_sha256': digest(before), 'after_sha256': digest(after)})
        if path.endswith('-api-changes.wikitext'):
            provenance = read(ROOT / path.replace('-api-changes.wikitext', '-api-changes.provenance.json'))
            flags = {(), ('--preserve-examples',), tuple(provenance.get('extractor_flags', []))}
            for mode in sorted(flags):
                previous = outcome(namespace['extract_text'], before.decode(), mode)
                current = outcome(extractor.extract_text, before.decode(), mode)
                assert previous == current, (path, mode)
                modes.append({'path': path, 'flags': mode, 'before': previous, 'after': current})
    dump('p548-input-preservation.json', {'base_revision': BASE, 'audit_revision': revision, 'rows': rows})
    dump('p548-extract-preservation.json', modes)


def main():
    spec = importlib.util.spec_from_file_location('extractor', ROOT / 'tools/extract_patch_non_inventory.py')
    extractor = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(extractor)
    revision = git('rev-parse', 'HEAD').decode().strip()
    result = reproduce(extractor, revision)
    preserve(extractor, revision)
    print(json.dumps(result))


if __name__ == '__main__':
    main()
