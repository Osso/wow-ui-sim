# Patch 1.0.0 SOURCE replay

Original seal/archive commit `f616d6481`:42 original sealed files,43-member155,329-byte archive. Original map SHA256 `c60a8cc891f28dd3b8175982a293463f636d7abda2382a019c944a28391c694d`; archive SHA256 `4db3045271c7043291e49572e571a18a0e507a6f8c6fb29e879c8a2e3ff95f0b`. Never reseal originals or backfill later receipts into original proof epochs.

## Fresh copied replay through Pyrun

```python
from pathlib import Path
import hashlib, json, tarfile, tempfile
root = Path('/home/osso/.worktrees/wow-ui-sim-p100-page')
e = root / 'data/patch-api/evidence/1.0.0-session-2026-10-09'
context = json.loads((e / 'portable-context.json').read_bytes())
archive_path = e / 'replay-archive.tar.gz'
assert hashlib.sha256(archive_path.read_bytes()).hexdigest() == context['archive_sha256']
with tempfile.TemporaryDirectory(prefix='replay-100-', dir=e) as directory:
    copied = Path(directory)
    with tarfile.open(archive_path) as archive:
        members = archive.getmembers()
        assert len(members) == context['archive_members'] == 43
        for member in members:
            path = Path(member.name)
            assert member.isfile() and not path.is_absolute()
            assert not set(path.parts) & {'..', '.git', 'target', 'Interface', '__pycache__'}
        archive.extractall(copied, filter='data')
    for script in ['audit.py', 'test_source_accounting.py', 'test_portable.py']:
        result = cli.command('/usr/bin/python3', '-I', '-B', str(copied / script)).cwd(str(root)).env('PATH', '/nonexistent').env('PYTHONDONTWRITEBYTECODE', '1').capture().run()
        print(script, result)
        assert result.exit_code == 0
```

All CLI cwd remains owned worktree. Copied modules resolve archived evidence and unchanged own-base historical tools; no Git, target, addon, current-tool, credential files or network/client/build requirement. `-I -B` isolates Python imports/bytecode; `PATH=/nonexistent` prevents executable discovery. Three commands above are development replay, not independent full acceptance.

## Expected evidence

- `audit.py`: exit0,42 original seals,854 inventory/854 unspecified signatures,859raw rows/859UNPROVEN contracts,856links,2headers/0counts/0defaults,2prose/1template; default register0/extract104bytes/error behavior byte-identical.
- `test_source_accounting.py`:8/8,4,287 occurrence omissions,16 count mutations,23 invention/semantic mutations; exact identity/source mutations and separate histories.
- `test_portable.py`:3/3, copied replay, serialized ledger reference omission/log fabrication rejected, exact original byte/hash/map restoration without resealing.

Own SOURCE GREEN8 at `039cfa29d`; fresh copied SOURCE8/portable3 at `f616d6481`, not base or later documenting HEAD. [Original proof ledger](proof-ledger.json), [later actual receipts](later-proof-ledger.json), [archive identity](archive-identity.json), [portable controls](portable-proof.json), [context](portable-context.json) and separate `receipt-seals.json` retain exact scopes. Original RED test/scaffold preserve executed pre-format bytes. `original-docs/` stays original pre-portable snapshot; live HANDOFF/REPLAY evolve separately. Full stdout/stderr retained independently; combined logs concatenate stdout then stderr, without chronological interleaving claims. Main owns historical-target research, integration and independent full acceptance; registry endpoint does not close parent.
