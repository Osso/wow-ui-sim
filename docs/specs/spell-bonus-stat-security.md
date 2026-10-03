# Spell bonus school authentication and existing stat outputs — B82

Bounded Retail 12.0.5 contract for `GetSpellBonusDamage(school)` original-input authentication, preserving the existing intellect-backed stat output. `GetSpellBonusHealing()` is an unchanged shared-output control, not a new input policy. Source: `src/lua_api/globals/cooldown_probes.rs`; output model: `src/c_api/c_secrets.rs`. See [Lua API boundary](../lua-api.md) and [existing stat output contract](unit-stat-output-restriction.md). Inputs `17211be121` compiled and executed before producer edits: **4 PASS /3 FAIL**. Main inspected expected school-auth failures; minimal producer now authenticates original argument1 before existing stat/output work. Main accepts independent716 bounded behavior and qualified gates below:12 current focused cases plus current startup/loaded probe. Compiler-check protocol deviation is retained; no native/full-API or argument-removal credit.

## What it must do

### Original school input

- [x] Accept original authentic HOST secret NUM school2 in an untainted caller; return the actual existing seeded intellect proxy2500, exactly one result. Preserve original wrapper identity/secrecy and caller trust through explicit restriction false→true→false.
- [x] Deny original secret school in a tainted caller **before state/output work**, including intellect0 and explicit restrictiontrue. Error identifies `GetSpellBonusDamage`, original argument1 and untainted-caller requirement without leaking private payloads. Denial preserves caller taint, host intellect/restriction and public-call recovery; outer secure context restores.
- [x] Authenticate original secretNil/BOOL/table/actual Frame payloads even though school is otherwise informational/unused. Tainted callers must receive authorization denial, not payload parsing or a returned proxy. Retain original visible table/Frame properties and wrapper secrecy. No secure malformed-input policy is introduced.
- [x] Public numeric school2 in a tainted caller continues working: public output with explicit flagfalse, opaque secret output with flagtrue, original caller taint retained.
- [x] Permanent Lua global roots and permanent global list retain actual host identities, wrapper allocation sequences, secrecy and original table/Frame properties across allocation/forced GC, secure acceptance, tainted denial and flag toggles. No tainted-secret BOOL equality.

### Existing output model — preservation, not new wrapping

- [x] Explicit `unit_stats_restricted` false→true→false governs both damage and unchanged healing outputs. Preserve actual value2500 and arity1; secure callers may inspect the restricted value through existing `secretunwrap`, addons cannot unwrap or perform arithmetic. Existing `canaccessvalue` predicate remains false for secret outputs. `C_Secrets.ShouldUnitStatsBeSecret()` stays a public flag.
- [x] Live intellect mutation2500→819 updates damage/healing outputs without new formulas; changing one environment's intellect/restriction does not affect a separate environment seeded731.

Cached Retail `Blizzard_APIDocumentationGenerated/PlayerScriptDocumentation.lua:869–883` declares damage `SecretArguments = "AllowedWhenUntainted"`, required `school` of type `luaIndex`, and `SecretWhenUnitStatsRestricted = true`. Healing retains the same output annotation without a school input. These are cached declarations, not native-client execution evidence. Exact482/484 retained deltas concern **output secret flags only**: no school argument removal or new argument-removal credit. Authentication scope follows the existing cached argument policy plus the observed runtime defect; retained484 is a preservation control only.

The simulator currently ignores school and returns a single intellect bucket through existing `push_stat_number`. This is an **inferred approximation**, not per-school damage, native spell-power calculation, or a claim that arbitrary inputs are valid. Tests use public school2 and original secret NUM2 for success; secure wrong-type/nil/range/per-school policy remains unmodeled. No expectations impose new ordinary type errors.

## How it works

- [Lua API / VM boundary](../lua-api.md)
- [Existing stat-output contract and proof](unit-stat-output-restriction.md)
- [B81 authentic host root/input patterns](tooltip-unit-debuff-security.md)

## Implementation inventory

