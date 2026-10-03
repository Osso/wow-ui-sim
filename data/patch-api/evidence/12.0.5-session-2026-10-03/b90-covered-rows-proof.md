# B90 covered-row verification
Revision: `4190a4a07e4182ff48c1d373b369607e6dae3e42`. `git diff 62d0ce70f HEAD -- src tests Cargo.toml Cargo.lock build.rs`: exit 0, empty. Read-only review; only this report written.

## A — ACCEPT WITH QUALIFICATIONS
Row `global api-C_UnitAuras-GetAuraBaseDuration-371`: `# arg3.Type number -> SpellIdentifier`.
Assertions in `tests/aura_refresh_duration.rs`: lines 133–136 iterate `{99002, 'DURATION Override'}` and assert `C_Spell.GetSpellIDForSpellIdentifier(identifier) == 99002`, `C_UnitAuras.GetAuraBaseDuration('player', 101, identifier) == 40`; lines 152–155 assert omitted base `== 20`, explicit numeric alias base `== 40`; lines 173–176 assert unresolved overrides `== nil`, exactly one return, and omitted result `~= nil`; lines 116–122 assert omitted/nil one numeric return, equal results and base `== 20`.
Non-vacuous: positive fixture outputs 20 versus 40 distinguish aura-derived identity from numeric/string override; unknown overrides cannot pass as blanket nil because positive controls exist. `src/c_api/aura_duration.rs:117–122` uses matched aura ID on nil and existing identifier resolver otherwise.
Qualification: demonstrates bounded numeric/seeded string alias migration, not every native SpellIdentifier form, production metadata, or native security policy. No assertion directly tests generated API type metadata. Spec explicitly marks alias/security/duration policy inferred.
Coverage: status `audit-pending`; capabilities `[]`; note `Source retained; behavioral applicability audit not completed.`
Existing capability `aura-refresh-duration` names `docs/specs/aura-refresh-duration.md` and `tests/aura_refresh_duration.rs`, both APIs, bounded independent pass. Row can join that bounded capability without claiming native parity.

## B — ACCEPT
Row `prose-2026-03-12-023`: all three named charge-duration APIs return zero-span objects at maximum charges.
`tests/cooldown_probes/charge_duration.rs:209` calls `seed_charge(&env, 2)`; fixture lines 158–169 set current charges from argument, max 2, nonzero recharge duration 40 and rate 2. Lines 212–216 define three concrete producer functions calling C_Spell, C_ActionBar and C_SpellBook respectively. Lines 221–226 assert `d ~= nil`, `d:GetStartTime() == d:GetEndTime()`, `d:GetTotalDuration() == 0 and d:GetModRate() == 2`, `d:IsZero() and d:HasExpired() and not d:IsActive()`, and zero elapsed/remaining quantities.
Non-vacuous: loop traverses three nonnil functions, not an array of nullable query results; each function is called and its result explicitly asserted nonnil. Positive below-max control lines 185–191 demands nonzero duration 20 and not expired. Shared producer `src/c_api/charge_state.rs:84–106` selects `(now, 0.0, rate)` when at/above max under `retail-12-0-5`.
Literal statement covered for all three APIs, bounded to configured charge state and tested profile. Automatic charge progression, native rate/security behavior and all profiles are outside this statement/proof; spec `docs/specs/spell-charge-state.md:9,15` records max/elapsed policy, with interpretation of fully elapsed explicitly inferred.
Coverage: status `audit-pending`; capabilities `[]`; note: `Batch5 charge-duration 1 PASS / 3 FAIL. GetSpellChargeDuration resolves through runtime_surface_bootstrap.lua __wow_namespace_mt.__index lazy nil closure (lines 65–76), explaining no-data PASS; C_Spell.GetSpellCharges uses a temporary zero-table default. Models/producers missing, not API-global absence; final provider behavior unresolved. Exact argv/logs: /tmp/patch-12.0.5-batch5-runs.json. Implementations pending agents 73/74/75.` This note is historical, contradicted by current concrete producers/tests.
No existing capability names `docs/specs/spell-charge-state.md` or `tests/cooldown_probes/charge_duration.rs`; cannot join an exact existing spec/test capability.

## C — ACCEPT WITH QUALIFICATIONS
Row `prose-2026-03-12-026`: “Duration objects that measure a zero-span are now considered fully elapsed.”
`tests/cooldown_probes/charge_duration.rs:111–115`: `assert(d:IsZero() and d:HasExpired(), 'zero span is fully elapsed')`; `assert(d:GetElapsedPercent() == 1 and d:GetRemainingPercent() == 0)`; same fractions with modifier 1; zero elapsed/remaining durations. Lines 117–128 invoke this control on default, future-start zero span, advanced clock, reset and defaults. Line 124 asserts nonzero control `not d:HasExpired() and d:GetElapsedPercent() == 0.5`.
Non-vacuous: direct method calls on created object, five explicit control invocations, nonzero contrasting state. `src/lua_api/globals/lua_duration_object/core.rs:302–311,328–334` explicitly implements zero elapsed fraction 1 and expired true under `retail-12-0-5`.
Qualification: “fully elapsed” is concretized as expired and elapsed fraction 1, remaining fraction 0, not HasStarted (tests intentionally assert false). Native meaning of all predicates and secret-configured/all-profile zero spans is not established by these tests. Spec `docs/specs/duration-core.md` and linked charge spec distinguish simulator policy from native parity.
Coverage: status `audit-pending`; capabilities `[]`; note `Source retained; behavioral applicability audit not completed.`
Existing `duration-common-formatters` names the same spec but only `tests/duration_numeric_formatters.rs` and formatter symbols, not these core tests or zero-span behavior. No existing capability names the same spec AND tests; do not silently credit zero-span semantics to formatter scope.

