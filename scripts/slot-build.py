#!/usr/bin/env python3
"""Launch an admitted Cargo build through a stable per-slot source mount."""
import argparse
from contextlib import contextmanager, ExitStack, redirect_stdout
import fcntl
import hashlib
import json
import os
import re
import shutil
import math
import stat
from pathlib import Path
import subprocess
import sys
import time
import tomllib


TARGET_ROOT = Path("/home/osso/.cache/wow-ui-sim-targets")
SOURCE_ROOT = Path("/home/osso/.cache/wow-ui-sim-srcmount")
# Slot 2 was retired on 2026-10-27 (user decision: three build slots).
SLOTS = (0, 1, 3)
CAPACITY = "/home/osso/.worktrees/logs/capacity.sh"
LIBRARY_SUFFIXES = {".rlib", ".rmeta", ".so", ".a", ".dll", ".dylib"}


def check_owner(slot, owner):
    if (slot / ".owner").read_text() != owner:
        raise ValueError(f"slot owner mismatch: {slot / '.owner'}")


def disk_inventory(slot):
    """Allocated bytes and mtimes, without following symlinks or double-counting links."""
    entries = {}
    for directory, children, files in os.walk(slot):
        for name in children + files:
            path = Path(directory) / name
            stat = path.lstat()
            entries[path] = stat
    return entries


def artifact_units(slot, entries):
    # Fingerprint package names can differ from bin/test names. Join aliases
    # only at the same hash; incremental's independent hashes remain separate.
    parents, members = {}, {}

    def root(key):
        parents.setdefault(key, key)
        if parents[key] != key:
            parents[key] = root(parents[key])
        return parents[key]

    def alias(base, name, artifact_hash):
        key = (base, name.replace("-", "_"), artifact_hash)
        return root(key)

    prefixes = ("test-integration-test-", "test-example-", "test-bench-",
                "test-lib-", "test-bin-", "run-build-script-", "build-script-",
                "example-", "lib-", "bin-")
    for path in entries:
        if path.parent.name not in {".fingerprint", "deps", "build", "incremental"}:
            continue
        kind = path.parent.name
        match = re.fullmatch(r"(.+)-([a-z0-9]+)(?:\..*)?", path.name)
        if not match:
            continue
        name = match[1]
        library_output = path.suffixes and path.suffixes[0] in LIBRARY_SUFFIXES
        if kind == "deps" and library_output and name.startswith("lib"):
            name = name[3:]
        base = path.parent.parent
        key = alias(base, name, match[2])
        if kind == ".fingerprint" and not path.is_symlink():
            for marker in path.iterdir():
                marker_name = marker.name.removeprefix("dep-").removesuffix(".json")
                for prefix in prefixes:
                    if marker_name.startswith(prefix):
                        other = alias(base, marker_name[len(prefix):], match[2])
                        parents[root(other)] = root(key)
                        break
        members[path] = key
    units = {}
    for path, key in members.items():
        units.setdefault(root(key), []).append(path)
    return list(units.values())


def previous_artifact_paths(slot):
    receipt = slot / ".verified-source-identity.json"
    if not receipt.exists():
        return set()
    previous = json.loads(receipt.read_text())
    paths = {slot / name for name in previous.get("artifacts", {})}
    for record in previous.get("reported_artifacts", []):
        paths.update(Path(name) for name in
                     [record.get("executable"), *record.get("filenames", [])] if name)
    return paths


def profile_lock_paths(slot):
    paths = set()
    for directory, children, files in os.walk(slot):
        if ".cargo-lock" in files or set(children) & {
                ".fingerprint", "deps", "build", "incremental"}:
            paths.add(Path(directory) / ".cargo-lock")
    return paths


@contextmanager
def hold_pruning_locks(slot, owner, dry_run):
    with ExitStack() as locks:
        if not dry_run:
            check_owner(slot, owner)
            for path in sorted(profile_lock_paths(slot)):
                stream = locks.enter_context(path.open("a+b"))
                try:
                    fcntl.flock(stream, fcntl.LOCK_EX | fcntl.LOCK_NB)
                except BlockingIOError as error:
                    raise ValueError(f"slot build lock busy: {path}") from error
        yield


def receipt_artifact_identities(paths):
    identities = set()
    for path in paths:
        try:
            info = path.lstat()
        except FileNotFoundError:
            # A removed receipt output has no current inode to protect.
            continue
        identities.add((info.st_dev, info.st_ino))
    return identities


def prune_slot(slot, owner, cap_bytes, recent_hours=2, dry_run=False, now=None):
    with hold_pruning_locks(slot, owner, dry_run):
        return prune_locked_slot(slot, owner, cap_bytes, recent_hours, dry_run, now)


