# Patch 8.1.0 publication sweep

Audit pinned Warcraft Wiki page 464337, revision 4462737 against current retail, not historical reconstruction. [Audit](../wiki/investigations/patch-8-1-0-api-audit.md) records models, exact gaps and proof.

## What it must do

- [x] Refetch and pin raw source, provenance and fetch receipt; retain every API/extract occurrence and both rename identities. Preserve CVar header counts and build/citation context.
- [x] Reuse the 8.1.5 opt-in bullet parser without a competing implementation; keep rename and CVar-table additions separately opt-in. Every formerly reproducible saved artifact must reproduce.
- [x] Enforce exact reviewed publication gaps through a prefork sweep, with integrated 8.1.5 then 8.2.0 registers followed by 8.2.5 and later master registers.
- [x] Implement supplied civil calendar comparison with documented rhs-relative sign, chronological component ordering, weekday independence and no mutation/clock defaults.
- [x] Retire only proven unused retail members; retain current cached consumers and live simulator provider callers. Preserve Mists paths and unmodified Blizzard Lua.
- [x] Retain complete whole-word qualified/bare cached and source/test scans plus later re-addition checks, including the integrated 8.1.5/8.2.0 registers.
- [x] Record passing targeted sweep/area/parser/reproduction/Mists/format gates and artifact acceptance, without a full integration suite.
- [x] Validator must work after merge and later audits: no absolute checkout assertions; pin the complete historical register/sweep set; preserve original before-hashes and accept only exact attributable later-audit replacements; counts derive from files.
- [x] Separate frozen145/87/58 results and two removal supersessions from old533c145/93/52/six-producer evidence; original validation9/9 passed.
- [x] New GUID cycle uses its own direct145-row capture and exact51-gap/seven-producer set; verified strict publication and retry validator reject old-map substitution, missing captures, stale/unreviewed closures and rewritten history while preserving both source captures.

## Current gap reconciliation — 2026-10-14