- `tests/spell_bonus_stat_security.rs`: seven authored public-API cases; actual host inputs, original VM wrappers and actual Frame backing, no query replacements or mocks. `#[cfg(feature = "retail-12-0-5")]`; existing `build.rs` automatically discovers it in grouped `tests/integration.rs`, no new Cargo target.
- `src/lua_api/globals/cooldown_probes.rs`: damage authenticates original school with VM `unwrap_secret` before state/output work under `retail-12-0-5`, adding API/argument context to denial. Payload remains informational after authentication; no new ordinary validation or school formulas. Below that feature, callback behavior is unchanged. Healing callback and shared existing intellect proxy unchanged.
- `src/c_api/c_secrets.rs`: unchanged `push_stat_number` and public restriction predicate; no output-wrapper reimplementation.
- `src/lua_api/state`: existing explicit `unit_stats_restricted` and player intellect inputs, unchanged.
- `tests/character_stats/stat_restriction_fixtures.lua`: existing damage2/healing500 output-model evidence; not duplicated as a whole stat matrix.

## Tests asserting this spec

All seven cases compiled and executed at `17211be121`: four secure/public/output controls PASS; three tainted-auth/GC schedules FAIL at `AllowedWhenUntainted school must be denied before output`. This is expected behavioral RED, not current GREEN or downstream GC/root completion. Tests invoke registered public APIs on real `WowLuaEnv` state, not stand-in callbacks. All seven cases now refreshed PASS at `d21d4a208`, independently inspected716; table credit applies current GREEN, not retroactive RED branch completion.

| Exact case | Concrete boundary | Proof |
|---|---|---|
| `secure_original_secret_school_returns_existing_intellect_and_preserves_trust` | Original HOST NUM2, intellect2500, arity1, secure trust, false→true→false and identical roots | Refreshed PASS |
| `tainted_original_secret_school_is_denied_even_for_zero_or_restricted_output` | NUM2 denial with (2500,false), (0,false), (0,true), contextual error, unchanged state and public recovery | Refreshed PASS |
| `tainted_secret_nil_bool_table_and_actual_frame_authenticate_before_unused_payload` | Original secretNil/false BOOL/private table/actual Frame, authorization before unused payload, no private marker leak | Refreshed PASS |
| `public_tainted_school_follows_explicit_output_flag_and_retains_caller_taint` | Public2, flag toggles, opaque addon results and preserved trust | Refreshed PASS |
| `damage_and_unchanged_healing_share_value_arity_and_false_true_false_output_control` | Damage/healing2500, one result, public predicate, secure value and addon opacity | Refreshed PASS |
| `live_intellect_mutation_and_restriction_are_environment_local` | 2500→819 versus separate731, live outputs and restriction isolation | Refreshed PASS |
| `permanent_global_list_and_wrapper_allocations_survive_gc_calls_denials_and_toggles` | Actual global/list identities and alloc_seq through allocations/GC/calls/denials/toggles; original properties | Refreshed PASS |

### Qualified prior full-runtime failure

Supplied corrected probe record `/tmp/patch-12.0.5-spell-bonus-school-probe-correction.md` preserves pinned producer `d7d7b41fc`, wow-sim SHA256 `a05b5a30e9775933661e005ebff8894497682ef3ec57e05cbd61b7ed7f932073`, observed HEAD `7168c1eb5`, four matched source/dependency guards, worker3876875 duration3.612368066s. **STDERR**251–252 contains `B82_SCHOOL_AUTH public-accepted secure-accepted tainted-accepted` followed by assertion error `GetSpellBonusDamage AllowedWhenUntainted school not denied`. This is local simulator failure, not native evidence or B82 grouped test execution.

Original runtimeJSON `probe_executed=false` arose from a stdout-only marker search, not an unexecuted probe. Corrected parser inspected both streams without rerunning; originals remain preserved. Exit0 means CLI completion, **not assertion success or API PASS**. FIRST_FRAME_RENDERED does not prove a displayed GUI/native desktop; local font/CASC logs confer no native/API parity. No new startup/acceptance credit.

Artifacts remain `/tmp/patch-12.0.5-spell-bonus-school-probe-ops/{status.json,runtime.json,runtime.stdout,runtime.stderr}` and `/tmp/patch-12.0.5-spell-bonus-school-probe.lua`. These supplied investigation paths are provenance references, not test dependencies.

## Known gaps (current cycle)