## Independent runtime proof
Command for each filter: `timeout 90 /home/osso-test/Projects/wow/wow-ui-sim/target/debug/deps/integration-8ea324359263a4d2 FILTER --test-threads=1`, cwd repository. Empty source/test/build diff satisfied rerun condition. Four bounded filters run concurrently; no build.
- `aura_refresh_duration::` exit 0: `test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 10056 filtered out; finished in 5.87s`
- `duration_core::` exit 0: `test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 10047 filtered out; finished in 8.07s`
- `cooldown_probes::charge_duration` exit 0: `test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 10064 filtered out; finished in 3.00s`
- `aura_table_shape::` exit 0: `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 10067 filtered out; finished in 2.44s`

Additional C evidence: `tests/duration_core.rs:297–304` asserts `d:GetElapsedPercent() == zeroElapsedPercent and d:GetRemainingPercent() == 0`, nil/default and both enum modifier variants, `d:IsZero() and d:GetTotalDuration() == 0`, zero duration quantities. Lines 306–316 exercise default, reset, and explicit zero interval. Helper lines 234–241 seeds expected percent from compile-time `retail-12-0-5`, not measured output; passing A's feature-gated 18 tests confirms this binary includes that feature.

## D — ACCEPT WITH QUALIFICATIONS
Row `prose-2026-03-12-029`: all five listed aura-data booleans are no longer secret.
`tests/aura_table_shape.rs:204–207` explicitly lists `isHelpful`, `isHarmful`, `isRaid`, `isNameplateOnly`, `isFromPlayerOrPlayerPet`. Lines 209–215 supply five concrete player/party/target fixtures with expected booleans. Lines 223–227 call slot/index/instance queries and assert `#queries == 3`. Lines 229–234 assert instance identity and, for EACH field, `assert(type(value) == "boolean", unit .. " " .. field .. " type")`, `assert(not issecretvalue(value), unit .. " " .. field .. " secrecy")`, `assert(value == expected[i], unit .. " " .. field .. " value")`.
Addon-tainted caller IS tested: line 217 asserts `debug.getstacktaint() == "AuraClassificationFixture"`; lines 241–242 stamp and invoke `addon`; lines 236–237 assert query preserves addon taint. Snapshot test lines 271,273,276–277 independently asserts all fresh fields boolean/non-secret and retained taint, with stamped caller. Lines 269–270 assert independent snapshot/mutation behavior; lines 283–298 assert host values unchanged.
Non-vacuous: concrete field/fixture tables, three-result count plus expected aura identities, nonnil dereferences, expected true AND false values for every flag across player/party fixtures. No nullable-result loop silently skips all evidence. All five writer fields are ordinary `Val::Bool` at `src/lua_api/globals/auras.rs:814–829`, not generic secret declassification.
Qualification: literal public representation proven for modeled aura tables and three query surfaces, including addon taint; no native combat/restricted-input transition or previously-secret aura fixture is asserted. Native classification/defaults, arbitrary aura query APIs and generic declassification are unproven. Thus bounded credit only; not whole native secrecy parity. Spec `docs/specs/aura-classification-flags.md:7–17` states exact public fields/taint contract and explicit plain-host-input limitation.
Coverage: status `audit-pending`; capabilities `[]`; note: `Bounded partial proof only: docs/specs/aura-classification-flags.md#reconciled-batch28-bounded-partial-proof--2026-10-01. Producer bff26b9c1 publishes explicit raid/nameplate/from-player plain booleans; deterministic fixture e652d9610 has saved seven PASS in shared build 7926dfdfe, independently accepted by final followup. Fresh fmt passed; source-identical prior check reused. Historical unique controls 121/123 PASS, two unresolved failures, preexisting UNPROVEN; BROAD NOT GREEN. Saved startup 0 []. Whole native five-boolean secrecy delta, native combat/secret policies, generic declassification, all profiles and public target-input model unproved. Row remains pending; no whole-row/page credit.`
No existing capability names `docs/specs/aura-classification-flags.md` or `tests/aura_table_shape.rs`; no exact capability to join. Existing pending note already recognizes bounded partial coverage but forbids whole-row native credit.

## Verdict table
| A: GetAuraBaseDuration arg3 | ACCEPT WITH QUALIFICATIONS — numeric/seeded-alias forms only |
| B: maximum-charge durations | ACCEPT — all three APIs non-vacuously exercised |
| C: zero-span fully elapsed | ACCEPT WITH QUALIFICATIONS — explicit expired/fraction interpretation |
| D: five aura booleans public | ACCEPT WITH QUALIFICATIONS — all five, addon-tainted caller; modeled plain inputs |

## Not verified
- Native WoW runtime/probes, native secret/combat restrictions, general declassification, historical before/after migration.
- Every SpellIdentifier form, live aura metadata/classification, all query APIs or all client profiles.
- Broad suite, CI, build/formatter/lint/check, deployment or GUI. No repo changes or capability reassignment performed.
- Independent binary provenance/hash-to-source attestation: reused designated prebuilt binary under explicitly permitted empty committed source diff.
- Concurrent unrelated working-tree modifications (`tests/font_api.rs`, `tests/housing_catalog_base_lookups.rs`) appeared by final status; not inspected or modified. Reviewed target files remained the scope of proof.
