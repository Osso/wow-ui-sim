# Proof ledger

Base: 7a292c9d0fc7458c648eceb6cf67a036b2e22ac8. Runtime/test scope: c9e249647. No src, shared helper, tool, Cargo, vendor or existing sweep changes.

- Register/extract reproduction at c9e249647: 62 registers byte-identical, 59/62 extracts; inherited 12.0.5, 12.0.7, 12.1.0 failures retained unchanged. Later source/tool changes invalidate this proof.
- tools-tests receipt: 89 Python fixtures pass. Later tools changes invalidate.
- format receipt: cargo fmt --check passes. Later Rust changes invalidate.
- Retail and Mists workers active; no completed result claimed until receipts appear.
- Docs-only additions do not invalidate runtime or reproduction proof.
