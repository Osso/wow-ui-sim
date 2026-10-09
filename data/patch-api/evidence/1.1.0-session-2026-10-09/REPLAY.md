# Frozen 1.1.0 SOURCE replay

Original `seals.json` and `replay-archive.tar.gz` are immutable once created. Archive contains all sealed original files plus exact seal map, not later receipts. No Git/target/current shared tools/network needed by copied replay; Python standard library only. Own-base generator/extractor copies preserve zero-entry generator/default106-byte extraction and malformed-input errors, not corrected literal11 inventory.

## Fresh replay with Pyrun

```python
from pathlib import Path
import hashlib, json, tarfile, tempfile
own = Path('/home/osso/.worktrees/wow-ui-sim-p110-page')
e = own / 'data/patch-api/evidence/1.1.0-session-2026-10-09'
identity = json.loads((e/'archive-identity.json').read_bytes())
archive = e/'replay-archive.tar.gz'
assert hashlib.sha256(archive.read_bytes()).hexdigest() == identity['archive_sha256']
with tempfile.TemporaryDirectory(prefix='p110-replay-') as directory:
    copied = Path(directory)
    with tarfile.open(archive) as bundle:
        bundle.extractall(copied, filter='data')
    assert not any((copied/name).exists() for name in ['.git','target','tools','Interface'])
    for name in ['audit.py','test_source_accounting.py','test_portable.py']:
        result = cli.command('python3','-I','-B',str(copied/name)).cwd(str(own)).capture().run()
        print(result.stdout, result.stderr)
        assert result.exit_code == 0
```

`audit.py` checks original seals, validates literal ledger, reproduces default bytes/errors. Copied SOURCE suite exercises all66occurrence omission controls plus counts/inventions/identity/source/history/default boundaries. Portable3 creates further fresh evidence-only copies; serialized ledger reference omission and successful SOURCE log fabrication each reject by original seal, restore exact bytes/hash/map without resealing, then replay succeeds.

Before invoking proof, inspect proof ledgers for scope/revision coverage. Main should capture real cwd/argv/revision/times/separate full streams/hash scopes in a new later receipt, not overwrite original files/map/archive or reuse historical execution timestamps. Later receipts do not change historical input/proof epoch; no parent/native acceptance is claimed. [Handoff](HANDOFF.md) and [audit](../../../../docs/wiki/investigations/patch-1-1-0-api-audit.md) bound claims.
