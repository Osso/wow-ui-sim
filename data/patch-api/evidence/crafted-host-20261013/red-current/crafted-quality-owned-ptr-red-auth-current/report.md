# PTR compile and sealed-copy verification

**PASS — compile evidence and copy authentication only. Runtime acceptance remains unexecuted.**

- Source: `c652aaa8824f87089f0747a3ad0e19a4dbe3350e`; production delta: `6c38a8273`. Not a current-HEAD runtime claim.
- Epoch: `<PRIVATE_HOME>/.local/state/wow-ui-sim/verification/widget-discrete-progress-current/crafted-quality-owned-ptr-red-20261014T014829545770Z`. All eight metadata files and both complete compile logs read.
- Existing no-run compile: unit exit **0**, Result=success; **741** JSON records (665 compiler artifacts, 75 build scripts, 1 build-finished); final `success=true`; **0 compiler diagnostics**, **6 manifest deprecations**. Cargo **4m 06s**.
- Unit: **October 13, 2026, 20:48:29–20:52:36 CDT**; peak **7,600,734,208 bytes**. Epoch's UTC date is October 14.
- New sealed integration copy: `<PRIVATE_HOME>/.local/state/wow-ui-sim/verification/widget-discrete-progress-current/crafted-quality-owned-ptr-red-20261014T014829545770Z/artifacts/deps/integration-020202567c2bd5a4`. Receipt absent before authentication. Exactly **one sequential SHA-256 pass**, **471,553,176 bytes**; pre/post path and descriptor stats matched seal.

SHA-256: `a184cc7ededb87b665131889e07d299be3140be032a985b89adab2baae198cdf`

Receipt: `<PRIVATE_HOME>/.local/state/wow-ui-sim/verification/widget-discrete-progress-current/crafted-quality-owned-ptr-red-20261014T014829545770Z/receipt-integration.json` — `PASS`, `sha256_matches_seal=true`, `hash_reads=1`, path, post-stat and digest recorded; mode **0600**, file/directory fsync, exact readback verified.

## Coverage

| Claim | Proof |
|---|---|
| PTR integration compilation | PASS, existing logs only |
| New sealed integration-copy identity | PASS, one hash pass |
| Five positive PTR cases | UNEXECUTED |
| One original whole-consumer PTR case | UNEXECUTED |
| Current HEAD runtime behavior | NOT CLAIMED |

No inference from Retail. No binary executed/copied; no build/test/check/formatter reruns, services, delegation, Bash or cwd tools. Old binaries, normal binary, libraries, archives and source tree not authenticated. Source identity is recorded metadata, not independent source authentication.
