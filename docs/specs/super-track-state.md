# Supplied SuperTrack state

CF-2 supplies tracked targets through `src/c_api/c_super_track.rs` and per-quest highlights through `src/c_api/c_quest_log.rs`. [System notes](../wiki/systems/super-track-state.md) describe ownership and dispatch. The content-only restoration below is verified; broader quest/highlight requirements remain staged.

## What it must do

- [ ] Register the modeled namespace on Retail, PTR and Forever beside C_QuestLog through one shared runtime profile predicate; preserve unrelated namespace members.
- [ ] Start with absent quest/content/map-pin/vignette. Untracked quest returns nil: **documented-optional representation**, not established native sentinel parity. Pair queries return no results when absent; populated zero enum values remain valid.
- [ ] Round-trip quest and content selection, replace targets within each category and clear them; quest setter argument 0 clears. Clear-all removes all supported targets. IsSuperTrackingAnything reflects supported stored targets without priority arbitration.
- [ ] Publish committed state to the original selected Blizzard_QuestSuperTracking consumer through SUPER_TRACKING_CHANGED; its cached quest/content/highlight outputs must change and clear, including matching/nonmatching turn-in/watch events and eligible/ineligible auto-track branches.
- [ ] Return the five highlight outputs from the supplied per-quest row. With no row, return map ID 0 and four false flags so the tracker can refresh without adding map highlights. This absence representation is inferred from the documented return types and the Blizzard consumer.

## How it works

- [Ownership, consumer selection and proof bounds](../wiki/systems/super-track-state.md)

## Implementation inventory

- `src/c_api/c_super_track.rs` — SuperTrackState and modeled getters/mutators.
- `src/c_api/c_quest_log.rs` — QuestAdditionalHighlights supplied-row projection.
- `src/c_api/mod.rs` — shared runtime Retail/PTR/Forever publication predicate.
- `src/lua_api/state{,/sim_state}.rs` — environment-owned `super_track`, all fields None by default.
- `src/lua_api/sim_substates/quest_log.rs` — quest-log-owned additional_highlights rows, empty by default.
- `src/lua_api/workarounds/temporary/super_track_defaults.rs` — only unsupported priority query remains; no target fallback installers.

## Tests asserting this spec

- `tests/cf2_supertrack_consumers.rs` — five original-consumer selectors, each with state transitions; original complete chunk loaded via selected TOC, no consumer/API replacements.
- `tests/cf2_supertracking_restored.rs` — restored content shape and original listener-time cache transitions on Retail/PTR/Forever.
- `tests/c_super_track_probes.rs` — public shapes after bootstrap, supplied map-pin/vignette state and empty highlight output for quests without a row.
- Existing `tests/c_factory_m4_contracts.rs` absent-target assertions remain preservation, not producer RED.

## Known gaps (current cycle)

- [ ] Execute the remaining quest/highlight packet beyond the content-only restoration below. Its broader historical staging claims are not established by the restored selectors.
- [ ] Native untracked quest sentinel, exact highlight absence values, event timing/deduplication and cross-target coupling remain unestablished. Mutation calls synchronously notify after releasing the state borrow.
- [ ] IsQuestTask is currently coupled to the static world-quest catalog. World/task rejection is covered together; independent task-only classification remains held, not mocked.

## Out of scope

Corpse, waypoint, scenario/party targets, priority arbitration, persistence, broader map-provider catalogs and user-facing map-pin/vignette setters. Existing highest-priority query remains a temporary unsupported nil response, retired only when native arbitration is modeled; namespace is not fully modeled. PTR readiness=false is preservation, not readiness RED.

## Deferred supertracking coverage restoration

`tests/cf2_supertracking_restored.rs` restores only the supertracking obligations from the deferred housing shape/boundary tests on Retail, PTR and Forever. Both exact selectors exercise supplied Appearance/101 and Achievement/202 pairs, clear, and re-entry through the selected original `Blizzard_QuestSuperTracking.lua` event/cache refresh. An external listener records the pair during notification; the original helper reads that cache. The shape selector also checks the documented zero-result absence versus populated two-number binding contract. The cache helper itself returns two fields, including two nils after clear.

Declarations were re-authenticated per profile at `Blizzard_APIDocumentationGenerated/SuperTrackManagerDocumentation.lua:32–40` and `ContentTrackingTypesDocumentation.lua:60–70`. These tests preserve an existing supplied producer; they do not establish native target policy, event deduplication or arbitration, and do not restore unrelated housing tests. At `15f338b2f`, both exact selectors pass on all three profiles. Byte-identical tests independently reject producer-subtracted sibling `b6e8274a2` at the original listener snapshot. No producer change was needed. Stock maps match base `7576b25e1` exactly: Retail 10 unique/12 occurrences, PTR 216/282, Forever 26/26, with no added/removed keys or count changes. All three full-feature all-target checks have zero warnings; fmt and manual Rust readability checks pass. [Proof bounds and original consumer path](../wiki/systems/super-track-state.md#deferred-content-coverage-restoration) retain the distinction from native policy and the broader staged packet.
