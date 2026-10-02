# Action display and use-count outputs — exact237/241

Batch66 covers only the Retail 12.0.5 output annotation deltas for `C_ActionBar.GetActionDisplayCount` (row237) and `C_ActionBar.GetActionUseCount` (row241). Primary source: profile cache `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ActionBarFrameDocumentation.lua`, lines190–208 and277–290. The parent correction supersedes the earlier assertion that full documentation was unavailable: the filename is **ActionBarFrameDocumentation.lua**, not ActionBarDocumentation.lua. This contract uses explicit simulator inputs; it does not claim native acquisition. See [Lua API architecture](../lua-api.md).

Both blocks declare `RequiresValidActionSlot=true`, `SecretWhenCooldownsRestricted=true`, and `SecretArguments="AllowedWhenUntainted"`. Display takes nonnil `actionID:luaIndex`, `maxDisplayCount:number` default9999 and `replacementString:cstring` default`"*"`, returning one nonnil STRING. Use takes nonnil `actionID:luaIndex`, returning one nonnil NUMBER. Display documentation: “Depending on the action type, return a string that is either the use count or number of charges.” Beyond the maximum, replacement is returned; equality does not replace. No per-argument NeverSecret annotation is present. Input authentication is necessary for the whole function contract, not new input-annotation credit.

## What it must do

### Explicit inputs and selection

- [x] Public `Debug + Clone + Copy + PartialEq + Eq` `ActionUseCountInfo { spell_id:u32, count:u32 }` and `SimState.action_use_counts:HashMap<u32,ActionUseCountInfo>` are gated by `retail-12-0-5`; the map is slot-keyed and empty by default. **Inferred simulator width/domain policy**, not a native cap or populated inventory.
- [x] Match stored `spell_id` to the slot's current `action_bars` binding. Distinct slots17→19750/count7 and19→642/count2 remain independent, even when both bind the same spell. No hidden inventory links, fake production counts, acquisition, catalog, or consumption.
- [x] **Inferred absence policy:** no assignment, missing record or stale identity yields use NUM0. Display with no source yields STRING`""`; explicit count0 yields `"0"`, distinct from absence. Unknown positive slots follow the same policy, without pruning stale host records.
- [x] **Inferred charge classification/priority:** mapped spell's existing typed `SpellChargeState.current_charges` supplies display quantity only when `max_charges>0`; otherwise use matched explicit count. Preserve existing valid-charge read semantics, including zero current charges and ignored max0 entries. Charges never supply use counts. Explicit current3/max5 plus host use7 yields use7/display`"3"`.
- [x] Mapping swaps and clears, count replacement/removal, charge replacement/clear and restriction-flag updates are immediately visible. Swap slot17 to642 invalidates old count without mutation; supplying count9 for642 yields use9/display`"9"` when no charges. Slot19 remains unchanged.
- [x] Reads, scalar/root replacement and aliases leave bindings, counts and charges unchanged; environments remain isolated. Return exactly ONE scalar, never a table, nil or multi-value DTO.

### Display formatting

- [x] Concrete quantities format as decimal ASCII, unlocalized integer strings. Quantity strictly greater than max returns the supplied replacement; equality returns digits. Default9999 returns `"9999"` for9999 and `"*"` for10000. Count7/max6 replaces; max7 returns`"7"`; max6.5 replaces.
- [x] **Inferred formatter domain/default policy:** omitted or nil max/replacement use9999/`"*"`; threshold accepts finite f64 including negative/fractional values. Negative−1 replaces supplied zero. No-source display remains empty regardless of threshold.
- [x] Empty and UTF8/nonASCII replacements are exact strings, including `""` and `"é雪"`. **Required representability policy, inferred native errors:** replacement must be a UTF8, NUL-free CString; reject wrong types, invalid UTF8 and embedded NUL even on unknown/no-source slots.
- [x] u32MAX is an exact public numeric use result and decimal`"4294967295"` when threshold permits it; lower/fractional/default thresholds replace it. This is local integer representation, not proof of native quantity limits.

### Argument authentication and errors

- [x] Authenticate original display positions1–3 and use position1 via the VM's `unwrap_secret` before type parsing, model lookup or source choice. Authenticate all documented positions before rejecting invalid public inputs. Public tainted calls remain allowed; secure secret NUM slot/max and secret STRING replacement work with meaningful assigned data.
- [x] **Inferred strict slot domain:** finite integral positive u32 only. Reject omitted/nil, BOOL, STRING, table/frame, zero, negative, fractional, nonfinite and out-of-range values. Reject secure authenticated wrong types/invalid numbers ordinarily; permit secure secret nil formatter defaults.
- [x] Tainted secrets of all six kinds (NUM, STRING, BOOL, nil, table, actual frame) deny before known/unknown/missing lookup, and before invalid/missing public arguments at other positions. No fake callbacks, security-query replacement or insecure declassification.
- [x] Nonempty public error strings identify exact API and argument position without private payloads; caller trust/taint survives rejection and recovery. Preserve authentic wrapper metadata/identity through rooted copies and GC; do not use tainted secret BOOL equality as identity proof.

