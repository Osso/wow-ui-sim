# Action display and use-count outputs — exact237/241

Batch66 covers only the Retail 12.0.5 output annotation deltas for `C_ActionBar.GetActionDisplayCount` (row237) and `C_ActionBar.GetActionUseCount` (row241). Primary source: profile cache `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ActionBarFrameDocumentation.lua`, lines190–208 and277–290. The parent correction supersedes the earlier assertion that full documentation was unavailable: the filename is **ActionBarFrameDocumentation.lua**, not ActionBarDocumentation.lua. This contract uses explicit simulator inputs; it does not claim native acquisition. See [Lua API architecture](../lua-api.md).

Both blocks declare `RequiresValidActionSlot=true`, `SecretWhenCooldownsRestricted=true`, and `SecretArguments="AllowedWhenUntainted"`. Display takes nonnil `actionID:luaIndex`, `maxDisplayCount:number` default9999 and `replacementString:cstring` default`"*"`, returning one nonnil STRING. Use takes nonnil `actionID:luaIndex`, returning one nonnil NUMBER. Display documentation: “Depending on the action type, return a string that is either the use count or number of charges.” Beyond the maximum, replacement is returned; equality does not replace. No per-argument NeverSecret annotation is present. Input authentication is necessary for the whole function contract, not new input-annotation credit.

## What it must do

### Explicit inputs and selection

- [ ] Public `Debug + Clone + Copy + PartialEq + Eq` `ActionUseCountInfo { spell_id:u32, count:u32 }` and `SimState.action_use_counts:HashMap<u32,ActionUseCountInfo>` are gated by `retail-12-0-5`; the map is slot-keyed and empty by default. **Inferred simulator width/domain policy**, not a native cap or populated inventory.
- [ ] Match stored `spell_id` to the slot's current `action_bars` binding. Distinct slots17→19750/count7 and19→642/count2 remain independent, even when both bind the same spell. No hidden inventory links, fake production counts, acquisition, catalog, or consumption.
- [ ] **Inferred absence policy:** no assignment, missing record or stale identity yields use NUM0. Display with no source yields STRING`""`; explicit count0 yields `"0"`, distinct from absence. Unknown positive slots follow the same policy, without pruning stale host records.
- [ ] **Inferred charge classification/priority:** mapped spell's existing typed `SpellChargeState.current_charges` supplies display quantity only when `max_charges>0`; otherwise use matched explicit count. Preserve existing valid-charge read semantics, including zero current charges and ignored max0 entries. Charges never supply use counts. Explicit current3/max5 plus host use7 yields use7/display`"3"`.
- [ ] Mapping swaps and clears, count replacement/removal, charge replacement/clear and restriction-flag updates are immediately visible. Swap slot17 to642 invalidates old count without mutation; supplying count9 for642 yields use9/display`"9"` when no charges. Slot19 remains unchanged.
- [ ] Reads, scalar/root replacement and aliases leave bindings, counts and charges unchanged; environments remain isolated. Return exactly ONE scalar, never a table, nil or multi-value DTO.

### Display formatting

- [ ] Concrete quantities format as decimal ASCII, unlocalized integer strings. Quantity strictly greater than max returns the supplied replacement; equality returns digits. Default9999 returns `"9999"` for9999 and `"*"` for10000. Count7/max6 replaces; max7 returns`"7"`; max6.5 replaces.
- [ ] **Inferred formatter domain/default policy:** omitted or nil max/replacement use9999/`"*"`; threshold accepts finite f64 including negative/fractional values. Negative−1 replaces supplied zero. No-source display remains empty regardless of threshold.
- [ ] Empty and UTF8/nonASCII replacements are exact strings, including `""` and `"é雪"`. **Required representability policy, inferred native errors:** replacement must be a UTF8, NUL-free CString; reject wrong types, invalid UTF8 and embedded NUL even on unknown/no-source slots.
- [ ] u32MAX is an exact public numeric use result and decimal`"4294967295"` when threshold permits it; lower/fractional/default thresholds replace it. This is local integer representation, not proof of native quantity limits.

### Argument authentication and errors

- [ ] Authenticate original display positions1–3 and use position1 via the VM's `unwrap_secret` before type parsing, model lookup or source choice. Authenticate all documented positions before rejecting invalid public inputs. Public tainted calls remain allowed; secure secret NUM slot/max and secret STRING replacement work with meaningful assigned data.
- [ ] **Inferred strict slot domain:** finite integral positive u32 only. Reject omitted/nil, BOOL, STRING, table/frame, zero, negative, fractional, nonfinite and out-of-range values. Reject secure authenticated wrong types/invalid numbers ordinarily; permit secure secret nil formatter defaults.
- [ ] Tainted secrets of all six kinds (NUM, STRING, BOOL, nil, table, actual frame) deny before known/unknown/missing lookup, and before invalid/missing public arguments at other positions. No fake callbacks, security-query replacement or insecure declassification.
- [ ] Nonempty public error strings identify exact API and argument position without private payloads; caller trust/taint survives rejection and recovery. Preserve authentic wrapper metadata/identity through rooted copies and GC; do not use tainted secret BOOL equality as identity proof.

