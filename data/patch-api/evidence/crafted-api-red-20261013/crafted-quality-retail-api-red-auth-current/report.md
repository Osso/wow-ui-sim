# PASS — Retail compile-only and sealed-copy authentication

Source code label: `8b46433a4568e26458a15907ea79636e38ff65f9`. Caller-supplied current HEAD `dd10be4ff` has later privacy-only changes; this is **not** a current-revision binary claim. No source tree was authenticated.

## Existing compilation evidence

- Full `verification/widget-discrete-progress-current/crafted-quality-retail-api-red-retry1-20261014T012534611408Z/compile.stdout` read and parsed: 741 JSON records (665 compiler artifacts, 75 build-script records, 1 build-finished), final `success=true`; 0 compiler diagnostics.
- Full `verification/widget-discrete-progress-current/crafted-quality-retail-api-red-retry1-20261014T012534611408Z/compile.stderr` read: six manifest deprecations plus one six-warning summary, three package-cache lock wait lines; Cargo finished in 1m 47s.
- `verification/widget-discrete-progress-current/crafted-quality-retail-api-red-retry1-20261014T012534611408Z/compile-terminal.txt`: `Result=success`, `SubState=exited`, `ExecMainStatus=0`; October 13, 2026, 20:25:34–20:27:22 CDT; memory peak 5,468,069,888 bytes.
- `verification/widget-discrete-progress-current/crafted-quality-retail-api-red-retry1-20261014T012534611408Z/submission.json` records `cargo test --offline --locked --no-run --test integration -j4 --message-format=json-render-diagnostics`; no test execution. Retail features match seal.

## Exactly-once new-copy authentication

- Seal: `verification/widget-discrete-progress-current/crafted-quality-retail-api-red-retry1-20261014T012534611408Z/seal-integration.json`.
- New normal single-link executable: `verification/widget-discrete-progress-current/crafted-quality-retail-api-red-retry1-20261014T012534611408Z/artifacts/deps/integration-9f07e4a55f2bbbf7`.
- Receipt absent before authentication; exclusively created before the sole sequential SHA-256 pass. Pre-stat, opened-descriptor stat, post-descriptor stat and post-path stat match seal.
- Digest: `cbae6489645597ef075b9f6d6406b35731248afa92391975502064ad46902d33`; `sha256_matches_seal=true`, `hash_reads=1`, `status=PASS`.
- Receipt: `verification/widget-discrete-progress-current/crafted-quality-retail-api-red-retry1-20261014T012534611408Z/receipt-integration.json`; mode 0600, fsynced and read back identically. No old binary, libraries, archive or source-tree authentication.

## Boundary

Actual crafted-feature RED remains **UNEXECUTED until main runs it**. Compile success is not feature acceptance. Prior 01:24:31 UTC admission denial preserved at `verification/widget-discrete-progress-current/crafted-quality-retail-api-red-20261014T012431198439Z/admission.json` (`allowed=false`). No builds, tests, checks, formatters, service actions or delegation performed.

Proof: `verification/crafted-quality-retail-api-red-auth-current/proof.json`.
