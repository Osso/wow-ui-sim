# Supplied party minimum and scenario availability (CF-2)

`C_PartyInfo.GetMinLevel` must derive from existing supplied player/active-party levels. Only scenario catalog projections needed by the original ScenarioFinder availability consumer belong in this slice. C namespace model/query code belongs in `src/c_api/`; legacy catalog globals may delegate there. [Consumer proof notes](../wiki/investigations/consumer-selection-and-red-proof.md#cf-2-party-minimum-staging) describe selection and proof boundaries.

## What it must do

- [x] Retail and PTR normally select the GroupFinder Game TOC and its shared ScenarioFinder Lua/XML. Use the original XML-created frame, cover, label and script bindings; prove Lua source provenance before calling availability. Never execute an unselected file or replace vendor functions/widgets.
- [x] With supplied player60 and an active connected member49, original availability shows the cover and the original localized next-level50 label. Moving that member to50 opens the cover at the inclusive minimum without changing player level.
- [x] Player48/member50 yields48; player100/member71 yields71 and next80. Members80 and90 are available; member91 has no available scenario or next threshold. Each standalone selector rejects constant party getters.
- [x] Project supplied `lfd_dungeons` scenario rows only: specific/random ranges50–70 and80–90, plus an ordinary-dungeon distractor1–100. Replacement lower thresholds55 then65 change the original label/cover; each standalone selector rejects constant catalogs and non-scenario inclusion.
- [x] Clearing the supplied roster and active flag recovers supplied player60. This is a bounded simulator policy, not established native ungrouped/category/disconnected semantics.

## How it works

- [Runtime selection and RED proof](../wiki/investigations/consumer-selection-and-red-proof.md#cf-2-party-minimum-staging).
- [C API boundary and profile rules](../../AGENTS.md#c-api-boundary).

## Pinned declarations

Retail `Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:323` and PTR `:332` declare `GetMinLevel(category?) -> number`; an omitted category uses the active party. The bounded consumer calls it without arguments at `Blizzard_GroupFinder/Shared/ScenarioFinder_Shared.lua:58`. Retail selects `Blizzard_GroupFinder_Mainline.toc`; PTR selects the unified `Blizzard_GroupFinder.toc` with `[Family]` and `AllowLoadGameType` filtering.

WoW Forever publishes the same `GetMinLevel` (its `PartyInfoDocumentation.lua:322-335` block is byte-identical to Retail's). Forever excludes Blizzard_GroupFinder (`## ExcludeLoadGameType: camelot`), so no original consumer is selected there; `tests/party_min_level_forever.rs` calls the API with supplied player and party levels.

## Implementation inventory

- `tests/cfactory_party_minlevel.rs` — isolated lane-owned original selected-consumer selectors for Retail/PTR.
- `src/c_api/c_party_info.rs` — Retail/PTR `GetMinLevel` reads the supplied player and, when active, party roster. Registration uses `client_profile::ACTIVE`; no Mists behavior is added.
- `src/lua_api/globals/battlefield_lfg_probes.rs` — Retail/PTR scenario projections reuse `lfd_dungeons`, retaining order and filtering subtype4 plus the supplied random flag. Specific choices exclude random rows; random count/info use random rows. Info exposes the six supplied fields read by the original availability consumer. Other profiles retain their empty surface. No new catalog or defaults.
- `src/lua_api/state_types/collections.rs` — existing `LfdDungeonInfo` catalog records, reused without a second catalog.
- `src/lua_api/game_data.rs`, `src/lua_api/state/sim_state.rs` — existing player/party inputs, no second roster.

## Tests asserting this spec

Both selectors in `tests/cfactory_party_minlevel.rs`, automatically discovered by the integration harness:

- `cfactory_party_minlevel::original_scenario_specific_catalog_uses_lowest_party_level`
- `cfactory_party_minlevel::original_scenario_random_catalog_uses_lowest_party_level`

Each runs all roster, boundary, replacement-catalog and supplied-solo observations independently under two disjoint catalog-ID sets. The random selector also shrinks its scenario count from two to one, rejecting a constant nonzero count. Registered only on Retail/PTR; no runtime success-return for unsupported profiles. Mists LFR availability is a separate deferred input boundary.

## Known gaps (current cycle)

- [x] Both Retail/PTR selectors independently list one case and fail at original `ScenarioFinder_Shared.lua:58`, after original XML bindings and Lua provenance. Tests are byte-identical across the producer-only sibling pair.
- [x] Both independent selectors pass GREEN on Retail/PTR. Retail/PTR/Era all-target checks have zero compiler warnings, including test targets. Empty-catalog preservation passes in both phases/profiles; full workspace formatting passes.
- [ ] Mists is BLOCKED on unresolved `C_LFGInfo.IsLFREnabled`. Preserve diagnostics; do not call that setup failure producer RED or add Mists behavior in this lane.
- [x] Compare complete producer-equivalent pinned-master and GREEN stock maps: Retail12/14→10/12, PTR244/312→242/310. Each removes only the original GetMinLevel primary/reporter pair; no additions or changed surviving counts. All launches use fresh current CVar copies with unchanged-original guards.

## Out of scope

- Squelch/LFR availability defaults, club state and generic LFG redesign.
- Unknown/disconnected roster, argument coercion and category semantics without separate evidence; native solo parity remains unproven.
- Vendor changes, test-only producer overrides, ambient/environment gameplay inputs, new availability shims and invented native defaults.
- Full-suite/prefork acceptance or seven-profile compatibility claims from this bounded slice.
