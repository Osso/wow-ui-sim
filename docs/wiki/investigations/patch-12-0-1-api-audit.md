# Patch 12.0.1 API page audit

Captured page 659762, revision 6747895; 225 inventory occurrences plus 252 non-inventory lines. Audit machinery is complete; compatibility is not: publication-only proof and pending behavioral contracts remain distinct.

## Source and coverage

Retrieved October 6, 2026. Retained revision is dated June 19, 2026; Blue posts are dated February 19, February 24 and March 21, 2026. Consolidated comparison: 12.0.0 build 65655 → 12.0.1 build 66838, April 3, 2026. No external linked page was expanded. No standalone Deprecated API section exists in this capture.

All eight added/removed inventory header counts match. Existing 12.0.0/12.0.5/12.0.7/12.1.0 registers regenerate byte-identically. Plaintext extraction retains seven enum parents/30 deltas, 21 structure parents/33 deltas, and Blue-post security/data statements. Existing 12.0.0 plaintext remains identical.

| Scope | Rows | Outcome | Proof level |
|---|---:|---|---|
| Inventory | 225 | 174 OK / 51 exact reviewed gaps | Loaded cached Game publication/absence; no result or security parity |
| Later supersession | 16 | All OK; metadata-only | Later add/remove expectations, not historical behavioral proof |
| Extract enum/structure | 91 | Audit-pending | Source and candidate test-body inspection only |
| Extract prose | 110 | Audit-pending | 74 spell-policy/data plus 36 security/cooldown/model statements |
| Extract editorial context | 51 | Metadata-only | Individual source-context rationale |

[New page ledger](../../../data/patch-api/sources/12.0.1-page-coverage.json): **477 unique IDs**, **120 partial-development-green / 38 bounded-coverage / 252 audit-pending / 67 metadata-only**. Added/changed inventory greens mean publication only. Removed inventory greens bound absence or exact cached deprecation fallback attribution; they do not claim successor behavior. Existing page ledgers were not modified.

## Root causes and fixes

Initial sweep observed 170 OK / 55 gaps. Two removed `C_NamePlate` clamping members were raw-absent but ordinary namespace lookup fabricated functions. Two removed `C_CombatAudioAlert` speaker-volume members still had explicit Rust publishers. Retire all four in `src/c_api/patch_retired_members.rs`; stop the explicit audio registrations/functions on the same gate. Existing volume-only tests retain their historical epoch gate; live format/speed tests no longer call removed APIs.

No 12.0.1 feature/epoch exists in `Cargo.toml` or `src/client_profile.rs`. Use `retail-12-0-5`, the first supported later epoch; gating at 12.0.0 would remove APIs from the preceding documented surface. Earlier-epoch preservation is source-reviewed, not separately runtime-tested. Current retail is the 12.1.0 surface, not a newly added 12.0.1 client profile.

Two extractor defects surfaced while retaining Blue posts: broad HTML stripping erased a charge-cooldown comparison; blanket nested-bullet metadata classification discarded contractual formulas. Behavioral fixtures reproduce both boundaries. Fixes preserve operators, qualified API names and nested contractual rows without changing old coverage ledgers or old plaintext.

The 12.0.0 sweep now includes the 12.0.1 register. Its fixture grows from 23 to 27: removed minimapTrackedInfov2/useCompactPartyFrames, CHAT_MSG_ENCOUNTER_EVENT and SetPreventSecretValues remain published. Speaker-volume supersession gaps are fixed, not accepted into the baseline.

## Remaining gaps and scout

Every 51-row non-OK observation is retained in [gap review](../../../data/patch-api/evidence/12.0.1-session-2026-10-06/p1201-gap-review.json): 27 namespace raw-publication gaps, 13 object methods (12 missing heal-prediction methods, one retained removed method), five removed CVars, three globals, two events and one unprobeable NamePlate acquisition. Autostub lookup does not prove native publication; no fake successor model was added.

[Exhaustive scout](../../../data/patch-api/evidence/12.0.1-session-2026-10-06/p1201-extract-scout.md) assigns every extract ID. Priorities: enums (37), damage-meter DTOs (21), cooldown hotfix (19), catalog DTOs (9), other DTO producers (24). Later batches: spell-policy data (74), remaining security/token/aura/macro boundaries (17). All 201 contractual extract occurrences remain pending despite inspected reusable tests. The teleport wildcard has no listed spell IDs; source `Texture:IsDesatured()` spelling is retained rather than silently mapped to another API.

## Verification

[Publication spec](../../specs/patch-12-0-1-publication-sweep.md) owns exact commands and final gate outcomes. Development sweep: 174/51 exact baseline, four retirement closures. Negative control flips `C_DamageMeter.GetSessionDurationSeconds`: exactly one new gap (51 → 52) and one changed observation, expected failure. Retirement test and 11 audio-filter tests pass; four Python extraction tests pass. Final isolated sweeps, startup and format/check evidence will be recorded in the same session evidence directory. No independent agent/model review was authorized or invoked.

## Sources

- [Source provenance](../../../data/patch-api/sources/12.0.1-api-changes.provenance.json).
- [Inventory register](../../../data/patch-api/sources/12.0.1-wikitext-register.json).
- [Sweep contract](../../specs/patch-12-0-1-publication-sweep.md).
- [Extract row scout](../../../data/patch-api/evidence/12.0.1-session-2026-10-06/p1201-extract-row-scout.json).

## See Also

- [[patch-12-0-0-api-audit]] — predecessor sweep/extract conventions and later-register supersession.
- [[patch-12-0-5-api-audit]] — first supported later retirement epoch.
- [[client-profiles]] — profile selection is separate from API epoch.
