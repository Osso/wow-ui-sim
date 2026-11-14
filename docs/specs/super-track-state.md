# Supplied SuperTrack state

CF-2 supplies tracked targets through `src/c_api/c_super_track.rs` and per-quest highlights through `src/c_api/c_quest_log.rs`. [System notes](../wiki/systems/super-track-state.md) describe ownership and dispatch. The content-only restoration below is verified; broader quest/highlight requirements remain staged.

## What it must do

- [ ] Register the modeled namespace on Retail, PTR and Forever beside C_QuestLog through one shared runtime profile predicate; preserve unrelated namespace members.
- [x] **VERIFIED (Retail 12.1.5.70688; optional original-consumer test):** `SetSuperTrackedUserWaypoint` uses Lua truthiness and is a no-op when the stored flag already matches. A changed true value with a valid point clears the quest, content, map-pin, vignette and user-waypoint flags plus one unnamed flag, then sets user-waypoint tracking. It preserves corpse tracking and one unnamed flag. A changed false value or invalid point clears only user-waypoint tracking; the saved point stays unchanged. `IsSuperTrackingUserWaypoint`, `IsSuperTrackingCorpse`, and `IsSuperTrackingAnything` expose the modeled flags. `IsSuperTrackingAnything` includes user-waypoint tracking. `ClearAllSuperTracked` clears modeled flags and selections but preserves the saved waypoint. The implementation uses zero setter values, one getter boolean, and no event as neutral seams; the new setter dispatches no event.
- [ ] The setter checks secret access before converting the required non-nil argument. Native return arity, the two getterless flags, and named-event effects remain UNKNOWN; see [system evidence bounds](../wiki/systems/super-track-state.md#user-waypoint-supertracking).
- [ ] **VERIFIED (Retail 12.1.5.70688, R88j.6):** quest, content and map-pin tracking flags are separate from their saved selections. Changed quest ID zero clears only the quest flag; changed nonzero quest IDs set it and clear content/map-pin flags and selections. Setting content stores its pair even when the ID is zero, sets its flag, and clears quest/map-pin flags and selections. Clearing content removes presence and its flag but keeps the saved pair. Clearing a map pin removes its selection only when its flag is true. Clear-all clears modeled flags/selections while preserving the saved content/map-pin values. `SetSuperTrackedMapPin` and `SetSuperTrackedVignette` producer acceptance remains deferred pending R88j.8.
- [ ] Start with absent quest/content/map-pin/vignette selections and false simulator defaults for corpse and user-waypoint flags; native initial flag values are UNKNOWN. Untracked quest returns nil: **documented-optional representation**, not established native sentinel parity. Pair queries return no results when absent; populated zero enum values remain valid.
- [ ] Round-trip quest and content selection, replace targets within each category and clear them; quest setter argument 0 clears. Clear-all removes modeled target selections and modeled flags without changing the saved waypoint. IsSuperTrackingAnything reflects modeled selections, corpse and user-waypoint state; two getterless native flags remain unmodeled.
- [ ] Publish committed state to the original selected Blizzard_QuestSuperTracking consumer through SUPER_TRACKING_CHANGED; its cached quest/content/highlight outputs must change and clear, including matching/nonmatching turn-in/watch events and eligible/ineligible auto-track branches.
- [ ] Return the five highlight outputs from the supplied per-quest row. **VERIFIED locally (WM71, Retail 12.1.5.70688):** a numeric cache miss or no selected POI returns map ID 0 and four false flags. A selected POI can return a nonzero map ID even when all flags are false. Zero/no quest produces defaults with the observed empty zero-key cache (**INFERRED**). Missing, nil, nonconvertible and out-of-range arguments error; finite in-range numbers truncate toward zero. Numeric coercion is attempted; exact string grammar is **UNKNOWN**. Secret access is AllowedWhenUntainted. No direct super-tracked-ID check exists; universal quest-log membership is **UNKNOWN**. Native flag masks for world, elite, dungeons and treasures are 8, 64, 16 and 32 respectively.

## How it works

- [Ownership, consumer selection and proof bounds](../wiki/systems/super-track-state.md)

## Implementation inventory

- `src/c_api/c_super_track.rs` — SuperTrackState and modeled getters/mutators.
- `src/c_api/c_quest_log.rs` — QuestAdditionalHighlights supplied-row projection.
- `src/c_api/mod.rs` — shared runtime Retail/PTR/Forever publication predicate.
- `src/lua_api/state{,/sim_state}.rs` — environment-owned `super_track`, with absent selections and false simulator flag defaults.
- `src/lua_api/sim_substates/quest_log.rs` — quest-log-owned additional_highlights rows, empty by default.
- `src/lua_api/workarounds/temporary/super_track_defaults.rs` — only unsupported priority query remains; no target fallback installers.

## Tests asserting this spec

- `tests/cf2_supertrack_consumers.rs` — five original-consumer selectors, each with state transitions; original complete chunk loaded via selected TOC, no consumer/API replacements.
- `tests/cf2_supertracking_restored.rs` — restored content shape and original listener-time cache transitions on Retail/PTR/Forever.
- `tests/c_super_track_probes.rs` — public shapes after bootstrap, supplied map-pin/vignette state and empty highlight output for quests without a row.
- `tests/test_c_map_user_waypoint.rs` — optional Retail original-consumer flow, waypoint flag, corpse preservation, clear-all and aggregate queries.
- Existing `tests/c_factory_m4_contracts.rs` absent-target assertions remain preservation, not producer RED.

## Known gaps (current cycle)

- [ ] Execute the remaining quest/highlight packet beyond the content-only restoration below. Its broader historical staging claims are not established by the restored selectors.
- [ ] Verify the exact setter/getter return arity and named-event effects. R88j.5 leaves two getterless native flags unmodeled. Agreement between the separate quest/content/map-pin flags and selections is only INFERRED from a consistent start; R88j.6 does not prove a public-function sequence that creates a mismatch. Clear-all keeps its pre-existing simulator event dispatch; native event identity remains UNKNOWN. Set-map-pin and vignette setter acceptance is pending R88j.8.
- [ ] Native untracked quest sentinel, live quest POI cache state and selection, event timing/deduplication and cross-target coupling remain unestablished. Mutation calls synchronously notify after releasing the state borrow. WM71 evidence: `native-evidence-70688/WM71-quest-additional-highlights.md` (local handoff).
- [ ] IsQuestTask is currently coupled to the static world-quest catalog. World/task rejection is covered together; independent task-only classification remains held, not mocked.

## Out of scope

Scenario/party targets, priority arbitration, persistence, broader map-provider catalogs and user-facing map-pin/vignette setters. Existing highest-priority query remains a temporary unsupported nil response, retired only when native arbitration is modeled; namespace is not fully modeled. PTR readiness=false is preservation, not readiness RED.

## Deferred supertracking coverage restoration

`tests/cf2_supertracking_restored.rs` restores only the supertracking obligations from the deferred housing shape/boundary tests on Retail, PTR and Forever. Both exact selectors exercise supplied Appearance/101 and Achievement/202 pairs, clear, and re-entry through the selected original `Blizzard_QuestSuperTracking.lua` event/cache refresh. An external listener records the pair during notification; the original helper reads that cache. The shape selector also checks the documented zero-result absence versus populated two-number binding contract. The cache helper itself returns two fields, including two nils after clear.

Declarations were re-authenticated per profile at `Blizzard_APIDocumentationGenerated/SuperTrackManagerDocumentation.lua:32–40` and `ContentTrackingTypesDocumentation.lua:60–70`. These tests preserve an existing supplied producer; they do not establish native target policy, event deduplication or arbitration, and do not restore unrelated housing tests. At `15f338b2f`, both exact selectors pass on all three profiles. Byte-identical tests independently reject producer-subtracted sibling `b6e8274a2` at the original listener snapshot. No producer change was needed. Stock maps match base `7576b25e1` exactly: Retail 10 unique/12 occurrences, PTR 216/282, Forever 26/26, with no added/removed keys or count changes. All three full-feature all-target checks have zero warnings; fmt and manual Rust readability checks pass. [Proof bounds and original consumer path](../wiki/systems/super-track-state.md#deferred-content-coverage-restoration) retain the distinction from native policy and the broader staged packet.