def prune_locked_slot(slot, owner, cap_bytes, recent_hours, dry_run, now):
    if cap_bytes <= 0 or not math.isfinite(cap_bytes):
        raise ValueError("slot cap must be positive and finite")
    if recent_hours < 0 or not math.isfinite(recent_hours):
        raise ValueError("recent hours must be nonnegative and finite")
    if not dry_run:
        check_owner(slot, owner)
    entries = disk_inventory(slot)
    inodes = {(stat.st_dev, stat.st_ino): stat.st_blocks * 512
              for stat in entries.values()}
    total = sum(inodes.values())
    result = {"slot": str(slot), "dry_run": dry_run, "total_bytes": total,
              "remaining_bytes": total, "watermark_bytes": int(cap_bytes * .8),
              "removed": []}
    if total <= cap_bytes:
        print(json.dumps(result), flush=True)
        return result
    protected = previous_artifact_paths(slot)
    protected_inodes = receipt_artifact_identities(protected)
    cutoff = (time.time() if now is None else now) - recent_hours * 3600
    candidates = []
    units = artifact_units(slot, entries)
    membership = {path: index for index, paths in enumerate(units) for path in paths}
    contents = [{} for _ in units]
    for path, info in entries.items():
        ancestor = path
        while ancestor != slot and ancestor not in membership:
            ancestor = ancestor.parent
        if ancestor in membership:
            contents[membership[ancestor]][path] = info
    for paths, covered in zip(units, contents):
        newest = max(stat.st_mtime for stat in covered.values())
        identities = {(stat.st_dev, stat.st_ino) for stat in covered.values()}
        if newest >= cutoff or protected.intersection(covered) or identities & protected_inodes:
            continue
        # Only count blocks that disappear: uplifted binaries can have hard links
        # outside the unit (or even outside this slot).
        links, metadata = {}, {}
        for info in covered.values():
            identity = (info.st_dev, info.st_ino)
            links[identity] = links.get(identity, 0) + 1
            metadata[identity] = info
        freed = sum(inodes[identity] for identity, count in links.items()
                    if count >= metadata[identity].st_nlink
                    or stat.S_ISDIR(metadata[identity].st_mode))
        candidates.append((newest, sorted(paths), freed))
    for _, paths, freed in sorted(candidates):
        if result["remaining_bytes"] < result["watermark_bytes"]:
            break
        manifest = {"paths": [str(path) for path in paths], "size_bytes": freed}
        if not dry_run:
            check_owner(slot, owner)
            # Cargo profile locks remain held through the whole-unit deletion.
            for path in paths:
                if path.is_dir() and not path.is_symlink():
                    shutil.rmtree(path)
                else:
                    path.unlink()
        result["removed"].append(manifest)
        result["remaining_bytes"] -= freed
        print(json.dumps(dict(manifest, dry_run=dry_run)), flush=True)
    print(json.dumps(result), flush=True)
    return result


def check_capacity(capacity_id):
    env = dict(os.environ, CAPACITY_ID=capacity_id)
    env.pop("CAPACITY_PEEK", None)
    result = subprocess.run([CAPACITY], env=env, check=True,
                            capture_output=True, text=True)
    verdict = result.stdout.strip().rsplit("|", 1)[-1].strip()
    if verdict != "ADD" and not verdict.startswith("ADD:"):
        raise ValueError(f"capacity refused: {result.stdout.strip()}")


def build_argv(slot, worktree, unit, cargo):
    target = TARGET_ROOT / f"slot-{slot}"
    # Incremental reuse requires this fixed BindPaths source path per target slot.
    mount = SOURCE_ROOT / f"slot-{slot}"
    return [
        "/usr/bin/systemd-run", "--user", "--no-block", "--remain-after-exit",
        f"--unit={unit}", "--slice=agents.slice",
        "--property=MemoryMax=9G", "--property=MemorySwapMax=0",
        "--property=CPUAccounting=yes",
        "--setenv=CARGO_INCREMENTAL=1", "--setenv=RUSTC_WRAPPER=",
        f"--setenv=CARGO_TARGET_DIR={target}",
        f"--property=BindPaths={worktree}:{mount}",
        f"--working-directory={mount}", "--", *cargo,
    ]


def git(worktree, *args):
    return subprocess.check_output(["git", "-C", str(worktree), *args])


def source_identity(worktree):
    head = git(worktree, "rev-parse", "HEAD").decode().strip()
    diff = git(worktree, "diff", "--binary", "HEAD", "--")
    dirty = set(os.fsdecode(p) for p in git(
        worktree, "diff", "--name-only", "-z", "HEAD", "--").split(b"\0") if p)
    untracked = git(worktree, "ls-files", "--others", "--exclude-standard", "-z")
    digest = hashlib.sha256(diff)
    for path in sorted(p for p in untracked.split(b"\0") if p):
        name = os.fsdecode(path)
        dirty.add(name)
        file = worktree / name
        content = os.fsencode(os.readlink(file)) if file.is_symlink() else file.read_bytes()
        digest.update(path + b"\0" + hashlib.sha256(content).digest())
    return {"head": head, "dirty_sha256": digest.hexdigest()}, sorted(dirty)


def full_source_inputs(worktree):
    tracked = {os.fsdecode(p) for p in git(worktree, "ls-files", "-z").split(b"\0") if p}
    result = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1", "--offline"],
        cwd=worktree, capture_output=True, check=False)
    if result.returncode:
        return tracked
    metadata = json.loads(result.stdout)
    roots = set()
    for package in metadata["packages"]:
        roots.add(Path(package["manifest_path"]).parent)
        roots.update(Path(dep["path"]) for dep in package["dependencies"] if dep.get("path"))
    # The root package covers build.rs's build/, installer/, native/intl/ and
    # Cargo.lock, plus test-harness/build.rs's tests reads, without a whitelist.
    inputs = {"build.rs", "Cargo.toml", "Cargo.lock"} & tracked
    inputs.update(name for name in tracked
                  if any((worktree / name).is_relative_to(root) for root in roots))
    return inputs


