"""Validate historical audit scope without accepting unrecorded input drift."""
import hashlib
import json
import subprocess

# The merged 8.2.5 audit (ending 127aa3724035d9e668143d9b28aaa4cf6e176fa1)
# closed only C_ClubFinder.ReportPosting in 9.2.5. These are the exact before
# and after blobs from 7ff3dc540^ and that audit endpoint, respectively.
# No row-level or whitespace drift beyond those committed bytes is accepted.
LATER_AUDIT_REPLACEMENTS = {
    'data/patch-api/sources/9.2.5-page-coverage.json': (
        'd022e992c195f5756620fc4534430cb4f2bf372cbb7b9ee3d9075f17ad5524df',
        '64896487daebe0b4710259a861edd9fe17a2114a8822e00e4363f0aa4c284e97',
    ),
    'tests/data/patch_9_2_5_sweep_known_gaps.json': (
        'a834106b494886a3ac4877e5a623d7552aefd27188fab04555257c533651f3e9',
        'f7370f8626dab454cb1815e064173a7ad12c36b53046c1db2ae44a31a32c524a',
    ),
}


def preserved_input_matches(root, path, recorded_digest):
    """Accept original bytes or a specific, merged later-audit replacement."""
    current_digest = hashlib.sha256((root / path).read_bytes()).hexdigest()
    return current_digest == recorded_digest or LATER_AUDIT_REPLACEMENTS.get(path) == (
        recorded_digest, current_digest
    )


def historical_json(root, path, revision):
    """Check current input provenance, then return the historical proof input."""
    original = subprocess.check_output(['git', 'show', f'{revision}:{path}'], cwd=root)
    digest = hashlib.sha256(original).hexdigest()
    assert preserved_input_matches(root, path, digest), f'unrecorded input change: {path}'
    return json.loads(original)


def read_audit_json(root, path, revision):
    """Use historical accounting only for explicitly recorded later closures."""
    relative = path.relative_to(root).as_posix()
    if relative in LATER_AUDIT_REPLACEMENTS:
        return historical_json(root, relative, revision)
    return json.loads(path.read_text())


def _historical_paths(root, revision, directory, pattern):
    names = subprocess.check_output(
        ['git', 'ls-tree', '-r', '--name-only', revision, directory],
        cwd=root, text=True,
    ).splitlines()
    paths = [root / name for name in names if (root / name).match(pattern)]
    assert paths, f'no {pattern} at {revision}'
    for path in paths:
        assert path.is_file(), f'missing historical input: {path.relative_to(root)}'
    return paths


def historical_registers(root, revision):
    """Keep the complete register set at the audit revision, not today's set."""
    return _historical_paths(root, revision, 'data/patch-api/sources', '*-wikitext-register.json')


def historical_sweep_tests(root, revision):
    """Keep every historical sweep source, including non-register sweeps."""
    return _historical_paths(root, revision, 'tests', 'patch_*_publication_sweep.rs')
