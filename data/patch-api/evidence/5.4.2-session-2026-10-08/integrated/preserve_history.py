"""Preserve historical receipts and original input blobs before replaying rebased pins."""
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent
HISTORY = HERE.parent
MASTER = '896086537'
OLD_BASE = 'a9d7c9566'
OLD_TIP = '8691fe460'
REBASED_TIP = '833a7d8a8'


def git(*args, input=None):
    return subprocess.check_output(['git', *args], cwd=ROOT, input=input)


def digest(value):
    return hashlib.sha256(value).hexdigest()


def dump(name, value):
    (HERE / name).write_text(json.dumps(value, indent=2) + '\n')


def patch_id(revision):
    return git('patch-id', '--stable', input=git('show', '--format=', '--binary', revision)).decode().split()[0]


if __name__ == '__main__':
    preserved = {}
    for path in sorted(HISTORY.iterdir()):
        if not path.is_file():
            continue
        name = path.relative_to(ROOT).as_posix()
        content = git('show', REBASED_TIP + ':' + name)
        if path.name != 'validate.py':
            assert path.read_bytes() == content
        preserved[name] = digest(content)
        if path.name == 'validate.py':
            (HERE / 'historical-validator.py.txt').write_bytes(content)
    dump('historical-preservation.json', preserved)
    rebased = {}
    for line in git('log', MASTER + '..' + REBASED_TIP, '--format=%H %s').decode().splitlines():
        revision, subject = line.split(' ', 1)
        rebased[subject] = revision
    originals = git('rev-list', OLD_BASE + '..' + OLD_TIP).decode().splitlines()
    rows = []
    for original in originals:
        subject = git('show', '-s', '--format=%s', original).decode().strip()
        revision = rebased[subject]
        rows.append({'recorded_revision': original, 'rebased_revision': revision, 'subject': subject,
                     'recorded_patch_id': patch_id(original), 'rebased_patch_id': patch_id(revision),
                     'rebased_tree': git('rev-parse', revision + '^{tree}').decode().strip()})
    pins = set()
    def collect(value):
        if isinstance(value, dict):
            for key, item in value.items():
                if key.endswith('revision') and isinstance(item, str):
                    pins.add(git('rev-parse', item).decode().strip())
                collect(item)
        elif isinstance(value, list):
            for item in value:
                collect(item)
    for path in HISTORY.glob('*.json'):
        if 'validator-gate' not in path.name:
            collect(json.loads(path.read_text()))
    external = []
    mapped = {row['recorded_revision']: row['rebased_revision'] for row in rows}
    master_subjects = {}
    for line in git('log', MASTER, '--format=%H %s').decode().splitlines():
        revision, subject = line.split(' ', 1)
        master_subjects.setdefault(subject, []).append(revision)
    for original in sorted(pins - set(mapped)):
        if subprocess.run(['git', 'merge-base', '--is-ancestor', original, MASTER], cwd=ROOT).returncode == 0:
            continue
        subject = git('show', '-s', '--format=%s', original).decode().strip()
        candidates = master_subjects.get(subject, [])
        assert len(candidates) == 1, (original, subject, candidates)
        revision = candidates[0]
        mapped[original] = revision
        external.append({'recorded_revision': original, 'rebased_revision': revision, 'subject': subject,
                         'recorded_patch_id': patch_id(original), 'rebased_patch_id': patch_id(revision),
                         'rebased_tree': git('rev-parse', revision + '^{tree}').decode().strip()})
    blobs = HERE / 'historical-blobs'
    blobs.mkdir(exist_ok=True)
    scopes = []
    contents = {}
    def tree(revision):
        return dict(line.split('\t', 1)[::-1] for line in git('ls-tree', '-r', revision).decode().splitlines())
    def load_blobs(ids):
        missing = sorted(set(ids) - set(contents))
        if not missing:
            return
        output = git('cat-file', '--batch', input=('\n'.join(missing) + '\n').encode())
        offset = 0
        for oid in missing:
            end = output.index(b'\n', offset)
            header = output[offset:end].decode().split()
            assert header[:2] == [oid, 'blob'], header
            size = int(header[2])
            offset = end + 1
            contents[oid] = output[offset:offset + size]
            offset += size + 1
    for original, revision in sorted(mapped.items()):
        inventories = {directory: git('ls-tree', '-r', '--name-only', original, directory).decode().splitlines()
                       for directory in ['data/patch-api/sources', 'tests', 'tools', 'data/patch-api/evidence', 'data/patch-api/evidence/5.4.2-session-2026-10-08']}
        selected = set(inventories['data/patch-api/sources'])
        selected.update(inventories['tests'])
        selected.update(git('ls-tree', '-r', '--name-only', original, 'src', 'crates', 'tools', 'Interface', 'Cargo.toml', 'Cargo.lock', 'build.rs').decode().splitlines())
        selected.update(['tools/gen_patch_wikitext_register.py', 'tools/extract_patch_non_inventory.py',
                         'tests/common/publication_sweep.rs', 'docs/wiki/index.md', 'docs/wiki/log.md'])
        old_tree, new_tree = tree(original), tree(revision)
        old_ids = {name: old_tree[name].split()[2] for name in selected}
        new_ids = {name: new_tree[name].split()[2] if name in new_tree else None for name in selected}
        load_blobs(old_ids.values())
        inputs = {}
        for name in sorted(selected):
            old = contents[old_ids[name]]
            row = {'recorded_blob': old_ids[name], 'rebased_blob': new_ids[name], 'sha256': digest(old)}
            if old_ids[name] != new_ids[name]:
                output = blobs / (row['recorded_blob'] + '.txt')
                output.write_bytes(old)
                row['historical_file'] = output.relative_to(HERE).as_posix()
            inputs[name] = row
        scopes.append({'recorded_revision': original, 'rebased_revision': revision,
                       'inventories': inventories, 'tree_blobs': {name: metadata.split()[2] for name, metadata in old_tree.items()}, 'inputs': inputs})
    for row in rows + external:
        if row['recorded_patch_id'] != row['rebased_patch_id']:
            name = 'historical-patch-' + row['recorded_revision'] + '.txt'
            (HERE / name).write_bytes(git('show', '--format=', '--binary', row['recorded_revision']))
            row['historical_patch'] = name
            row['conflict_reason'] = {'Account 5.4.2 gaps and prove bounded current roster and enum behavior': 'Wiki index/log insertions moved below newly merged 5.4.7/5.5.x headings; historical wiki blobs preserve original ordering. No behavior change.', 'Pin retail 5.4.2 source and parse Mists automated inventories': 'Opt-in Mists extractor argument and CLI forwarding coexist with canonical-patch-navigation; generator insertion context includes combat-restriction-bullets and client-line. Original parser blobs are preserved for historical replay.'}[row['subject']]
    dump('rebase-mapping.json', {'original_base': git('rev-parse', OLD_BASE).decode().strip(),
                               'rebased_base': git('rev-parse', MASTER).decode().strip(),
                               'commits': rows, 'external_commits': external, 'pinned_inputs': scopes})
    print(json.dumps({'mapped_commits': len(rows), 'external_commits': len(external), 'historical_files': len(preserved)}))
