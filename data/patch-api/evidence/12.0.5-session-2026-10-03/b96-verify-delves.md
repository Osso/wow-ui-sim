# C_DelvesUI verification — a0e23199d

Read-only verification started. Skill read; no delegation, compilation or repository mutation.

## Initial evidence
HEAD a0e23199dc60d4056add5f5a15b68f23dcac38cc; source/test diff empty; working tree clean. Three producers read/write explicit state. Eligibility unwrap_secret precedes validation. Curio rarity NeverSecret guard precedes real map lookup, not placeholder output. Cached declaration permits secret arg1 (AllowedWhenTainted), while public identifier reader rejects it: bounded qualification required.

## Proof ledger: delves_api_inputs::
Command: target/debug/deps/integration-8ea324359263a4d2 delves_api_inputs:: --test-threads=1
Source scope: a0e23199d; HEAD src/tests diff empty. Prebuilt binary provenance not independently authenticated.
Exit: 0
```text

running 9 tests
test delves_api_inputs::active_delve_does_not_access_secret_extras_for_either_caller ... ok
test delves_api_inputs::active_delve_reads_live_host_boolean_and_ignores_all_extras ... ok
test delves_api_inputs::curio_link_rejects_invalid_public_arguments ... ok
test delves_api_inputs::curio_link_uses_aliases_rarity_and_live_host_records_without_mutation ... ok
test delves_api_inputs::curio_rarity_is_never_secret_even_after_gc_and_for_tainted_callers ... ok
test delves_api_inputs::delve_inputs_are_isolated_between_environments ... ok
test delves_api_inputs::eligibility_authenticates_secret_map_before_validation_or_mutation ... ok
test delves_api_inputs::eligibility_invalid_inputs_preserve_last_request ... ok
test delves_api_inputs::eligibility_request_records_map_and_returns_zero_values ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 10152 filtered out; finished in 0.85s


```

## Proof ledger: delves_ui::
Command: target/debug/deps/integration-8ea324359263a4d2 delves_ui:: --test-threads=1
Source scope: a0e23199d; HEAD src/tests diff empty. Prebuilt binary provenance not independently authenticated.
Exit: 0
```text

running 1 test
test delves_ui::delves_ui_entrance_methods_use_seeded_tier_state ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10160 filtered out; finished in 0.13s


```

## Final assessment

### Per-row verdict

| Row | Verdict | Earned coverage / qualification |
|---|---|---|
| 262 | ACCEPT | HasActiveDelve reads live host boolean, not former mapID. Public extras ignored; default false and true→false transitions tested. |
| 263 | ACCEPT | No argument authentication/conversion; authentic secret number/string extras ignored for both caller classes, preserving taint/secrecy. |
| 265 | ACCEPT WITH QUALIFICATIONS | mapID recorded and replaced, zero return values, invalid-input nonmutation, AllowedWhenUntainted authentication tested. Signed i32 limits are inferred; eligibility responses/events remain out of scope. |
| 257 | ACCEPT WITH QUALIFICATIONS | Public numeric and case-insensitive alias identifiers resolve real keyed host links. Secret arg1 AllowedWhenTainted permission is NOT implemented; cached callers require host records absent by default. Not full GetCurioLink parity. |
| 258 | ACCEPT | Authentic secret rarity denied for both caller classes before conversion/lookup, including after full GC; recovery returns exact seeded links. Guard protects a real state-backed producer, not a placeholder. Does not earn secret arg1 policy. |

### Defects and merge risk

1. **Cached curio click-path compatibility regression:** src/lua_api/globals/missing_surface/delves_ui.rs:168–171 raises on every missing record; src/lua_api/state.rs:189 starts curio_links empty. Scanning src shows no population path. Cached Blizzard_DelvesCompanionConfiguration/Blizzard_DelvesCompanionConfiguration.lua:399 and :676 call GetCurioLink then InsertLink without pcall. Thus these calls error in the default environment whenever those branches execute; only manually supplied matching host records make them work. This is source-established, not a live UI reproduction. Do not restore fabricated placeholder links; provide actual host records or explicitly retain the compatibility gap. Nonnullable return documentation alone does not establish native miss errors.
2. **Secret identifier compatibility gap:** delves_ui.rs:158–162 delegates to src/c_api/c_spell.rs:673–676, which rejects secret spell identifiers for both caller classes. Cached DelvesUIDocumentation.lua:47 declares AllowedWhenTainted. Spec acknowledges this at docs/specs/delves-api-inputs.md:38; tests/delves_api_inputs.rs:263–265 assert rejection, not native permission. Narrow row 257 type-expansion credit only.
3. **Stale completion documentation:** docs/specs/delves-api-inputs.md:60 still says host fields/defaults need applying and all requirements remain unchecked, although those fields/defaults exist and the authorized binary tests passed. Authoring-slice history is not final integration status.

