# Forever Spell Action-Bar Status

Forever's full UI uses the same spell-passive and action-slot queries as Retail's action-bar status consumer. The simulator had published neither query path on Forever: the spell APIs were Retail-gated, and `FindSpellActionButtons` returned a constant empty table outside its Retail registration epoch.

## Findings

### API evidence

- **VERIFIED (Forever `SpellDocumentation.lua:821–834`):** `C_Spell.IsRangedAutoAttackSpell` accepts a non-nil `SpellIdentifier` and returns a non-nil Boolean; `Blizzard_SpellSearchUtil.lua:82` calls it.
- **VERIFIED (Forever `SpellDocumentation.lua:965–977`):** `C_Spell.IsSpellPassive` accepts a non-nil `SpellIdentifier` and returns a non-nil Boolean; `ActionButtonUtil.lua:312` and `SpellFlyout.lua:186` call it.
- **VERIFIED (Forever `ActionBarFrameDocumentation.lua:74–88`):** `C_ActionBar.FindSpellActionButtons` accepts a spell identifier and an action-bar set; its `slots` return is a table, and `MayReturnNothing` is set. `ActionButtonUtil.lua:111` calls it.
- **VERIFIED (observed Forever spell cache `spells.json`, build 1.60.1.70685):** the selected 74 `SpellMisc` rows contain `Attributes_9`, but none has bit 64 set. This narrow cache result does not establish that the full client has no passive spells.
- **VERIFIED (R88i.14, relayed by aso):** `IsSpellPassive` returns false for ID 0 or a missing spell record and checks mask 64 for an existing record. **UNKNOWN (R88i.14):** the native table and field identity. Therefore the simulator does not use Forever DB2 `Attributes_9` as the passive source.

### Root causes and mapping

- **VERIFIED (simulator source before the fix):** `IsSpellPassive` was compiled only for Retail 12.1.5, and `IsRangedAutoAttackSpell` was registered only when Retail was active.
- **VERIFIED (Forever full-UI trace before the fix):** the original consumer failed at `ActionButtonUtil.lua:312` because `C_Spell.IsSpellPassive` was nil.
- **VERIFIED (simulator source before the fix):** the non-Retail `find_spell_action_buttons` producer returned an empty table; the state-backed implementation was registered only for `retail-12-0-5`. **VERIFIED (Forever full-UI trace):** the consumer supplied MKB style and slot 13, but `FindSpellActionButtons(467, Mkb)` returned an empty table.
- **INFERRED (Forever spell-passive mapping):** reuse Retail's host-supplied `spell_passive_attribute_masks` model with mask 64; use the profile-aware spell catalog for Forever record existence. The map is empty by default.
- **INFERRED (Forever ranged-spell mapping):** reuse the existing `ranged_auto_attack_spell_id` input. The generated documentation verifies the call shape, not native writer behavior.
- **VERIFIED (simulator source):** the shared slot query reads `action_bars` and does not inspect its second `setToSearch` argument. The action-bar-set distinction remains unmodeled.

## Fix

- Publish the two `C_Spell` queries on Retail 12.1.5 and Forever only; do not publish them on Mists or Wrath.
- Use `spell_lookup::get_spell` for Forever's runtime-loaded catalog while preserving Retail's `SPELL_DB` lookup.
- Publish only `C_ActionBar.FindSpellActionButtons` on Forever through the existing direct-slot model. Keep the other action-bar slot queries and the Mists/Wrath empty producer unchanged.

## Behavioral proof

`tests/forever_spell_passive_consumer.rs` loads Forever's full game startup inventory and invokes the original `ActionButtonUtil.GetActionBarStatusForSpell` implementation. It supplies catalogued spell 131 with host passive mask 64, spell 467 in MKB slot 13, and known non-passive spell 782 off all bars. **VERIFIED (Forever exact test):** the consumer returns `NotMissing` for the passive and slotted spells, `MissingFromAllBars` for the unslotted spell, and the two C_Spell queries return their supplied values. ID 0 is not passive.

## Sources

- [Spell API implementation](../../../src/c_api/c_spell.rs)
- [Spell passive state](../../../src/lua_api/state/sim_state.rs)
- [Action-bar slot query](../../../src/c_api/c_action_bar_spell_slots.rs)
- [Action-bar registration](../../../src/lua_api/globals/action_bar_api/registration.rs)
- [Forever original-consumer test](../../../tests/forever_spell_passive_consumer.rs)
- Forever runtime cache: `Blizzard_APIDocumentationGenerated/SpellDocumentation.lua`, `Blizzard_APIDocumentationGenerated/ActionBarFrameDocumentation.lua`, `Blizzard_ActionBar/Shared/ActionButtonUtil.lua`, `Blizzard_ActionBar/Shared/SpellFlyout.lua`, `Blizzard_SpellSearch/Blizzard_SpellSearchUtil.lua`

## See Also

- [[spellbook-r88m4-followups]] — related original spellbook consumer coverage.