The saved `983f02fbf` default prefork sweep failed only on resolved/stale entries for `C_UIWidgetManager.GetSpellDisplayVisualizationInfo`; no new gap was reported. Reconcile the current known-gap fixture (50 → 49) with the already-modeled [SpellDisplay contract](ui-widget-spell-display.md). Source rows, IDs, later-register ordering and exact set equality remain unchanged. Historical reviewed captures/counts remain historical, not rewritten. This closes only obsolete missing-endpoint expectations; native/signature/payload parity is not inferred. The original default Retail prefork body passes at compiled`1b94ffff3`; [bounded repair matrix](../wiki/investigations/fullsuite-983f02-2026-10-14.md#bounded-repair-follow-up) owns the independent proof and retained diagnostics. Historical broad FAIL is unchanged.

## How it works

- [Page audit, coverage matrix and proof boundaries](../wiki/investigations/patch-8-1-0-api-audit.md).
- [Exact gap review](../../data/patch-api/evidence/8.1.0-session-2026-10-08/p810-gap-review.json).

## Implementation inventory

- `data/patch-api/sources/8.1.0-*` — pinned source, extract, register and occurrence ledger.
- `src/c_api/c_date_and_time.rs` — supplied civil-time value model and comparison.
- `src/c_api/patch_retired_members.rs` — retail-only three-member list.
- `tests/patch_8_1_0_*.rs`, `tests/data/patch_8_1_0_sweep_known_gaps.json` — bare/cached/classic behavior and exact publication gap enforcement.
- `data/patch-api/evidence/8.1.0-session-2026-10-08/` — exhaustive scans, frozen gap review/results, reproduction, validator and command receipts. Validator separates pinned historical fixtures from explicit current modeled-producer closures.
- `data/patch-api/evidence/publication-533c-reconciliation-2026-10-12/current-publication-results.json` — immutable old533c145/93/52 capture.
- `data/patch-api/evidence/guid-is-player-current-2026-10-12/current-publication-results.json` — required new GUID-cycle direct capture; never substituted by old/frozen observations.

## Tests asserting this spec

- `tests/patch_8_1_0_publication_sweep.rs`.
- `tests/patch_8_1_0_calendar_compare.rs`.
- `tests/patch_8_1_0_publication_fixes.rs`, `tests/patch_8_1_0_cached_surfaces.rs`.
- `tools/test_extract_patch_non_inventory.py`, `tools/test_gen_patch_wikitext_register.py`.
- Evidence `validate.py` and recorded scoped calendar/map/date-provider/configuration-provider/Mists gates.
- `tools/test_patch_8_1_0_validation.py` — historical/current acceptance and concrete rejection of rewritten frozen data, missing current capture, stale/unreviewed closures and invalid current observations. Fixture writes must leave an existing source capture byte-exact; isolation regression reproduced safely in scratch storage before excluding the current packet directory. Independent2879 original9/9 tests and real-validator exit0 remain historical. Main-verified `d21ca3f64` retry9/9 and actual validator exit0 establish new GUID-cycle path/count/negative controls and both-capture isolation. Earlier8/9FAIL/standalone NOT RUN remains history; this validator proof alone gives no native/full-suite credit. Later saved broad PASS is separate below.

## Current reconciliation — 2026-11-04

Remove only `C_PetJournal.GetPetInfoByItemID-82` from the current expected gaps (41 → 40). Producer `9c773cad6` registers the Retail getter and reads the generated pet-item catalog; see [pet-item contract](pet-info-by-item.md). The row tests publication, not complete output or native parity. Earlier cycle counts below are historical; the current fixture owns the remaining gaps.

## Known gaps (current cycle)

- [ ] Frozen audit's 58 exact publication gaps retain historical per-ID model, identity, metadata, policy, request-lifecycle or protected-provider reasons. Current producer closures are recorded separately, not retroactively applied to that snapshot.
- [ ] The GUID-cycle snapshot had 51 publication gaps; old533c52 gaps remain historical. New GUID cycle initial strictFAIL/exit1 solely GUID83 stale expectation and wrong101 worker guard remain failures. Main-verified `a555f927a` post-reconciliation strictPASS and `d21ca3f64` retry validatorPASS establish the separate current seven-closure set; frozen87/58 and two supersessions unchanged. Publication-before fixture-race/coherenceFAIL and earlier8/9validatorFAIL/standalone NOT RUN retained. [GUID packet](../../data/patch-api/evidence/guid-is-player-current-2026-10-12/README.md) owns details; independent2939 retention/link/privacy PASS at19988. Previously: no signature/output/security/native parity inferred. `533c086e0` failed only two stale expectations, now removed at61cd81963; independent2877 original-prefork case PASS145/93/52. Independent2879 validator9/9 tests and real-validator exit0, source capture byte-exact. Saved suite FAIL, initial validator/preflight FAILs and safe isolation RED remain history; strict set comparison and frozen87/58 unchanged.
- [ ] GUID packet retention/link/privacy and collision disclosure independently2939 PASS at19988; original RED JSON recovery remains NOT_RECOVERED, reconstruction not byte-exact. Saved d21 broad suite PASS, including original strict publication; [broad packet](../../data/test-perf/evidence/fullsuite-d21-2026-10-12/README.md) owns counts/raw result/log/proof. New packet retention verified by independent2944 checks/2945 semantic correction (original word-only checker FAIL preserved/rejected as noncontract), distinct from unchanged GUID independent2939 PASS; not a replacement PASS for533cFAIL/BA3FAIL. GUI authenticated-user review, native/other-profile/external acceptance, credits, Pi provenance and full handoff remain OPEN. No successful body-timing/cache-content/historical-equivalence/causal credit.
- [ ] Two literal `?` source statements specify no implementable contract; retained as uncertainty, never behavioral credit.
- [ ] Current publication is not signature, populated-output, payload, security or historical parity proof. Calendar invalid civil-date/native security parity remains unverified.
- [ ] Inherited 12.0.5/12.0.7/12.1.0 saved extracts remain non-reproducible; unchanged failure boundaries are recorded.

## Out of scope

No invented API defaults or shim shortcuts, no vendor/cache/Wowless/WowlessData edits, no Blizzard Lua monkey-patching, no other-page implementation, no session-cwd changes, push, merge, agents/models or full integration suite. Host has no WoW install; CASC texture acceptance is not claimed.