def read_content_changes(worktree, identity, dirty, previous):
    if previous and identity == previous.get("identity"):
        return set()
    try:
        if not previous:
            raise ValueError("no previous source")
        paths = git(worktree, "diff", "--name-only", "--no-renames", "-z",
                    previous["identity"]["head"], identity["head"], "--")
        changed = {os.fsdecode(p) for p in paths.split(b"\0") if p}
        changed.update(previous["dirty_paths"])
    except (subprocess.CalledProcessError, KeyError, ValueError):
        changed = full_source_inputs(worktree)
    return changed.union(dirty)


def prepare_source(worktree, previous, attempted=None):
    identity, dirty = source_identity(worktree)
    content_changed = read_content_changes(worktree, identity, dirty, previous)
    changed = set(content_changed)
    last_attempt = attempted if attempted is not None else previous
    retry = bool(last_attempt and last_attempt.get("result", "success") != "success")
    if not previous or identity != previous.get("identity") or retry:
        if retry:
            changed.update(full_source_inputs(worktree))
        changed.update(dirty)
        now = time.time_ns()
        for name in sorted(changed):
            path = worktree / name
            if path.is_file() and not path.is_symlink():
                os.utime(path, ns=(path.stat().st_atime_ns, now))
    return {"identity": identity, "dirty_paths": dirty,
            "changed_paths": sorted(changed),
            "content_changed_paths": sorted(content_changed)}


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix(".tmp")
    temporary.write_text(json.dumps(value, indent=2) + "\n")
    temporary.replace(path)


def summarize_receipt(current):
    """Receipts keep a certificate count and digest; slot identity files keep the full map."""
    summary = dict(current)
    certificates = summary.pop("artifact_certificates", {})
    summary["artifact_certificate_count"] = len(certificates)
    summary["artifact_certificates_sha256"] = hashlib.sha256(
        json.dumps(certificates, sort_keys=True).encode()).hexdigest()
    return summary


def read_package_roots(worktree):
    result = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1", "--offline"],
        cwd=worktree, capture_output=True, check=False)
    if result.returncode:
        # Match the existing all-tracked-input policy when metadata is unavailable.
        return {None: [str(worktree)]}
    packages = json.loads(result.stdout)["packages"]
    return {package["id"]: [str(Path(package["manifest_path"]).parent),
                           *(dep["path"] for dep in package["dependencies"]
                             if dep.get("path"))]
            for package in packages}


def package_inputs_changed(current, roots):
    worktree = Path(current["worktree"])
    return any((worktree / name).is_relative_to(root)
               for name in current["changed_paths"] for root in roots)


def dep_info_words(value):
    words = re.findall(r"(?:\\.|[^\s\\])+", value)
    return [re.sub(r"\\([ #\\:])", r"\1", word).replace("$$", "$")
            for word in words]


def parse_dep_info(content, worktree):
    outputs, inputs = set(), set()
    for line in content.replace("\\\n", " ").splitlines():
        target, separator, sources = line.partition(": ")
        if not separator or not sources.strip():
            continue
        outputs.update(os.path.realpath(worktree / word, strict=False)
                       for word in dep_info_words(target))
        # Keep symlink-sensitive /.. traversal until classification and existence checks.
        inputs.update(str(worktree / word) for word in dep_info_words(sources))
    if not outputs or not inputs:
        raise ValueError("no dependency rules")
    return {"outputs": sorted(outputs), "inputs": sorted(inputs)}


def read_dep_info_snapshot(slot, worktree):
    files, executables, identities = {}, [], {}
    for directory, children, names in os.walk(slot):
        children[:] = [name for name in children
                       if not name.startswith(".") and name not in {"incremental", "receipts"}]
        for name in names:
            path = Path(directory) / name
            if path.is_file() and (path.stat().st_mode & 0o111 or path.suffix == ".exe"):
                executables.append(str(path))
                stat = path.stat()
                identities.setdefault((stat.st_dev, stat.st_ino), []).append(str(path))
            if path.suffix != ".d":
                continue
            try:
                files[str(path)] = parse_dep_info(path.read_text(), worktree)
            except (OSError, UnicodeError, ValueError) as error:
                files[str(path)] = {"error": str(error)}
    aliases = {path: paths for paths in identities.values() for path in paths}
    return {"files": files, "executables": executables, "aliases": aliases}


