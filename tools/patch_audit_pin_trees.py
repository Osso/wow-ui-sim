"""Validate compact rebase input trees without resolving pre-rebase commits.

Full rebased trees pin bytes, names and modes. A projected recorded tree preserves
exactly the original input set; retained historical blobs replace rebased exceptions.
Only hashes of recorded objects are needed, never the recorded commit or tree object.
"""
import gzip
import hashlib
import json
from pathlib import Path
import subprocess

_PROOFS = {}


def object_id(kind, content):
    header = kind.encode() + b' ' + str(len(content)).encode() + b'\0'
    return hashlib.sha1(header + content).hexdigest()


def tree_id(entries):
    """Compute Git's recursive tree hash from relative paths and mode/type/blob IDs."""
    children = {}
    for path, entry in entries.items():
        name, separator, rest = path.partition('/')
        if separator:
            children.setdefault(name, {})[rest] = entry
        else:
            children[name] = entry
    records = []
    for name, entry in children.items():
        if isinstance(entry, dict):
            mode, kind, oid = '40000', 'tree', tree_id(entry)
        else:
            mode, kind, oid = entry
        encoded_name = name.encode('utf-8', 'surrogateescape')
        order = encoded_name + (b'/' if kind == 'tree' else b'')
        record = mode.lstrip('0').encode() + b' ' + encoded_name + b'\0' + bytes.fromhex(oid)
        records.append((order, record))
    return object_id('tree', b''.join(record for _, record in sorted(records)))


def git(root, *args):
    proof = _PROOFS.get(str(root))
    if proof is not None:
        return proof(*args)
    return subprocess.check_output(['git', *args], cwd=root)


def directory_id(root, revision, directory):
    if not directory:
        return git(root, 'rev-parse', revision + '^{tree}').decode().strip()
    entries = git(root, 'ls-tree', '-z', revision, '--', directory).split(b'\0')
    if entries == [b'']:
        return None
    assert len(entries) == 2 and entries[1] == b'', directory
    metadata, name = entries[0].split(b'\t', 1)
    mode, kind, oid = metadata.split()
    assert kind == b'tree' and name.decode() == directory, directory
    return oid.decode()


def exclude_entries(entries, excluded, directory):
    assert len(excluded) == len(set(excluded)), directory
    for path in excluded:
        removed = [name for name in entries if name == path or name.startswith(path + '/')]
        assert removed, (directory, path)
        for name in removed:
            del entries[name]


def snapshot_entries(root, revision, directory, pin):
    """Rebuild recorded entries from reachable rebased content plus exact differences."""
    assert directory_id(root, revision, directory) == pin['rebased_tree'], directory
    target = revision + ':' + directory if directory else revision
    entries = {}
    if pin['rebased_tree'] is not None:
        for line in git(root, 'ls-tree', '-r', '-z', target).split(b'\0'):
            if line:
                metadata, path = line.split(b'\t', 1)
                entries[path.decode('utf-8', 'surrogateescape')] = tuple(metadata.decode().split())
    if 'suffix' in pin:
        entries = {name: entry for name, entry in entries.items() if name.endswith(pin['suffix'])}
    exclude_entries(entries, pin['excluded'], directory)
    entries.update({path: tuple(entry) for path, entry in pin['overrides'].items()})
    assert tree_id(entries) == pin['recorded_tree'], 'recorded snapshot: ' + directory
    return entries


def verify_scope_trees(root, scope):
    revision = scope['rebased_revision']
    for directory, pin in scope['inventories'].items():
        snapshot_entries(root, revision, directory, pin)
    for directory, pin in scope['trees'].items():
        assert directory_id(root, revision, directory) == pin['rebased_tree'], directory
        raw = git(root, 'ls-tree', '-r', '-z', revision + ':' + directory)
        entries = {}
        for line in raw.split(b'\0'):
            if not line:
                continue
            metadata, path = line.split(b'\t', 1)
            mode, kind, oid = metadata.decode().split()
            entries[path.decode('utf-8', 'surrogateescape')] = (mode, kind, oid)
        exclude_entries(entries, pin['excluded'], directory)
        for path, row in scope['inputs'].items():
            if path.startswith(directory + '/'):
                relative = path[len(directory) + 1:]
                mode, kind, oid = entries[relative]
                assert oid == row['rebased_blob'], path
                entries[relative] = (mode, kind, row['recorded_blob'])
        assert tree_id(entries) == pin['recorded_tree'], 'recorded input tree: ' + directory


