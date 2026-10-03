# Click-binding spell identifier — row 255

`C_ClickBindings.CanSpellBeClickBound(spellID)` reports host-declared spell eligibility. The 12.0.5 [GlobalAPI source](../../data/patch-api/sources/12.0.5-api-changes.txt) line 255 changes `# arg1.Type number -> SpellIdentifier`. Source ID: `global api-C_ClickBindings-CanSpellBeClickBound-255`.

Cached retail `Blizzard_APIDocumentationGenerated/ClickBindingsDocumentation.lua` lines 11–23 declare `SecretArguments = "AllowedWhenTainted"`, one nonnil `spellID: SpellIdentifier`, documented as **"Base spellID for spell, spellID for PetAction"**, and one nonnil `canBeBound: bool`. There is no second argument. Declaration is contract context, not native execution evidence.

## What it must do

### Explicit eligibility input

- [ ] Own feature-gated per-environment `SimState.click_bindable_spells: HashSet<u32>` with explicit host-declared resolved spell IDs. **INFERRED model policy:** no eligibility catalog, spellbook/action-bar derivation or acquisition; existing selected bindings do not imply eligibility.
- [ ] Return exactly one public boolean from live set membership. **INFERRED miss policy:** unresolved public strings and resolved nonmembers return false.
- [ ] **INFERRED empty default:** no declared eligible spells initially. Preserve existing left-target/right-menu interaction bindings; profile replacement/reset neither grants nor removes eligibility.
- [ ] Host set insertion/removal and alias replacement/removal take effect immediately. Queries do not mutate either input; environments isolate their inputs.

### Identifier boundary

- [ ] Resolve arg1 through existing `read_public_spell_identifier_at`: explicit aliases first, numeric identity otherwise, lowercase normalization for seeded names. Seeded numeric aliases override numeric identity. Numeric strings and full colored links require explicit aliases; do not parse links.
- [ ] **INFERRED shared strict representation policy:** accept public UTF-8 strings and finite integral u32 numbers, including zero/u32MAX. Reject missing/nil, bool, table, actual Frame, function, thread, invalid UTF-8 and nonfinite/fractional/negative/out-of-range numbers before alias lookup.
- [ ] Public identifiers work from secure and actually tainted callers without changing caller taint. Results remain public and have exact arity on hits and misses.
- [ ] **INFERRED conservative secret policy:** reject authentic VM secret strings/numbers in secure and tainted contexts, for eligible IDs, aliases, links and misses. Never unwrap inputs or inspect private payloads. Preserve identity, secrecy and caller taint after full GC; subsequent public queries recover.

## How it works

- [Lua API architecture](../lua-api.md)
- [Existing interaction profile](click-binding-interaction-profile.md)
- [Shared classification identifier boundary](aura-spell-classification-identifiers.md)

## Implementation inventory

- `src/c_api/c_click_bindings_spell.rs` — membership getter and modeled namespace registration.
- `src/c_api/c_spell.rs` — existing strict public identifier validator and alias-first resolver; unchanged.
- `src/lua_api/state/sim_state.rs` — required field insertion, owned by integrating session.
- `src/lua_api/state.rs` — required empty default insertion, owned by integrating session.
- `src/c_api/mod.rs` and `src/lua_api/env_init/mod.rs` — required module/registration insertions, owned by integrating session.
- `src/lua_api/workarounds/temporary/click_bindings_defaults.rs` — existing unconditional-true Lua provider must be removed by integrating session; other methods unchanged.

## Tests asserting this spec

`tests/click_binding_spell_identifier.rs`: nine feature-gated behavioral tests cover default interactions, numeric/name/miss controls, explicit full-link aliases, numeric precedence, live alias/set mutation, environment isolation, input immutability, selected-profile independence, strict invalid representations/endpoints, public tainted calls and authentic GC-rooted host-secret rejection/recovery in both caller contexts. No API replacement or marker secrets.

Existing `tests/blizzard_click_binding_ui_loads.rs`, `tests/click_targeting.rs` and `src/lua_api/workarounds/temporary/click_bindings_defaults.rs` tests provide integration controls. No existing direct test of this getter was found.

## Known gaps (current cycle)

- [ ] Integrate field/default/module/registration and remove the old provider using handoff instructions.
- [ ] Main session must format, compile and run new tests; no compiled RED/GREEN or runtime proof exists for this slice.
- [ ] Verify unchanged Blizzard UI load/show/hide and interaction-click controls. Cached `Blizzard_ClickBindingUI.lua:429` gates cursor additions; `Blizzard_SpellBookItem.lua:469` gates highlights/covers only when in click-bind mode, with an action ID and a learned spell. Neither use establishes eligibility or requires a true result during startup. Empty default is not a claim of native catalog completeness; hosts must declare IDs before spell binding is offered.

## Out of scope

- Native `AllowedWhenTainted` secret permissions, eligibility rules, miss/type/error wording and native parity: unknown; conservative simulator policies are labeled INFERRED.
- Base-spell/PetAction normalization: declaration specifies input meaning but existing interaction profile has no eligibility/normalization state. Host declares resolved IDs; no relationship is guessed.
- A second parameter: none declared. Additional Lua arguments are ignored by the single-argument callback; no second-argument semantics invented.
- Spell/macro/pet-action execution, persistence, full click-casting UI and earlier client epochs. Existing profile remains independent.
