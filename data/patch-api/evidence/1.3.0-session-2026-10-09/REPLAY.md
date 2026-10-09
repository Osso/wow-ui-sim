# Patch 1.3.0 original pre-copy replay instructions

Original archive/seal map will retain this epoch unchanged. Extract the future replay-archive.tar.gz to a fresh safe data-only directory. Use absolute copied paths and owned CLI cwd `/home/osso/.worktrees/wow-ui-sim-p130-page`.

```text
/usr/bin/env PATH=/nonexistent PYTHONNOUSERSITE=1 /usr/bin/python3 -I -B /ABSOLUTE/COPY/audit.py
/usr/bin/env PATH=/nonexistent PYTHONNOUSERSITE=1 /usr/bin/python3 -I -B /ABSOLUTE/COPY/test_source_accounting.py
/usr/bin/env PATH=/nonexistent PYTHONNOUSERSITE=1 /usr/bin/python3 -I -B /ABSOLUTE/COPY/test_portable.py /ABSOLUTE/NEW-RECEIPT.json
```

Copied modules consume only retained evidence/own-base tools; no Git/target/addons/current tools/network. Receipt path must not exist. Serialized ledger omission and fabricated native log must reject and restore original bytes/hashes/map without resealing. Later actual execution receipts stay separate. SOURCE only, no runtime/native or parent acceptance.
