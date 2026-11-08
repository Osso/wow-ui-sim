# PASS — existing PTR compile evidence and sealed-copy authentication

Scope: compile only, recorded source `9cc2112267ea139e3e13408a16787edd53c232dd`, production delta `8b46433a4`. Revision identifiers are recorded metadata; source trees/archives were not authenticated. No runtime RED or current-HEAD binary claim.

## Recorded compile evidence

- All 741 JSON records parsed: 665 compiler artifacts, 75 build-script records, final `build-finished: success=true`; 0 compiler diagnostics.
- Stderr: six manifest deprecations (plus their summary), one package-cache lock wait; Cargo `1m 19s`.
- Actual unit termination: `Result=success`, `SubState=exited`, `ExecMainCode=1` (exited), `ExecMainStatus=0`. Launcher exit is not substituted for actual unit exit.
- October 13, 2026, 20:32:00–20:33:20 CDT (October 14, 01:32:00–01:33:20 UTC); peak 4,851,388,416 bytes.
- Retry admitted; recorded working directory `/home/USER/Projects/wow/wow-ui-sim`; command used `--no-run`, `--test integration`, `--no-default-features`, `sound,gui,casc,client-ptr`.

Evidence files:

- `verification/widget-discrete-progress-current/crafted-quality-ptr-api-red-retry1-20261014T013200049067Z/compile.stdout`
- `verification/widget-discrete-progress-current/crafted-quality-ptr-api-red-retry1-20261014T013200049067Z/compile.stderr`
- `verification/widget-discrete-progress-current/crafted-quality-ptr-api-red-retry1-20261014T013200049067Z/compile-terminal.txt`
- `verification/widget-discrete-progress-current/crafted-quality-ptr-api-red-retry1-20261014T013200049067Z/compile-result.json`
- `verification/widget-discrete-progress-current/crafted-quality-ptr-api-red-retry1-20261014T013200049067Z/submission.json`
- `verification/widget-discrete-progress-current/crafted-quality-ptr-api-red-retry1-20261014T013200049067Z/admission.json`
- `verification/widget-discrete-progress-current/crafted-quality-ptr-api-red-retry1-20261014T013200049067Z/revision.json`

## New sealed copy

Path: `verification/widget-discrete-progress-current/crafted-quality-ptr-api-red-retry1-20261014T013200049067Z/artifacts/deps/integration-020202567c2bd5a4`

Seal: `verification/widget-discrete-progress-current/crafted-quality-ptr-api-red-retry1-20261014T013200049067Z/seal-integration.json`

Receipt: `verification/widget-discrete-progress-current/crafted-quality-ptr-api-red-retry1-20261014T013200049067Z/receipt-integration.json`

SHA-256: `ab32a4174443cdc6efe7399d76eb49dd7c599e17ac055756a1570e96a0c42229`

Receipt absent before authentication. One complete file hash pass (`hash_reads=1`); pre/post device, inode, size, and mtime match seal. Computed SHA-256 matches seal. Receipt created exclusively, mode 0600, fsynced, read back exactly; parent directory fsynced. Normal build executable was not read or authenticated.

## Prior denial retained

`verification/widget-discrete-progress-current/crafted-quality-ptr-api-red-20261014T012904034210Z`: admission `allowed=false`, submission absent. Prior 01:29:04 UTC denial remains separate from admitted 01:32:00 UTC retry. No launch performed during verification.

## Limits

No binaries executed; no build/test/check/formatter reruns, services, delegation, Bash, or cwd tools. No old binaries, libraries, archives, source trees, or external cache/settings authenticated. Runtime test result remains unverified.

Proof: `verification/crafted-quality-ptr-api-red-auth-current/proof.json`
