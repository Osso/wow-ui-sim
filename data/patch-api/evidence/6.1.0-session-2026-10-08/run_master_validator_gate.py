"""Execute the archived, byte-identical master gate against this audit checkout."""
import json
from pathlib import Path
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent


def main():
    source = (HERE / 'p610-master-validator-gate.py').read_text()
    namespace = {'__name__': 'pinned_master_gate',
                 '__file__': str(ROOT / 'tools/check_patch_validators.py')}
    exec(compile(source, 'pinned_master_gate', 'exec'), namespace)
    original_git = namespace['git']

    def check_wiki_before_commit(root, *args):
        if 'commit' in args:
            for path in ('docs/wiki/index.md', 'docs/wiki/log.md'):
                before = original_git(root, 'show', 'HEAD:' + path)
                after = (root / path).read_text()
                assert len(after.splitlines()) >= len(before.splitlines()), path
        return original_git(root, *args)

    namespace['git'] = check_wiki_before_commit
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    report = namespace['check_revision'](ROOT, revision)
    (HERE / 'p610-master-validator-gate-report.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({'revision': report['revision'], 'status': report['status'],
                      'summary': report['summary'], 'error': report.get('error')}, indent=2))
    return int(report['status'] != 'PASS')


if __name__ == '__main__':
    sys.exit(main())