def target_dep_inputs(record, snapshot, allow_missing=False):
    artifacts = {os.path.realpath(path, strict=False) for path in
                 [record.get("executable"), *record.get("filenames", [])] if path}
    # Cargo may report only the uplifted bin, while rustc writes deps/<name>-<hash>.d.
    # Snapshot hard-link identities bind that bin to the correct metadata hash.
    aliases = snapshot.get("aliases", {})
    artifacts.update(alias for path in list(artifacts) for alias in aliases.get(path, []))
    candidates = {str(Path(path).with_suffix(".d")) for path in artifacts}
    matches = {name: info for name, info in snapshot["files"].items()
               if name in candidates or artifacts.intersection(info.get("outputs", []))}
    rustc_matches = {name: info for name, info in matches.items()
                     if Path(name).parent.name == "deps"}
    if rustc_matches:
        matches = rustc_matches
    if not matches and not allow_missing:
        raise ValueError(f"missing dep-info for {artifact_path(record)}")
    inputs = set()
    for name, info in matches.items():
        if "error" in info:
            raise ValueError(f"unreadable dep-info {name}: {info['error']}")
        inputs.update(info["inputs"])
    return inputs


def package_build_inputs(current, record):
    manifest = record.get("manifest_path")
    roots = current["package_roots"].get(record.get("package_id"), [])
    package = Path(current["worktree"])
    if manifest:
        package = Path(manifest).parent
    elif roots:
        package = Path(roots[0])
    inputs = {str(package / name) for name in ("build.rs", "Cargo.toml", "Cargo.lock")}
    inputs.add(str(Path(current["worktree"]) / "Cargo.lock"))
    return inputs


def repo_relative_inputs(current, inputs):
    worktree = Path(current["worktree"])
    mount = SOURCE_ROOT / Path(current["slot"]).name
    utility_mount = SOURCE_ROOT / f"ct-bins-{Path(current['slot']).name}"
    roots = [Path(os.path.realpath(root, strict=False))
             for root in (mount, utility_mount, worktree)]
    relative = set()
    for name in inputs:
        path = Path(os.path.realpath(worktree / name, strict=False))
        for root in roots:
            if path.is_relative_to(root):
                relative.add(str(path.relative_to(root)))
                break
    return relative


def generated_input_paths(current, inputs):
    slot = Path(os.path.realpath(current["slot"], strict=False))
    generated = set()
    for name in inputs:
        path = Path(os.path.realpath(Path(current["worktree"]) / name, strict=False))
        if path.is_relative_to(slot) and not repo_relative_inputs(current, [str(path)]):
            generated.add(str(path))
    return generated


def artifact_path(record):
    if record.get("executable"):
        return record["executable"]
    filenames = record.get("filenames", [])
    return filenames[0] if filenames else record.get("target", {}).get("name", "unknown")


def is_repo_artifact(current, record):
    if record.get("package_id", "").startswith(("registry+", "git+")):
        return False
    paths = [path for path in [record.get("manifest_path"),
                              record.get("target", {}).get("src_path")] if path]
    # Records without ownership metadata cannot prove that they are external.
    return not paths or bool(repo_relative_inputs(current, paths))


def read_record_inputs(current, record, before, after):
    compiler_inputs = target_dep_inputs(record, after)
    inputs = repo_relative_inputs(current, compiler_inputs)
    if not inputs:
        raise ValueError(f"zero repo-relative inputs for {artifact_path(record)}")
    previous = target_dep_inputs(record, before, allow_missing=record.get("fresh") is False)
    inputs.update(repo_relative_inputs(current, previous))
    inputs.update(generated_input_paths(current, previous | compiler_inputs))
    inputs.update(repo_relative_inputs(current, package_build_inputs(current, record)))
    return sorted(inputs)


def read_target_inputs(current, before, after):
    targets, errors = {}, []
    for record in current["reported_artifacts"]:
        if not is_repo_artifact(current, record):
            continue
        try:
            targets[artifact_path(record)] = read_record_inputs(current, record, before, after)
        except ValueError as error:
            errors.append(str(error))
    return targets, errors


def input_content_digest(path, cache, ancestors=frozenset()):
    # Resolve before cache lookup: missing ancestors and symlink-sensitive /..
    # spellings must still fail, even if the resolved input was already hashed.
    resolved = Path(os.path.realpath(path, strict=True))
    if resolved in ancestors:
        raise ValueError(f"cyclic directory input: {path}")
    if resolved in cache:
        return cache[resolved]
    if path.is_dir():
        digest = hashlib.sha256(b"directory\0")
        for child in sorted(path.iterdir(), key=lambda child: os.fsencode(child.name)):
            name = os.fsencode(child.name)
            content = input_content_digest(child, cache, ancestors | {resolved})
            digest.update(len(name).to_bytes(8, "big") + name + content.encode("ascii") + b"\0")
        value = "tree-sha256:" + digest.hexdigest()
    else:
        with path.open("rb") as stream:
            value = hashlib.file_digest(stream, "sha256").hexdigest()
    cache[resolved] = value
    return value


def read_input_hashes(worktree, names):
    hashes, cache = {}, {}
    for name in sorted(names):
        try:
            hashes[name] = input_content_digest(worktree / name, cache)
        except FileNotFoundError:
            hashes[name] = None
    return hashes


