# Initial crafted-quality public-copy verification

**OVERALL: PASS** — bounded public-copy/privacy/link/count/scope gate at `c652aaa8824f87089f0747a3ad0e19a4dbe3350e`. No feature GREEN claim.

Public root: `data/patch-api/evidence/crafted-item-quality-initial-api-red-2026-10-13`.
Authority: `/home/USER/.local/state/wow-ui-sim/handoff/crafted-initial-api-red-retention-final-stage-current/manifest.json` and `/home/USER/.local/state/wow-ui-sim/handoff/crafted-initial-api-red-retention-final-stage-current/private-mappings.json`.

## Copy, membership and wiring
- 60/60 whole original UTF-8 texts match 14 ordered literal replacements and terminal LF only; 60/60 original and public manifest size/SHA-256 checks pass. Selected JSON was not reserialized or clipped.
- Exactly 62 current tracked public files: 60 selected texts + README + publication metadata. Every current Git blob/size matches; metadata has exactly 60 unique entries with matching sizes/hashes.
- 60 unique original paths: 58 under old-stage `files/.originals/` (one component), 2 independent PTR originals at authority-designated verification paths. No fabricated all-60-old-stage assertion.
- README:12 links to existing `docs/specs/crafted-item-quality-info.md#initial-crafted-api-red-checkpoint`; spec:42 links back to existing README. All five Markdown links checked. README and owning spec read completely. Pending public-copy labels are retained historical publication state, not a feature acceptance label.

## Privacy
PASS: no raw account/suffix-family, email, realm/character identity, identifying home path, raw systemd unit or 32-hex invocation residual observed in all 62 public files; no six-digit account pattern observed. All 14 old literals absent. `/home/USER` and `retained-unit-NN.service` are placeholders. Cargo package IDs and the Markdown checkpoint anchor are classified by actual context. `0#iced-layout-inspector@0.1.0` and `0#peercred-ipc@0.1.0` are Cargo package-ID fragments (two each), not accounts. Slash pairs are actual paths/URLs or technical labels such as Retail/PTR and omitted/nil, not blanket-redacted. Exact candidate counts and classifications in proof.json.

## Exact observed API FAILs
Two distinct test names × two profiles = four failed outcomes. Each 0 PASS / 1 FAIL, exit 101. These are expected retained RED evidence, not failures of this copy gate.

### PTR — `crafted_quality_api::crafted_quality_required_item_info_errors_have_context`
- Source `9cc2112267ea139e3e13408a16787edd53c232dd`; exit 101; 0 passed / 1 failed.
- `data/patch-api/evidence/crafted-item-quality-initial-api-red-2026-10-13/widget-discrete-progress-current/crafted-quality-ptr-api-red-retry1-20261014T013200049067Z/runtime-crafted-required-api-red-2g/stderr:13`: `thread 'crafted_quality_api::crafted_quality_required_item_info_errors_have_context' (2936192) panicked at tests/crafted_quality_api.rs:27:6:`
- `data/patch-api/evidence/crafted-item-quality-initial-api-red-2026-10-13/widget-discrete-progress-current/crafted-quality-ptr-api-red-retry1-20261014T013200049067Z/runtime-crafted-required-api-red-2g/stderr:14`: `public omitted/nil calls must reach contextual ItemInfo validation: Lua(Runtime(RuntimeError { message: "required ItemInfo must reject omitted/nil input", level: 0, traceback: [] }))`
- Exact stdout: `data/patch-api/evidence/crafted-item-quality-initial-api-red-2026-10-13/widget-discrete-progress-current/crafted-quality-ptr-api-red-retry1-20261014T013200049067Z/runtime-crafted-required-api-red-2g/stdout`; result: `data/patch-api/evidence/crafted-item-quality-initial-api-red-2026-10-13/widget-discrete-progress-current/crafted-quality-ptr-api-red-retry1-20261014T013200049067Z/runtime-crafted-required-api-red-2g/result.json`.
- Reached: first omitted-input public call returned normally. Unreached: explicit nil second call and contextual checks. Return payload unknown.

### PTR — `crafted_quality_api::crafted_quality_secret_nil_authenticates_before_required_validation`
- Source `9cc2112267ea139e3e13408a16787edd53c232dd`; exit 101; 0 passed / 1 failed.
- `data/patch-api/evidence/crafted-item-quality-initial-api-red-2026-10-13/widget-discrete-progress-current/crafted-quality-ptr-api-red-retry1-20261014T013200049067Z/runtime-crafted-secret-api-red-2g/stderr:13`: `thread 'crafted_quality_api::crafted_quality_secret_nil_authenticates_before_required_validation' (2936220) panicked at tests/crafted_quality_api.rs:72:6:`
- `data/patch-api/evidence/crafted-item-quality-initial-api-red-2026-10-13/widget-discrete-progress-current/crafted-quality-ptr-api-red-retry1-20261014T013200049067Z/runtime-crafted-secret-api-red-2g/stderr:14`: `AllowedWhenUntainted must authenticate real VM secret nil before parsing: Lua(Runtime(RuntimeError { message: "untainted secret nil must reach required-input validation", level: 0, traceback: [] }))`
- Exact stdout: `data/patch-api/evidence/crafted-item-quality-initial-api-red-2026-10-13/widget-discrete-progress-current/crafted-quality-ptr-api-red-retry1-20261014T013200049067Z/runtime-crafted-secret-api-red-2g/stdout`; result: `data/patch-api/evidence/crafted-item-quality-initial-api-red-2026-10-13/widget-discrete-progress-current/crafted-quality-ptr-api-red-retry1-20261014T013200049067Z/runtime-crafted-secret-api-red-2g/result.json`.
- Reached: genuine VM-secret-nil setup passed; first untainted public call returned normally. Unreached: contextual checks; tainted denial; wrapper preservation; taint restoration/recovery; subsequent validation/unwrap. Return payload unknown.