- [x] Main compiled grouped RED before producer edit: `17211be121743b9261e986f96bbe3d3b36d6ccf6`, compile exit0/zero diagnostics90.716985s; run101/1.115939s, **4 PASS /3 FAIL**, SHA256 `7944d930d15983447909f49a0dc0d9be2953bde4d64ebf496f576f35caa1682d`. `/tmp/patch-12.0.5-spell-bonus-stat-red-ops/` retains exact argv/full streams/artifact feature records. Failures reach tainted-school acceptance before downstream root/GC assertions, not missing target or compile failure.
- [x] Before-state/output ordering is the required boundary. Contextual denial and unchanged state assert externally visible behavior; they do not instrument/count internal reads. Downstream roots/GC assertions may remain unreached when denial fails.
- [x] Main accepts independent716 bounded behavior; retained482/484 receive explicit-state output-annotation credit only. Independent metadata validation remains separate; original inputs commit supplied no current acceptance or output rewrap implementation.

## Independent bounded acceptance — 2026-10-03

Main accepts716 `/tmp/patch-12.0.5-spell-bonus-stat-independent-proof.{md,json}`: **12 distinct refreshed PASS** =7 new+4 existing restriction controls+1 proxy control. Producer `d21d4a208da4b911f003181e5ca33163f42b92be` (parent `71c384eec`), compile exit0/zero diagnostics108.741957s; runs1.407465s/0.727129s/0.278497s. Integration SHA256 `90f26231deda006a6dc80603fbb4ea2eb18eb932ba0cf831bd8bba61b35661f5`. Full source/test/embedded Lua/readability/wiring and seven source/dependency guards accepted; no new findings or behavioral counterexample. Old-profile body reconstructed byte-identically after removing the eight-line modern gate, not executed.

Current local startup exit0 `[]`/4.739207s, CLEAN0 and `casc=false`; same loaded school probe exit0/3.507810s has STDERR marker **public-accepted secure-accepted tainted-denied** and no exec-Lua error. The probe's actual assertions establish denial, not process0 alone. Simulator SHA256 `82e09ca5ecfce367d82fa8ba88f81903060408a8991a6d56a8093358ce71086d`. Probe font logs `casc=true` and FIRST_FRAME_RENDERED event confer no native/GUI/CASC integration clearance.

Scoped format exit0. **Qualified check:** a single `cargo check --message-format=json` produced complete streams, build-finished.success=true and zero diagnostics (stderr15.80s), but verifier mistakenly executed it synchronously through terminal `stderr_to_file`; direct exit was not retained. This deviation is not cleared, no exit0/asynchronous-check claim and no rerun merely to recover it. Actual current test build exit0 independently supplies compilation proof. Dirty-combined provenance/guarded source equality is not clean whole-tree identity; wiki-only `d5a6c4e2a` drift does not alter owned inputs. Historical globalfmt1 at unowned `aura_duration.rs:44` remains.

Exact482/484 add output annotations only. Current tests establish bounded explicit restriction-state wrapping, arity/value preservation, opacity and live/intellect isolation for both real registered producers. The output helper was preserved, not newly implemented; cached damage school authentication is separate from retained delta credit. [Page accounting](../../data/patch-api/sources/12.0.5-page-coverage.json) owns86 capabilities/362 ordered IDs and165 pending/179 bounded/11 partial/7 metadata. Full APIs, per-school/native formulas, activation, ordinary malformed/nil/range policy, older profiles and broad suites remain unaccepted. Original runtime failure and RED4/3/downstream limitations above retained.

## Out of scope

- Native-client parity, per-school formulas/buckets, native nilability/error wording, required/omitted/nil/wrong-type/range validation: unverified; current ignored informational school and ordinary public type policy are unmodeled.
- Automatic activation of unit-stat restriction from combat, auras, access context or other state: explicit host flag only.
- Older-profile behavior: new tests gated off below `retail-12-0-5`; no old-profile verification or compatibility credit.
- New healing input policy, producer changes, output-wrapper duplication, whole-stat matrix duplication, aura/vendor/Cargo changes, other files, builds/checks/tests/readability/coverage/startup/acceptance execution, push/deploy: unauthorized for B82 inputs-only work.
