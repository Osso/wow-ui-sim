# SuperTrack state

CF-2 stages environment-owned tracked targets and quest-log-owned highlight rows for the original Blizzard_QuestSuperTracking consumer. Content cache/listener restoration is verified below; broader quest/highlight acceptance remains pending. [Contract](../../specs/super-track-state.md).

## Ownership and publication

`SimState.super_track` holds `SuperTrackState { quest_id, content, map_pin, vignette }`; every field defaults to None. `c_api::c_super_track` owns the API endpoints. `QuestLogState.additional_highlights` holds supplied QuestAdditionalHighlights rows keyed by quest ID; no parallel quest log or fabricated row exists. `c_api::c_quest_log` projects the five fields in documented order.

`c_api/mod.rs::supports_super_tracking` selects Retail/PTR/Forever once at runtime, alongside the existing C_QuestLog registration. Modeled target installers were removed from temporary defaults and world-service defaults. Only the unsupported highest-priority nil query remains temporary, with retirement tied to native priority modeling. It is not inferred priority behavior.

Mutators commit state, release the borrow, then synchronously dispatch SUPER_TRACKING_CHANGED. The original manager refreshes its cache and publishes Supertracking.OnChanged. Per-category replacement, clear(quest setter 0), and clear-all are modeled without inventing cross-target exclusivity. Equal-write notifications and synchronous timing are simulator policies, not native evidence.

## Consumer selection and observations

Retail/PTR select the generic Blizzard_FrameXMLUtil TOC, Family=Mainline/Game=Standard; Forever selects Family=Mainline/Game=Camelot. All permit the mainline-gated QuestSuperTracking entry. Mists/Era/Anniversary reject this entry; no Wrath chain is established. Tests resolve the profile-selected TOC, check the selected consumer entry, retain its source hash in output, load dependencies and the complete original chunk through the loader, and observe manager notifications. They never replace producer closures or rewrite vendor Lua.

The earlier staged stock selector also exercises content transitions: startup absence alone cannot reject constant nil. Active-quest selectors supply two different highlight tuples, assert all cached fields and map helpers, then clear stale outputs. Matching/nonmatching quest events and offer auto-tracking use the original event handlers. Task-only classification cannot be varied independently of the static world catalog and is explicitly held. These broader selectors are not established by the content-only proof below.

## Evidence bounds

Untracked quest nil follows the documented optional return. When a tracked quest lacks a highlight row, the query returns map ID 0 and four false flags. This lets `Blizzard_QuestSuperTracking.lua` complete its cache refresh and publish `Supertracking.OnChanged` while each map highlight predicate remains false. The five non-nil return types are documented, and the exact absence values are inferred.

Historical staging expectation, not the current baseline: frozen Retail stock receipt: 36 unique messages / 42 occurrences. Only the QuestSuperTracking :52 error and its ScriptErrors wrapper (count 2 each) are attributed here. Conditional expected remainder: 34 unique / 38 occurrences, with those exact two messages removed and every other message/count unchanged. :56 is latent, not part of the 36; any new :56 error fails acceptance. This is an expectation, not an executed result.

## Deferred content coverage restoration

On base `7576b25e1`, the existing `SuperTrackState.content: Option<(i32, i32)>` already satisfies the restored content obligations. No producer change was needed. `tests/cf2_supertracking_restored.rs` recovers only the supertracking cases from the deferred housing shape/boundary files, not their failing housing siblings or the accepted house-or-plot tests.

Both exact selectors pass at `15f338b2f` on Retail, PTR and Forever. Producer-subtracted sibling `b6e8274a2` projects no content results; each selector independently fails at the original listener snapshot with byte-identical tests (SHA256 `c42711b377c55a63980dd6b6322350e5dc08c221e75872dc2c7d796aa6750953`). This is preservation with a demonstrated RED counterexample, not a new producer-defect claim.

The selected original `Blizzard_QuestSuperTracking.lua:52` reads `(0,101)` then `(2,202)`, clear and re-entry. Its `:59` notification exposes the refreshed tracker; an external listener records the pair during notification through the original helper at `:107–108`. The shape selector also preserves the closed documented enum and distinguishes the zero-result absent binding from the cache helper's two nil fields after clear. No consumer/API replacement or direct mutation of the original manager's cache is used.

Per-profile declarations were authenticated at `Blizzard_APIDocumentationGenerated/SuperTrackManagerDocumentation.lua:32–40` and `ContentTrackingTypesDocumentation.lua:60–70`. Forever runtime evidence uses a private copy of the supplied profile cache.

| Profile / pinned version | Base → restored stock, unique/occurrences | Exact selectors | All-target warnings |
|---|---|---|---|
| Retail 12.1.5.70448 | 10/12 → 10/12 | 2 GREEN / 2 counterexample RED | 0 |
| PTR 12.1.7.70493 | 216/282 → 216/282 | 2 GREEN / 2 counterexample RED | 0 |
| Forever 1.60.1.70532 | 26/26 → 26/26 | 2 GREEN / 2 counterexample RED | 0 |

Raw full-message keys and counts have no additions, removals or changes; these are not clean-startup claims. Retail stock ran at `9e53bbfe3` and carries over because `15f338b2f` changes only a test assertion. Formatting and manual Rust readability checks pass. Quest/highlight branches and native sentinel/timing/deduplication/arbitration policy remain outside this proof.

## Sources

- [Restored original-consumer tests](../../../tests/cf2_supertracking_restored.rs) — exact selectors and supplied transitions.

- [Contract and staged tests](../../specs/super-track-state.md)
- [M4 world inputs](../../specs/c-factory-m4-world-inputs.md) — historical absent-service policy, superseded for these targets.
- `Blizzard_FrameXMLUtil/Mainline/Blizzard_QuestSuperTracking.lua` and generated SuperTrackManager/QuestLog documentation in each profile cache — original consumer and nullable signatures; no vendor edits.

## See Also

- [[lua-api]] — C API subsystem boundary.
- [[c-factory-startup-followup]] — factory-removal residuals, not a clean-startup claim.
