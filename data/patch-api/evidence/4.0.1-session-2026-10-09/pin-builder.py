"""Build compact self-contained historical proof inputs; never pin later state."""
import base64
import gzip
import hashlib
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
SOURCE_REVISION = '567654d75'


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT)


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def write_archive(name, value):
    raw = json.dumps(value, sort_keys=True, separators=(',', ':')).encode()
    data = gzip.compress(raw, mtime=0)
    assert len(data) < 5_000_000, name
    (HERE / name).write_bytes(data)
    return digest(data)


def main():
    evidence = sorted(path for path in HERE.iterdir() if path.is_file())
    exclude = {'pin-builder.py', 'validate.py', 'historical-context.json',
               'historical-inputs.json.gz', 'historical-scopes.json.gz'}
    seals = {path.name: digest(path.read_bytes()) for path in evidence
             if path.name not in exclude and not path.name.startswith('validator-')}
    receipts = {path.stem.removesuffix('.proof'): json.loads(path.read_text())
                for path in evidence if path.name.endswith('.proof.json')
                and not path.name.startswith('validator-')}
    revisions = {row['revision'] for row in receipts.values()} | {SOURCE_REVISION}
    scopes, trees = {}, {}
    for revision in sorted(revisions):
        commit = git('cat-file', 'commit', revision)
        root = []
        for record in git('ls-tree', '-z', revision).split(b'\0'):
            if record:
                metadata, name = record.split(b'\t')
                root.append([name.decode(), *metadata.decode().split()])
        scope = {'commit': base64.b64encode(commit).decode(),
                 'commit_id': git('rev-parse', revision).decode().strip(), 'root': root,
                 'scope_trees': {}}
        for directory in ('src', 'tests', 'tools', 'data', '.cargo'):
            oid = git('rev-parse', revision + ':' + directory).decode().strip()
            scope['scope_trees'][directory] = oid
            if oid not in trees:
                rows = []
                for record in git('ls-tree', '-r', '-z', revision + ':' + directory).split(b'\0'):
                    if record:
                        metadata, name = record.split(b'\t')
                        rows.append([name.decode(), *metadata.decode().split()])
                trees[oid] = rows
        scopes[revision] = scope
    scope_digest = write_archive('historical-scopes.json.gz', {'revisions': scopes, 'trees': trees})
    own_sweep = git('show', SOURCE_REVISION + ':tests/patch_4_0_1_publication_sweep.rs').decode()
    paths = set(re.findall(r'include_str!\("\.\./([^"\n]+)"\)', own_sweep))
    paths.update('data/patch-api/sources/4.0.1-' + suffix for suffix in (
        'api-changes.wikitext', 'api-changes.provenance.json', 'api-changes.txt',
        'wikitext-register.json', 'page-coverage.json'))
    paths.update(('tests/patch_4_0_1_publication_sweep.rs', 'tests/patch_4_0_1_retirements.rs',
                  'tests/data/patch_4_0_1_sweep_known_gaps.json', 'tests/common/publication_sweep.rs',
                  'tools/gen_patch_wikitext_register.py', 'tools/extract_patch_non_inventory.py',
                  'tools/test_patch_cataclysm_register.py', 'tools/patch_audit_pin_trees.py',
                  'src/c_api/patch_retired_members.rs', 'src/c_api/mod.rs',
                  'src/lua_api/globals/stubs/global_stubs.rs',
                  'Cargo.toml', 'Cargo.lock', 'build.rs', '.cargo/config.toml'))
    inputs = {}
    for path in sorted(paths):
        raw = git('show', SOURCE_REVISION + ':' + path)
        inputs[path] = {'content': base64.b64encode(raw).decode(), 'sha256': digest(raw),
                        'blob_id': git('rev-parse', SOURCE_REVISION + ':' + path).decode().strip()}
    input_digest = write_archive('historical-inputs.json.gz', inputs)
    context = {'schema': 'p401-historical-proof/v1', 'source_revision': SOURCE_REVISION,
               'source_revid': 1271877, 'pageid': 129645,
               'validator_sha256': digest((HERE / 'validate.py').read_bytes()),
               'archives': {'historical-inputs.json.gz': input_digest,
                            'historical-scopes.json.gz': scope_digest},
               'evidence_sha256': seals, 'proofs': receipts,
               'retired_globals': ['CollapseSkillHeader', 'ExpandSkillHeader'],
               'pending_successors': ['4.1.0', '4.2.0', '4.3.0', '4.3.4'],
               'policy': 'Historical source/input integrity and recorded targeted proof only; no current-head, native, broad-gate or full runtime replay claim.'}
    (HERE / 'historical-context.json').write_text(json.dumps(context, indent=2) + '\n')
    print(json.dumps({'inputs': len(inputs), 'evidence_seals': len(seals),
                      'scope_revisions': len(scopes), 'archive_bytes': sum(
                          (HERE / name).stat().st_size for name in context['archives'])}))


if __name__ == '__main__':
    main()
