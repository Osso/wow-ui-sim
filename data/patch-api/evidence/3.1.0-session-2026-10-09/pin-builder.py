"""Capture original own source/receipts once; never reseal historical closures.

Generation needs Git in this worktree. validate.py does not need Git or target.
"""
import base64
import gzip
import hashlib
import json
from pathlib import Path
import re
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
BASE = 'c17f1b4bb'


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT)


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n')


def capture_revision(revision, selected, bundle):
    commit = git('cat-file', 'commit', revision)
    scope = dict(commit_base64=base64.b64encode(commit).decode(),
                 root_tree=git('rev-parse', revision + '^{tree}').decode().strip(),
                 selected_files={}, tree_objects={})
    for path in selected:
        result = subprocess.run(['git', 'show', revision + ':' + path], cwd=ROOT, capture_output=True)
        if result.returncode:
            # Test/parser additions do not exist at earlier RED revisions.
            assert 'does not exist' in result.stderr.decode() or 'exists on disk, but not in' in result.stderr.decode(), result.stderr
            continue
        content = result.stdout
        snapshot = 'revision/' + revision + '/' + path
        bundle[snapshot] = content.decode()
        scope['selected_files'][path] = dict(snapshot_path=snapshot, sha256=digest(content),
                                            git_blob=git('rev-parse', revision + ':' + path).decode().strip())
        parents = [None] + [str(parent) for parent in Path(path).parents if str(parent) != '.']
        for parent in parents:
            selector = revision + (':' + parent if parent else '^{tree}')
            tree_id = git('rev-parse', selector).decode().strip()
            if tree_id not in scope['tree_objects']:
                tree = git('cat-file', 'tree', tree_id)
                scope['tree_objects'][tree_id] = base64.b64encode(tree).decode()
    return scope


def main():
    assert not (HERE / 'historical-inputs.json').exists(), 'Historical manifest already sealed; retain original, do not regenerate.'
    sources = ROOT / 'data/patch-api/sources'
    for suffix, target in [('api-changes.wikitext', 'historical-source.wikitext'),
                           ('api-changes.provenance.json', 'historical-provenance.json'),
                           ('api-changes.txt', 'historical-extract.txt'),
                           ('wikitext-register.json', 'historical-register.json'),
                           ('page-coverage.json', 'historical-page-coverage.json'),
                           ('signatures.json', 'historical-signatures.json')]:
        (HERE / target).write_bytes((sources / ('3.1.0-' + suffix)).read_bytes())
    (HERE / 'historical-known-gaps.json').write_bytes((ROOT / 'tests/data/patch_3_1_0_sweep_known_gaps.json').read_bytes())
    command_specs = {
        'parser-red': (1, 'FAILED (failures=2)'),
        'parser-first-green': (1, 'FAILED (failures=1)'),
        'parser-green': (0, 'Ran 2 tests'),
        'headless-compile': (101, 'could not find `objective_tracker_tree`'),
        'own-sweep-red': (None, 'new gaps:'),
        'own-sweep-green': (0, 'passed'),
        'negative': (None, 'p310-negative-control'),
    }
    commands = []
    for name, (expected_exit, required) in command_specs.items():
        receipt = json.loads((HERE / (name + '-receipt.json')).read_bytes())
        if expected_exit is None:
            assert receipt['exit_code'] != 0, (name, receipt)
            expected_exit = receipt['exit_code']
        assert receipt['exit_code'] == expected_exit, (name, receipt)
        assert required in (HERE / (name + '.log')).read_text(), (name, required)
        commands.append(dict(receipt, artifact=name + '.log', expected_exit_code=expected_exit, required_log_text=required))
    write_json(HERE / 'command-ledger.json', dict(commands=commands, policy='Exact original targeted development receipts; no current/native acceptance credit.'))
    sweep = (ROOT / 'tests/patch_3_1_0_publication_sweep.rs').read_text()
    later = re.findall(r'include_str!\("\.\./(data/patch-api/sources/[^"\n]+)"\)', sweep.split('later_registers:', 1)[1])
    snapshot = set(later) | {
        'tools/gen_patch_wikitext_register.py', 'tools/extract_patch_non_inventory.py',
        'tools/build_patch_3_1_0_accounting.py', 'tools/test_patch_3_1_0_source.py',
        'tests/patch_3_1_0_publication_sweep.rs', 'tests/common/publication_sweep.rs',
        'data/patch-api/sources/api-change-pages-remaining.json',
        'data/patch-api/source-cache/legacy-2026-10-09/manifest.json',
    }
    bundle = {path: (ROOT / path).read_text() for path in sorted(snapshot)}
    bundle['baseline/gen_patch_wikitext_register.py'] = git('show', BASE + ':tools/gen_patch_wikitext_register.py').decode()
    selected = ['tools/gen_patch_wikitext_register.py', 'tools/test_patch_3_1_0_source.py',
                'tools/build_patch_3_1_0_accounting.py', 'tests/patch_3_1_0_publication_sweep.rs',
                'tests/data/patch_3_1_0_sweep_known_gaps.json', 'tests/common/publication_sweep.rs',
                'data/patch-api/sources/3.1.0-api-changes.wikitext',
                'data/patch-api/sources/3.1.0-api-changes.provenance.json',
                'data/patch-api/sources/3.1.0-wikitext-register.json',
                'Cargo.toml', 'Cargo.lock', 'build.rs', '.cargo/config.toml']
    scopes = {}
    for revision in sorted({command['revision'] for command in commands}):
        scopes[revision] = capture_revision(revision, selected, bundle)
    bundle['proof-scopes.json'] = json.dumps(scopes, sort_keys=True)
    archive = gzip.compress(json.dumps(bundle, sort_keys=True, separators=(',', ':')).encode(), mtime=0)
    assert len(archive) <= 5_000_000, 'archive too large'
    (HERE / 'historical-bundle.json.gz').write_bytes(archive)
    exclude = {'pin-builder.py', 'validate.py', 'PLAN.md', 'historical-inputs.json'}
    sealed = {}
    for path in sorted(HERE.iterdir()):
        if path.is_file() and path.name not in exclude and not path.name.startswith('validator-'):
            raw = path.read_bytes()
            assert len(raw) <= 5_000_000, path.name
            sealed[path.name] = dict(sha256=digest(raw), bytes=len(raw))
    manifest = dict(schema='p310-historical-source-proof/v1', sealed_files=sealed,
                    snapshot_sha256={path: digest(content.encode()) for path, content in bundle.items()},
                    later_registers=later, pending_successors=['3.2.0', '3.3.0', '3.3.3', '3.3.5', '4.0.1'],
                    limits='Selected-source accounting and retained targeted command evidence only. No native/full runtime reconstruction, future closures, broad or coordinator acceptance.')
    write_json(HERE / 'historical-inputs.json', manifest)
    seal = digest((HERE / 'historical-inputs.json').read_bytes())
    print(json.dumps(dict(manifest_sha256=seal, archive_bytes=len(archive), archived_files=len(bundle), sealed_files=len(sealed), commands=len(commands), revisions=len(scopes))))


if __name__ == '__main__':
    main()