### Output privacy

- [x] After positive assigned provider assertions, apply existing `charge_state::cooldowns_are_restricted` Retail125/PTR predicate separately to actual host-secret STRING display and NUM use outputs. Unrestricted values have public Lua string/number types and exact payloads. Do not borrow output-field policies from rows231/233/239.
- [x] **Inferred zero/empty restriction policy:** restricted missing use0 and no-source empty display are typed secrets, as are supplied zero and replacement strings. Restricted outputs are authentic opaque VM userdata, not nominal primitive Lua types. Assert secret metadata and trusted-host `Val::Num`/`Val::Str` exact UTF8 payloads; no secret nominal LuaType requirement.
- [x] Public tainted callers can receive/copy restricted values without changing caller frames, but cannot access payloads. Tainted NUM math and STRING concatenation/length fail without private leaks; ordinary opaque observations such as tostring must not expose payloads. No blanket native/global privacy parity claim.
- [x] Root returned opaque values on Lua globals before host inspection/GC. Tainted copies preserve wrappers; original and fresh outputs survive GC with exact host payloads and metadata identity. After flag-off fresh outputs are public while old secrets retain privacy; secure recovery remains available.
- [x] After main establishes compiled behavioral RED, implement first-class getters in planned `src/c_api/c_action_bar_counts.rs`, imported into the single existing namespace registration with inverse-gated old stubs. No handler, publication, provider-selection or charge-helper changes before RED.

## How it works

- [Lua API architecture](../lua-api.md)
- [Client profiles](../wiki/systems/client-profiles.md)
- [Audit accounting](../wiki/systems/patch-api-audit-manifest.md)

## Implementation inventory

- `src/c_api/action_count_info.rs`: explicit two-field public host snapshot only.
- `src/c_api/mod.rs`: epoch125 input module/root export and crate-visible getter module; no parallel registration.
- `src/lua_api/state/sim_state.rs`: public epoch125 slot-keyed map.
- `src/lua_api/state.rs`: empty epoch125 initializer.
- `src/lua_api/globals/action_bar_api.rs`: epoch125 imports replace the two existing callback names; local display-nil/use0 callbacks are inverse-gated for earlier epochs/Forever. Other methods remain unchanged.
- `src/lua_api/globals/action_bar_api/registration.rs`: unchanged existing callback entries.
- `src/c_api/charge_state.rs`: existing typed charge model/restriction predicate; only `read_charge_input` visibility becomes crate-visible. Its body and semantics are unchanged.
- `src/c_api/c_action_bar_counts.rs`: first-class getters authenticate original VM inputs, validate strict domains, snapshot matching slot use counts/current spell charges/restriction under one immutable borrow, format concrete u32 quantities and immediately root typed host-secret or public scalar results. No new state, acquisition, consumption, pruning, classification catalog or alternate publication.

## Tests asserting this spec

`tests/action_count_outputs.rs`: 28 substantive cases, grouped integration autodiscovery; no new Cargo target. Real namespace/model fixtures, authentic VM inputs and outputs, scoped Retail125/PTR gate. Embedded conditions are split to avoid dense expression chains. Real `register_table_security` fixture helper is permitted; no API callbacks or security queries are replaced.

Bounded test-only readability follow-up extracts scalar publication, table/frame wrapping/publication and input metadata validation from `secret_fixture`; expands remaining dense Lua calls, assertions, loops and callback bodies. Same publication order, root push/pop, pinned table-security registration, caller frames, queries, model inputs, errors and metadata checks. No production/state/Cargo/Forever changes or additional tests. Formatting is test-file-only with `skip_children=true`; compilation, refreshed current28 runtime and independent readability/equivalence acceptance remain pending main67. Source237/241 and all requirement/accounting statuses remain pending/unchanged.

## Saved parent GREEN and initial independent gate — 2026-10-01

Default integration compiled `f081fedb9af52e8642abf3fcd810513ee6f87a4a` successfully in234.80088983406313s with zero diagnostics. `/tmp/patch-12.0.5-batch66-green-build-result.json` and full compiler streams bind integration SHA256 `3c33bc66c4e58aaf3ca1a6b59076ed0cb6081c19b727a184306245d21f6a41e5`. Seven finite runs record145 distinct Retail PASS:28 focused plus117 controls,42.36772962694522s; startup returns `[]`, exit0,7.412627534009516s. This is bounded development below60s, not padded whole-goal acceptance.

