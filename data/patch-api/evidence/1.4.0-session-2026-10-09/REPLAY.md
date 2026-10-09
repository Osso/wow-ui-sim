# Patch 1.4.0 copied SOURCE replay

Original `replay-archive.tar.gz` contains44 sealed files plus original map (45 members,84,207 bytes). It preserves the pre-category-correction SOURCE7 epoch, not corrected metadata. Separate `current-replay-extension.tar.gz` contains current accounting/test/ledger plus its own three-file map. Neither archive is rewritten by later receipts.

Current integration uses `current-ledger.json`: only Changed TogglePVP category is null rather than last New category Miscellaneous. Contracts/counts/defaults/history/coverage unchanged. All32 substantive contracts remain UNPROVEN.

## Fresh-copy Pyrun commands

Run in owned worktree only; all paths absolute, every CLI uses owned cwd. Standard-library Python, no Git, current tools, build target, runtime, addons or network. Isolated interpreter/module path and `PATH=/nonexistent`. Commands below are replay instructions, not a claim that later docs HEAD was executed.

```python
from pathlib import Path
import hashlib, json, sys, tarfile, tempfile
root = Path('/home/osso/.worktrees/wow-ui-sim-p140-page')
evidence = root / 'data/patch-api/evidence/1.4.0-session-2026-10-09'
with tempfile.TemporaryDirectory(prefix='manual-replay140-', dir=evidence) as folder:
    copied = Path(folder)
    identity = json.loads((evidence / 'archive-identity.json').read_bytes())
    archive_path = evidence / 'replay-archive.tar.gz'
    assert hashlib.sha256(archive_path.read_bytes()).hexdigest() == identity['sha256']
    with tarfile.open(archive_path, 'r:gz') as archive:
        archive.extractall(copied, filter='data')
    original_map = (copied / 'seals.json').read_bytes()
    assert hashlib.sha256(original_map).hexdigest() == identity['original_map_sha256']
    for name, expected in json.loads(original_map).items():
        assert hashlib.sha256((copied / name).read_bytes()).hexdigest() == expected
    for script in ['audit.py', 'test_source_accounting.py', 'test_portable.py']:
        code = f'import sys,runpy; sys.path.insert(0, {str(copied)!r}); runpy.run_path({str(copied / script)!r}, run_name="__main__")'
        result = cli.command(sys.executable, '-I', '-B', '-c', code).cwd(str(root)).env({'PATH': '/nonexistent'}).capture().run()
        print(script, result.exit_code, result.stdout, result.stderr)
        assert result.exit_code == 0
    extension_identity = json.loads((evidence / 'current-extension-identity.json').read_bytes())
    extension = evidence / 'current-replay-extension.tar.gz'
    assert hashlib.sha256(extension.read_bytes()).hexdigest() == extension_identity['sha256']
    with tarfile.open(extension, 'r:gz') as archive:
        archive.extractall(copied, filter='data')
    for name, expected in json.loads((copied / 'current-extension-seals.json').read_bytes()).items():
        assert hashlib.sha256((copied / name).read_bytes()).hexdigest() == expected
    code = f'import sys,runpy; sys.path.insert(0, {str(copied)!r}); runpy.run_path({str(copied / "test_current_accounting.py")!r}, run_name="__main__")'
    result = cli.command(sys.executable, '-I', '-B', '-c', code).cwd(str(root)).env({'PATH': '/nonexistent'}).capture().run()
    print('current category correction', result.exit_code, result.stdout, result.stderr)
    assert result.exit_code == 0
    assert (copied / 'seals.json').read_bytes() == original_map
```

Expected original validator exit0, SOURCE7/7 with197 omission/count controls, portable3/3; corrected SOURCE3/3 with55 category/omission controls. Portable tests alter **copied** serialized ledger/log, demand seal rejection, restore exact bytes/hash/map, then replay without resealing. Historical default generator zero-entry bytes, extractor2,250-byte output and two exact errors remain distinct from literal27 declarations. All32 contracts remain UNPROVEN; no current/historical native/model or parent acceptance credit.

## Actual proof epochs

- `5c4bde798`: original SOURCE7/7.
- `edbe7d9a1`: fresh original copied validator/SOURCE7/portable3; original44 seals/archive unchanged.
- `8bfc74ee9`: separate original-plus-extension copied current SOURCE3/3;55 controls, original/default preservation.

[Original ledger](proof-ledger.json), [later ledger](later-proof-ledger.json), [original scopes](proof-scope-index.json), [later scopes](later-proof-scope-index.json), [portable receipts](portable-proof.json), and [contexts](portable-context.json)/[current context](current-portable-context.json) retain actual revisions/cwd/argv/times/full streams/hash scopes. Logs concatenate stdout then stderr, not chronological interleaving. Original map/archive and current extension map/archive remain immutable; `receipt-seals.json` covers separate later artifacts, never backfills original seals.