def blob_digests(root, oids):
    """Hash each reachable blob once even when several recorded scopes share it."""
    oids = sorted(oids)
    data = ('\n'.join(oids) + '\n').encode()
    proof = _PROOFS.get(str(root))
    raw = (proof('cat-file', '--batch', input=data) if proof is not None else
           subprocess.check_output(['git', 'cat-file', '--batch'], cwd=root, input=data)) if oids else b''
    digests = {}
    offset = 0
    for oid in oids:
        end = raw.index(b'\n', offset)
        actual, kind, size = raw[offset:end].decode().split()
        assert (actual, kind) == (oid, 'blob'), oid
        start = end + 1
        offset = start + int(size)
        content = raw[start:offset]
        assert object_id('blob', content) == oid and raw[offset:offset + 1] == b'\n', oid
        digests[oid] = hashlib.sha256(content).hexdigest()
        offset += 1
    assert offset == len(raw)
    return digests


def expand_pinned_inputs(root, here, scopes):
    """Supply old replay interfaces from checked trees, not stored per-file lists."""
    mapping = json.loads((Path(here) / 'rebase-mapping.json').read_text()) if (Path(here) / 'rebase-mapping.json').exists() else {}
    _PROOFS[str(root)] = ProofGit(root, here, mapping)
    expanded = []
    for scope in scopes:
        verify_scope_trees(root, scope)
        row = dict(scope)
        row['inventories'] = {}
        for directory, pin in scope['inventories'].items():
            entries = snapshot_entries(root, scope['rebased_revision'], directory, pin)
            row['inventories'][directory] = sorted(directory + '/' + path for path in entries)
        row['inputs'] = dict(scope['inputs'])
        for directory, pin in scope['trees'].items():
            target = scope['rebased_revision'] + ':' + directory
            for line in git(root, 'ls-tree', '-r', '-z', target).split(b'\0'):
                if not line:
                    continue
                metadata, relative = line.split(b'\t', 1)
                relative = relative.decode('utf-8', 'surrogateescape')
                path = directory + '/' + relative
                if any(relative == name or relative.startswith(name + '/') for name in pin['excluded']) or path in row['inputs']:
                    continue
                oid = metadata.decode().split()[2]
                row['inputs'][path] = {'recorded_blob': oid, 'rebased_blob': oid}
        if 'tree_blobs' in scope:
            entries = snapshot_entries(root, scope['rebased_revision'], '', scope['tree_blobs'])
            row['tree_blobs'] = {path: entry[2] for path, entry in entries.items()}
        expanded.append(row)
    digests = blob_digests(root, {row['rebased_blob'] for scope in expanded
                                 for row in scope['inputs'].values() if 'sha256' not in row})
    for scope in expanded:
        for row in scope['inputs'].values():
            if 'sha256' not in row:
                row['sha256'] = digests[row['rebased_blob']]
    return expanded


def verify_pinned_inputs(root, here, scopes):
    for scope in scopes:
        verify_scope_trees(root, scope)
        revision = scope['rebased_revision']
        for path, row in scope['inputs'].items():
            actual = git(root, 'rev-parse', revision + ':' + path).decode().strip()
            assert actual == row['rebased_blob'], path
            content = ((Path(here) / row['historical_file']).read_bytes()
                       if 'historical_file' in row else git(root, 'show', revision + ':' + path))
            assert hashlib.sha256(content).hexdigest() == row['sha256'], path
            assert object_id('blob', content) == row['recorded_blob'], path


