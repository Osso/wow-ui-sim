# Saved Mists currency / observer epoch audit

**Epoch:** `20261010T043019Z`  
**Compiled revision:** `0148437a7868b0a25d0ce2fca504378d4558b5ee`  
**Artifact integrity:** PASS. **Functional result: FAIL (currency remains RED).**

## Saved execution coverage

| Target | Passed | Failed | Exit | Proof |
|---|---:|---:|---:|---|
| mists_currency_list | 1 | 4 | 101 | Five named saved outcomes; complete stdout/stderr privately read |
| mists_compat_bootstrap | 23 | 0 | 0 | Includes Honor singleton negative case |
| mists_dialog_helpers | 2 | 0 | 0 | Includes dialog singleton negative case |

Total: **26 passed, 4 failed**, no ignored or filtered tests. Build exit 0, jobs 12. Independent before/after source-map equality; audited source files match captured hashes. Selected test artifacts match build-end seals and invocation/after-run hashes. All six runtime stream hashes match saved receipts. Cargo features include `client-mists` and exclude other client profiles. No new commands executed against compiled artifacts; no native or prefork claim.

## Currency boundaries — main review controls acceptance

Saved currency target remains **1 PASS / 4 FAIL**, not CurrencyGREEN. Initial report's precise failed-assertion locations, observed field values and successful-name assertions are **withheld**: saved streams lack panic diagnostics. Source-grounded only: list projection omits watched/max fields and adapter emits false/nil. Current vendor source orders a hybrid-scroll dependency before currency lookup; exact runtime failing callee remains unobserved. Neither existing failure proves the intended currency lookup ran.

[Main review](main-review.json) records accepted integrity/counts and withheld claims. Source `2e46082d0` adds bounded diagnostics; fresh `518` diagnostics are separate and not imported. Source518 autohide tests remain unchecked/unexecuted; current CurrencyGREEN is not proven.

## Nil observer negative cases

- **Honor:** named test `mists_honor_frame_shared_reproduces_missing_honor_system_enabled` actually reports `ok` among 23. Hash-matched source `tests/mists_compat_bootstrap.rs:371-400` asserts false call status, nil-call failure text, observer length **exactly 1**, container **_G**, key **HonorSystemEnabled**.
- **Dialog:** named test `money_frame_set_type_reproduces_missing_basic_message_dialog_helper` actually reports `ok` among 2. Hash-matched source `tests/mists_dialog_helpers.rs:37-67` asserts same failure conditions and singleton **_G / SetBasicMessageDialogText**.

Both remove prior observations from measured interval, reset dedupe table during call, restore dedupe and prior observations afterward, and bind handler before call. Passing saved test outcomes establish that failure and singleton assertions executed; this is not merely source presence or a test-count inference. Observation records themselves are not separately dumped in receipts, so no source-line/addon-owner attribution beyond asserted container/key is claimed. Observer writer is wired in `src/lua_api/globals/compat_overrides.rs:277,293-323`, called by `runtime_surface_bootstrap.lua:81-109`.

## Warnings

Current full Cargo JSON parsed: **no `is_enabled` warning**, no compiler-message records. Historical Mists JSON `mists-final-profile-check/20261010T031637Z/compile.stdout` contains **two** diagnostics: function `is_enabled` is never used, source `src/lua_api/handler_timing.rs:12`. Absence now is established for this saved compilation only, not every profile.

**Six external `iced-wgpu-patched/Cargo.toml` deprecated Clippy-key warnings remain** in compile stderr; manifest summary also reports six. Not a warning-free build.

## Preservation and scope

Every epoch file outside `audit/` privately read in full; raw streams/code/payloads not reproduced here. `hashmanifest.json` records original receipts, supporting source hashes, historical evidence, and output hashes. Only requested three audit files written; original receipts and original source/tests untouched. No reruns, builds, tests, polling, delegation, operations, or native/prefork verification.
