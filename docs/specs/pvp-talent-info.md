# PvP talent info

Retail `C_SpecializationInfo.GetPvpTalentInfo` publishes a supplied PvP talent record to the `Blizzard_PlayerSpells` list-button consumer. This spec records the verified field names and the simulator host-input contract; it does not infer native field providers or missing-row behavior.

## What it must do

- [ ] Register the getter for Retail only and validate its required signed 32-bit `talentID` argument with the verified Usage text.
- [ ] For a supplied host record, return one table with `talentID`, `name`, `icon`, `selected`, `available`, `spellID`, `unlocked`, `known`, `grantedByAura`, `dependenciesUnmet`, and `dependenciesUnmetReason`.
- [ ] Copy each supplied field into the result without deriving native state from catalog data or player state.
- [ ] Let `PvPTalentListButtonMixin:Update` use supplied name, icon, and `unlocked` values for button presentation.
- [ ] Keep the host record map empty by default. An absent host row returns no values as a simulator policy; this does not claim native miss parity.

## How it works

- [C API and Lua API state](../wiki/systems/lua-api.md)

## Implementation inventory

- `src/c_api/c_spec.rs` — Retail record input, getter registration, argument validation, and table publication.
- `src/lua_api/workarounds/temporary/pvp_talent_defaults.rs` — remaining temporary PvP talent defaults; the static `GetPvpTalentInfo` placeholder is removed.
- `tests/pvp_talent_native_consumers.rs` — locked/unlocked list-button consumer and absent-host-row tests.

## Tests asserting this spec

- `tests/pvp_talent_native_consumers.rs::player_spells_locked_talent_button_uses_its_supplied_info`
- `tests/pvp_talent_native_consumers.rs::player_spells_unlocked_talent_button_uses_its_supplied_info`
- `tests/pvp_talent_native_consumers.rs::empty_host_records_do_not_publish_placeholder_info`

## Known gaps (current cycle)

- [ ] UNKNOWN (R88f.A.51): Native field-provider mapping and native missing-row or invalid-ID result remain unmodelled.
- [ ] UNKNOWN (R88f.A.51): Native record types, updates, invalidation, events, and server effects remain unverified. The simulator accepts explicit typed host records only.

## Out of scope

- Registration or behavior on profiles not covered by R88f.A.51.
- Seeding PvP talent rows, learning rules, player progression, or native writer lifecycles.

## Evidence

- **VERIFIED (R88f.A.51):** Retail 12.1.5.70688 publishes the eleven named fields above in one table after an ID-keyed lookup; the exact Usage text is recorded in the decomp answer.
- **UNKNOWN (R88f.A.51):** Field providers, native miss behavior, default state, wrapper forwarding, extra arguments, invalidation, and events are not established.
- **VERIFIED (Retail 12.1.5.70783 cache):** `Blizzard_PlayerSpells/Blizzard_PlayerSpells.toc` loads `PvPTalents/Blizzard_PvPTalentListTemplates.xml`, which loads `Blizzard_PvPTalentListTemplates.lua`; its `PvPTalentListButtonMixin:Update` reads `unlocked`, `name`, `icon`, and dependency fields.
- Decomp citation: `pr46-49-decomp-answers.md`, section `R88f.A.51`.
