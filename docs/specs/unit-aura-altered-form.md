# Unit aura altered-form query

`C_UnitAuras.WantsAlteredForm(unit: string) -> wantsAlteredForm: bool` reads explicit player input. Cached retail `Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua` declares one unit token, one boolean and `SecretArguments = "AllowedWhenUntainted"`. Patch source `data/patch-api/sources/12.0.5-api-changes.txt:407` changes `AllowedWhenTainted` to `AllowedWhenUntainted` (exact row `global api-C_UnitAuras-WantsAlteredForm-407`). Input lives in `src/lua_api/state_types/character_world.rs`; query producer/registration is deliberately absent from this inputs/tests/spec slice. See [Lua API architecture](../lua-api.md).

## What it must do

### State and query

- [ ] **INFERRED:** `PlayerState.wants_altered_form` is one independent boolean, false in empty/default and seeded player state. No PartyMember field or unit-state map.
- [ ] Return exactly one public boolean for a required unit string; changing the explicit input false/true/false changes the modeled player's result without query-induced state changes or events.
- [ ] Use actual `existing_guid_for_unit` identity resolution: tokens resolving to the modeled player's GUID read the player input. Existing `TargetUnit('player'); FocusUnit('target')` aliases must follow that identity; retargeting and clearing must stop reading it when identity no longer matches.
- [ ] **INFERRED:** other, unknown and unmodeled identities return false. Do not invent `self` or `raidplayer` aliases or assume `raid1` is player.
- [ ] Never derive this input from race, stance/shapeshift state, barber data, `is_alternate_form` or `alternate_form_is_default`. Query must preserve those independent fields.
- [ ] **INFERRED:** missing/nil or nonstring units error (including numbers; no coercion); error text is not native-characterized. Valid public calls still work after rejection.

### Secret unit argument

- [ ] Authenticate only the documented unit argument through the VM's exact `rilua::table_security::unwrap_secret` boundary. This is AllowedWhenUntainted access, not generic declassification or caller-taint clearing.
- [ ] Accept a real host-secret **string** containing `player` from an untainted caller and return the same one public boolean as the public token. Preserve caller taint and input secrecy.
- [ ] Reject that string from a tainted closure; preserve the original stack-taint label. Public-string recovery must work inside the same tainted closure; secure secret-string recovery must work afterward.
- [ ] Rooted host-secret string remains secret and retained across full GC, with secure acceptance, tainted rejection and public recovery unchanged.

### Cached consumer

- [ ] Load complete, unmodified cached `Blizzard_SharedXML/UnitUtil.lua`. Assert real `PlayerUtil.ShouldUseNativeFormInModelScene()` returns true for Human, and the explicit boolean for Worgen/Dracthyr, using actual existing race data and query. No vendor patch, query override, fake race data or 3D rendering.

## How it works

- [Lua API architecture](../lua-api.md).
- [Client-profile runtime cache](../wiki/systems/client-profiles.md).
- [Existing unit identity behavior](unit-identity-equality.md).

## Implementation inventory

- `src/lua_api/state_types/character_world.rs`: explicit player boolean; derived Default supplies false, seeded initialization inherits it.
- `src/lua_api/globals/unit_misc.rs`: existing identity resolver; unchanged, producer must reuse it.
- `tests/unit_aura_altered_form.rs`: ten tests in existing autodiscovered integration target, gated by `retail-12-0-5`.
- Active-profile cache `AddOns/Blizzard_SharedXML/UnitUtil.lua`: real scene-selection consumer loaded unchanged by one test.

## Tests asserting this spec

`tests/unit_aura_altered_form.rs`:

- `player_input_defaults_false_in_empty_and_seeded_state`, `player_query_returns_one_public_boolean_for_false_true_false`.
- `target_and_focus_follow_existing_player_guid_identity`, `other_unknown_and_unmodeled_identities_return_false`.
- `explicit_input_is_independent_of_race_stance_and_legacy_form_flags`, `required_unit_string_rejects_missing_nil_and_wrong_types`.
- `untainted_caller_accepts_actual_host_secret_unit_string`, `tainted_caller_rejects_host_secret_string_and_recovers_with_public_input`, `rooted_secret_string_survives_gc_with_identity_security_and_public_recovery`.
- `cached_unit_util_consumer_uses_input_for_worgen_and_dracthyr_only`.

Filter: `unit_aura_altered_form::` in Cargo target `integration`. No new Cargo target/manifest/build-script edits.

## Known gaps (current cycle)

- [ ] Parent owns RED compilation/runtime and subsequent query producer/registration. This slice runs no builds, tests, checks, lint or readability gates; no passing claims or row promotion.
- [ ] Native source/default/non-player/error/security semantics remain unproven. False default/misses and strict representation errors are explicit simulator inferences.
- [ ] Consumer test requires populated active-profile cache with `Blizzard_SharedXML/UnitUtil.lua`; missing cache/source fails explicitly, never skips or substitutes code. The inspected retail file consists of function definitions and can be supplied unchanged to existing `WowLuaEnv::exec`; actual load/runtime remains parent verification. This is not full SharedXML/addon/model-scene closure.
- [ ] GC fixture asserts retained reference equality via secure `rawequal`, secrecy and authenticated behavior, not raw VM pointer identity. Race/stance/legacy-field nonmutation is asserted; barber independence and absence of events are requirements without dedicated effect-monitoring tests in this slice.

## Out of scope

- Query producer/registration in this commit: intentionally reserved for parent after RED.
- 3D rendering and model-scene construction: cached consumer's boolean choice suffices.
- PartyMember fields, per-unit state map, fabricated race records or aliases: unsupported by chosen minimal model.
- Race/stance/barber-derived input, native probes, broader profiles and real cosmetic-state production: no evidence or authorization for this slice.
- Audit/accounting/wiki updates, ignored PLAN, concurrent documentation, delegation and verification commands: excluded by bounded request.
