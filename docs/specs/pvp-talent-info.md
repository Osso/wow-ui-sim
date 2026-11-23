# PvP talent info

`C_SpecializationInfo.GetPvpTalentInfo` publishes a supplied PvP talent record to the `Blizzard_PlayerSpells` list-button consumer. This spec records the verified field names and the simulator host-input contract; it does not infer native field providers or missing-row behavior.

## What it must do

- [x] Register the table getter for Retail and WoW Forever; the consumer tests call it through `Blizzard_PlayerSpells`.
- [x] Assert that missing, nil, wrong-type, out-of-range, and tainted-secret `talentID` inputs fail and include the exact verified Usage text.
- [x] For a supplied host record, return one table with `talentID`, `name`, `icon`, `selected`, `available`, `spellID`, `unlocked`, `known`, `grantedByAura`, `dependenciesUnmet`, and `dependenciesUnmetReason`.
- [x] Copy each supplied field into the result without deriving native state from catalog data or player state.
- [x] Let `PvPTalentListButtonMixin:Update` use supplied name, icon, and `unlocked` values for button presentation.
- [x] Keep the host record map empty by default. An absent host row returns no values as a simulator policy; this does not claim native miss parity.

## Related global lookup

- [x] Register `GetPvpTalentInfoByID(talentID [, specGroupIndex, isInspect, inspectUnit])` for Retail and WoW Forever.
- [x] **VERIFIED (R88eu):** For a supplied row, return eleven values with these types: numeric `talentID`, string `name`, number-or-nil `icon`, boolean `selected`, boolean `available`, numeric `spellID`, boolean-or-nil `unlocked`, numeric `tier`, numeric `column`, boolean `known`, and boolean `grantedByAura`.
- [x] **VERIFIED (R88eu):** Return no values when the talent ID is zero or has no supplied row.
- [x] **VERIFIED (R88eu):** Reject non-number `talentID` arguments with an argument error that includes the Usage text.
- [ ] Optional context arguments are accepted but their native effects are not modeled.
- **INFERRED (R88eu):** Tuple order is `talentID`, `name`, `icon`, `selected`, `available`, `spellID`, `unlocked`, `tier`, `column`, `known`, `grantedByAura`; absent `tier` and `column` host inputs default to zero.

## How it works

- [C API and Lua API state](../wiki/systems/lua-api.md)

## Implementation inventory

- `src/c_api/c_spec.rs` — Retail record input, getter registration, argument validation, and table publication.
- `src/lua_api/workarounds/temporary/pvp_talent_defaults.rs` — remaining temporary PvP talent defaults; the static `GetPvpTalentInfo` placeholder is removed.
- `src/lua_api/globals/real/pvp_talent_info.rs` — inferred `GetPvpTalentInfoByID` tuple backed by the supplied record map.
- `tests/pvp_talent_native_consumers.rs` — locked/unlocked list-button consumer and absent-host-row tests.
- `tests/pvp_talent_state.rs` — Retail original Player Spells and Inspect consumers; Retail and Forever global tuple checks.

## Tests asserting this spec

- `tests/pvp_talent_native_consumers.rs::player_spells_locked_talent_button_uses_its_supplied_info`
- `tests/pvp_talent_native_consumers.rs::player_spells_unlocked_talent_button_uses_its_supplied_info`
- `tests/pvp_talent_native_consumers.rs::empty_host_records_do_not_publish_placeholder_info`
- `tests/pvp_talent_native_consumers.rs::get_pvp_talent_info_validates_verified_argument_paths`

## Known gaps (current cycle)

- [ ] UNKNOWN (R88f.A.51): Native field-provider mapping and native missing-row or invalid-ID result remain unmodelled.
- [ ] UNKNOWN (R88f.A.51): Native record types, updates, invalidation, events, and server effects remain unverified. The simulator accepts explicit typed host records only.

## Out of scope

- Registration or behavior on profiles not covered by R88f.A.51.
- Seeding PvP talent rows, learning rules, player progression, or native writer lifecycles.

## Evidence

- **VERIFIED (R88f.A.51):** Retail 12.1.5.70688 publishes the eleven named fields above in one table after an ID-keyed lookup; the exact Usage text is recorded in the decomp answer.
- **UNKNOWN (R88f.A.51):** Field providers, native miss behavior, default state, wrapper forwarding, extra arguments, invalidation, and events for `C_SpecializationInfo.GetPvpTalentInfo` are not established.
- **VERIFIED (R88eu):** `GetPvpTalentInfoByID` returns eleven values with the types listed above; a missing row or `talentID` zero returns no values, and a non-number ID raises an argument error.
- **INFERRED (R88eu):** The eleven-value order is listed above. The supplied record model defaults `tier` and `column` to zero. The effects of optional context arguments remain UNKNOWN.
- **VERIFIED (Retail 12.1.5.70783 cache):** `Blizzard_PlayerSpells/Blizzard_PlayerSpells.toc` loads `PvPTalents/Blizzard_PvPTalentListTemplates.xml`, which loads `Blizzard_PvPTalentListTemplates.lua`; its `PvPTalentListButtonMixin:Update` reads `unlocked`, `name`, `icon`, and dependency fields.
- Decomp citation: `pr46-49-decomp-answers.md`, section `R88f.A.51`.
