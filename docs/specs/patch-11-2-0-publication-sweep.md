# Patch 11.2.0 publication sweep

## Contract

Probe all 162 consolidated inventory occurrences from page 636685, revision 6726773, in unmodified cached Game UI. Publication/absence only; no signatures, outputs, security, behavior or native parity claim. No 11.x Cargo feature exists. Default retail carries 12.1.0; apply every later register chronologically: 11.2.5, 11.2.7, 12.0.0, 12.0.1, 12.0.5, 12.0.7, 12.1.0. Latest later add/remove wins; changed rows preserve publication. Retain source direction and supersession IDs.

Use `tests/common/publication_sweep.rs`; exact observed gap IDs equal `tests/data/patch_11_2_0_sweep_known_gaps.json`. `P1120_SWEEP_OUT` writes all observations before assertion; `P1120_SWEEP_REGISTER` accepts a full same-sized negative-control register. One altered unsuperseded row must add exactly one gap. Retail-only file gating. No Blizzard/vendor modifications, fabricated models, catalog shop changes or 11.2.7 fixture edits.

## Acceptance

- [x] Review every non-OK row; fix cheap root causes, retain exact gaps.
- [x] Exact 26-gap fixture and one-row negative control (26 → 27).
- [x] Unique 244-ID inventory/extract ledger and exhaustive ranked scout.
- [x] Eight isolated local debug retail sweeps; later observations unchanged.
- [x] Formatting, retail check, Mists test check without non-vendor warnings, startup `[]`.

## Bounded simulator fixes

Initial cached sweep: 124 OK / 38 gaps. Twelve rows close through existing simulator machinery: register the live player combat field reader; read FontString gradient slot 0; add four documented zero fog defaults; mark five removed namespace members; exclude Browser NavigateTo on supported retail epochs. New helpers/registrations use `retail-12-0-0`; classic contracts stay unchanged. Gradient cleared/default `(0, 0)` is inferred simulator policy, not native proof. Five new behavioral tests reproduce all initial defect groups, then pass. Final per-direction review identifies removed C_Bank.FetchNextPurchasableBankTabCost as another namespace-autostub retirement, not a missing bank producer; its added repeated-lookup assertion fails first, then passes.

Retained gaps: 24 explicitly missing namespace producers and two global-retirement/deprecation-attribution defects. GetLootMethod retains the simulator's legacy string/zero-index contract while C_PartyInfo.GetLootMethod returns numeric/nil-index results; migration needs separate consumer proof. SendChatMessage is registered directly and republished after cached chat activation, replacing a deprecated forwarding wrapper; cold-environment and cached post-load chat behavior need lifecycle proof. Neither is disguised as native compatibility or an accepted deprecation alias.

Page-specific extract seeding recognizes five Deprecated resource links and the 11.1.7→11.2.0 comparison as editorial. Nested StaticPopup advice stays pending. No non-inventory behavior credit.

## Local proof — October 6, 2026

Final runtime/test revision `2bd77f4d4`; subsequent changes only evidence/docs/validation. [Proof ledger](../../data/patch-api/evidence/11.2.0-session-2026-10-06/p1120-proof.json) retains exact commands, revisions, failures, invalidations and local log paths. Initial RED, extractor RED and intermediate Rust borrow-check failure remain visible. No agents/models/native acceptance.

| Isolated sweep | Rows | OK | Exact gaps | Result |
|---|---:|---:|---:|---|
| 11.2.0 | 162 | 136 | 26 | PASS |
| 11.2.5 | 163 | 118 | 45 | PASS |
| 11.2.7 | 508 | 387 | 121 | PASS |
| 12.0.0 | 1010 | 987 | 23 | PASS |
| 12.0.1 | 225 | 222 | 3 | PASS |
| 12.0.5 | 363 | 352 | 11 | PASS |
| 12.0.7 | 174 | 171 | 3 | PASS |
| 12.1.0 | 778 | 773 | 5 | PASS |

Each sweep runs alone, local debug retail, one filter/process, `--nocapture --test-threads=1`. Seven later observation maps match retained 11.2.5 proof exactly. All seven existing registers regenerate byte-identically. Twenty-eight existing ledgers/registers/raw sources/gap fixtures remain unchanged against starting master. Four source expectations reverse through later supersession, all OK; no CVar default mismatches.

Negative control flips only C_ChallengeMode.GetLeaverPenaltyWarningTimeLeft added → removed: expected exit 101, exactly one changed observation/new gap and none resolved. Five bounded behavioral tests, eight extractor fixtures and two register fixtures pass. Format and retail check pass. Mists test check passes with zero errors/non-vendor warnings; six pre-existing iced manifest deprecations plus vendor summary remain unsuppressed. Later bank change lies entirely in Mists-excluded code, so that proof remains valid without redundant compilation. Changed Rust lines manually audited for readability.

Separate local debug build succeeds; bounded 90-second startup run exits 0 with `[]`, CLEAN, zero unique errors/occurrences. No release, extra target directory, vendor edit, Blizzard monkey-patch, other worktree edit, push, merge or delegation.

Coverage: 162 inventory + 82 extract = 244 unique IDs; 75 partial-development-green, 57 bounded-coverage, 91 audit-pending, 21 metadata-only. Of 57 successful unsuperseded removals, 46 establish absence/registration rejection and 11 accept exact cached deprecated wrappers/aliases, not strict raw absence. Extract: 65 pending / 17 editorial; scout batches 4, 8, 34, 9 and 10 rows.

[Artifact validator](../../data/patch-api/evidence/11.2.0-session-2026-10-06/p1120-validate.py) checks hashes, source identities, chronological expectations, exact gaps, reviewed outcomes, negative control, ledger credit, complete scout allocation and revision-scoped proof. Local build/check logs are ignored artifacts, referenced by the retained proof ledger.