def snapshot_source_hashes(current, snapshot):
    worktree = Path(current["worktree"])
    names = {os.fsdecode(p) for p in git(
        worktree, "ls-files", "--cached", "--others", "--exclude-standard", "-z").split(b"\0") if p}
    paths = {name for info in snapshot["files"].values() for name in info.get("inputs", [])}
    roots = {root for roots in current["package_roots"].values() for root in roots}
    roots.add(str(worktree))
    paths.update(str(Path(root) / name) for root in roots
                 for name in ("build.rs", "Cargo.toml", "Cargo.lock"))
    names.update(repo_relative_inputs(current, paths))
    # A newly emitted dep-info file can name a source directory for the first
    # time. Capture source parents before the child runs, not after it.
    names.update(str(parent) for name in list(names) if not Path(name).is_absolute()
                 for parent in Path(name).parents if parent != Path("."))
    return read_input_hashes(worktree, names)


class LockClosureRejected(ValueError):
    pass


class UncertifiedFreshArtifact(ValueError):
    pass


def lock_closure_digest(lock_bytes, package_id):
    packages = tomllib.loads(lock_bytes.decode())["package"]
    if "#" not in package_id:
        raise ValueError(f"unrecognized package ID: {package_id}")
    source, fragment = package_id.rsplit("#", 1)
    if "@" in fragment:
        name, version = fragment.rsplit("@", 1)
    elif source.startswith("path+file://"):
        name, version = Path(source.removeprefix("path+file://")).name, fragment
    else:
        raise ValueError(f"unrecognized package ID: {package_id}")
    source = None if source.startswith("path+") else source

    def resolve(name, version=None, source=None):
        matches = [p for p in packages if p["name"] == name
                   and (version is None or p["version"] == version)
                   and (source is None or p.get("source", "").split("#", 1)[0] == source)]
        if len(matches) != 1:
            raise ValueError(f"ambiguous or missing lock package: {name} {version}")
        return matches[0]

    roots = [p for p in packages if p["name"] == name and p["version"] == version
             and (p.get("source", "").split("#", 1)[0] if source else p.get("source")) == source]
    if len(roots) != 1:
        raise ValueError(f"ambiguous or missing lock package: {name} {version}")
    pending = roots
    entries = set()
    while pending:
        package = pending.pop()
        key = (package["name"], package["version"], package.get("source", ""))
        if key in entries:
            continue
        entries.add(key)
        for dependency in package.get("dependencies", []):
            match = re.fullmatch(r"([^ ]+)(?: ([^ ]+))?(?: \(([^)]+)\))?", dependency)
            if not match:
                raise ValueError(f"invalid lock dependency: {dependency}")
            pending.append(resolve(*match.groups()))
    content = [(p["name"], p["version"], p.get("source", ""),
                p.get("checksum", ""), sorted(p.get("dependencies", [])))
               for p in packages
               if (p["name"], p["version"], p.get("source", "")) in entries]
    return hashlib.sha256(json.dumps(sorted(content), separators=(",", ":")).encode()).hexdigest()


def certify_record(current, record, previous, source_before, artifacts, snapshot):
    path = artifact_path(record)
    compiler_inputs = target_dep_inputs(record, snapshot)
    required = repo_relative_inputs(current, compiler_inputs)
    if not required:
        raise ValueError(f"missing compiler inputs for {path}")
    generated = generated_input_paths(current, compiler_inputs)
    required.update(generated)
    for name in compiler_inputs:
        if not (repo_relative_inputs(current, [name])
                or generated_input_paths(current, [name])):
            continue
        if not os.path.exists(Path(current["worktree"]) / name):
            raise ValueError(f"missing compiler input content for {path}: {name}")
    inputs = required | repo_relative_inputs(current, package_build_inputs(current, record))
    hashes = read_input_hashes(Path(current["worktree"]), inputs)
    if any(hashes.get(name) is None for name in required):
        raise ValueError(f"missing compiler input content for {path}")
    # Slot-local generated files may be produced by this build; source inputs must predate it.
    if any(name not in source_before or source_before[name] != digest
           for name, digest in hashes.items() if name not in generated):
        raise ValueError(f"compiler inputs changed during build for {path}")
    outputs = set(record.get("filenames", []))
    if record.get("executable"):
        outputs.add(record["executable"])
    if not outputs:
        raise ValueError(f"missing compiler outputs for {path}")
    certificates = {}
    for output in sorted(outputs):
        name = os.path.relpath(output, current["slot"])
        if name not in artifacts:
            raise ValueError(f"missing output hash for {output}")
        certificate = {"inputs": hashes, "sha256": artifacts[name]}
        package_id = record.get("package_id", "")
        lock_path = Path(current["worktree"]) / "Cargo.lock"
        try:
            certificate["lock_closure"] = lock_closure_digest(lock_path.read_bytes(), package_id)
        except (OSError, ValueError, KeyError) as error:
            raise LockClosureRejected(f"unverifiable lock closure for {output}: {error}") from error
        old = previous.get(name)
        if record.get("fresh") is not False:
            comparable = dict(old) if old else {}
            if "lock_closure" in certificate and old:
                if "lock_closure" not in comparable:
                    revision = current.get("previous_revision")
                    try:
                        lock = git(Path(current["worktree"]), "show", f"{revision}:Cargo.lock") if revision else b""
                        if hashlib.sha256(lock).hexdigest() != old.get("inputs", {}).get("Cargo.lock"):
                            raise ValueError("legacy lock digest mismatch")
                        comparable["lock_closure"] = lock_closure_digest(lock, package_id)
                    except (OSError, subprocess.CalledProcessError, ValueError, KeyError):
                        raise LockClosureRejected(f"unverifiable legacy lock for {output}") from None
                if comparable["lock_closure"] != certificate["lock_closure"]:
                    raise LockClosureRejected(f"lock closure changed for {output}")
                comparable["inputs"] = dict(comparable["inputs"], **{"Cargo.lock": hashes.get("Cargo.lock")})
            if comparable != certificate:
                raise UncertifiedFreshArtifact(f"uncertified fresh artifact {output}; rebuild required")
        certificates[name] = certificate
    return certificates