### RETAIL — `crafted_quality_api::crafted_quality_required_item_info_errors_have_context`
- Source `8b46433a4568e26458a15907ea79636e38ff65f9`; exit 101; 0 passed / 1 failed.
- `data/patch-api/evidence/crafted-item-quality-initial-api-red-2026-10-13/widget-discrete-progress-current/crafted-quality-retail-api-red-retry1-20261014T012534611408Z/runtime-crafted-required-api-red-2g-retry1/stderr:13`: `thread 'crafted_quality_api::crafted_quality_required_item_info_errors_have_context' (2914571) panicked at tests/crafted_quality_api.rs:27:6:`
- `data/patch-api/evidence/crafted-item-quality-initial-api-red-2026-10-13/widget-discrete-progress-current/crafted-quality-retail-api-red-retry1-20261014T012534611408Z/runtime-crafted-required-api-red-2g-retry1/stderr:14`: `public omitted/nil calls must reach contextual ItemInfo validation: Lua(Runtime(RuntimeError { message: "required ItemInfo must reject omitted/nil input", level: 0, traceback: [] }))`
- Exact stdout: `data/patch-api/evidence/crafted-item-quality-initial-api-red-2026-10-13/widget-discrete-progress-current/crafted-quality-retail-api-red-retry1-20261014T012534611408Z/runtime-crafted-required-api-red-2g-retry1/stdout`; result: `data/patch-api/evidence/crafted-item-quality-initial-api-red-2026-10-13/widget-discrete-progress-current/crafted-quality-retail-api-red-retry1-20261014T012534611408Z/runtime-crafted-required-api-red-2g-retry1/result.json`.
- Reached: first omitted-input public call returned normally. Unreached: explicit nil second call and contextual checks. Return payload unknown.

### RETAIL — `crafted_quality_api::crafted_quality_secret_nil_authenticates_before_required_validation`
- Source `8b46433a4568e26458a15907ea79636e38ff65f9`; exit 101; 0 passed / 1 failed.
- `data/patch-api/evidence/crafted-item-quality-initial-api-red-2026-10-13/widget-discrete-progress-current/crafted-quality-retail-api-red-retry1-20261014T012534611408Z/runtime-crafted-secret-api-red-2g-retry1/stderr:13`: `thread 'crafted_quality_api::crafted_quality_secret_nil_authenticates_before_required_validation' (2914613) panicked at tests/crafted_quality_api.rs:72:6:`
- `data/patch-api/evidence/crafted-item-quality-initial-api-red-2026-10-13/widget-discrete-progress-current/crafted-quality-retail-api-red-retry1-20261014T012534611408Z/runtime-crafted-secret-api-red-2g-retry1/stderr:14`: `AllowedWhenUntainted must authenticate real VM secret nil before parsing: Lua(Runtime(RuntimeError { message: "untainted secret nil must reach required-input validation", level: 0, traceback: [] }))`
- Exact stdout: `data/patch-api/evidence/crafted-item-quality-initial-api-red-2026-10-13/widget-discrete-progress-current/crafted-quality-retail-api-red-retry1-20261014T012534611408Z/runtime-crafted-secret-api-red-2g-retry1/stdout`; result: `data/patch-api/evidence/crafted-item-quality-initial-api-red-2026-10-13/widget-discrete-progress-current/crafted-quality-retail-api-red-retry1-20261014T012534611408Z/runtime-crafted-secret-api-red-2g-retry1/result.json`.
- Reached: genuine VM-secret-nil setup passed; first untainted public call returned normally. Unreached: contextual checks; tainted denial; wrapper preservation; taint restoration/recovery; subsequent validation/unwrap. Return payload unknown.

## Scope and build context
Retail first source `8b46433a4`; PTR first source `9cc211226`. Saved Git comparisons establish the initial test file unchanged at current publication commit (both exit 0, empty diff). Later doc/publication context and passive/positive stage `6c38a827` do not replace first-initial evidence. Historical unexecuted labels in source-only reports do not override later actual runs. No production/runtime-source equality or fresh authentication claimed.

Each raw build stdout has 741 JSON records: 665 compiler-artifact, 75 build-script-executed, 1 successful build-finished, zero compiler-message diagnostics. Each compile stderr retains 6 manifest deprecations; cache-lock waits Retail 3 / PTR 1. Initial Retail/PTR compile admissions and two prior Retail runtime admissions were denied: four zero-launch records distinct from four executed API assertion failures.

No positive-data, getter, native, whole-consumer, explicit-nil/context, tainted-denial or recovery promotion. Normal first-call returns establish no throw, not a particular returned payload.

## Gate failures
None.

## Artifact handling
Only new `verification/crafted-quality-initial-copy-current/report.md` and `verification/crafted-quality-initial-copy-current/proof.json` written; exclusive creation, mode0600, file and directory fsync, full-byte readback. No runtime/build/test/check/fmt reruns, services, delegation, Bash, cwd tools or canonical edits. No Bullet300/initial Bullet26 packet, binary/library/archive or vendor-content access; no source-tree authentication.
