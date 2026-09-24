# Forever character-panel remaining stat rows

## What it must do

- [ ] Forever publishes `LE_UNIT_STAT_SPIRIT = 5` without changing other profiles. The existing `UnitStat('player', 5)` reads the explicitly unseeded spirit value.
- [ ] Forever `IsDualWielding()` is true only when equipped main-hand slot 16 and off-hand slot 17 both resolve to one-hand weapons in the item catalog; a shield, missing item, or two-hand weapon is not a dual-wield pair. `IsRangedWeapon()` is true when slot 18 resolves to a ranged weapon. Both respond to item removal/replacement using existing item inventory-type classification, not class guesses.
- [ ] Forever `GetRangedHitModifier()`, `GetArmorPenetration()`, `GetSpellPenetration()`, `GetOverrideAPBySpellPower()`, and `GetOverrideSpellPowerByAP()` each return one number from their own player CharacterStats fields: `ranged_hit_modifier_pct`, `armor_penetration`, `spell_penetration`, `spell_power_to_attack_power`, and `attack_power_to_spell_power`. These fields start at explicit simulator zero, not claimed native defaults. Armor and spell penetration are flat amounts; ranged hit is a percent; override coefficients are direct configured numbers.
- [ ] Forever `GetRangedHaste()` returns `(haste_pct(), quiver_haste_pct)`; other profiles retain their existing single haste return. Unseeded quiver haste is explicitly zero.
- [ ] Forever `UnitDefenseSkill(unit)` uses the same modeled unit/level resolution as `UnitDefense(unit)` and returns `(max(level * 5, 0), 0)`. The second value is an explicit zero because defense bonuses have no modeled source. Existing `UnitDefense` retains one return on all profiles.

Cached `PlayerScriptDocumentation.lua` supplies the two-argument return shapes and `UnitDocumentation.lua` the defense tuple. Cached `ARMOR_PENETRATION_TOOLTIP` says attacks ignore `%d` armor, establishing flat units. The item catalog and `C_Item`'s inventory-type classifier supply equipment classification. These are simulator model inputs, not native-verified stat formulas.

## Limits

No combat resolution, talents, auras, quiver simulation, native coefficient claim, vendor changes, or global implementations of vendor-local `GetEffectiveDefenseSkill` and `GetRangedDamage`.

## Tests

- `tests/forever_character_remaining_stats.rs`: grouped observable Lua constant/arity, configured modifiers and isolation, unit-resolution tuple, equipped-item lifecycle (one-hand, shield, bow, ranged-right, removal). Execution deferred to main's final verifier under the explicit no-Cargo batch instruction.
- `tests/wowforever_character_panel.rs`: main-owned unchanged full-panel acceptance, followed by all 25 actual cached vendor handlers.
