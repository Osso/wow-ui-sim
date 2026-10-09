"""Reproduce the complete register/extract set at one recorded Git revision."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
BASE = '4d046d99d29f71edcd83e55076fd0b07d8f9ff4f'
FLAG_PREFIX = 'data/patch-api/evidence/5.4.8-session-2026-10-08/integrated/'


def blob(revision, path):
    return subprocess.check_output(['git', 'show', f'{revision}:{path}'], cwd=ROOT)


def main():
    revision = subprocess.check_output(['git', 'rev-parse', '--verify', sys.argv[1] + '^{commit}'],
                                       cwd=ROOT, text=True).strip()
    names = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', revision,
                                    'data/patch-api/sources'], cwd=ROOT, text=True).splitlines()
    register_flags = {row['patch']: row['verified_flags'] for row in
                      json.loads(blob(BASE, FLAG_PREFIX + 'p548-register-reproduction.json'))}
    extract_flags = {row['patch']: row['verified_flags'] for row in
                     json.loads(blob(BASE, FLAG_PREFIX + 'p548-saved-extract-reproduction.json'))}
    own = json.loads(blob(revision, 'data/patch-api/sources/5.5.4-api-changes.provenance.json'))
    register_flags['5.5.4'] = own['generator_flags']
    extract_flags['5.5.4'] = own['extractor_flags']
    extractor = {'__file__': str(ROOT / 'tools/extract_patch_non_inventory.py'), '__name__': 'pinned_extractor'}
    exec(compile(blob(revision, 'tools/extract_patch_non_inventory.py'), 'pinned_extractor', 'exec'), extractor)
    inherited_extracts = {row['patch']: row for row in json.loads(blob(BASE, FLAG_PREFIX + 'p548-saved-extract-reproduction.json'))}
    records = []
    with tempfile.TemporaryDirectory(prefix='p550-reproduction-') as directory:
        scratch = Path(directory)
        (scratch / 'tools').mkdir()
        sources = scratch / 'data/patch-api/sources'
        sources.mkdir(parents=True)
        for tool in ['gen_patch_wikitext_register', 'extract_patch_non_inventory']:
            path = f'tools/{tool}.py'
            (scratch / path).write_bytes(blob(revision, path))
        for source_path in (name for name in names if name.endswith('.wikitext')):
            content = blob(revision, source_path)
            (scratch / source_path).write_bytes(content)
        for register_path in sorted(name for name in names if name.endswith('-wikitext-register.json')):
            patch = Path(register_path).name.removesuffix('-wikitext-register.json')
            raw_path = f'data/patch-api/sources/{patch}-api-changes.wikitext'
            expected_register = blob(revision, register_path)
            register = json.loads(expected_register)
            provenance = json.loads(blob(revision, f'data/patch-api/sources/{patch}-api-changes.provenance.json'))
            register_flags[patch] = provenance.get('generator_flags', register_flags.get(patch, []))
            extract_flags[patch] = provenance.get('extractor_flags', extract_flags.get(patch, []))
            out = scratch / register_path
            command = [sys.executable, '-B', str(scratch / 'tools/gen_patch_wikitext_register.py'),
                       patch, str(scratch / raw_path), str(register['source']['revid']), str(out),
                       *register_flags[patch]]
            result = subprocess.run(command, cwd=scratch, capture_output=True, text=True)
            record = {'patch': patch, 'register_exit': result.returncode,
                      'register_sha256': hashlib.sha256(expected_register).hexdigest(),
                      'register_byte_identical': result.returncode == 0 and out.read_bytes() == expected_register,
                      'register_error': result.stderr,
                      'generator_flags': register_flags[patch],
                      'extractor_flags': extract_flags[patch]}
            text_path = f'data/patch-api/sources/{patch}-api-changes.txt'
            if text_path in names:
                expected_text = blob(revision, text_path)
                command = [sys.executable, '-B', str(scratch / 'tools/extract_patch_non_inventory.py'),
                           '--patch', patch, '--text-only', *extract_flags[patch]]
                result = subprocess.run(command, cwd=scratch, capture_output=True, text=True)
                record.update(extract_exit=result.returncode,
                              extract_sha256=hashlib.sha256(expected_text).hexdigest(),
                              extract_byte_identical=result.returncode == 0 and
                              (scratch / text_path).read_bytes() == expected_text,
                              extract_error=result.stderr)
            options = {flag.removeprefix('--').replace('-', '_'): True for flag in extract_flags[patch]}
            try:
                generated = extractor['extract_text'](blob(revision, raw_path).decode(), **options)
                record['generated_extract_sha256'] = hashlib.sha256(generated.encode()).hexdigest()
                record['generated_extract_error'] = None
            except ValueError as error:
                record['generated_extract_sha256'] = None
                record['generated_extract_error'] = str(error)
            if patch in inherited_extracts:
                before = inherited_extracts[patch]
                assert (record['generated_extract_sha256'], record['generated_extract_error'], record['extract_byte_identical']) == (before['sha256'], before['error'], before['byte_identical']), patch
            records.append(record)
    report = {'revision': revision, 'historical_flags_revision': BASE,
              'historical_flags_sources': [FLAG_PREFIX + 'p548-register-reproduction.json',
                                           FLAG_PREFIX + 'p548-saved-extract-reproduction.json'],
              'records': records}
    report_path = HERE / 'p550-reproduction.json'
    previous_report = report_path.read_bytes() if report_path.exists() else None
    report_path.write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({'registers': len(records),
                      'register_failures': [row['patch'] for row in records if not row['register_byte_identical']],
                      'extracts': sum('extract_exit' in row for row in records),
                      'extract_failures': [row['patch'] for row in records
                                           if 'extract_exit' in row and not row['extract_byte_identical']]}, indent=2))


if __name__ == '__main__':
    main()
