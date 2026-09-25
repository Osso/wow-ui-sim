# Forever character-panel remaining stat rows

Forever Camelot character-panel handlers consume these Forever-only stat globals and state reads. This is the contract; implementation details and diagnostic history are in [[forever-character-panel]].

## What it must do

- [ ] **Spirit.** Forever publishes `LE_UNIT_STAT_SPIRIT = 5`; existing `UnitStat('player', 5)` reads the explicitly unseeded spirit value without changing other profiles.
- [ ] **Equipment predicates.** `IsDualWielding()` is true only when slots 16 and 17 both resolve through the item catalog to one-hand weapons. A shield, missing item, or two-hand weapon is not a pair. `IsRangedWeapon()` is true when slot 18 resolves to a ranged weapon. Both reflect item replacement and removal through existing inventory-type classification, not class guesses.
- [ ] **Modifier reads.** `GetRangedHitModifier()`, `GetArmorPenetration()`, `GetSpellPenetration()`, `GetOverrideAPBySpellPower()`, and `GetOverrideSpellPowerByAP()` each return one number from the corresponding player `CharacterStats` field. These fields start at explicit simulator zero. Armor and spell penetration are flat amounts; ranged hit is a percent; override values are direct configured coefficients.
- [ ] **Ranged haste.** Forever `GetRangedHaste()` returns `(haste_pct(), quiver_haste_pct)` with unseeded quiver haste zero. Other profiles retain their existing one-return shape.
- [ ] **Defense skill.** `UnitDefenseSkill(unit)` shares `UnitDefense(unit)`'s modeled unit/level resolution and returns `(max(level * 5, 0), 0)`. The second value is explicit zero because defense bonuses have no modeled source. `UnitDefense` retains one return on every profile.
- [x] **Relic slot.** Forever `UnitHasRelicSlot(unit)` requires a string token and returns one boolean. Existing unit resolution supports `player`/`self`, `target`, and `focus`; missing or unmodeled units return false. The Classic Vanilla class rule—Paladin (2), Shaman (7), Druid (11) true; other classes false—is an inference, **not native-verified Forever behavior**. Changes in resolved unit class must change the result. Three focused cases passed in final scoped verification at `72d6d8b45`.

Cached `PlayerScriptDocumentation.lua` supplies the two-return shapes and `UnitDocumentation.lua` the defense tuple and required-token/boolean relic-slot shape. Cached `ARMOR_PENETRATION_TOOLTIP` establishes flat armor-penetration units. These inputs and zero defaults are simulator policy, not native-verified formulas.

## How it works

- [[forever-character-panel]] — handler diagnostic and current proof boundary.
- [Lua API](../lua-api.md) — global registration and state boundary.
- [Forever character stat contributions](forever-character-stat-contributions.md) — adjacent primary-stat and regeneration contract.

## Implementation inventory

- `src/lua_api/env_init/enums.rs` — Forever spirit constant.
- `src/lua_api/globals/real/combat_stats.rs` — modifier reads, predicates, relic-slot query, ranged-haste shape, and state-error propagation.
- `src/lua_api/globals/unit_stats.rs` — spirit and defense-skill reads.
- `src/lua_api/state_types/character_world.rs` — modeled modifier and quiver-haste fields.

## Tests asserting this spec

- `tests/forever_character_remaining_stats.rs` — observable Lua tests for stats, equipment lifecycle, relic class transitions and resolved aliases, and required token/return shape. The three relic cases passed at `72d6d8b45`; other row coverage remains unverified in this cycle.
- `tests/wowforever_character_panel.rs` — main-owned full-panel open/close/reopen acceptance passed once at `72d6d8b45`; it does not prove all 25 cached vendor handlers.

## Known gaps (current cycle)

- [ ] Obtain verifier GREEN for the remaining non-relic stat rows.
- [ ] Run all 25 cached vendor handlers after the remaining-stat proof; the one full-panel lifecycle pass is not that diagnostic.
- [ ] Obtain final release visual proof from the registered install; the isolated 69977 cache/run does not establish it.

## Out of scope

Combat resolution; talents, auras, or quiver simulation; native coefficient equivalence; vendor Lua changes; global implementations of vendor-local `GetEffectiveDefenseSkill` or `GetRangedDamage`; and changes to non-Forever profiles.