def certify_artifacts(current, previous, source_before, artifacts, snapshot):
    certificates = dict((previous or {}).get("artifact_certificates", {}))
    current["previous_revision"] = (previous or {}).get("identity", {}).get("head")
    current["closure_verified"] = []
    current["rejected_artifacts"] = []
    errors = []
    for record in current["reported_artifacts"]:
        if not is_repo_artifact(current, record):
            continue
        try:
            certified = certify_record(
                current, record, certificates, source_before, artifacts, snapshot)
            certificates.update(certified)
            if record.get("fresh") is not False and all(
                    "lock_closure" in cert for cert in certified.values()):
                current["closure_verified"].append(artifact_path(record))
        except (OSError, ValueError, KeyError, tomllib.TOMLDecodeError) as error:
            errors.append(str(error))
            if isinstance(error, (LockClosureRejected, UncertifiedFreshArtifact)):
                current["rejected_artifacts"].append(artifact_path(record))
    return certificates, errors


def matched_input_changes(current, inputs, changes):
    inputs = set(inputs)
    if not inputs:
        return set()
    worktree = Path(current["worktree"])
    return {name for name in changes if name in inputs or any(
        str(parent) in inputs and (worktree / parent).is_dir()
        for parent in Path(name).parents)}


def find_stale_artifacts(current, previous_artifacts, artifacts):
    stale = []
    changed = set(current["changed_paths"])
    content_changed = set(current["content_changed_paths"])
    for record in current["reported_artifacts"]:
        path = artifact_path(record)
        inputs = current["target_inputs"].get(path, [])
        relevant_changes = matched_input_changes(current, inputs, changed)
        if path in current.get("closure_verified", []):
            relevant_changes.discard("Cargo.lock")
        if not relevant_changes:
            continue
        name = os.path.relpath(path, current["slot"])
        if record.get("fresh") is not False:
            stale.append(name)
            continue
        if not record.get("executable"):
            continue
        changed_hash = name in artifacts and previous_artifacts.get(name) != artifacts[name]
        needs_changed_hash = bool(matched_input_changes(current, inputs, content_changed))
        if name not in artifacts or (needs_changed_hash and not changed_hash):
            stale.append(name)
    return sorted(set(stale))


def unread_package_paths(current):
    inputs = {name for names in current["target_inputs"].values() for name in names}
    roots = {root for paths in current["package_roots"].values() for root in paths}
    roots = repo_relative_inputs(current, roots)
    changes = set(current["content_changed_paths"])
    read_changes = matched_input_changes(current, inputs, changes)
    return sorted(name for name in changes - read_changes
                  if any(Path(name).is_relative_to(root) for root in roots))