### Output privacy

- [ ] After positive assigned provider assertions, apply existing `charge_state::cooldowns_are_restricted` Retail125/PTR predicate separately to actual host-secret STRING display and NUM use outputs. Unrestricted values have public Lua string/number types and exact payloads. Do not borrow output-field policies from rows231/233/239.
- [ ] **Inferred zero/empty restriction policy:** restricted missing use0 and no-source empty display are typed secrets, as are supplied zero and replacement strings. Restricted outputs are authentic opaque VM userdata, not nominal primitive Lua types. Assert secret metadata and trusted-host `Val::Num`/`Val::Str` exact UTF8 payloads; no secret nominal LuaType requirement.
- [ ] Public tainted callers can receive/copy restricted values without changing caller frames, but cannot access payloads. Tainted NUM math and STRING concatenation/length fail without private leaks; ordinary opaque observations such as tostring must not expose payloads. No blanket native/global privacy parity claim.
- [ ] Root returned opaque values on Lua globals before host inspection/GC. Tainted copies preserve wrappers; original and fresh outputs survive GC with exact host payloads and metadata identity. After flag-off fresh outputs are public while old secrets retain privacy; secure recovery remains available.
- [ ] After main establishes compiled behavioral RED, implement first-class getters in planned `src/c_api/c_action_bar_counts.rs`, imported into the single existing namespace registration with inverse-gated old stubs. No handler, publication, provider-selection or charge-helper changes before RED.

## How it works

- [Lua API architecture](../lua-api.md)
- [Client profiles](../wiki/systems/client-profiles.md)
- [Audit accounting](../wiki/systems/patch-api-audit-manifest.md)

## Implementation inventory

- `src/c_api/action_count_info.rs`: explicit two-field public host snapshot only.
- `src/c_api/mod.rs`: epoch125 module and C API root export; no registration changes.
- `src/lua_api/state/sim_state.rs`: public epoch125 slot-keyed map.
- `src/lua_api/state.rs`: empty epoch125 initializer.
- `src/lua_api/globals/action_bar_api.rs`: unchanged current local display-nil/use0 stubs; no provider implemented.
- `src/lua_api/globals/action_bar_api/registration.rs`: unchanged existing callback entries.
- `src/c_api/charge_state.rs`: existing typed charge model/restriction predicate; unchanged.
- Planned `src/c_api/c_action_bar_counts.rs`: not created in this scaffold; getter work waits for compiled RED.

## Tests asserting this spec

`tests/action_count_outputs.rs`: 28 substantive cases, grouped integration autodiscovery; no new Cargo target. Real namespace/model fixtures, authentic VM inputs and outputs, scoped Retail125/PTR gate. Embedded conditions are split to avoid dense expression chains. Real `register_table_security` fixture helper is permitted; no API callbacks or security queries are replaced.

## Known gaps (current cycle)

- [ ] **Proof ledger:** no build, test, check, lint, readability gate or operations executed for batch66. Owned Rust formatting only. All requirements remain unchecked; no RED/GREEN/acceptance claim. Current nil/0 stubs are expected to fail meaningful data/format assertions, but expectation is not executed evidence; downstream security/GC proof remains absent.
- [ ] Main must compile grouped tests and establish behavioral RED before getter work. Compilation errors are not RED. Later implementation/verification is outside this input-only change.
- [ ] Native probes deferred, not a gate: actual use-count versus charge action kinds; concrete9999/10000 default-boundary quantities; secure and tainted scalar opacity/type; missing data, coercion and error behavior. None captured here.
- [ ] Parent-reported checkpoint:194 pending/153 bounded/14 partial/1 metadata,362 IDs/72 capabilities after combined233/239 acceptance `5523335834f0b3ed26d1c1168f2551b82419111a`;237/241 remain pending. That acceptance supplies no credit here. No other capability data, wiki, PLAN or specs changed.

## Out of scope

Native UI/consumer parity, primitive nominal secret types, native datasets/source acquisition, full action-type classification, all-profile or global privacy parity. Legacy `GetActionCount`, Consumable/Stackable predicates and other property parsing/methods remain untouched. No native charge acquisition or automatic consumption. No protected aura implementation access or modification. No publication/provider/charge-helper logic until main's compiled RED.