class ProofGit:
    """Read historical Git labels from verified trees rooted in master ancestry.

    Labels are metadata, not commit dependencies. No historical tree objects are
    opened: names/modes/IDs are reconstructed and checked against their tree hash.
    """
    def __init__(self, root, here, mapping):
        self.root = root
        self.here = Path(here)
        self.pins = mapping.get('revision_pins', {})
        self.archives = mapping.get('archived_blobs', {})
        self.cache = {}

    def raw(self, *args, input=None):
        return subprocess.check_output(['git', *args], cwd=self.root, input=input)

    def label(self, value):
        revision = value.split(':', 1)[0].split('^', 1)[0]
        matches = [label for label in self.pins if label.startswith(revision)]
        assert len(matches) <= 1, value
        return matches[0] if matches else None

    def entries(self, revision):
        label = self.label(revision)
        if label in self.cache:
            return self.cache[label]
        if label is None:
            result = {}
            for line in self.raw('ls-tree', '-r', '-z', revision).split(b'\0'):
                if line:
                    metadata, path = line.split(b'\t', 1)
                    result[path.decode()] = tuple(metadata.decode().split())
            return result
        pin = self.pins[label]
        self.raw('merge-base', '--is-ancestor', pin['revision'], 'HEAD')
        result = snapshot_entries(self.root, pin['revision'], '', pin['snapshot'])
        self.cache[label] = result
        return result

    def blob(self, oid):
        if oid in self.archives:
            content = gzip.decompress((self.here / self.archives[oid]).read_bytes())
        else:
            content = self.raw('cat-file', 'blob', oid)
        assert object_id('blob', content) == oid, oid
        return content

    def patch(self, label):
        pin = self.pins[label]
        self.entries(label)
        patch = self.raw('show', '--format=', '--binary', pin['revision'])
        actual = self.raw('patch-id', '--stable', input=patch).decode().split()[0]
        assert actual == pin['reachable_patch_id'], label
        if 'patch_file' in pin:
            patch = gzip.decompress((self.here / pin['patch_file']).read_bytes())
            assert hashlib.sha256(patch).hexdigest() == pin['patch_sha256'], label
        return patch

    def __call__(self, *args, input=None):
        if args == ('cat-file', '--batch') and self.archives:
            oids = input.decode().splitlines()
            reachable = [oid for oid in oids if oid not in self.archives]
            raw = self.raw('cat-file', '--batch', input=('\n'.join(reachable) + '\n').encode()) if reachable else b''
            records, offset = {}, 0
            for oid in reachable:
                start = offset
                end = raw.index(b'\n', start)
                actual, kind, size = raw[start:end].decode().split()
                assert (actual, kind) == (oid, 'blob'), oid
                offset = end + 1 + int(size) + 1
                records[oid] = raw[start:offset]
            assert offset == len(raw)
            for oid in set(oids) & self.archives.keys():
                content = self.blob(oid)
                records[oid] = f'{oid} blob {len(content)}\n'.encode() + content + b'\n'
            return b''.join(records[oid] for oid in oids)
        labels = [self.label(arg) for arg in args]
        if not any(labels):
            return self.raw(*args, input=input)
        if args[0] == 'rev-parse':
            value = args[-1]
            label = self.label(value)
            entries = self.entries(label)
            if ':' in value:
                path = value.split(':', 1)[1]
                if path in entries:
                    oid = entries[path][2]
                else:
                    children = {name[len(path)+1:]: entry for name, entry in entries.items()
                                if name.startswith(path + '/')}
                    assert children, path
                    oid = tree_id(children)
            elif value.endswith('^{tree}'):
                oid = self.pins[label]['snapshot']['recorded_tree']
            else:
                assert value == label or value.endswith('^{commit}') or label.startswith(value), value
                oid = label
            return (oid + '\n').encode()
        if args[0] == 'show':
            value = args[-1]
            label = self.label(value)
            if ':' in value:
                return self.blob(self.entries(label)[value.split(':', 1)[1]][2])
            if args[1:-1] == ('-s', '--format=%s'):
                self.entries(label)
                return self.raw('show', '-s', '--format=%s', self.pins[label]['revision'])
            assert args[1:-1] == ('--format=', '--binary'), args
            return self.patch(label)
        if args[0] == 'ls-tree':
            index = next(i for i, label in enumerate(labels) if label)
            entries = self.entries(args[index])
            if ':' in args[index]:
                directory = args[index].split(':', 1)[1]
                entries = {path[len(directory)+1:]: entry for path, entry in entries.items()
                           if path.startswith(directory + '/')}
            paths = [path for path in args[index+1:] if path != '--']
            selected = {path: entry for path, entry in entries.items()
                        if not paths or any(path == name or path.startswith(name + '/') for name in paths)}
            if '-r' not in args:
                assert len(paths) == 1, args
                path = paths[0]
                selected = ({path: ('040000', 'tree', tree_id({name[len(path)+1:]: entry for name, entry in selected.items()}))}
                            if selected else {})
            separator = b'\0' if '-z' in args else b'\n'
            records = [path.encode() if '--name-only' in args else
                       (' '.join(selected[path]) + '\t' + path).encode() for path in sorted(selected)]
            return separator.join(records) + (separator if records else b'')
        if args[0] == 'diff':
            index = 2 if args[1] == '--name-only' else 1
            before, after = self.entries(args[index]), self.entries(args[index+1])
            assert args[index+2] == '--', args
            paths = args[index+3:]
            changed = sorted(path for path in before.keys() | after.keys()
                             if before.get(path) != after.get(path) and
                             any(path == name or path.startswith(name + '/') for name in paths))
            # These validators use full diff only as an equality predicate.
            return ('\n'.join(changed) + ('\n' if changed else '')).encode()
        raise AssertionError('unsupported historical Git operation: ' + repr(args))
