# Forever character stat contributions

Forever Camelot PaperDoll consumes primary-stat contribution globals backed by player state. These are explicit coarse simulator policies, not native Forever coefficients. See [Lua API](../lua-api.md) for the state/registration boundary.

## What it must do

- [ ] Forever alone exposes `GetCritChanceFromStat(stat, value)`, `GetSpellCritChanceFromStat(stat, value)`, and `GetRangedAttackPowerForStat(stat, value)` with one numeric return each. Agility (index 2) contributes melee/ranged crit at 100 points per 1% (fraction `0.01`); intellect (index 4) contributes spell crit at the same rate. Other stat indices contribute zero. Negative input contributes zero. Ranged AP uses passed agility: Hunter 2/point, Warrior/Rogue 1/point, other classes zero.
- [ ] Forever player `UnitStat('player', 5)` reads per-environment spirit state as `(stat, base, positive, negative)`; the explicit unseeded/default value is zero. Other profiles retain their existing index-5 behavior.
- [ ] Forever alone exposes `GetHealthRegenFromSpirit()`, `GetManaRegenFromSpirit()`, and `GetHealthRegen()` with two numeric returns each. Spirit contributes 0.2 health and 0.1 mana per second per point; the second (combat) contribution is zero. There is no other modeled health regeneration, so total health regen equals spirit health regen.
- [ ] Existing `GetManaRegen()` intellect baseline remains intact; Forever adds spirit mana regen to its first (total) return, leaving its second (casting baseline) unchanged. Other profiles retain both original returns. Separate environments do not share spirit inputs.

These coefficients and class selection are **simulator guesses**, not native-verified semantics. The cached `PlayerScriptDocumentation.lua` establishes signatures/return counts; `Camelot/PaperDollFrameStats.lua` multiplies the crit fraction by 100 and displays spirit rates over five seconds. No native Forever probe is available.

## How it works

- [Lua API](../lua-api.md)
- [Character-stat data flow](../frame-data-flow.md)

## Implementation inventory

- `src/lua_api/state_types/character_world.rs` — Forever-only unseeded spirit field.
- `src/lua_api/globals/unit_stats.rs` — player `UnitStat` spirit read.
- `src/lua_api/globals/real/forever_stat_contributions.rs` — pure conversion helpers and modeled global functions.
- `src/lua_api/globals/real/combat_stats.rs` — Forever-only addition to existing mana total.
- `src/lua_api/globals/real/mod.rs`, `src/lua_api/globals/register.rs` — Forever-only registration.

## Tests asserting this spec

- `tests/forever_character_stat_contributions.rs` — grouped behavioral tests for passed-input scaling, class/stat selection, return counts, spirit isolation, and baseline coherence. Execution deferred to main's batched Cargo verification; boxes remain unchecked until GREEN proof.
- Main-owned `tests/wowforever_character_panel.rs` — one open/close/reopen full-panel acceptance passed at `72d6d8b45`; a later release-binary smoke repeated the lifecycle in the isolated 69977 fixture. Neither validates these stat-contribution formulas.

## Known gaps (current cycle)

- [ ] Run grouped behavioral tests for these contribution formulas. The final 12/12 verification covers panel lifecycle, relic, atlas, identity, and CDN cases, not this formula contract.

## Out of scope

Native coefficient equivalence; new C API or admin controls; Blizzard Lua edits; crit-rating totals; other profiles; broader stat model redesign.
