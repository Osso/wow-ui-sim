"""Reproduce the complete register/extract set at one recorded Git revision."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
BASE = '787b47591824a1be135c14b395d539dd2438a95e'
FLAG_PREFIX = 'data/patch-api/evidence/6.2.0-session-2026-10-08/integrated/'


def blob(revision, path):
    return subprocess.check_output(['git', 'show', f'{revision}:{path}'], cwd=ROOT)


def main():
    revision = sys.argv[1]
    names = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', revision,
                                    'data/patch-api/sources'], cwd=ROOT, text=True).splitlines()
    register_flags = {row['patch']: row['verified_flags'] for row in
                      json.loads(blob(BASE, FLAG_PREFIX + 'p620-register-reproduction.json'))}
    extract_flags = {row['patch']: row['verified_flags'] for row in
                     json.loads(blob(BASE, FLAG_PREFIX + 'p620-saved-extract-reproduction.json'))}
    own = json.loads(blob(revision, 'data/patch-api/sources/5.5.4-api-changes.provenance.json'))
    register_flags['5.5.4'] = own['generator_flags']
    extract_flags['5.5.4'] = own['extractor_flags']
    records = []
    with tempfile.TemporaryDirectory(prefix='p554-reproduction-') as directory:
        scratch = Path(directory)
        (scratch / 'tools').mkdir()
        sources = scratch / 'data/patch-api/sources'
        sources.mkdir(parents=True)
        for tool in ['gen_patch_wikitext_register', 'extract_patch_non_inventory']:
            path = f'tools/{tool}.py'
            (scratch / path).write_bytes(blob(revision, path))
        for register_path in sorted(name for name in names if name.endswith('-wikitext-register.json')):
            patch = Path(register_path).name.removesuffix('-wikitext-register.json')
            raw_path = f'data/patch-api/sources/{patch}-api-changes.wikitext'
            expected_register = blob(revision, register_path)
            register = json.loads(expected_register)
            (scratch / raw_path).write_bytes(blob(revision, raw_path))
            out = scratch / register_path
            command = [sys.executable, '-B', str(scratch / 'tools/gen_patch_wikitext_register.py'),
                       patch, str(scratch / raw_path), str(register['source']['revid']), str(out),
                       *register_flags[patch]]
            result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
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
                result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
                record.update(extract_exit=result.returncode,
                              extract_sha256=hashlib.sha256(expected_text).hexdigest(),
                              extract_byte_identical=result.returncode == 0 and
                              (scratch / text_path).read_bytes() == expected_text,
                              extract_error=result.stderr)
            records.append(record)
    report = {'revision': revision, 'historical_flags_revision': BASE,
              'historical_flags_sources': [FLAG_PREFIX + 'p620-register-reproduction.json',
                                           FLAG_PREFIX + 'p620-saved-extract-reproduction.json'],
              'records': records}
    (HERE / 'p554-reproduction.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({'registers': len(records),
                      'register_failures': [row['patch'] for row in records if not row['register_byte_identical']],
                      'extracts': sum('extract_exit' in row for row in records),
                      'extract_failures': [row['patch'] for row in records
                                           if 'extract_exit' in row and not row['extract_byte_identical']]}, indent=2))


if __name__ == '__main__':
    main()
