"""Reproduce the complete committed register set and saved extracts at HEAD."""
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


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def read_recipes():
    recipes, extracts = {}, {}
    paths = git('ls-tree', '-r', '--name-only', 'HEAD', 'data/patch-api/evidence').splitlines()
    for name in sorted(paths):
        if not name.endswith(('-register-reproduction.json', '-saved-extract-reproduction.json')):
            continue
        rows = json.loads(git('show', 'HEAD:' + name))
        if isinstance(rows, list):
            for row in rows:
                if 'patch' in row and 'verified_flags' in row:
                    destination = extracts if name.endswith('-saved-extract-reproduction.json') else recipes
                    destination[row['patch']] = row['verified_flags']
    return recipes, extracts


def reproduce():
    revision = git('rev-parse', 'HEAD').strip()
    names = git('ls-tree', '-r', '--name-only', revision, 'data/patch-api/sources').splitlines()
    registers = [name for name in names if name.endswith('-wikitext-register.json')]
    recipes, extract_recipes = read_recipes()
    spec = importlib.util.spec_from_file_location('extractor', ROOT / 'tools/extract_patch_non_inventory.py')
    extractor = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(extractor)
    results = []
    with tempfile.TemporaryDirectory(prefix='p520-reproduce-') as temporary:
        for name in registers:
            path = ROOT / name
            patch = path.name.removesuffix('-wikitext-register.json')
            prefix = ROOT / 'data/patch-api/sources' / patch
            provenance = json.loads(Path(str(prefix) + '-api-changes.provenance.json').read_text())
            flags = provenance.get('generator_flags', recipes.get(patch))
            assert flags is not None, patch
            output = Path(temporary) / path.name
            command = [sys.executable, '-B', str(ROOT / 'tools/gen_patch_wikitext_register.py'),
                       patch, str(prefix) + '-api-changes.wikitext',
                       str(json.loads(path.read_text())['source']['revid']), str(output), *flags]
            result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
            identical = result.returncode == 0 and output.read_bytes() == path.read_bytes()
            assert identical, (patch, result.stderr)
            extract_flags = provenance.get('extractor_flags', extract_recipes.get(patch, []))
            options = {flag.removeprefix('--').replace('-', '_'): True for flag in extract_flags}
            saved = Path(str(prefix) + '-api-changes.txt').read_bytes()
            error = None
            try:
                generated = extractor.extract_text(Path(str(prefix) + '-api-changes.wikitext').read_text(), **options).encode()
                extract_identical = generated == saved
            except ValueError as exception:
                error = str(exception)
                extract_identical = False
            if not extract_identical:
                assert patch in ('12.0.5', '12.0.7', '12.1.0'), (patch, error)
            results.append({'patch': patch, 'path': name, 'revision': revision,
                            'generator_flags': flags, 'register_identical': identical,
                            'register_sha256': digest(path.read_bytes()),
                            'extractor_flags': extract_flags, 'extract_identical': extract_identical,
                            'extract_error': error, 'extract_sha256': digest(saved)})
    (HERE / 'reproduction.json').write_text(json.dumps(results, indent=2) + '\n')
    print(json.dumps({'registers': len(results), 'extracts': sum(r['extract_identical'] for r in results),
                      'inherited_extract_failures': [r['patch'] for r in results if not r['extract_identical']]}))


if __name__ == '__main__':
    reproduce()
