# Patch 12.0.1 API page audit

Captured page 659762, revision 6747895; 225 inventory occurrences plus 252 non-inventory lines. Audit machinery is complete; compatibility is not: publication-only proof and pending behavioral contracts remain distinct.

## Source and coverage

Retrieved October 6, 2026. Retained revision is dated June 19, 2026; Blue posts are dated February 19, February 24 and March 21, 2026. Consolidated comparison: 12.0.0 build 65655 → 12.0.1 build 66838, April 3, 2026. No external linked page was expanded. No standalone Deprecated API section exists in this capture.

All eight added/removed inventory header counts match. Existing 12.0.0/12.0.5/12.0.7/12.1.0 registers regenerate byte-identically. Plaintext extraction retains seven enum parents/30 deltas, 21 structure parents/33 deltas, and Blue-post security/data statements. Existing 12.0.0 plaintext remains identical.

| Scope | Rows | Outcome | Proof level |
|---|---:|---|---|
| Inventory | 225 | 222 OK / 3 exact reviewed gaps | Loaded cached Game publication/absence; bounded producer tests, no native parity |
| Later supersession | 16 | All OK; metadata-only | Later add/remove expectations, not historical behavioral proof |
| Extract enum/structure | 91 | Audit-pending | Source and candidate test-body inspection only |
| Extract prose | 110 | Audit-pending | 74 spell-policy/data plus 36 security/cooldown/model statements |
| Extract editorial context | 51 | Metadata-only | Individual source-context rationale |

[New page ledger](../../../data/patch-api/sources/12.0.1-page-coverage.json): **477 unique IDs**, **120 partial-development-green / 38 bounded-coverage / 252 audit-pending / 67 metadata-only**. Added/changed inventory greens mean publication only. Removed inventory greens bound absence or exact cached deprecation fallback attribution; they do not claim successor behavior. Existing page ledgers were not modified.

## Root causes and fixes

Initial sweep observed 170 OK / 55 gaps. Two removed `C_NamePlate` clamping members were raw-absent but ordinary namespace lookup fabricated functions. Two removed `C_CombatAudioAlert` speaker-volume members still had explicit Rust publishers. Retire all four in `src/c_api/patch_retired_members.rs`; stop the explicit audio registrations/functions on the same gate. Existing volume-only tests retain their historical epoch gate; live format/speed tests no longer call removed APIs.

No 12.0.1 feature/epoch exists in `Cargo.toml` or `src/client_profile.rs`. Use `retail-12-0-5`, the first supported later epoch; gating at 12.0.0 would remove APIs from the preceding documented surface. Earlier-epoch preservation is source-reviewed, not separately runtime-tested. Current retail is the 12.1.0 surface, not a newly added 12.0.1 client profile.

Two extractor defects surfaced while retaining Blue posts: broad HTML stripping erased a charge-cooldown comparison; blanket nested-bullet metadata classification discarded contractual formulas. Behavioral fixtures reproduce both boundaries. Fixes preserve operators, qualified API names and nested contractual rows without changing old coverage ledgers or old plaintext.

The initial 12.0.0 fixture grew from 23 to 27 when 12.0.1 supersession reopened minimapTrackedInfov2/useCompactPartyFrames, CHAT_MSG_ENCOUNTER_EVENT and SetPreventSecretValues. The follow-up retires all four from `retail-12-0-5`, restoring the fixture to 23. Strict 12.0.0 publishers remain source-gated; no separate strict-epoch runtime proof is claimed.

## Remaining gaps and scout

All original 51 IDs remain in [gap review](../../../data/patch-api/evidence/12.0.1-session-2026-10-06/p1201-gap-review.json), with baseline/current observations, per-ID outcomes and bounded behavioral filters. Forty-eight now pass publication. Three remain: `C_CatalogShop.BulkRefundDecors` lacks a refundable-decor transaction/service model; `C_Secrets.ShouldUnitThreatValuesBeSecret` lacks numeric threat output/secrecy state; `NamePlate:SetStackingBoundsFrame` cannot be acquired through the intentionally unsupported 3D nameplate subsystem. No no-op refund, invented secrecy predicate or synthetic acquisition factory closes these boundaries.

[Exhaustive scout](../../../data/patch-api/evidence/12.0.1-session-2026-10-06/p1201-extract-scout.md) assigns every extract ID. Priorities: enums (37), damage-meter DTOs (21), cooldown hotfix (19), catalog DTOs (9), other DTO producers (24). Later batches: spell-policy data (74), remaining security/token/aura/macro boundaries (17). Scout-time bookkeeping kept all 201 contractual extract occurrences pending despite inspected reusable tests. The separate extract follow-up below does not mutate that page ledger. The teleport wildcard has no listed spell IDs; source `Texture:IsDesatured()` spelling is retained rather than silently mapped to another API.

