# Patch API blocker inventory

## UnitPowerPercent curve verification pending

Tests `eb495dbc7` reached RED with two ordinary nil/omitted queries passing and four supplied-curve failures. Runtime `2ba41cda2` evaluates non-nil argument 4 with the existing evaluator. Runtime/profile proof is pending, so this is not credited and does not change totals, snapshot, manifests or generated inventory. The current `0..100` input is simulator policy; `unmodified`, native scale, unknown units, security and full-LoD remain unverified. [Spec](../../specs/unit-power-percent-curves.md); [[patch-12-0-0-api-audit]].

## UnitHealthPercent curve refresh

One bounded best-effort credit covers supplied scalar/color curves, live player/target health, one-result arity, nil/omitted queries and invalid-curve rejection. Runtime `92b4f8c4f` passes 5/5 on retail 12.0.0/12.0.5/12.0.7; 12.0.0 exact-byte proof was reused and later profiles were fresh. Fmt/check/build/startup passed with startup `[]`. Final metadata proof `68ad13265`: **15,063 fresh / zero stale**, 59 renewals, seven additions, validator exit 0 and all 3,410 rows matching. Totals **2345 / 1063 / 2**; snapshot **1,089 / 282**. The `0..100` curve input is simulator policy, not native scale proof. Prediction, native scale, unknown units, security and full-LoD remain unproven; broad audit and eight Mists failures remain open.

## Historical aura-instance enumeration refresh

One bounded credit covers `C_UnitAuras.GetUnitAuraInstanceIDs`: public ordinary filtering, live updates, sort rules and limits. Historical publication was corrected at `ba31c6b94`; private, caster and enum gates remain later-only. Final metadata proof at `e9f56b974` records **15,056 fresh / zero stale**, 59 renewals, five additions, validator exit 0 and all 3,410 rows matching. Totals **2344 / 1064 / 2**; snapshot **1,090 rows / 282 plans**. Native filter domains, ordering, defaults, identity, security and full-LoD behavior remain unproven. Broad audit and eight Mists failures remain open.

## Global outfit-situations setting refresh

Two bounded credits cover explicit/repeated global boolean writes, one-boolean getter, zero-return setter and environment isolation. Runtime `457a8ae88`, tests `14ac47769`: independent 3/3 each on retail 12.0.0/12.0.5/12.0.7 in `/tmp/verify-outfit-situations-enabled-ledger.json`. Final metadata proof at `9c6404e68` records **15,051 fresh / zero stale**, 107 renewals, eight additions, validator exit 0 and all 3,410 rows matching. Totals **2343 / 1065 / 2**; snapshot **1,091 rows / 282 plans**. Initial false is simulator policy; native defaults, pending/per-outfit behavior, reset, persistence, events, UI/consumer execution and security remain unproven. Broad audit and eight Mists failures remain open.

## CombatText active-unit refresh

Only `GetActiveUnit` and `SetActiveUnit` leave the prior 1,097-row snapshot. The bounded model stores copied explicit `player`/`vehicle` selection independently per environment; it does not model identity, token validation, defaults, routing, events or consumers. Runtime `375c71192` passes 3/3 on each target profile; final metadata proof at `c099a68e6` records **15,035 fresh / zero stale**, 95 renewals, eight additions, validator exit 0 and all 3,410 rows matching. Totals **2339 / 1069 / 2**; snapshot **1,095 rows / 282 plans**. `GetCurrentEventInfo`, native/security obligations and eight Mists AccountStore failures remain open. `/tmp/verify-combat-text-active-unit-metadata-ledger.json`.

## Tracked-task membership refresh

Four bounded credits cover Add/Remove/GetTrackedInitiativeTasks and InitiativeTasksTracked.trackedIDs: ordinary numeric membership, copied arrays, return arity and bidirectional environment isolation. Tests `3abb84e1c` reached RED 0/3 because additions left the tracked-ID list empty. Runtime `d6d48d1e5` stopped at compiler error E0308 before tests ran; corrected runtime `27223f82a` has 3/3 each targeted profile (12.0.0 exact-byte reuse; 12.0.5/12.0.7 fresh), plus fmt/check/build/startup/readability PASS in `/tmp/verify-neighborhood-tracked-tasks-ledger.json`. Final metadata proof at `71209241a`: **15,027 fresh / zero stale**, 151 renewals, 16 additions, validator exit 0 and all 3,410 rows matching. Totals **2337 / 1071 / 2**; snapshot **1,097 rows / 282 plans**. Native validation, initial/order/duplicate/unknown-ID policies, task records/InitiativeTaskInfo.tracked, events, refresh, persistence, lifecycle, consumers and security remain unproven. Broad audit and eight Mists AccountStore failures remain open. All 282 plans remain; the neighborhood plan retains task-record/native/event/consumer obligations.

