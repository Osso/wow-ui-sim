# B83 row486 independent read-only verification — 2026-10-03

Scope: `global api-PlayerScript GetSpellCritChance-486` only. Read and followed `/home/osso-test/AgentConfig/skills/verify/SKILL.md` as verifier; no delegation. Evidence below comes from local files and read-only Git commands, not newly executed behavioral tests.

Initial and final HEAD: `49c667ad40241d690afb385eb621ab283a4981dd`. Initial and final `git status --porcelain`: ` M src/c_api/aura_duration.rs`, and nothing else. All Git commands reported exit0 and empty stderr. Repo paths below are relative to `/home/osso-test/Projects/wow/wow-ui-sim`.

## 1. Retained delta and register — PASS

`data/patch-api/sources/12.0.5-api-changes.txt:485–486`:

> PlayerScript GetSpellCritChance
>   + SecretWhenUnitStatsRestricted

`data/patch-api/sources/12.0.5-register.json:3182–3191` records:

> "id": "global api-PlayerScript GetSpellCritChance-486"
> "kind": "delta"
> "subject": "PlayerScript GetSpellCritChance"
> "change": "+ SecretWhenUnitStatsRestricted"
> "source_lines": [486]

The array is physically split across lines3187–3189. Full-file search for `GetSpellCritChance` found exactly one occurrence, line485. The next subject begins at487. No other delta for this subject exists in that retained text. Output annotation only; no added signature, argument, formula or activation policy.

## 2. Cached declaration — PASS

`/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PlayerScriptDocumentation.lua:894–903` is the complete entry, with named declaration at895:

> Name = "GetSpellCritChance",
> Type = "Function",
> SecretWhenUnitStatsRestricted = true,
> Returns =
> {
>     { Name = "result", Type = "number", Nilable = false },
> },

Neither `Arguments` nor `SecretArguments` occurs in this entry. Exactly one nonnullable numeric return is declared. The supplement's895–903 range is accurate. This verifies cached text, not native execution or cache provenance.

## 3. Provider and wrapping — PASS, with wording qualification

`src/lua_api/globals/real/combat_stats.rs:109–118`:

> fn get_crit_chance(state: &mut LuaState) -> LuaResult<u32> {
>     let crit = borrow_state(state)?.player.stats.crit_pct() + 5.0;
>     push_stat_number(state, crit)?;
>     Ok(1)
> }
> fn get_spell_crit_chance(state: &mut LuaState) -> LuaResult<u32> {
>     let _school = Option::<i32>::from_stack(state, 1)?;
>     get_crit_chance(state)
> }

Registration is explicit at `combat_stats.rs:356`: `("GetSpellCritChance", get_spell_crit_chance)`. Lines413–418 register every `COMBAT_STAT_GLOBALS` entry through `LuaApiMut::register_function`. External wiring: `src/lua_api/globals/register.rs:283`, `super::real::combat_stats::register_all(lua)?;`.

`src/c_api/c_secrets.rs:67–79` reads `unit_stats_restricted` under `retail-12-0-5`, selects `rilua::table_security::wrap_host_secret_number(state, number)` when restricted, otherwise `Val::Num(number)`, then executes exactly one `state.push(value)`. Without that feature it selects plain `Val::Num(number)`. The successful provider path reports exactly one result; conversion/state errors can terminate before a result.

Qualification: “optional numeric” and “discards it” at supplement line168 are shorthand, not unrestricted argument acceptance. The actual field is `Option<i32>` and parsing is fallible before delegation. `src/lua_bridge/from_stack.rs:246–258` rejects nonintegral numbers (`"expected integer, got non-integer number at argument {index}"`); lines268–275 range-check i32 (`"expected i32, value {n} out of range at argument {index}"`). Lines319–325 return `None` for nil/absent positions and otherwise call `T::from_stack(...)`. Only the successfully parsed payload is unused. No new B82-style explicit guard exists here.

## 4. Fixture, model and assertions — PASS

`tests/character_stats/stat_restriction_fixtures.lua:31`:

> {'GetSpellCritChance', {2}, {7}},

`tests/character_stats/stat_restriction.rs:19` seeds `player.stats.crit_rating = 360;`. `src/lua_api/state_types/character_world.rs:108–110` implements:

