"""Record original-to-rebased patch IDs and preserve historical shared input blobs."""
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent
HISTORY = HERE.parent
BASE = 'e958b7d38'
MASTER = 'd0fabed03'
OLD_TIP = 'cfee5d4ce'
REBASED_TIP = '08de02e35'


def git(*args, input=None):
    return subprocess.check_output(['git', *args], cwd=ROOT, input=input)


def dump(name, value):
    (HERE / name).write_text(json.dumps(value, indent=2) + '\n')


def digest(value):
    return hashlib.sha256(value).hexdigest()


def patch_id(revision):
    patch = git('show', '--format=', '--binary', revision)
    return git('patch-id', '--stable', input=patch).decode().split()[0]


def main():
    preserved = {}
    for path in sorted(HISTORY.iterdir()):
        if not path.is_file():
            continue
        name = path.relative_to(ROOT).as_posix()
        content = git('show', REBASED_TIP + ':' + name)
        if path.name != 'validate.py':
            assert path.read_bytes() == content, name
        preserved[name] = digest(content)
        if path.name == 'validate.py':
            (HERE / 'historical-validator.py.txt').write_bytes(content)
    dump('historical-preservation.json', preserved)
    rebased = {}
    for revision in git('rev-list', MASTER + '..' + REBASED_TIP).decode().splitlines():
        subject = git('show', '-s', '--format=%s', revision).decode().strip()
        rebased[subject] = revision
    rows = []
    originals = git('rev-list', BASE + '..' + OLD_TIP).decode().splitlines()
    for original in originals:
        subject = git('show', '-s', '--format=%s', original).decode().strip()
        revision = rebased[subject]
        rows.append({'recorded_revision': original, 'rebased_revision': revision,
                     'subject': subject, 'recorded_patch_id': patch_id(original),
                     'rebased_patch_id': patch_id(revision),
                     'rebased_tree': git('rev-parse', revision + '^{tree}').decode().strip()})
    # Runtime/source/test/tool pins in historical receipts retain their exact old bytes.
    pins = {}
    for path in sorted(HISTORY.glob('*.json')):
        value = json.loads(path.read_text())
        def collect(value):
            if isinstance(value, dict):
                for key, item in value.items():
                    if key.endswith('revision') and isinstance(item, str):
                        pins[item] = None
                    collect(item)
            elif isinstance(value, list):
                for item in value:
                    collect(item)
        collect(value)
    blobs = HERE / 'historical-blobs'
    blobs.mkdir(exist_ok=True)
    by_original = {row['recorded_revision']: row['rebased_revision'] for row in rows}
    scopes = []
    external = []
    master_subjects = {}
    for line in git('log', '--format=%H %s', MASTER).decode().splitlines():
        commit, subject = line.split(' ', 1)
        master_subjects.setdefault(subject, []).append(commit)
    later_scans = json.loads((HISTORY / 'p602-later-register-scan.json').read_text())
    for pin in sorted(pins):
        original = git('rev-parse', pin).decode().strip()
        own = original in by_original
        if not own:
            if subprocess.run(['git', 'merge-base', '--is-ancestor', original, MASTER], cwd=ROOT).returncode == 0:
                continue
            subject = git('show', '-s', '--format=%s', original).decode().strip()
            candidates = master_subjects[subject]
            assert len(candidates) == 1, (pin, candidates)
            revision = candidates[0]
            external.append({'recorded_revision': original, 'rebased_revision': revision,
                             'subject': subject, 'recorded_patch_id': patch_id(original),
                             'rebased_patch_id': patch_id(revision),
                             'rebased_tree': git('rev-parse', revision + '^{tree}').decode().strip()})
            paths = sorted({row['path'] for row in later_scans if row['revision'] == pin})
        else:
            revision = by_original[original]
            paths = git('ls-tree', '-r', '--name-only', original, 'data/patch-api/sources', 'tests', 'tools', 'src/c_api', 'data/patch-api/evidence').decode().splitlines()
        inventories = {directory: git('ls-tree', '-r', '--name-only', original, directory).decode().splitlines()
                       for directory in ('data/patch-api/sources', 'tests', 'data/patch-api/evidence')}
        inputs = {}
        for name in paths:
            if not (name.startswith('data/patch-api/sources/') or
                    name in ('tools/gen_patch_wikitext_register.py', 'tools/extract_patch_non_inventory.py',
                             'tests/common/publication_sweep.rs', 'src/c_api/c_scenario_bonus.rs',
                             'src/c_api/patch_retired_members.rs') or
                    (name.startswith('data/patch-api/evidence/') and name.endswith('/validate.py')) or
                    (name.startswith('tests/') and (name.endswith('_publication_sweep.rs') or 'patch_6_0_2' in name))):
                continue
            old = git('show', original + ':' + name)
            new = git('show', revision + ':' + name)
            row = {'recorded_blob': git('rev-parse', original + ':' + name).decode().strip(),
                   'rebased_blob': git('rev-parse', revision + ':' + name).decode().strip(),
                   'sha256': digest(old)}
            if old != new:
                output = blobs / (row['recorded_blob'] + '.txt')
                output.write_bytes(old)
                row['historical_file'] = output.relative_to(HERE).as_posix()
            inputs[name] = row
        scopes.append({'recorded_revision': original, 'rebased_revision': revision,
                       'inventories': inventories, 'inputs': inputs})
    dump('rebase-mapping.json', {'original_base': git('rev-parse', BASE).decode().strip(),
                               'rebased_base': git('rev-parse', MASTER).decode().strip(),
                               'commits': rows, 'external_commits': external, 'pinned_inputs': scopes})
    print(json.dumps({'mapped_commits': len(rows), 'pinned_revisions': len(scopes),
                      'historical_files': len(preserved)}))


if __name__ == '__main__':
    main()