Format-setting refresh: only Get/SetFormatSetting leave the prior 1,110-row snapshot. All 282 plans remain; 281 unchanged. Ordinary keyed state is independently proven 3/3 per target profile at `b36c27ce3`; final metadata proof at `19c423c5f` records 14,976 fresh hashes, zero stale, 43 renewals, 12 additions, validator exit 0 and 3,410 matching rows. Remaining audio plan retains spec/throttle, enable/speech and native/default/range/CVar/output/callback/consumer/security obligations. `/tmp/verify-audio-format-setting-metadata-ledger.json`.

Snapshot date: **2026-09-14**, source baseline `88ccc385a` plus the retained StatusBar correction and separately proven housing/free-place, combat-log, speaker-speed/volume and historical public-aura credits. Per-manifest hashes identify current inputs. [Row inventory](../../generated/patch-api-blockers.json) maps every `evidence-required` occurrence to its literal gap, references and evidence plan. **1,090 remaining rows after one bounded historical public-aura credit.** Native behavior and executable readiness are not inferred from inventory membership.

## Speaker-volume refresh

Only the historical Get/SetSpeakerVolume rows leave the prior 1,112-row snapshot. All 282 plans remain; 281 are unchanged. Final metadata proof at `dcf344b01` records 14,964 fresh hashes, zero stale hashes, 31 renewals, 12 evidence additions, and validator exit 0 with all 3,410 rows matching. Remaining audio plans retain format/spec/throttle, enable-state and speech obligations, plus native/default/range/CVar/callback/persistence/playback gaps; neither speed nor volume storage proof establishes those effects. Runtime proof: `/tmp/verify-audio-speaker-volume-ledger.json`; metadata proof: `/tmp/verify-audio-speaker-volume-metadata-ledger.json`.

## Coverage

| Manifest | Rows |
|---|---:|
| 12.0.0 | 1,070 |
| 12.0.5 probes | 4 |
| 12.0.7 | 0 |
| 12.1 behaviors | 12 |
| 12.1 FrameXML | 0 |
| 12.1.5 | 10 |

Counts exclude `best-effort` residual gaps and `exception-requested` rows. Retail 12.0.0 remains **2344 best-effort / 1064 evidence-required / 2 exceptions**. Eight Mists AccountStore failures remain unresolved; this inventory supplies no Mists proof.

## How to use a row

1. Resolve `(manifest, id)` and read `missing_evidence` with `basis_references`. Notes are copied from the manifest, not newly established native facts.
2. Follow each row's `probe_plan` for observations, dependencies and remaining design gaps. `probe_protocol` is only the high-level queue label, not an alternative executable specification.
3. Establish concrete inputs, producer/consumer and target build before writing assertions. If these are absent, record that probe-design blocker rather than inventing values.
4. Separate ordinary behavior from secret/security/VM obligations before execution. A mixed row is not wholly excluded merely because its notes mention secrets.
5. Capture observations, then reassess only the demonstrated scope. Native absence, simulator state, rendering and lifecycle remain distinct proof levels.

## Work queues

Protocol assignment uses literal change/owner fields, with explicit later-patch cases. It is a routing aid, **not semantic classification of every row**.

| Proposed protocol | Rows | Missing prerequisite / next observation |
|---|---:|---|
| API/state | 446 | Establish behavioral inputs/model; observe named returns and state changes. Signature-only cases remain design-blocked. |
| Structure producer | 318 | Identify populated producer/consumer; vary input and observe exact field relationships. |
| Publication/removal | 233 | Capture exact old symbol across base/target load phases; distinguish field producers, CVar lookup and globals. |
| Event producer | 88 | Trigger an actual transition; capture tuple, order, multiplicity and unchanged-state control. |

The removal queue includes **192 enum/constant/CVar rows**. Local full-LoD simulator loading is useful simulator evidence, not native removal timing or dynamic-publication proof. Record failed/skipped addon loads; incomplete loading is not absence proof.

### Explicit later-patch cases

| Protocol | Rows | Next observation / blocker |
|---|---:|---|
| Deferred security | 15 | Exact forbidden/private/secret boundaries remain outside this phase. |
| Access predicate | 1 | Locate annotated API; `RequiresTimedSignalMapAccess` is not an ordinary constructor. Enforcement deferred. |
| Region parameters | 5 | Locate the constructor consuming `CreateRegionParams`; observe each supplied field's effect. |
| Timed entry | 3 | Locate map entry producer/consumer; establish key/time semantics from populated transitions. |
| Window transition | 1 | Capture actual maximize/restore signal; equal dimensions alone cannot identify transitions. |
| Store population capture | 1 | Existing capture never called the missing entry point. Find a build/state with callable population; observe non-secret UI state separately from protection. |

