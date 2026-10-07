# Patch 11.1.7 publication sweep

## Contract

Probe all 48 inventory occurrences from page 628473, revision 6726774 in unmodified cached Game UI. Default retail carries 12.1.0; no 11.x feature exists. Apply later 11.2.0, 11.2.5, 11.2.7, 12.0.0, 12.0.1, 12.0.5, 12.0.7 and 12.1.0 registers chronologically. Latest add/remove wins; changes preserve publication. Preserve source direction and supersession IDs.

Shared publication sweep requires exact reviewed gap IDs. P1117_SWEEP_OUT writes all observations before assertion; P1117_SWEEP_REGISTER permits a full same-sized negative control. Publication/absence only, not signature, output, security, behavior or native parity. Cached Blizzard deprecated wrappers remain intact.

## Acceptance

- [x] Review all gaps and fix bounded model-backed defects.
- [x] Retain exact fixture, non-inventory extract and exhaustive page ledger.
- [x] Run nine isolated publication sweeps and one-row negative control.
- [x] Run new behavioral tests, formatting, Mists test check and startup [].

## Bounded fixes

Two removed C_Debug members were registered unconditionally. Move the retained pre-retirement implementation under src/c_api/c_debug.rs and register only outside retail-12-0-0; mark removed keys to prevent namespace autostub fabrication. Never remove cached Blizzard deprecated wrappers. Existing legacy test runs only before retirement. Two documented graphics commands join the existing typed console catalog, not CVar storage; execution and native metadata remain unmodeled.

Two behavioral tests reproduce all four defects before implementation. An earlier assisted-slot candidate did not compile: AssistedCombatState contains only next_cast_spell_id, not an assisted action spell identity. Next-cast recommendation is not the rotation action spell; defer that producer rather than fabricate it.

## Local proof

Runtime/test revision `210e4e23c`; later changes are evidence/docs/validation only. [Proof ledger](../../data/patch-api/evidence/11.1.7-session-2026-10-06/p1117-proof.json) preserves exact commands, revisions, initial RED and withdrawn compile-failing candidate. No independent/native acceptance.

| Isolated sweep | Rows | OK | Exact gaps | Result |
|---|---:|---:|---:|---|
| 11.1.7 | 48 | 40 | 8 | PASS |
| 11.2.0 | 162 | 136 | 26 | PASS |
| 11.2.5 | 163 | 118 | 45 | PASS |
| 11.2.7 | 508 | 414 | 94 | PASS |
| 12.0.0 | 1010 | 987 | 23 | PASS |
| 12.0.1 | 225 | 222 | 3 | PASS |
| 12.0.5 | 363 | 352 | 11 | PASS |
| 12.0.7 | 174 | 171 | 3 | PASS |
| 12.1.0 | 778 | 773 | 5 | PASS |

Each sweep runs alone, local debug retail, one filter/process, `--nocapture --test-threads=1`. Eight later known-gap fixtures remain byte-identical to starting master. Seven observation maps equal older 11.2.0 evidence; 11.2.7 changes are exactly 27 closures already present on starting master, reconciled against its retained follow-up outcomes. Eight existing registers regenerate byte-identically; 32 existing source/register/coverage/fixture files remain unchanged.

Negative control changes only C_ActionBar.ForceUpdateAction added → removed: exit 101, exactly one new gap and no resolutions (8 → 9). Two behavioral tests pass after RED. Eleven extractor/register fixtures and extract reproduction pass. `cargo fmt` and `cargo fmt --check` pass. `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` passes with zero non-vendor warnings; six pre-existing iced manifest warnings plus summary remain unsuppressed. Separate default retail build succeeds; `timeout 90 target/debug/wow-sim --no-addons --no-saved-vars lua-errors` exits 0 with `[]`, CLEAN, zero unique errors/occurrences. Changed Rust lines manually reviewed for readability.

Coverage: 48 inventory + 27 extract = 75 unique IDs; 28 partial-development-green, five bounded-coverage, 24 audit-pending, 18 metadata-only. Seven later reversals receive metadata-only status, no historical credit. Five unsuperseded removals establish strict absence/registration rejection; zero cached-alias acceptances. Sixteen extract candidates remain pending, eleven editorial rows are metadata-only. No historical 11.1.7/native parity or graphics command execution claim.

[Per-ID review](../../data/patch-api/evidence/11.1.7-session-2026-10-06/p1117-gap-review.json) covers all 12 initial gaps: four bounded closures, eight missing producer/policy contracts. [Scout](../../data/patch-api/evidence/11.1.7-session-2026-10-06/p1117-extract-scout.md) assigns every non-inventory ID once to summary (3), enums (7), DTOs (6) or editorial context (11). [Validator](../../data/patch-api/evidence/11.1.7-session-2026-10-06/p1117-validate.py) checks source hashes, chronological expectations, exact gaps, negative control, ledger credit, complete allocation, preserved inputs and revision-scoped proof. Local Cargo logs are ignored artifacts. Requested evidence path keeps the October 6 name; provenance records retrieval October 7, 2026; host clock stamped October 6.