Independent521 confirms source/model/security/wiring and frozen145 PASS, fresh scoped fmt/default check exit0, plus36 separate existing Forever controls and successful GUI-enabled compile. `/tmp/patch-12.0.5-action-count-independent-proof.md` and `/tmp/batch66-independent-gates.json` retain full hashes/costs/provenance. Three dense embedded-Lua literals were accepted after source inspection and rewritten multiline; assertions/order/values remain identical. Refreshed28-test runtime and independent readability/equivalence proof remain pending.

Concurrent Batch67 added only a Retail125-gated spell-count map/default to two shared state files. Initial default check/format remain pre67 evidence, not current-whole-tree proof. Forever’s feature projection excludes that field; prior Forever compile/36 controls remain source-valid, without claiming unchanged whole-file hashes. Main will refresh current-default compilation/check with the next source snapshot rather than replay valid controls blindly. Typed-secret payload/native nominal-type and all inference/process/global-format/dirty limits remain.

Header uses applicable instructional date; observed host Git/build/gate timestamps are separately retained in artifacts and may differ. No provenance redating.

## Independent bounded acceptance — 2026-10-01

Parent accepts independent521 source/model/security/wiring and frozen145 Retail PASS, independent523 fresh default check including Batch67’s empty gated state field, and525 final source-equivalence/readability/format/current28 runtime audit. `/tmp/patch-12.0.5-action-count-final-followup.md` and `.json` own reconciliation:28 refreshed PASS at `23ac1c89d4437343726a6b814515c14c8399d254`, integration SHA256 `a65bdce18f20fe0e23dfeedeb5939c1f672ae9a55b660151f66f6144437ab7db`,6.491112876916304s. Test SHA256 `ecae155e6db44ab1601d2a7e3a5eb47b954eb0be5d27bae6c3200aec207494d4`; full test/helper audit finds no violations. Shared refresh compilation420.46007691998966s is separate from execution.

Twenty-eight refreshed plus117 inherited controls =**145 distinct Retail PASS**, not fresh145 or173. Historical startup0 `[]` and36 separate existing Forever controls remain source-valid; two new state entries compile out of Forever. Default check523 exits0/no warnings in29.718483s on current typed additions; test-only fmt525 exits0. Main/source scopes and whole-file hash changes remain explicit, not a whole-tree or unchanged-allowlist assertion. No valid control/startup/Forever gates rerun solely for milestones.

Only exact237/241 promote: **194 pending /153 bounded /14 partial /1 metadata →192 /155 /14 /1**,362 ordered unique IDs,73 capabilities. Prior72 capabilities,360 unrelated rows/source hashes retained in `/tmp/patch-12.0.5-batch66-accounting-before.json` and postcommit `batch66-accounting-validation.json`. Acquisition, priority/domain/default/native primitive/global privacy/UI/profile limitations remain; globalfmt1, dirty-combined and historical502 process failure remain failures. Broader goal stays open. Header uses applicable main instructional date; observed child/host gate dates remain separate.

## Known gaps (current cycle)

- [x] **Historical RED ledger:** saved parent-reported compiled RED at `d0c7969641adf566502d840d9651ed54c8fedca5` precedes this producer. Build exit0,608.5359100290807s; auto-background job completed and full log was recovered, with no rerun and zero diagnostics. Integration artifact hash `aec74c6d04bfef08bb9ec35c0538f080bda1a90a69989b8f15f3d4164e5e84ba`; 28 tests,0PASS/28FAIL, exit101,7.2183314569992945s. Exact parent command/log path was not supplied here; no replacement execution or provenance claim is invented. Historical build/host timestamps may be2026-10-02 and are not rewritten.
- [x] Earliest RED failures are positive use0 versus7/u32MAX and display-nil versus the scalar contract. Many secret/GC cases stop at positive baseline assertions; this is not28 independent authentication/GC failures. Fixtures use real namespace declarations and valid modeled inputs; no fixture changes are justified.
- [x] Producer, current focused GREEN, inherited controls/startup, fresh default compilation, formatting, full readability/equivalence and independent exact237/241 acceptance supplied above.
- [ ] Native probes deferred, not a gate: actual use-count versus charge action kinds; concrete9999/10000 default-boundary quantities; secure and tainted scalar opacity/type; missing data, coercion and error behavior. None captured here.
- [ ] Native acquisition/priority/domain/default/width/global-privacy/primitive nominal-type/UI/full-profile limitations remain unproved; broader goal stays open.

## Out of scope

Native UI/consumer parity, primitive nominal secret types, native datasets/source acquisition, full action-type classification, all-profile or global privacy parity. Legacy `GetActionCount`, Consumable/Stackable predicates and other property parsing/methods remain untouched. No native charge acquisition or automatic consumption. No protected aura implementation access or modification. No tests, new state or Cargo changes in this producer. No unrelated data, wiki or PLAN changes. Saved compiled RED authorizes only the exact237/241 getters and minimal charge-reader visibility change.
