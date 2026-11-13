# Trade skill crafting UI availability

**VERIFIED (PTR 70493 runtime test):** clicking the Blacksmithing button reaches `C_TradeSkillUI.CanTradeSkillShowCraftingUI`. **VERIFIED (PTR 12.1.7.70668):** the native predicate requires a non-null opaque-helper result, an existing spell, and `SpellEffect` difficulty 0 / effect index 0 with `Effect == 47`.

## What it must do

- [ ] Register `CanTradeSkillShowCraftingUI` on PTR `C_TradeSkillUI`, accept one required spell ID, and return one Boolean.
- [ ] Return true only when the supplied opaque-helper input is true, the spell exists in simulator spell data, and generated `SpellEffect` effect 0 equals 47.
- [ ] Return false when the helper input is not supplied, the spell is unknown, or effect 0 is absent or not 47.

## How it works

- [C API surface and simulator architecture](../lua-api.md)

## Implementation inventory

- `src/c_api/c_trade_skill_crafting_ui.rs` — PTR API registration, host input, and predicate.
- `src/c_api/mod.rs` and `src/c_api/registration.rs` — module and API registration.
- `src/lua_api/state/sim_state.rs` and `src/lua_api/state.rs` — per-environment host input storage and default.
- `src/bin/wow_cli/gen_spells.rs` — narrow generated effect-0 projection.
- `crates/sim-data/src/lib.rs` and `crates/sim-data/data/spell_effects.rs` — required-spell effect-0 data.

## Tests asserting this spec

- `tests/workarounds_professions.rs`

## Known gaps (current cycle)

- **UNKNOWN (PTR 12.1.7.70668):** the opaque helper's meaning and internal state dependencies are not established; the simulator uses a neutral supplied Boolean, default false.
- **UNKNOWN (PTR 12.1.7.70668):** exact Game-versus-Glue registration route is not established.
- **INFERRED (data source mismatch):** the generated effect rows come from Retail DB2 build 12.1.5.70391, while the native predicate evidence is PTR build 12.1.7.70668; parity for individual spell rows is not established.
- **INFERRED (PTR shared parser):** numeric strings and signed-32-bit conversion follow the shared parser; exact accepted string grammar and formatted error text remain UNKNOWN.

## Out of scope

- Do not model the opaque helper as player, SkillLine, known-spell, NPC, guild, linked-profession, or view-only state. **VERIFIED (PTR 12.1.7.70668):** the decoded predicate has no direct lookup for those inputs; hidden helper dependencies remain UNKNOWN.
