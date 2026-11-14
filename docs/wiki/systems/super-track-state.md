# SuperTrack state

CF-2 models environment-owned tracked targets and quest-log-owned highlight rows for original Blizzard consumers. The user-waypoint tracking flag, corpse flag and their original World Map consumer are now modeled separately from the stored waypoint. [Contract](../../specs/super-track-state.md).

## Ownership and publication

`SimState.super_track` holds `SuperTrackState { quest_id, content, map_pin, vignette, user_waypoint, corpse }`. The four selection fields default to None; the booleans default to false as simulator inputs, not native initial-state claims. `c_api::c_super_track` owns the API endpoints. `QuestLogState.additional_highlights` holds supplied QuestAdditionalHighlights rows keyed by quest ID; no parallel quest log or fabricated row exists. `c_api::c_quest_log` projects the five fields in documented order.

`c_api/mod.rs::supports_super_tracking` selects Retail/PTR/Forever once at runtime, alongside the existing C_QuestLog registration. Modeled target installers were removed from temporary defaults and world-service defaults. Only the unsupported highest-priority nil query remains temporary, with retirement tied to native priority modeling. It is not inferred priority behavior.

Existing modeled mutators commit state, release the borrow, then dispatch SUPER_TRACKING_CHANGED as pre-existing simulator policy. The new user-waypoint setter dispatches no event. Native named-event effects remain UNKNOWN; ClearAllSuperTracked keeps its existing simulator dispatch. Per-category replacement, clear(quest setter 0), and clear-all are modeled without inventing cross-target exclusivity.

## User waypoint supertracking

**VERIFIED (Retail 12.1.5.70688, R88i.3/R88j.5):** `SetSuperTrackedUserWaypoint` checks a required non-nil argument, applies Lua truthiness, and returns early when the requested flag is unchanged. On a changed true request with a valid saved waypoint, the native handler clears the quest, content, map-pin, vignette, user-waypoint and one getterless flag, then sets user-waypoint tracking true. It preserves corpse tracking and one other getterless flag. A changed false request or an invalid point clears only the user-waypoint flag; it does not change the saved map or coordinates.

**VERIFIED (Retail 12.1.5.70688):** `IsSuperTrackingCorpse` reads a flag that the waypoint setter preserves. `IsSuperTrackingAnything` includes user-waypoint tracking and checks all eight native flags. Native `ClearAllSuperTracked` clears all eight flags and the four selections but preserves the saved waypoint coordinates. The simulator models corpse and user-waypoint flags plus existing selections; the two getterless flags remain unmodeled. **UNKNOWN (R88j.6 pending):** the relationship between native quest/content/map-pin boolean getters and the simulator's optional selection fields.

**VERIFIED (optional Retail prefork case):** the selected `WaypointLocationDataProvider.lua` click path passes line 101, and its original pin `OnAcquired` consumer reads the true user-waypoint flag to select the tracked atlas. The test also checks that the setter does not dispatch SUPER_TRACKING_CHANGED, that ClearAllSuperTracked clears modeled flags, and that the saved waypoint remains available afterward. The setter/getter use neutral zero/one result seams; their exact native return arity and named-event effects remain UNKNOWN. The simulator preserves ClearAllSuperTracked's pre-existing event dispatch; native event identity is UNKNOWN.

**VERIFIED (Retail, PTR and Forever declarations):** their pinned `SuperTrackManagerDocumentation.lua` files publish the user-waypoint and corpse queries. The behavior above is verified only for Retail; no PTR or Forever behavior is claimed.

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

- [User waypoint system](c-map-user-waypoint.md) — stored point, original provider and setter boundaries.
- [SuperTrack implementation](../../../src/c_api/c_super_track.rs) — modeled selection, corpse and user-waypoint state.
- [Optional original-consumer test](../../../tests/test_c_map_user_waypoint.rs) — click, pin acquisition, transitions and saved-point preservation.
- [Restored original-consumer tests](../../../tests/cf2_supertracking_restored.rs) — exact selectors and supplied transitions.

- [Contract and staged tests](../../specs/super-track-state.md)
- [M4 world inputs](../../specs/c-factory-m4-world-inputs.md) — historical absent-service policy, superseded for these targets.
- `Blizzard_FrameXMLUtil/Mainline/Blizzard_QuestSuperTracking.lua` and generated SuperTrackManager/QuestLog documentation in each profile cache — original consumer and nullable signatures; no vendor edits.

## See Also

- [[lua-api]] — C API subsystem boundary.
- [[c-factory-startup-followup]] — factory-removal residuals, not a clean-startup claim.