def finish_build(slot, receipt, current, previous, artifacts, returncode):
    previous_artifacts = (previous or {}).get("artifacts", {})
    stale_paths = find_stale_artifacts(
        current, previous_artifacts, artifacts) if returncode == 0 else []
    roots = [root for roots in current["package_roots"].values() for root in roots]
    missing_evidence = (returncode == 0 and not current["reported_artifacts"]
                        and package_inputs_changed(current, roots))
    dep_info_errors = current.get("dep_info_errors", []) if returncode == 0 else []
    stale = bool(stale_paths) or missing_evidence or bool(dep_info_errors)
    result = "success" if returncode == 0 else "failed"
    if stale:
        result = "stale-artifacts"
    rejected = {os.path.relpath(path, slot)
                for path in current.get("rejected_artifacts", [])}
    evicted = []
    eviction_errors = []
    for record in current["reported_artifacts"]:
        if os.path.relpath(artifact_path(record), slot) not in rejected:
            continue
        outputs = {Path(p) for p in [record.get("executable"), *record.get("filenames", [])] if p}
        fingerprints = set()
        executable = record.get("executable")
        if executable:
            uplift = Path(executable)
            if uplift in outputs and uplift.parent.name not in {"deps", "build"} and uplift.is_file():
                name = record.get("target", {}).get("name", "").replace("-", "_")
                suffix = uplift.suffix
                candidates = []
                for candidate in (uplift.parent / "deps").glob(f"{name}-*{suffix}"):
                    stem = candidate.name[:-len(suffix)] if suffix else candidate.name
                    if (re.fullmatch(re.escape(name) + r"-[a-f0-9]+", stem)
                            and candidate.is_file() and not candidate.is_symlink()
                            and candidate.read_bytes() == uplift.read_bytes()):
                        candidates.append(candidate)
                if len(candidates) == 1:
                    outputs.add(candidates[0])
        identified = False
        for output in outputs:
            if output.parent.parent.name == "build":
                fingerprint = output.parent.name
                profile = output.parent.parent.parent
            elif output.parent.name == "deps":
                match = re.search(r"-([a-f0-9]+)(?:\.[^.]+)*$", output.name)
                profile = output.parent.parent
                package_id = record.get("package_id", "")
                source, _, fragment = package_id.rpartition("#")
                package = (fragment.split("@", 1)[0] if "@" in fragment
                           else Path(source.removeprefix("path+file://")).name)
                fingerprint = f"{package}-{match.group(1)}" if match and package else ""
            else:
                fingerprint = ""
            if fingerprint:
                candidate = profile / ".fingerprint" / fingerprint
                if candidate.is_dir():
                    fingerprints.add(candidate)
                    identified = True
        if not identified:
            eviction_errors.append(f"{artifact_path(record)}: fingerprint could not be identified")
            outputs = {Path(p) for p in [record.get("executable"), *record.get("filenames", [])] if p}
        for path in sorted(outputs | fingerprints):
            if not path.is_relative_to(slot):
                continue
            if path.is_dir() and not path.is_symlink():
                shutil.rmtree(path)
            elif path.exists() or path.is_symlink():
                path.unlink()
            else:
                continue
            evicted.append(str(path.relative_to(slot)))
    current = dict(current, artifacts=artifacts, returncode=returncode, result=result,
                   evicted_artifacts=evicted, unread_package_paths=unread_package_paths(current))
    write_json(receipt, summarize_receipt(current))
    write_json(slot / ".source-identity.json", current)
    if result == "success":
        write_json(slot / ".verified-source-identity.json", current)
    if stale:
        reason = ("no Cargo compiler-artifact records visible for changed package inputs"
                  if missing_evidence else f"missing fresh=false or changed SHA256: {stale_paths}")
        if dep_info_errors:
            reason = "; ".join(dep_info_errors)
        if eviction_errors:
            reason += "; " + "; ".join(eviction_errors)
        raise ValueError(f"stale artifacts: {reason}; see {receipt}")


def hash_artifacts(slot, paths):
    artifacts = {}
    for path in sorted(paths):
        if path.is_file() and not path.is_symlink():
            with path.open("rb") as stream:
                artifacts[os.path.relpath(path, slot)] = hashlib.file_digest(
                    stream, "sha256").hexdigest()
    return artifacts


def collect_artifacts(slot, started_ns, previous, records):
    # Retain discovery/hash evidence for capturing wrappers. Only current Cargo
    # records select artifacts for the freshness decision.

    paths = {slot / name for name in (previous or {}).get("artifacts", {})}
    for directory, children, files in os.walk(slot):
        children[:] = [name for name in children
                       if not name.startswith(".") and name not in {"incremental", "receipts"}]
        for name in files:
            path = Path(directory) / name
            if path.is_symlink() or not path.is_file():
                continue
            stat = path.stat()
            artifact = (stat.st_mode & 0o111 or path.suffix in LIBRARY_SUFFIXES
                        or path.suffix == ".exe")
            if artifact and max(stat.st_mtime_ns, stat.st_ctime_ns) >= started_ns:
                paths.add(path)
    paths.update(Path(name) for record in records
                 for name in [record.get("executable"), *record.get("filenames", [])] if name)
    return hash_artifacts(slot, paths)


def run_build_command(command):
    records = []
    with subprocess.Popen(command, stdout=subprocess.PIPE) as child:
        for line in child.stdout:
            sys.stdout.buffer.write(line)
            sys.stdout.buffer.flush()
            try:
                record = json.loads(line)
            except (ValueError, UnicodeDecodeError):
                continue
            if isinstance(record, dict) and record.get("reason") == "compiler-artifact":
                records.append(record)
        return child.wait(), records


def execute_build(request_path, cap_gib=60, recent_hours=2):
    request = json.loads(request_path.read_text())
    slot = Path(request["slot"])
    check_owner(slot, request["owner"])
    with redirect_stdout(sys.stderr):
        prune_slot(slot, request["owner"], cap_gib * 1024 ** 3, recent_hours)
    state = slot / ".source-identity.json"
    verified = slot / ".verified-source-identity.json"
    previous = json.loads(verified.read_text()) if verified.exists() else None
    attempted = json.loads(state.read_text()) if state.exists() else None
    worktree = Path(request["worktree"])
    current = prepare_source(worktree, previous, attempted)
    current.update(unit=request["unit"], cargo=request["cargo"], worktree=str(worktree),
                   slot=str(slot), package_roots=read_package_roots(worktree))
    # Attempts can replace Cargo fingerprints, but cannot certify source content.
    # Only finish_build advances the separate verified baseline after acceptance.
    write_json(state, dict(current, artifacts={}, result="building"))
    receipt = Path(request["receipt"])
    write_json(receipt, summarize_receipt(dict(current, artifacts={}, result="building")))
    before = read_dep_info_snapshot(slot, worktree)
    source_before = snapshot_source_hashes(current, before)
    started_ns = time.time_ns()
    returncode, records = run_build_command(request["cargo"])
    current["reported_artifacts"] = records
    after = read_dep_info_snapshot(slot, worktree)
    current["target_inputs"], current["dep_info_errors"] = read_target_inputs(
        current, before, after)
    artifacts = collect_artifacts(slot, started_ns, previous, records)
    current["artifact_certificates"], certificate_errors = certify_artifacts(
        current, previous, source_before, artifacts, after)
    current["dep_info_errors"].extend(certificate_errors)
    finish_build(slot, receipt, current, previous, artifacts, returncode)
    return returncode


