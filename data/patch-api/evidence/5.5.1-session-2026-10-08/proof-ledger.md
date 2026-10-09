# Proof ledger

Runtime scope: d56ab05934e52b124622ccf074749ce382d6e8e8; baseline 0469e5592fa74e051d9b01d85368b351f24e7d09. Exact commands, revision, target, environment, exits and log hashes are retained in each *.proof.json.

- Retail publication sweeps and line controls: running; retail baseline follows sequentially in the retail worker.
- Mists 5.5.1–5.5.4, cargo check --tests, injected-row negative: running sequentially in the Mists worker.
- Python tools/test_*.py: 87/87 pass. Cargo fmt --check: pass.
- Reproduction: 60 registers identical, 57 extracts identical, three inherited failures exactly preserved.
- Scans: /usr/bin/grep whole-word; full untruncated stdout retained. No source-listed retirement identities.

Source/runtime/tool/Cargo changes invalidate intersecting proof; wiki/spec/evidence-only updates do not. No shared src changes: broader integration/prefork/lib regression suites are not required by this page. No proof is reused from previous page execution. Final session seals and portable gate pending.

Additional independent proof: run the pinned-master validator gate once for the git ls-tree prior set; this does not replace the required final branch gate.

Gate-output recovery: session continuation orphaned the already-running pinned-master gate (PID 2785718, PPID 1) and removed its original Pyrun pipe reader before a result was saved. recover_gate_output.py drains that same process stdout into master-gate-report.json; the gate is not rerun. Build workers already had file-backed output and remain intact.
