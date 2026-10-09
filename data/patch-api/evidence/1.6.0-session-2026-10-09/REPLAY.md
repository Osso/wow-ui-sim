# Patch 1.6.0 SOURCE replay

Original seal/archive commit `a0e7ba9016e7c2034ada53e1c3d029599c192e28`:30 sealed files,31-member53,161-byte archive. Original map SHA256 `b193b6752c7fb6e2686f27aef28b7bd2321f7a8c18846c23198e27bf8aa7702f`. Archive SHA256 is retained in `portable-context.json` and each later invocation. Do not reseal or backfill later proof into originals.

## Fresh copied replay via Pyrun

```python
from pathlib import Path
import hashlib, json, tarfile, tempfile
root = Path('/home/osso/.worktrees/wow-ui-sim-p160-page')
e = root / 'data/patch-api/evidence/1.6.0-session-2026-10-09'
context = json.loads((e / 'portable-context.json').read_bytes())
archive_path = e / 'replay-archive.tar.gz'
assert hashlib.sha256(archive_path.read_bytes()).hexdigest() == context['archive_sha256']
with tempfile.TemporaryDirectory(prefix='replay-160-', dir=e) as directory:
    copied = Path(directory)
    with tarfile.open(archive_path) as archive:
        members = archive.getmembers()
        assert len(members) == 31
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

All CLI cwd remains owned worktree as instructed. Isolated copied modules load only archived evidence and historical tool copies; copied directory has no Git, target, addon, current tools or credential files. `PATH=/nonexistent` prevents executable discovery; `-I -B` isolates Python imports and bytecode. No runtime/client/network/build launch.

## Expected proof

- `audit.py`: exit0;30 seals, one physical/nonblank/metadata row/link/UNPROVEN contract, zero inventory/signatures/defaults/prose/headers/templates; byte-identical default generator/extractor and malformed-input errors.
- `test_source_accounting.py`:5/5; identity/source mutations,25 ledger omission/count/invention/expansion controls, separate histories, defaults/errors.
- `test_portable.py`:3/3; copied replay, serialized ledger reference omission and fabricated log rejection, exact original byte/hash/map restoration without resealing.

Executed at `a0e7ba901`, not base or later documentation HEAD. [Original proof](proof-ledger.json), [later actual receipts](later-proof-ledger.json), [context](portable-context.json) and [tamper/restoration receipts](portable-proof.json) preserve epochs. Separate `receipt-seals.json` seals later artifacts only. `original-docs/` remains the original pre-portable-proof snapshot, never rewritten. Target remains unexpanded; main owns target research, integration and independent acceptance.