> pub fn crit_pct(&self) -> f64 {
>     self.crit_rating as f64 / 180.0
> }

Consequently the real provider computes `360 / 180 + 5 = 7`. Expected7 is a fixture constant; produced7 is computed from seeded state, not a constant-return provider.

`stat_restriction_all_supported_outputs_preserve_values_arity_and_toggle` (`stat_restriction.rs:39–65`) runs `assertStatOutputs(false)` at41, sets restriction true at42, runs `assertStatOutputs(true)` at46, resets false at62 and runs the plain assertion again at63. The shared Lua helper (`stat_restriction_fixtures.lua:44–57`) invokes the registered `_G[name]` with fixture arguments and asserts:

> assert(actual.n == #expected, name .. ' arity')
> assert(issecretvalue(result) == restricted, name .. ' secrecy ' .. index)
> assert(canaccessvalue(result) == not restricted, name .. ' access ' .. index)
> if restricted then result = secretunwrap(result) end
> assert(math.abs(result - value) < 1e-10, name .. ' value ' .. index)

For this fixture that means arity1, value7 within1e-10, and plain→restricted→plain secrecy/access checks.

`stat_restriction_tainted_callers_receive_opaque_host_results` (`stat_restriction.rs:68–101`) sets restriction true at72 and iterates every fixture at78. Lines81–90 assert exact arity, secret/inaccessible results, failed `pcall(secretunwrap, value)`, failed arithmetic `pcall(function() return value + 1 end)`, and `debug.getstacktaint() == 'StatRestrictionProbe'`. Lines95–97 stamp that caller taint, invoke the function, then run the secure value checks. The tainted loop itself does not inspect secret payload7; the secure helper does.

The other two tests (`stat_restriction.rs:104–137`, absent-unit/constants; `140–158`, default predicate) never invoke `GetSpellCritChance` or `assertStatOutputs`. The former initializes fixtures but does not execute them. They are controls, not direct spell-crit evidence.

Test wiring: `tests/character_stats.rs:9–11` includes the restriction module under `retail-12-0-5`; fixture loading occurs at `stat_restriction.rs:5,31`. `git diff f35d0293f HEAD -- tests/character_stats/stat_restriction.rs tests/character_stats/stat_restriction_fixtures.lua` returned empty output/exit0. Git history identifies `f35d0293f` as “Test secrecy and tainted callers for 40 supported stat queries,” supporting the unchanged-prior40 assertion.

## 5. Reuse boundary and unrelated dirty change — PASS

Fresh command:

`git diff d21d4a208 HEAD -- src tests Cargo.toml Cargo.lock`

Result: exit0, stdout empty, stderr empty. Thus the named tracked code/test/Cargo inputs are unchanged between those commits at current HEAD, not merely at the historical `684670add` cited in the supplement.

`git diff -- src/c_api/aura_duration.rs` returned only a registration for `DoesAuraHaveExpirationTime` and its new callback. The callback calls `read_public_arguments`/`find_public_aura`, checks `aura.expiration_time != 0.0`, pushes `Val::Bool(expires)`, and returns `Ok(1)`. Hunks are at old44 and52/new44 and53. No change touches combat-stat providers, `push_stat_number`, the player stats model or restriction tests. This is not clean whole-tree proof, and no claim is made about unrelated aura behavior.

## 6. Committed reused-proof record — PASS as recorded evidence only

`data/patch-api/sources/12.0.5-page-coverage.json:1409–1423` records capability `spell-bonus-stat-security`. Line1420 gives:

> "compiled_revision": "d21d4a208da4b911f003181e5ca33163f42b92be"

Line1423 `gate_scope` begins:

> Main accepts716 bounded behavior:12 ALL refreshed PASS=7new+4stat+1proxy, compile0/108.741957s zeroDiag

Its `tests` array at1416–1418 names only `tests/spell_bonus_stat_security.rs`; it is not an individually enumerated list of the four stat-control results.

`docs/specs/spell-bonus-stat-security.md:66–68`, “Independent bounded acceptance — 2026-10-03,” explicitly records:

> 12 distinct refreshed PASS =7 new+4 existing restriction controls+1 proxy control.

The same line names producer `d21d4a208da4b911f003181e5ca33163f42b92be`, compilation and run timings, and integration SHA256 `90f26231deda006a6dc80603fbb4ea2eb18eb932ba0cf831bd8bba61b35661f5`. Together with the actual four-test module, this supports the supplement's aggregate reuse claim. Neither committed record reproduces four individually named raw test-result lines.

Line72 retains the check qualification: `build-finished.success=true`/zero diagnostics were recorded, but synchronous execution/direct-exit loss means no exit0/asynchronous-check clearance. The supplement line179 carries this qualification forward.

Python `Path('/tmp').glob('patch-12.0.5-*')` returned `[]`, checked twice. Referenced raw artifacts are absent at those paths on this host. I validated the committed record, not raw run output. `git show HEAD:data/patch-api/sources/12.0.5-page-coverage.json` matched the working file exactly. Historical PASS, timing/hash accuracy and historical execution provenance were not independently re-established.

## 7. Limits and unsupported extensions — PASS for bounded credit; qualifications required

Supplement `docs/specs/unit-stat-output-restriction.md:179,183–187` explicitly excludes raw-artifact/current-process proof, spell-specific/per-school/native formulas, restricted no-argument coverage, wrong-type/nil/secret-extra-argument parity, automatic activation, older profiles, native/GUI parity and full-suite readiness. Row486 remains pending. These limits are honest for the single seeded output boundary.

Two wording/proof-precision qualifications remain:

1. Line168 should not be read as silently ignoring arbitrary numeric extras: the conversion is optional i32, fallible and executed before output. Fractional/out-of-range extras are specifically excluded by the actual converter; error paths are not fixture evidence.
2. Line184's no-argument “plain value” control is weaker than exact-value preservation. `tests/character_stats.rs:173–174` includes both calls, but lines200–203 only reject values when `type(value) ~= "number" or value < 0`. It does not assert exact7, equality between calls, exact arity, `issecretvalue`, or `canaccessvalue`. It is a safe-number smoke assertion, not restricted no-argument proof. Nothing in this report grants that additional credit.

No substantive overclaim was found about the seeded `(2)` output-annotation case. The record is aggregate prior proof plus fresh source inspection, not newly observed runtime success. Nondefault rating values, live crit mutation, environment isolation and wrapper/GC lifetime are not established specifically for this row by these two tests; the one-seeded-fixture boundary must remain literal. The supplement does not claim those extensions.

Artifact checks: all cited files exist and contain substantive declarations/code/assertions; provider and fixture are wired as shown above. B83 section contains zero occurrences of `TODO`, `FIXME`, `HACK`, `XXX` or `except: pass`; no new implementation was audited or executed.

## 8. Current accounting — PASS

`data/patch-api/sources/12.0.5-page-coverage.json:3170–3173`:

> "source_id": "global api-PlayerScript GetSpellCritChance-486"
> "capabilities": []
> "status": "audit-pending"
> "note": "Source retained; behavioral applicability audit not completed."

Parsed current `source_rows` counts:

| Status | Count |
|---|---:|
| `audit-pending` | 165 |
| `bounded-coverage` | 179 |
| `partial-development-green` | 11 |
| `metadata-only` | 7 |
| **Total source rows** | **362** |

Capabilities: **86**. No accounting changes made.

## Overall verdict — ACCEPT WITH QUALIFICATIONS

Bounded output-annotation credit for row486 only is supported by the unchanged registered producer, computed seeded fixture, actual assertion coverage and committed aggregate prior-PASS record.

Qualifications: preserve the optional-i32 conversion/error boundary; treat the no-argument control as safe-number smoke evidence only; treat historical PASS as committed-record evidence, not independently inspected raw output or a fresh run. B82's check-protocol qualification remains unresolved.

Not verified: raw historical run artifacts, execution of any test/build/check/formatter/simulator, native behavior/cache provenance, restricted no-argument behavior, extra-argument compatibility, other fixtures/profiles, GUI/deployment, or broad-suite readiness. No agents/models/agent CLIs, Bash, repo writes, staging, commits or operational changes were used. Only this authorized report was written.
