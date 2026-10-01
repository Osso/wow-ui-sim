# Unit aura altered-form query

`C_UnitAuras.WantsAlteredForm(unit: string) -> wantsAlteredForm: bool` reads explicit player input. Cached retail `Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua` declares one unit token, one boolean and `SecretArguments = "AllowedWhenUntainted"`. Patch source `data/patch-api/sources/12.0.5-api-changes.txt:407` changes `AllowedWhenTainted` to `AllowedWhenUntainted` (exact row `global api-C_UnitAuras-WantsAlteredForm-407`). Input lives in `src/lua_api/state_types/character_world.rs`; `src/c_api/c_unit_aura_altered_form.rs` owns the `retail-12-0-5` producer. See [query implementation](../wiki/systems/lua-api.md#retail-1205-altered-form-query).

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

- [Query implementation](../wiki/systems/lua-api.md#retail-1205-altered-form-query).
- [Client-profile runtime cache](../wiki/systems/client-profiles.md).
- [Existing unit identity behavior](unit-identity-equality.md).

## Implementation inventory

- `src/lua_api/state_types/character_world.rs`: explicit player boolean; derived Default supplies false, seeded initialization inherits it.
- `src/c_api/c_unit_aura_altered_form.rs`: authenticates the unit argument, strictly decodes a string, compares resolved identity with the actual player GUID and reads the explicit bool; one public boolean result.
- `src/c_api/mod.rs`, `src/lua_api/globals/register.rs`: epoch-gated module and registration into the existing shared namespace after aura setup.
- `src/lua_api/globals/unit_misc.rs`: unchanged identity resolver reused for both supplied unit and player.
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

## Producer coverage and proof boundary — 2026-10-01

Implemented requirements map to the existing ten fixtures above: explicit default/input, identity aliases and misses, independent state, required-string validation, secure/tainted secret access and GC recovery, and the unchanged cached consumer. No tests changed. Read-only borrowing and no dispatch enforce query immutability; barber/event monitoring gaps below remain.

No explicit same-function default was found in source; the namespace's generic missing-method metatable supplies the old synthesized function. The real namespace slot supersedes it; generic defaults and unrelated methods remain unchanged.

## Reconciled batch39 parent proof — 2026-10-01

Saved evidence only; this docs audit runs no builds, tests or gates. Inputs/spec/ten tests are from `732d1c5e65fa5dc23116ed55326bc5a5e7a6c393`; producer is `795e2042e7cdbec0426e563272cf6b8f4d44ae15`. The test file is unchanged between those revisions.

| Evidence | Revision | Recorded result |
| --- | --- | --- |
| `/tmp/patch-12.0.5-batch39-red-build-result.json` | Inputs | Cargo compiler success, 1060s (17m40s); wrapper failed after build when calling `.run()` on returned CommandResult. OS exit not retained. |
| `/tmp/patch-12.0.5-batch39-red-run.json`, `/tmp/patch-12.0.5-batch39-red-run.log` | Inputs | 10 selected: **1 PASS / 9 FAIL**, 8.878s observed; OS exit not retained. Default-input PASS is not producer proof. |
| `/tmp/patch-12.0.5-batch39-green-build-result.json` | Producer | Integration build exit **0**, 816.140s. |
| `/tmp/patch-12.0.5-batch39-green-altered-form.json`, `/tmp/patch-12.0.5-batch39-green-altered-form.log` | Producer | **10/10 PASS**, exit **0**, 16.099s wall time (15.86s test summary); 9424 tests filtered out. |

Integration binary `target/debug/deps/integration-a11e89d240f9bd0c` SHA256: RED `4dd33f290f64317f9f0880a9e0ef4ac9b765b59ddcbb856615633542a6f4f2f3`; GREEN `ebde3a19745cab8140e280c2ec4867766695cdb1462a3cd2ff7cf7551d03efd4`. Saved run-log SHA256: RED `ed6fc0b94d366cfaf505896bbf9b6aca921f562f410c87bde0524bf32ca2bb44`; GREEN `ecedc60e2f7e5165e89b6f97e364f33a814d539178ec3c00220ca1342ea6aa81`. RED compilation success does not establish exit 0; RED test failure does not establish exit 101.

| Bounded capability | Proof level |
| --- | --- |
| Explicit false/true/false input, one public boolean, defaults, GUID target/focus aliases and identity misses | Parent GREEN fixtures |
| Race/stance/legacy-field independence and strict required-string rejection/recovery | Parent GREEN fixtures; policy remains inferred |
| Actual host-secret string accepted securely, denied in tainted closures without clearing taint; public/secure recovery and rooted full-GC preservation | Parent GREEN VM boundary fixtures; not native security characterization |
| Complete unchanged cached `UnitUtil.lua`, real Human/Worgen/Dracthyr data and `PlayerUtil.ShouldUseNativeFormInModelScene()` | Parent GREEN consumer fixture; no SharedXML/addon/3D closure |
| Broader controls, normal startup, independent checks/readability and final acceptance | **Pending**, not established by these ten tests or fixture initialization logs |

Native defaults, misses, representation errors and security edge parity remain unknown. No source-accounting credit or whole-row completion follows from this bounded parent GREEN.

## Known gaps (current cycle)

- [ ] Parent GREEN covers the ten fixtures in the proof table; broader controls, normal startup, independent checks/readability and final acceptance remain pending. Requirement boxes remain open for final acceptance; no row promotion.
- [ ] Native source/default/non-player/error/security semantics remain unproven. False default/misses and strict representation errors are explicit simulator inferences.
- [ ] Consumer test requires populated active-profile cache with `Blizzard_SharedXML/UnitUtil.lua`; missing cache/source fails explicitly, never skips or substitutes code. The inspected retail file consists of function definitions and can be supplied unchanged to existing `WowLuaEnv::exec`; actual load/runtime passed in the bounded parent consumer fixture. This is not full SharedXML/addon/model-scene closure.
- [ ] GC fixture asserts retained reference equality via secure `rawequal`, secrecy and authenticated behavior, not raw VM pointer identity. Race/stance/legacy-field nonmutation is asserted; barber independence and absence of events are requirements without dedicated effect-monitoring tests in this slice.

## Out of scope

- 3D rendering and model-scene construction: cached consumer's boolean choice suffices.
- PartyMember fields, per-unit state map, fabricated race records or aliases: unsupported by chosen minimal model.
- Race/stance/barber-derived input, native probes, broader profiles and real cosmetic-state production: no evidence or authorization for this slice.
- Audit/accounting, ignored PLAN, concurrent documentation, delegation and verification commands: excluded by bounded request.
