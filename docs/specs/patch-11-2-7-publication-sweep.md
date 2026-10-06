# Patch 11.2.7 publication sweep

## Contract

Probe all 508 consolidated inventory occurrences from page 649551, retained revision 6726771, in the unmodified cached Game environment. Publication/absence only: no signature, output, security, behavior or native parity credit. Preserve raw wikitext and source provenance. Regenerate all five existing registers byte-identically.

Use `tests/common/publication_sweep.rs`. Apply 12.0.0, 12.0.1, 12.0.5, 12.0.7 and 12.1.0 registers chronologically. Latest later add/remove wins; changed rows do not reverse publication. Source direction remains recorded alongside effective expectation and supersession ID. The shared implementation compares symbol keys, not the starting register's position; an older-than-all register requires no index special case.

Require the exact observed non-OK ID set to equal `tests/data/patch_11_2_7_sweep_known_gaps.json`. `P1127_SWEEP_OUT` writes observations before assertion; `P1127_SWEEP_REGISTER` accepts a full scratch register of the same size. A one-row negative control must add exactly one gap.

## Epoch and scope

Earliest supported retail epoch is `retail-12-0-0`; no 11.x runtime exists. Default retail carries 12.1.0. Unsuperseded 11.2.7 removals are baseline absence for every supported **retail epoch**, not every client profile: classic clients have independent historical contracts. Any publication fix must preserve those profile boundaries. No new 11.x feature, fabricated backing model, vendor rewrite or Blizzard monkey-patch is authorized.

## Acceptance

- [x] Review every non-OK row; fix cheap root-cause publication defects and retain explicit model gaps.
- [x] Exact known-gap GREEN and one-row negative control.
- [x] Create unique inventory/extract source-ID ledger with publication-only credit.
- [x] Scout every non-inventory statement without claiming behavioral parity.
- [x] Run all six publication sweeps alone, local debug retail.
- [x] Formatting, default local check, warning-free non-vendor Mists test check, startup Lua errors `[]`.

## Sources

- `data/patch-api/sources/11.2.7-api-changes.wikitext` and provenance.
- `data/patch-api/sources/11.2.7-wikitext-register.json`.
- [Session evidence](../../data/patch-api/evidence/11.2.7-session-2026-10-06/).

## Retirement fixes

Initial cached sweep exposes explicit global `JoinBattlefield` and fabricated lookup members `C_ReturningPlayerUI.AcceptPrompt` / `DeclinePrompt` after removal. Stop the legacy global publisher from `retail-12-0-0`; mark both namespace keys removed in the already retail-gated retirement module. Preserve classic publication and the existing state-backed `C_PvP.JoinBattlefield` queue producer. Regression asserts repeated raw/ordinary absence and concrete successor queue state. Existing queue tests use the live namespace successor rather than the retired global. No new backing model or fallback.

The same fabricated-member defect affects removed `C_CharacterServices.RPEResetCharacter`. Extend the retirement list and repeated-lookup regression; no character reset behavior is invented. All four initial unsuperseded removal defects now have producer/lookup fixes.

Ten missing 11.2.7 Command records are a catalog omission, not a missing execution model. Publish their exact source names/types on supported retail epochs using `c_api::c_console`; classic catalogs remain unchanged. Empty native metadata remains the existing documented inference. Concrete command RED observed zero of ten records.

## Extract compatibility

Validation exposed an unintended 12.0.7 extractor change: resuming at Structures on headed pages now parsed templates the old tool excluded. Restrict wikitable/Structures boundary handling to pages without a Global API heading; preserve all existing headed-page rendering, including pre-existing unsupported-template outcomes. New RED fixture proves the old headed inventory exclusion. This Python-only correction leaves Rust sweep/check/startup proof valid.

## Final local proof — 2026-10-06

Rust/data runtime revision `9cc60a03dc2dc2b51ff2675b00f9e5688228b0e5`; later evidence/docs and extractor-only changes do not invalidate it. No agents/models or independent/native acceptance.

| Isolated sweep | Rows | OK | Exact gaps | Result |
|---|---:|---:|---:|---|
| 11.2.7 | 508 | 387 | 121 | PASS |
| 12.0.0 | 1010 | 987 | 23 | PASS |
| 12.0.1 | 225 | 222 | 3 | PASS |
| 12.0.5 | 363 | 352 | 11 | PASS |
| 12.0.7 | 174 | 171 | 3 | PASS |
| 12.1.0 | 778 | 773 | 5 | PASS |

All six use local debug retail, one filter per process, `--nocapture --test-threads=1`. All five later baseline files remain byte-identical to base `ba5170301`. Results and exact commands are in [proof ledger](../../data/patch-api/evidence/11.2.7-session-2026-10-06/p1127-proof.json).

Initial 508-row sweep: 373 OK / 135 non-OK. Per-ID review closes four removal defects and ten catalog omissions; 121 retained gaps: 110 namespace members, six globals, two acquisition failures, two 3D method gaps, one later-superseded event removal. All nine source removals now observe baseline absence. Fifty-one effective expectations are superseded, 50 OK. No page CVar default mismatches were reported; this remains publication-only proof.

Negative control flips `C_BattleNet.InstallHighResTextures` added → removed: 121 → 122 gaps, exactly one new ID and one changed observation; expected exit 101. Retirement/catalog regressions 2 PASS; battlefield regressions 9 PASS. Register-parser fixtures 2 PASS; extractor fixtures 6 PASS. Existing extractor outcomes match base for all five later pages, including the already unsupported 12.1.0 template outcome. Existing extracted files were never rewritten.

Artifact validation PASS: 508 inventory + 19 extract = 527 unique IDs; ten header counts exact; source hashes, gap sets, all-later supersession expectations and all 19 scout assignments valid. Ledger: 328 partial-development-green / 9 bounded-coverage / 131 audit-pending / 59 metadata-only. All five existing coverage ledgers unchanged; all five existing registers regenerate byte-identically.

`cargo fmt --manifest-path /home/osso/.worktrees/wow-ui-sim-p1127-page/Cargo.toml -- --check` PASS. Local helper `--check` PASS. `cargo check --tests --no-default-features --features "sound gui casc client-mists"` PASS, no errors or non-vendor warnings. Six pre-existing iced manifest-key deprecations plus their vendor summary match initial builds; no suppression/vendor edit. Changed Rust manually audited for readability/profile scope.

Startup built separately with no timeout; `timeout 90 python3 /home/osso/.worktrees/wow-ui-sim-p1127-page/scripts/build-host.py --build-host local --no-build --run -- --no-addons --no-saved-vars lua-errors` exits 0, JSON `[]`, stderr CLEAN / zero unique and zero occurrences. Helper artifact announcement is separate from Lua-error JSON. No extra target directory or release build.

## Gap closure follow-up — 2026-10-06

Neighborhood invitation getters/setters reuse existing per-environment boolean state, now published on Retail 12.0.0+ as well as Forever. Classic publication unchanged. INFERRED initial false; documented omitted setter resets false. Behavioral RED: nil global. GREEN pending after commit. No persistence or notification event invented.