def print_status(unit):
    properties = ["SubState", "Result", "CPUUsageNSec",
                  "ExecMainStartTimestampMonotonic", "ExecMainExitTimestampMonotonic"]
    argv = ["/usr/bin/systemctl", "--user", "show", unit]
    for name in properties:
        argv.extend(["-p", name])
    result = subprocess.run(argv, check=True, capture_output=True, text=True)
    values = dict(line.split("=", 1) for line in result.stdout.splitlines() if "=" in line)
    for name in properties[:3]:
        print(f"{name}={values[name]}")
    start = int(values[properties[3]])
    end = int(values[properties[4]]) or time.monotonic_ns() // 1000
    wall = f"{(end - start) / 1e6:.6f}" if start else "unavailable"
    print(f"WallTimeSec={wall}")


def requests_json_messages(cargo):
    cargo_args = cargo[:cargo.index("--")] if "--" in cargo else cargo
    pairs = zip(cargo_args, cargo_args[1:] + [""])
    return any(arg.startswith("--message-format=json")
               or (arg == "--message-format" and value.startswith("json"))
               for arg, value in pairs)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--execute", type=Path, help=argparse.SUPPRESS)
    parser.add_argument("--status", metavar="UNIT")
    parser.add_argument("--slot", type=int, choices=SLOTS)
    parser.add_argument("--worktree", type=Path)
    parser.add_argument("--unit")
    parser.add_argument("--owner")
    parser.add_argument("--capacity-id")
    parser.add_argument("--slot-cap-gib", type=float, default=60,
                        help="prune above this allocated size (default: 60 GiB)")
    parser.add_argument("--prune-recent-hours", type=float, default=2,
                        help="retain units modified within this age (default: 2 hours)")
    parser.add_argument("--prune-dry-run", action="store_true",
                        help="list pruning for --slot read-only; no owner claim required")
    parser.add_argument("cargo", nargs=argparse.REMAINDER)
    args = parser.parse_args(argv)
    if args.execute:
        return execute_build(args.execute, args.slot_cap_gib, args.prune_recent_hours)
    if args.prune_dry_run:
        if args.slot is None:
            parser.error("--prune-dry-run requires --slot")
        prune_slot(TARGET_ROOT / f"slot-{args.slot}", None,
                   args.slot_cap_gib * 1024 ** 3, args.prune_recent_hours, dry_run=True)
        return
    if not math.isfinite(args.slot_cap_gib) or args.slot_cap_gib <= 0:
        parser.error("--slot-cap-gib must be positive and finite")
    if not math.isfinite(args.prune_recent_hours) or args.prune_recent_hours < 0:
        parser.error("--prune-recent-hours must be nonnegative and finite")
    if args.status:
        if any([args.slot is not None, args.worktree, args.unit,
                args.owner, args.capacity_id, args.cargo]):
            parser.error("--status cannot be combined with launch arguments")
        print_status(args.status)
        return
    if args.slot is None or not all([args.worktree, args.unit, args.owner, args.capacity_id]):
        parser.error("launch requires --slot, --worktree, --unit, --owner and --capacity-id")
    cargo = args.cargo[1:] if args.cargo[:1] == ["--"] else args.cargo
    if not cargo:
        parser.error("provide a Cargo command after --")
    if Path(cargo[0]).name != "cargo":
        parser.error(f"the command after -- must start with cargo, got {cargo[0]!r}")
    if not requests_json_messages(cargo):
        parser.error("the Cargo command must pass --message-format=json before any --; "
                     "artifact certification needs compiler-artifact records")
    if not args.worktree.is_absolute() or not args.worktree.is_dir():
        parser.error("--worktree must be an absolute directory path")
    worktree = args.worktree.resolve()
    target = TARGET_ROOT / f"slot-{args.slot}"
    check_owner(target, args.owner)
    check_capacity(args.capacity_id)
    check_owner(target, args.owner)
    (SOURCE_ROOT / f"slot-{args.slot}").mkdir(parents=True, exist_ok=True)
    if Path(args.unit).name != args.unit or args.unit in {".", ".."}:
        parser.error("--unit must be a unit name, not a path")
    request = target / "receipts" / f"{args.unit}.request.json"
    write_json(request, {"slot": str(target), "worktree": str(worktree),
                         "owner": args.owner, "unit": args.unit, "cargo": cargo,
                         "receipt": str(request.with_name(f"{args.unit}.json"))})
    worker = [sys.executable, str(Path(__file__).resolve()), "--execute", str(request),
              "--slot-cap-gib", str(args.slot_cap_gib),
              "--prune-recent-hours", str(args.prune_recent_hours)]
    subprocess.run(build_argv(args.slot, worktree, args.unit, worker), check=True)
    print(args.unit)


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        raise SystemExit(str(error)) from error
