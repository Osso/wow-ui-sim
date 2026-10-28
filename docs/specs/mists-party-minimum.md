# Mists supplied active-party minimum

Mists `C_PartyInfo.GetMinLevel()` returns an environment-local supplied number. Pinned Mists 5.5.4.70268 `Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:125–135` declares the query and non-nil numeric return; line130 says an omitted category uses the active party. See the [scenario system](../wiki/systems/mists-scenario-finder.md#supplied-active-party-minimum).

## What it must do

- [ ] Publish only on Mists, using `client_profile::ACTIVE`; leave Retail/PTR producers unchanged.
- [x] Return exactly `SimState.mists_party_min_level: Option<f64>` when supplied. Supplied values take precedence over the scenario default.
- [x] When solo and unsupplied, return the player level as a best-guess scenario default; native behavior is UNKNOWN. When grouped and unsupplied, raise `native-unknown: C_PartyInfo.GetMinLevel has no supplied active-party minimum or documented default`.
- [x] Only the omitted-category call is supported. An explicit argument raises `unsupported: C_PartyInfo.GetMinLevel explicit category`; no category lookup is invented. Both errors are simulator policies, not native wording.
- [x] With a supplied minimum, the selected original `ScenarioFinder_Shared.lua:58` must complete through the empty-catalog branch with its original cover shown and `NO_SCENARIO_AVAILABLE` label. This is preservation, not output discrimination.

## How it works

- [Supplied active-party minimum](../wiki/systems/mists-scenario-finder.md#supplied-active-party-minimum)

## Implementation inventory

- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs` — optional supplied input and unset initialization.
- `src/c_api/c_party_info.rs` — Mists-only publication and explicit query boundaries.

## Tests asserting this spec

The publication/unchanged-other-profile requirement has Mists runtime and seven-profile compile proof; other-profile runtime parity is not claimed.

`tests/mists_scenario_minlevel.rs` covers selected TOC/Lua/XML identity, unsupplied original58 boundary, empty-catalog preservation at85/90, original cover transitions at40/55, exact direct-query values and count, explicit-category rejection, clearing and environment isolation.

## Known gaps (current cycle)

- [x] The selected original consumer discriminates solo levels 40 and 55 using an explicitly supplied presentation-list entry and dungeon fixture. This does not establish native catalog behavior; no catalog producer was changed.

## Out of scope

Native minimum/default/category inference, catalog producer changes, vendor edits and changes to other profiles.
