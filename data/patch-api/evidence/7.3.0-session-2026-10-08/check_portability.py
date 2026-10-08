"""Second-checkout, later-file, source-tamper and read-only validator proof."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent


def run(command):
    result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
    return {'command': command, 'exit': result.returncode,
            'stdout': result.stdout, 'stderr': result.stderr}


def digests(directory):
    return {str(path.relative_to(directory)): hashlib.sha256(path.read_bytes()).hexdigest()
            for path in directory.iterdir() if path.is_file()}


def main():
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    relative = HERE.relative_to(ROOT)
    output = {'revision': revision, 'cases': {}}
    with tempfile.TemporaryDirectory(prefix='wow-ui-sim-p730-validator-') as temporary:
        clone = Path(temporary) / 'checkout'
        cloned = run(['git', 'clone', '--shared', '--no-checkout', str(ROOT), str(clone)])
        assert cloned['exit'] == 0, cloned
        checked = run(['git', '-C', str(clone), 'checkout', '--detach', revision])
        assert checked['exit'] == 0, checked
        validator = clone / relative / 'validate.py'
        before = digests(clone / relative)
        base = run([sys.executable, '-B', str(validator)])
        assert base['exit'] == 0, base
        output['cases']['second_checkout'] = base
        # Future audits add files; frozen historical scope must not become a glob.
        register = clone / 'data/patch-api/sources/7.2.5-wikitext-register.json'
        sweep = clone / 'tests/patch_7_2_5_publication_sweep.rs'
        assert not register.exists() and not sweep.exists()
        register.write_text(json.dumps({'schema': 'patch-api-wikitext-register/v1',
                                       'patch': '7.2.5', 'entries': []}) + '\n')
        sweep.write_text('// Future audit scope sentinel; not historical proof.\n')
        later = run([sys.executable, '-B', str(validator)])
        assert later['exit'] == 0 and later['stdout'] == base['stdout'], later
        output['cases']['later_audit_files_do_not_expand_scope'] = later
        register.unlink()
        sweep.unlink()
        source = clone / 'data/patch-api/sources/7.3.0-api-changes.wikitext'
        original = source.read_bytes()
        try:
            source.write_bytes(original + b'\n')
            tamper = run([sys.executable, '-B', str(validator)])
            assert tamper['exit'] != 0 and 'changed retained artifact' in tamper['stderr'], tamper
            output['cases']['source_tamper_rejected'] = tamper
        finally:
            source.write_bytes(original)
        rows = []
        for path in sorted((clone / 'data/patch-api/evidence').glob('*/validate.py')):
            row = run([sys.executable, '-B', str(path)])
            row['path'] = str(path.relative_to(clone))
            rows.append(row)
        assert all(row['exit'] == 0 for row in rows), rows
        output['cases']['all_validators_in_second_checkout'] = rows
        after = digests(clone / relative)
        assert after == before, 'validation changed retained evidence'
        output['read_only'] = True
        output['temporary_checkout_removed_on_exit'] = True
    (HERE / 'p730-validator-portability.json').write_text(json.dumps(output, indent=2) + '\n')
    print(json.dumps({'status': 'PASS', 'validators': len(rows),
                      'second_checkout': True, 'later_scope': True,
                      'tamper_rejected': True, 'read_only': True}))


if __name__ == '__main__':
    main()
