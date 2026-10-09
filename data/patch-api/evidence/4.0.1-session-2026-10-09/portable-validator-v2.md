# Portable historical validator v2 — 2026-10-09

`validate-v1.py.txt` and `historical-context-v1.json` preserve exact original v1 bytes, before validation code or metadata changes:

- Validator v1 SHA256: `ab28d73b49bef4a6e428fc060258dbe0f5d5cc53e1490db5b985249a8bb91cc6`.
- Context v1 SHA256: `a7ae9cdfb4506ff99f15e4254ddc464fe11723b9563aa662f7d26b446c7c92c0`.

Executable v2 `validate.py` SHA256: `12bd752b78ff9d47f91cec474dceca6d48b991faab43d6287bc2ec94e2fa6606`. Its sole code change removes `dir=ROOT / 'target'` from the reproduction workspace; Python now selects the system temporary directory. No repository `target` creation or fallback was added. Active context changes only the executable self-seal and adds seals for the preserved v1 files. Original archive, evidence, source, runtime, log, proof and artifact seals remain unchanged.

This is new executable metadata, not a retroactive claim that original recorded invocations used v2. Recorded historical inputs and results remain 419 inventory / 302 covered / 117 publication gaps; the negative fixture retains 118 gaps. Current ledgers and later audits are outside this change.

Reproducer: `python3 -B tools/test_patch_4_0_1_validator.py -v`. Fixtures copy only sealed evidence, validator and pinned source into disposable fresh roots, then execute a separate Python process with no Git/Cargo on PATH and no `target` directory. Both valid historical-scope cases failed before the fix with `FileNotFoundError` at `check_reproduction`; the three serialized archive/source/log tamper cases rejected correctly. Tests assert `target` stays absent before and after execution and historical counts remain unchanged. Parent owns global portability/final acceptance.
