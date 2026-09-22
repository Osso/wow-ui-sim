# Public base-spell lookup

Forever `C_Spell.GetBaseSpell` resolves public identifiers against explicit specialization-specific base relationships in `src/c_api/spell_base.rs`. The observed ActionBarAuras failure passed a nil base result into a table key. This bounded model is not native-conformance certification. See [Lua API architecture](../lua-api.md).

## What it must do

- [ ] Return the supplied positive spell ID when no relationship is modeled, including the twelve observed default action spells. The pinned Forever `SpellDocumentation.lua:90–104` explicitly documents no-override identity.
- [ ] Resolve public numeric strings and known spell names through the existing spell identifier resolver; case-insensitive names follow existing simulator behavior.
- [ ] Resolve an explicitly configured relationship for the requested specialization. Omitted/zero specialization uses the player's current class specialization. Treating explicit specialization numbers as specialization IDs is a simulator policy, not native-probed behavior.
- [ ] Keep specialization relationships separate from identifier aliases; never infer a base by reversing arbitrary aliases.
- [ ] Reject invalid identifiers, malformed specialization arguments, and secret arguments explicitly without unwrapping secrets. Invalid-input errors are bounded simulator policy.
- [ ] Leave non-Forever profile registrations unchanged.

## How it works

- [Lua API architecture](../lua-api.md)

## Implementation inventory

- `src/c_api/spell_base.rs` — relationship state, public argument validation and lookup.
- `src/c_api/c_spell.rs`, `src/c_api/mod.rs` — Forever-only namespace registration/module.
- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs` — per-environment relationship storage, initially empty.

## Tests asserting this spec

- `tests/spell_base.rs` — grouped into the existing generated integration target; twelve default action IDs, public names, configured relationships, explicit/current specialization, alias independence, malformed inputs and secret rejection.
- Existing RED: `/tmp/forever-addon-audit/base-spell-red.lua`; `/tmp/forever-addon-runtime/actionbarauras-base-spell-15bkm7sb/stdout` observes nil for all twelve action spells and the unchanged addon failure.
- Targeted Rust compilation/tests and real addon replay remain pending; implementation performed without Cargo authorization.

## Known gaps (current cycle)

- [ ] No real-world override relationship dataset is populated. Test relationships are deliberately configured fixtures, not invented live mappings.
- [ ] Native documentation permits secret spell identifiers when tainted; this implementation rejects secrets explicitly. No VM extension or declassification is included.
- [ ] Native Forever probe evidence is unavailable; invalid-input behavior, name disambiguation and explicit specialization interpretation are not native-verified.

## Out of scope

- VM/secret access changes and other spell override APIs: outside this public-identifier failure slice.
- Full ActionBarAuras or inventory-wide compatibility: separate runtime acceptance remains required.
