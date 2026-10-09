# Patch 1.5.0 SOURCE replay

Original31 seals/32-member53,739-byte archive sealed at `a4f4d59ba`; actual executed epoch `d209973b3d20c2b894e27384e57b33d704ee3a75`. `portable-context.json` pins archive/map hashes; never reseal or backfill later proof into originals.

## Fresh copied replay via Pyrun

```python
from pathlib import Path
import hashlib, json, tarfile, tempfile
root = Path('/home/osso/.worktrees/wow-ui-sim-p150-page')
e = root / 'data/patch-api/evidence/1.5.0-session-2026-10-09'
context = json.loads((e / 'portable-context.json').read_bytes())
archive_path = e / 'replay-archive.tar.gz'
assert hashlib.sha256(archive_path.read_bytes()).hexdigest() == context['archive_sha256']
assert hashlib.sha256((e / 'seals.json').read_bytes()).hexdigest() == context['original_seals_sha256']
with tempfile.TemporaryDirectory(prefix='replay-150-', dir=e) as directory:
    copied = Path(directory)
    with tarfile.open(archive_path) as archive:
        members = archive.getmembers()
        assert len(members) == 32
        for member in members:
            path = Path(member.name)
            assert member.isfile() and not path.is_absolute()
            assert not set(path.parts) & {'..', '.git', 'target', 'Interface', '__pycache__'}
        archive.extractall(copied, filter='data')
    for script in ['audit.py', 'test_source_accounting.py', 'test_portable.py']:
        result = cli.command('/usr/bin/python3', '-I', '-B', str(copied / script)).cwd(str(root)).env('PATH', '/nonexistent').env('PYTHONDONTWRITEBYTECODE', '1').capture().run()
        print(script, result.exit_code, result.stdout, result.stderr)
        assert result.exit_code == 0
```

Every CLI cwd remains owned worktree. Isolated imports use copied evidence/historical tools only; copied directory contains no Git, target, current tools, addon or credentials. No client/runtime/network/build launch.

## Expected bounded proof

- `audit.py`:31 seals; one physical/nonblank/metadata row/link/UNPROVEN contract, zero local declarations; exact historical default bytes and malformed-input errors.
- `test_source_accounting.py`:6/6,25 ledger controls and six source identity mutations/restorations, separate histories/defaults.
- `test_portable.py`:3/3; copied replay plus serialized ledger omission/log fabrication rejection, exact original byte/hash/map restoration without resealing.

[Original proof](proof-ledger.json), [later receipts](later-proof-ledger.json), [context](portable-context.json) and [restorations](portable-proof.json) retain exact epochs. Seven later artifacts have separate `receipt-seals.json`; originals unchanged. `original-docs/` describes original pre-portable scope. Redirect stays unexpanded; main owns research/integration/independent acceptance. No native/runtime/parent goal credit.