These explicit deferrals are **not an exhaustive count of security-related rows**. Earlier inventory drafts incorrectly treated all `C_` rows mentioning secrets as excluded and routed region parameters through timed-map probes; neither inference is retained.

## Concrete probe-design follow-up

**All 1,101 remaining rows link to 282 plans** in the JSON: 943 rows use 124 family-specific plans, and 158 rows preserve explicit probe requirements already written in their manifest notes. No rows rely on a routing label alone. Extraction of a source requirement does not count as fresh semantic validation. All 282 plans remain `ready_to_execute: false`: native fixtures, an exact producer/consumer or an unresolved contract is missing. Plans specify observations, not guessed native outcomes.

| Plan family | Concrete distinction to capture |
|---|---|
| Cast/channel identities | Preserve nil tuple positions; compare castID and castBarID across consecutive/replaced casts without aliasing identities. |
| Unit realm/base sex | Same-realm versus cross-realm return values; displayed versus base sex across a real disguise fixture. |
| Numeric formatting/markup | Signed tie-boundary outputs and locale; isolated markup plus all 16 combinations of four pinned 12.0.0 controls, including stripNewlines. A later fifth control needs separate build proof. |
| Action charges/aura time | Populated producer transitions, recharge/refresh/removal and exact field relationships; no zero/default-table credit. |
| CVar/Perks/math publication | Compare both CVar spellings read-only; distinguish raw/ordinary lookup and base/target load phases from native removal proof. |
| Crafting fields | Identify a real populated structure producer before asserting a removed key; unrelated internal itemID tables are not substitutes. |
| PTR region/map structures | Find an actual consuming/exposing API; global nil or scalar key/time returns do not imply a missing public constructor. |

Investigation suggestions rejected: mutating the CVar is unnecessary for the recorded getter case-sensitivity gap; later cached cast/action signatures cannot replace the pinned 12.0.0 contract; a fifth cached StripHyperlinks control cannot expand the four-control 12.0.0 matrix without target-build evidence. Security enforcement remains deferred even when an ordinary producer observation is possible.

Historical source-alignment verification at `8cae7bfdf` covers the prior 1,121-row snapshot only. Correction-only metadata verification at `804a6b073` refreshes the housing delta: 1,119 rows/282 plans, 281 plans unchanged, exact hashes/notes/references matching, 14,917 fresh hashes, zero stale, 139 renewals and 14 additions. Proof: `/tmp/verify-housing-freeplace-metadata-corrected-ledger.json`. Its validator output has 3,410 matching rows and empty stderr, but the exit status was not retained; no second run was made.

Corrections distinguish numeric error codes from localized strings, callback/strided `mapvalues` from table mapping, real message operations from friend queries, and housing refundability from crafting data. Missing signatures remain explicit design gaps. Literal-source plans preserve complete notes, including Lua member names and expressions; they are not newly established native outcomes.

The combat-log setting refresh removes only four credited rows from the preceding 1,119-row snapshot. All 282 plans remain used; 281 are unchanged. The remaining combat-log plan now targets `GetMessageLimit`, `DoesObjectMatchFilter(mask, flags)` and `IsCombatLogRestricted`, not already-proven setting storage. Filtering/matching/restriction/pruning/native/lifecycle gaps remain explicit. The earlier housing validator-exit qualification above is unchanged. Metadata verification at `19eab5ef4` records 14,933 fresh hashes, zero stale, zero renewals, 16 additions, four credits, validator exit 0 and 3,410 matching rows. Proof: `/tmp/verify-combatlog-settings-metadata-ledger.json`.

The message-limit refresh removes only `added:C_CombatLog.GetMessageLimit` from the preceding 1,115-row snapshot. All 282 plans remain used; 281 are unchanged. Its remaining combat-log plan now covers `DoesObjectMatchFilter(mask, flags)` and `IsCombatLogRestricted` only. One storage/query credit changes retail totals to **2320 / 1088 / 2**; independent `b91e21c62` proof is 3/3 per target profile with unchanged-runtime reuse. The existing setter event credit remains intact. Final committed metadata verification at `5c2d06cb7` records 14,936 fresh hashes, zero stale, 16 renewals and three additions. It reuses the `152154a11` saved validator CommandResult—exit 0 and 3,410 matching rows—because all six final manifest byte streams are identical; no validator, test or runtime-gate rerun occurred. Proof: `/tmp/verify-combatlog-message-limit-metadata-ledger.json`. Earlier snapshot/validator evidence below retains its historical scope.

## Speaker-speed refresh