Merging today preserves the bounded host-model gains but leaves curio chat-link branches broken without injected records and secret arg1 parity incomplete. No runtime caller-compatibility test covers those branches.

### Tests and pre-commit discrimination

Observed: delves_api_inputs:: **9 passed, 0 failed**, exit 0; delves_ui:: **1 passed, 0 failed**, exit 0. Total **10/10**, no ignored tests. HEAD equals a0e23199dc60d4056add5f5a15b68f23dcac38cc; required src/tests diff empty; working tree clean. No cargo or repository mutations. Binary build provenance was not independently authenticated.

All nine new tests assert Lua outputs/errors or concrete host state, not implementation shape. Static comparison against a0e23199d^ producers predicts each fails (parent execution forbidden/no rebuilt parent):

| Test at tests/delves_api_inputs.rs | Parent failure discriminator |
|---|---|
| :75 active live state | Old HasActiveDelve(2339) returns true despite default false; host activation ignored. |
| :87 secret extras | Old producer cannot return seeded true independently of secret map conversion. |
| :113 request recording | Old no-op leaves last request None. |
| :133 invalid requests | Old no-op succeeds instead of erroring. |
| :155 authenticated request | Old no-op leaves None instead of Some(2339). |
| :196 curio records | Old fabricated [Curio] differs from exact fixture link. |
| :223 invalid curio inputs | Old defaulting producer succeeds instead of rejecting missing inputs. |
| :244 NeverSecret | Old defaulting producer returns fabricated link instead of NeverSecret error. |
| :283 environment isolation | Old fabricated link differs from second environment's seeded fixture link. |

Changed entrance test tests/delves_ui.rs:24 fails against parent because old producer returns true for 2339. Expectation change is justified by source lines 262–263 removing mapID and its argument annotation, plus cached zero-argument declaration. It does not prove native default false; default false is explicit host model policy tested separately.

### Secret authentication and cached callers

Eligibility delves_ui.rs:333–352 calls VM unwrap_secret before numeric validation or mutation. Pinned rilua 6044544 table_security.rs:232–245 authenticates via ensure_secure_caller before returning payload. Behavioral test :155 compares tainted secret-number/string denial, checks retained wrapper/taint, verifies no mutation and public-input recovery.

Cache search found five Lua call sites (excluding generated declarations): two zero-arg HasActiveDelve calls at Blizzard_FrameXML/InstanceDifficulty.lua:49 and Blizzard_DelvesDifficultyPicker/Blizzard_DelvesDifficultyPicker.lua:375 remain signature-compatible; request at difficulty picker :166 passes GetDelveEntranceMapID() (2339), valid and recorded; two curio calls fail on absent host links as above. Caller compatibility is static analysis, not execution of cached UI branches.

### Spec checkboxes provably earned

- Full bounded behavioral coverage: docs/specs/delves-api-inputs.md **:19, :21, :25–29, :33, :35–36, :39**. Inferred domains/miss behavior remain simulator policy, not native proof. Line :39 isolation test observes second environment's unchanged values; does not independently verify every first-environment postmutation value.
- **:20 partial:** untainted invalid public extras and tainted/untainted secret extras covered, but no tainted invalid-public-extra case.
- **:34 partial:** exact keyed outputs, arity/publicness/nonmutation tested; initially-empty curio map verified in initializer only, not an unseeded behavioral test.
- **:37 partial:** malformed rarity rejection tested; ordinary public-invalid-rarity versus invalid arg1 order, successful zero/u32-max rarity, and non-enum successful rarity not directly tested.
- **:38 partial:** public ordinary strings/numbers and secret-ID rejection tested; malformed UTF-8 and successful u32 boundaries not exercised. This checkbox explicitly excludes AllowedWhenTainted parity and cannot earn it.

Artifact checks: EXIST PASS (all requested files read); SUBSTANTIVE PASS for three bounded state-backed producers; WIRED PASS via missing_surface.rs:238 and 10 executed tests; ANTI-PATTERN PASS for changed producer file (TODO/FIXME/HACK/XXX each zero). Overall: ACCEPT WITH QUALIFICATIONS for bounded slice, NOT full cached caller/native compatibility.