## Verification

[Publication spec](../../specs/patch-12-0-1-publication-sweep.md) owns exact commands and final gate outcomes. Development sweep: 174/51 exact baseline, four retirement closures. Negative control flips `C_DamageMeter.GetSessionDurationSeconds`: exactly one new gap (51 → 52) and one changed observation, expected failure. Retirement test and 11 audio-filter tests pass; four Python extraction tests pass. Final isolated sweeps all pass: 12.0.0 983/27, 12.0.1 174/51, 12.0.5 352/11, 12.0.7 171/3, 12.1.0 773/5 (OK/exact gaps). Startup returns `[]`; formatting/local check and 477-row validation pass. [Proof ledger](../../../data/patch-api/evidence/12.0.1-session-2026-10-06/p1201-proof.json) records revision `a69dcfd66`, exact commands and unchanged six vendor-manifest warnings. No independent agent/model review was authorized or invoked.

## Gap closure verification

Verified runtime revision `eeccdea61`: five isolated local debug-retail sweeps pass with OK/gap counts **987/23, 222/3, 352/11, 171/3, 773/5** for 12.0.0/12.0.1/12.0.5/12.0.7/12.1.0. Only the 12.0.0 and 12.0.1 known-gap fixtures changed. Startup returns `[]`; formatting and local Cargo check pass. Thirteen new targeted regressions pass, plus audio (12 across direct/prefork phases), prediction (17), death recap (6), outfit (24), secrets (6), frame security (1) and CVar storage (9).

Moving the prediction factory initially dropped `__tostring`; the existing regression caught the missing identity prefix. Master `a3332a7fe` source preserves that prefix; the fix restores it and all 17 prediction tests pass. This comparison is source-level, not a full master runtime run. [Follow-up proof ledger](../../../data/patch-api/evidence/12.0.1-session-2026-10-06/p1201-gaps-proof.json) owns commands, revisions, inference/native limits and unchanged six vendor-manifest warnings. Changed-file Rust metrics stay within cognitive/cyclomatic thresholds; no suppression, vendor, crafting/order or page-coverage edits. No agents/models, push or merge.

## B01–B04 extract follow-up

[Per-source proof](../../../data/patch-api/evidence/12.0.1-session-2026-10-06/p1201-extract-proof.md) owns all 86 outcomes, exact revision-scoped commands, baseline comparisons and pending boundaries; page ledgers remain unchanged.

The loaded enum probe exposed the obsolete five-member combat-audio table: publish explicit current Retail name/value pairs and recomputed Meta in `c_api`. Charge DTOs omitted activity; derive the documented boolean and zero inactive recharge spans. Nonpositive regular intervals also normalize inactive. Active duration fixtures had negative start times because they subtracted seconds from a fresh near-zero clock; positive elapsed-clock fixtures preserve their original exact timing assertions. Numeric cooldown setters silently coerced VM secrets to zero; authenticate all numeric inputs before mutation and retain private timing for secure accepted values. Cached Blizzard Lua remains untouched. Secret-bearing duration-object consumption still rejects tainted callers; its separate ignored RED is explicitly executed and recorded as failing, not credited as a pass.

Catalog bundle/section DTOs were seeded temporary Lua outputs, including synthetic quantity one. Typed host records now preserve quantities, display order, disclaimer changes, optional omissions and independent snapshots. [Catalog contract](../../specs/catalog-shop-product-structures.md#1201-extract-bundle-and-section-dtos) owns missing-input policies. DamageMeter uses existing aggregate/detail host records; parent closure remains withheld for missing combat ConditionalSecret policy.

[Cooldown follow-up](../../specs/cooldown-restriction.md#1201-extract-follow-up) retains LoC flag derivation, aura-driven intervals and opaque duration-consumption gaps. Startup prints `[]`/CLEAN but process termination times out at 90 seconds; no clean-exit claim or master termination comparison. Two broad-filter failures already occur at the original master baseline; all isolated publication sweeps still pass.

## Sources

- [Source provenance](../../../data/patch-api/sources/12.0.1-api-changes.provenance.json).
- [Inventory register](../../../data/patch-api/sources/12.0.1-wikitext-register.json).
- [Sweep contract](../../specs/patch-12-0-1-publication-sweep.md).
- [Extract row scout](../../../data/patch-api/evidence/12.0.1-session-2026-10-06/p1201-extract-row-scout.json).

## See Also

- [[patch-12-0-0-api-audit]] — predecessor sweep/extract conventions and later-register supersession.
- [[patch-12-0-5-api-audit]] — first supported later retirement epoch.
- [[client-profiles]] — profile selection is separate from API epoch.