Only `added:C_CombatAudioAlert.GetSpeakerSpeed` and `added:C_CombatAudioAlert.SetSpeakerSpeed` leave the preceding 1,114-row snapshot. All 282 plans remain used; 281 are unchanged. The remaining audio plan targets speaker-volume, format/spec/throttle settings, `IsEnabled` and speech behavior, not already-proven speaker-speed storage. Rust registration is gated at `retail-12-0-0`; generic earlier-profile fallback callability/absence remains unproven. Fresh runtime proof is 3/3 per targeted profile, with initial `0` and accepted-write `true` explicitly simulator policy. CVar/native/playback/security gaps remain open. Final committed metadata validation at `519dad999` reuses exact runtime/test bytes and saved validator exit 0 with 3,410 matching rows: 14,952 fresh hashes, zero stale, 21 renewals and 16 additions. The 1,112-row snapshot has no missing/extra keys or orphan plans. Proof: `/tmp/verify-audio-speaker-speed-metadata-ledger.json`. Prior proof paragraphs retain their historical scope.

## Source correction: StatusBar

The pinned `changed:StatusBar` before/after definitions both retain `reverseFill` and `rotatesTexture`. Only `fillStyle` changes from stringenum/`STANDARD` to numeric enum/init `0`. The previous manifest note incorrectly required removed-field behavior. The manifest, occurrence inventory and snapshot now agree; status and credits are unchanged. Existing Get/SetFillStyle proof covers explicit numeric state, not native defaults or the complete UI-object contract.

## Refresh and verification

The JSON records each source manifest's SHA-256 and exact row keys. On refresh, compare all six committed manifests: every `evidence-required` row must occur once, other statuses must not appear, literal gap text and evidence references must match, and every protocol must resolve. Reassess explicit cases rather than inferring semantics from keywords. Keep manifest classifications unchanged unless separate behavioral evidence justifies credit.

Historical verification at `8cae7bfdf` remains scoped to the prior 1,121-row/282-plan snapshot, including its 14,903-fresh/zero-stale and observed validator-exit evidence. The housing refresh removes two credited rows, updates all six manifest hashes and narrows one basic-mode plan; 1,119 rows and 281 plans remain unchanged, with all 282 plans still used. Correction-only metadata verification at `804a6b073` independently validates the housing snapshot and references without rerunning runtime tests. It records complete validator output with 3,410 matching rows and empty stderr; its exit status is unavailable, so this page makes no validator-exit claim. The later combat-log metadata verification at `19eab5ef4` runs its own validator successfully; that exit 0 applies to the 1,115-row refreshed snapshot and does not retroactively change the housing-run record. Runtime proof remains separate in `/tmp/verify-housing-freeplace-ledger.json`.

## Sources

- [Audit contract](../../specs/patch-api-audit-manifest.md) — evidence and bounded-credit requirements.
- [Inventory JSON](../../generated/patch-api-blockers.json) — exact row keys, source hashes, missing evidence and acquisition protocols.
- [12.0.0 manifest](../../../data/patch-api/12.0.0.json) — primary remaining occurrence set; other manifest paths and hashes are listed in the JSON.

## See Also

- [[patch-12-0-0-api-audit]] — accumulated bounded implementation proof.
- [[patch-12-0-5-api-audit]] — retained native-capture gaps.
- [[patch-api-audit-manifest]] — manifest system and provenance.

## Bounded custom-set CRUD refresh

Seven historical CRUD methods receive bounded ordinary-state credit at runtime `df6264867`: concrete records, numeric IDs, ID/info/item retrieval, modify/rename/delete, copied input/output values, arity and record/environment isolation. Tests `ef9bbe70c` reached RED 0/4 (initial NewCustomSet returned nil); independent `/tmp/verify-custom-set-crud-ledger.json` records 4/4 each on retail 12.0.0/12.0.5/12.0.7 (12.0.0 exact-byte reuse), fmt/check/build/startup PASS. Final metadata proof `/tmp/verify-custom-set-crud-metadata-ledger.json` at `2531a0e61` records **15,011 fresh / zero stale**, 86 renewals, 35 additions, seven credits, validator exit 0 and all 3,410 rows matching. Readability length/local-Vec-loop findings remain a reviewed signal; main rejected mandatory `read_items` refactoring, not a clean-readability claim. Totals: **2333 / 1075 / 2**; snapshot **1,101 rows / 282 plans**. Native validation/error policy/defaults/limits/events/persistence/hyperlinks/consumer behavior remain unproven. Eight Mists AccountStore failures remain unresolved; broad audit remains open. The `collection_custom_sets` plan remains non-executable and retains native validation/error/default/persistence/events/hyperlink/consumer obligations.
