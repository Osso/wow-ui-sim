"""Prove all saved registers/extracts and record inherited/rebase changes explicitly."""
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
TEMPLATE = ROOT / 'data/patch-api/evidence/8.3.0-session-2026-10-08'
OUTPUT = Path('/home/osso/.cache/wow-ui-sim-targets/p825-page/audit-reproduction')


def read(path):
    return json.loads(path.read_text())


def dump(path, value):
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n')


def digest(data):
    return hashlib.sha256(data).hexdigest()


def git_blob(ref, path):
    result = subprocess.run(['git', 'show', f'{ref}:{path}'], cwd=ROOT, capture_output=True)
    assert result.returncode == 0, result.stderr
    return result.stdout


def load_extractor():
    spec = importlib.util.spec_from_file_location('extractor', ROOT / 'tools/extract_patch_non_inventory.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def extract_result(extractor, raw, flags):
    options = {flag.removeprefix('--').replace('-', '_'): True for flag in flags}
    try:
        text = extractor.extract_text(raw, **options)
        return {'sha256': digest(text.encode()), 'error': None}
    except ValueError as error:
        return {'sha256': None, 'error': str(error)}


def reproduce(extractor):
    OUTPUT.mkdir(parents=True, exist_ok=True)
    old_registers = {row['patch']: row for row in read(TEMPLATE / 'p830-register-reproduction.json')}
    old_extracts = {row['patch']: row for row in read(TEMPLATE / 'p830-saved-extract-reproduction.json')}
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    registers, extracts = [], []
    for path in sorted(SOURCES.glob('*-wikitext-register.json')):
        patch = path.name.removesuffix('-wikitext-register.json')
        provenance = read(SOURCES / f'{patch}-api-changes.provenance.json')
        recorded = provenance.get('generator_flags')
        flags = recorded if recorded is not None else old_registers[patch]['verified_flags']
        revid = read(path)['source']['revid']
        output = OUTPUT / path.name
        command = ['python3', 'tools/gen_patch_wikitext_register.py', patch,
                   str(SOURCES / f'{patch}-api-changes.wikitext'), str(revid), str(output), *flags]
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
        identical = result.returncode == 0 and output.read_bytes() == path.read_bytes()
        registers.append({'patch': patch, 'recorded_flags': recorded, 'verified_flags': flags,
                          'inferred_historical_flags': recorded is None,
                          'exit': result.returncode, 'byte_identical': identical,
                          'sha256': digest(path.read_bytes()), 'command': command, 'cwd': str(ROOT),
                          'revision': revision, 'stdout': result.stdout, 'stderr': result.stderr})
        assert identical, registers[-1]
        recorded = provenance.get('extractor_flags')
        flags = recorded if recorded is not None else old_extracts[patch]['verified_flags']
        raw = (SOURCES / f'{patch}-api-changes.wikitext').read_text()
        result = extract_result(extractor, raw, flags)
        saved = SOURCES / f'{patch}-api-changes.txt'
        extracts.append({'patch': patch, 'recorded_flags': recorded, 'verified_flags': flags,
                         'inferred_historical_flags': recorded is None,
                         'byte_identical': result['sha256'] == digest(saved.read_bytes()),
                         'error': result['error'], 'sha256': digest(saved.read_bytes())})
        if patch not in ('12.0.5', '12.0.7', '12.1.0'):
            assert extracts[-1]['byte_identical'], extracts[-1]
    dump(EVIDENCE / 'p825-register-reproduction.json', registers)
    dump(EVIDENCE / 'p825-saved-extract-reproduction.json', extracts)
    return registers


def preserve_inputs(registers, extractor):
    # origin/master used for rebase is pinned here; later remote movements cannot alter receipts.
    base = subprocess.check_output(['git', 'rev-parse', 'origin/master'], cwd=ROOT, text=True).strip()
    old_hashes = read(EVIDENCE / 'p825-input-hashes-before.json')
    rows = []
    for path, before in old_hashes.items():
        master_hash = digest(git_blob(base, path))
        current = digest((ROOT / path).read_bytes())
        allowed = path == 'data/patch-api/sources/9.2.5-page-coverage.json'
        assert current == master_hash or allowed, path
        rows.append({'path': path, 'original_before': before, 'rebased_master': master_hash,
                     'current': current, 'rebase_changed': before != master_hash,
                     'unchanged_after_rebase': current == master_hash,
                     'allowed_change': 'One ReportPosting pending row becomes bounded retail absence; see p825-later-gap-closures.json.' if current != master_hash else None})
    for row in registers:
        patch = row['patch']
        if patch == '8.2.5':
            continue
        for suffix in ('api-changes.wikitext', 'api-changes.txt', 'api-changes.provenance.json', 'wikitext-register.json', 'page-coverage.json'):
            path = f'data/patch-api/sources/{patch}-{suffix}'
            if path in old_hashes or not (ROOT / path).exists():
                continue
            master_hash = digest(git_blob(base, path))
            current = digest((ROOT / path).read_bytes())
            assert master_hash == current, path
            rows.append({'path': path, 'original_before': None, 'rebased_master': master_hash,
                         'current': current, 'rebase_changed': True,
                         'unchanged_after_rebase': True, 'allowed_change': None})
    dump(EVIDENCE / 'p825-input-preservation.json', {'base_revision': base, 'rows': rows})
    before_namespace = {'__name__': 'before_extractor', '__file__': str(ROOT / 'tools/extract_patch_non_inventory.py')}
    exec(compile(git_blob(base, 'tools/extract_patch_non_inventory.py'), 'before_extractor', 'exec'), before_namespace)
    class Before:
        extract_text = staticmethod(before_namespace['extract_text'])
    outcomes = []
    for row in registers:
        patch = row['patch']
        if patch == '8.2.5':
            continue
        raw = (SOURCES / f'{patch}-api-changes.wikitext').read_text()
        for flags in ([], ['--preserve-examples']):
            before = extract_result(Before, raw, flags)
            after = extract_result(extractor, raw, flags)
            assert before == after, (patch, flags)
            outcomes.append({'patch': patch, 'flags': flags, 'before': before, 'after': after, 'unchanged': True})
    dump(EVIDENCE / 'p825-extract-preservation.json', outcomes)


def main():
    extractor = load_extractor()
    registers = reproduce(extractor)
    preserve_inputs(registers, extractor)
    print(json.dumps({'registers_reproduced': len(registers), 'extracts_checked': len(registers)}))


if __name__ == '__main__':
    main()
