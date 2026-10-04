# Classic secret policy review — ce418cbe4

## Verdict: ACCEPT WITH QUALIFICATIONS

**Merge risk: low for this bounded marker/closure correction. No confirmed merge-blocking defect. Do not account it as disabling the entire Classic secret system or completing rows 213–215.**

Reviewed HEAD `ce418cbe46049c28ed7798c67a929a0dbb6e4354`, its diff against parent `4d142e325`, current producer/registration paths, and supplied RED/GREEN logs. No repo edits, tests, cargo commands, git mutations, agents, or model CLIs were run.

### Qualifications with file:line

- **Native secrecy remains supported, not globally disabled:** `src/lua_api/globals/security/secret_values.rs:93,116` still recognize authentic VM secrets before consulting marker/closure classification. Classic-reachable duration and SecondsFormatter code can propagate authentic secret inputs (`src/lua_api/globals/lua_duration_object/core.rs:98–100`; `src/c_api/seconds_formatter.rs:165,174`). No ordinary Classic input path in the searched producers was found to originate these wrappers. Host-injected authentic wrappers remain a limitation, not a new regression introduced by this commit.
- **Expectation/policy coupling:** `tests/security_api.rs:774,815,841,869,901,917,932` derive expected results from the same predicate as production. With today's exhaustive match, Retail/PTR/Forever assertions retain their original truth values. A future erroneous predicate change could switch both implementation and expectations together. Existing tests do not independently pin the three true-profile assignments. This is a coverage qualification, not evidence of a current retail regression.
- **Execution evidence is Mists-only and pre-integration:** supplied logs belong to the original isolated branch, not a fresh run of the cherry-pick atop B99. Wrath/Era/Anniversary and Retail/PTR/Forever were not executed here. Source review establishes preservation of changed paths, not whole-profile runtime parity.

## 1. Predicate and complete caller trace

`src/client_profile.rs:95–99` returns **false for Wrath, Mists, Era, Anniversary**, and **true for Retail, Ptr, WowForever**. Cargo declares the four official Classic bundles without retail epochs (`Cargo.toml:151–158`); retail epoch features on Classic/Forever are rejected by `src/client_profile.rs`. Forever remains a separate profile, intentionally outside disablement.

Only two production sites consult `uses_secret_values()`:

| Site | Wiring and effect |
|---|---|
| `secret_values.rs:73` | `mark_secret_value` returns before registry creation/write on Classic. `unit_misc.rs:77` receives identity name/realm/GUID outputs through `identity_output`; cached raid roster names call it at `group_queries.rs:335`. `security/value_access.rs:26` exposes the same helper through `__sim_mark_secret_value`, registered at `security/mod.rs:60–64` and used by attribute/event handling at `frame/methods/text_attribute_event/mod.rs:425,436,442`. These marker paths all reach the guard. |
| `secret_values.rs:170` | `function_is_secret` no longer treats the ForceTaint_Strong loadstring closure marker as secrecy on Classic. Both shallow checks (`:125`) and recursive table checks (`:99`, `:196`) reach it. `issecretvalue`, `canaccessvalue`, `canaccessallvalues`, and `canaccesstable` use these checks through `security/value_access.rs`. Loadstring taint stamping and enablement remain untouched (`env_init/mod.rs:150–159`). |

All remaining predicate references are tests. For Retail/PTR/Forever the new guard is false and the new conjunction has a true left operand; original marker writes and closure classification are preserved exactly. Retail/PTR 12.0.5 identity host wrappers at `unit_misc.rs:64–69` remain outside the marker path and unchanged. Native checks and registration gates are unchanged. This proves no behavioral delta in the changed security paths for those profiles by source inspection; it does not replace execution.

## 2. Secret-producing paths outside the predicate

“Compiled” below refers to normal official Classic feature bundles, not unsupported combinations of retail epoch features with Classic markers.

| Producer/path | Classic compilation/reachability and policy |
|---|---|
| `mark_secret_value`, identities, roster, attribute/event helpers | Compiled/reachable; centrally guarded, so no new marker is produced. |
| `identity_output` native string wrapper (`unit_misc.rs:69`) | Wrapper branch compiled out: requires retail-12-0-5 plus Retail/PTR. Classic takes ordinary string plus now-disabled marker branch. |
| `push_stat_number` (`c_secrets.rs:67–79`) | Compiled/reachable. Native number wrapper is compiled out without retail-12-0-5; explicit restriction flag cannot change Classic outputs. |
| Charge fields (`charge_state.rs:75`), spell cooldown (`c_spell.rs:465`), action cooldown (`c_action_bar.rs:49`), loss-of-control serializer (`loss_of_control.rs:22`) | Shared code compiles; public snapshot paths are reachable. Their restriction input comes from `cooldowns_are_restricted` (`charge_state.rs:25–30`), statically false on official Classic, independently of the new predicate. Thus wrapper branches cannot execute through these callers. |
| Spell count/max-aura/display wrappers (`c_spell_counts.rs:28,50,70`); action count/display wrappers (`c_action_bar_counts.rs:21,40`); action loss-of-control namespace | Modules compiled out by retail-12-0-5 (`c_api/mod.rs:24–27,120–121`). `unit_auras_restricted` is stored on all profiles (`state/sim_state.rs:305`) but its wrapper-producing consumer at `c_spell_counts.rs:47–50` is absent on Classic. |
| Spellbook cast-count wrapper (`c_spell_book.rs:424`) | Function compiled out by retail-12-0-5 (`:406`). |
| Scenario numeric/string wrappers (`c_scenario_info.rs:76,83`) | Producer functions/registration compiled out by retail-12-0-5, though the backing record/module exists. |
| UnitSpellTargetName string wrapper (`globals/real/unit_spell_target_name.rs:37`) | Entire module absent on Classic: retail-12-0-5 plus Retail/PTR gate (`globals/real/mod.rs:80–84`). |
| Aura dispel-color `wrap_secret` (`c_unit_aura_dispel_color.rs:43`) | Module compiled out by retail-12-0-5 (`c_api/mod.rs:155–156`). |
| Duration slots (`lua_duration_object/core.rs:100`) | Compiled/reachable, not profile guarded. Propagates secrecy only from existing authentic secret arguments/slots (`:187–189`), not ordinary tainted closures or strings. Host timed-duration producers pass ordinary numbers (`lua_duration_object.rs:140–154`). |
| Duration formatting host number/string wrappers (`lua_duration_object/formatting.rs:70,121`); cooldown formatter wrapper (`widgets/cooldown/countdown_formatter.rs:104`) | Modules compiled out by retail-12-0-5 (`lua_duration_object.rs:19–20`; `widgets/cooldown.rs:3–6`). |
| SecondsFormatter string/native wrap (`seconds_formatter.rs:165,174`) | Compiled and registered on Classic (`env_init/mod.rs:63`). Private callbacks are captured into formatter implementation (`seconds_formatter.rs:196–211`), not exposed as a general secret producer. Lua invokes wrap only when its inputs already carry authentic secrecy (`seconds_formatter/configuration.lua:88`; `format.lua:77,89`). Ordinary Classic inputs remain ordinary; injected authentic wrappers would propagate. |
| AbbreviatedNumberFormatter output/breakpoint wraps (`abbreviated_number_formatter.rs:147`; `config.rs:144`) | Module compiled out by retail-12-0-5 (`c_api/mod.rs:6–7`). |
| Duration text binding wrap (`duration_text_binding.rs:343`) | Function compiled, but registration returns before installing callbacks for Classic (`:260–267`); not reachable through Classic binding APIs. |
| Frame context-access boolean wrapper (`frame/methods/misc/secret.rs:164`) | Function and registration are Forever-only (`:34,143`); absent on official Classic. |
| Native `secretwrap` (`security/secret_values.rs:41`) | Function/registration compiled out by retail-12-1-0 (`:30,37`; `env_init/mod.rs:55–56`). Classic compatibility Lua `secretwrap` merely returns its one argument unchanged (`workarounds/temporary/debug_environment_defaults.rs:39–43`). |
| VM table-security registration | Simulator registration is Forever-only (`env_init/mod.rs:76–81`). Inspection of pinned rilua `6044544` found no call automatically registering it elsewhere. Native wrapper recognition/unwrap machinery still exists in the VM; this commit does not disable it. |

**Conclusion:** searched ordinary Classic producers are either newly marker-guarded, previously statically non-restricted, or absent. Native propagation remains compiled in shared consumers. No demonstrated unguarded ordinary Classic producer warrants rejecting this bounded change; blanket “entirely disabled” remains unproven.

## 3. Ungated stat policy state

`state/sim_state.rs:68` and `state.rs:74` now store/default `unit_stats_restricted` on every profile. This adds a public boolean/default storage slot where previously absent; it does not activate a consumer. Both reads remain gated by retail-12-0-5 (`c_secrets.rs:68–69,82–86`), and Classic plus Forever cannot enable that epoch. Retail/PTR already had the identical field and false default. Older retail epochs gain unused storage. No Lua-output behavior change follows from this ungating on any profile.

## 4. Seven existing expectation changes

These are **runtime branches/expected booleans**, not cfg removal of Retail tests:

| Test | Original Retail/PTR/Forever assertion retained today |
|---|---|
| SecureMap secret keys/values | Both original error substring assertions still run when predicate is true; final stored value assertion remains unconditional. |
| Tainted loadstring `issecretvalue` | Still expects true. |
| Tainted loadstring `canaccessvalue` | Still expects false. |
| Mixed `canaccessallvalues` | Still expects false. |
| Party roster name | Still expects secret=true, accessible=false. |
| Party full name/realm | Still expects both secret=true, bulk access=false. |
| Table containing party identity | Still expects inaccessible. |

The explicit Retail/PTR secret-GUID fixture survives at `tests/security_api.rs:884–893`. A current Retail regression with the unchanged true predicate cannot hide behind the Classic branch. A future predicate regression could; see qualification above.

## 5. New tests

`tests/classic_secret_policy.rs:2–7` gates the **whole module**, including imports, to four official Classic profiles. Retail/PTR/Forever compile it out. Test discovery is wired through `build.rs:54–85,463–470` and `tests/integration.rs:1`.

All seven tests exercise behavior: concrete identity payloads/arity/access and caller taint; cached roster name; five armor values under explicit restriction; charge fields under explicit cooldown restriction; manual-clock duration lifecycle and numeric/color curves; loadstring accessibility while closure/slot taint survives; protected combat width denial, exact blocked event, and secure recovery. None asserts implementation shape. Stat/cooldown/curve/protected-action tests were already GREEN before secrecy fixes; they are controls, not evidence those systems were newly implemented.

Read supplied complete logs: corrected RED **4 pass / 3 fail**, failing exactly identity marker, roster marker, closure secrecy; GREEN **7/7 + 46/46 + 15/15 = 68/68**. Three retail-gated identity filters executed **zero** tests, not parity proof. Author reports Mists cargo check exit 0; its saved log has a successful Finished line and no warning/error matches. No checks were rerun. These artifacts are original branch evidence, not an independent post-cherry-pick runtime gate.

## 6. B99 merge consistency

The cherry-pick changes only the matching cfg lines in the two state files. Exactly one stat field/default exists (`sim_state.rs:68`, `state.rs:74`). All six B99 additions remain present once in both struct/default with original gates and values:

| Field | Struct line | Default line/value |
|---|---:|---|
| special_bar_spells | 120 | 134 / empty HashSet |
| tiered_entrance_pde_id | 203 | 191 / 0 |
| npc_follower_guids | 261 | 234 / empty HashSet |
| nearest_party_member_token | 334 | 294 / None |
| aura_entry_ids | 337 | 296 / default |
| housing_bundles | 362 | 318 / default |

No duplicated or misplaced field/default, lost B99 initializer, or mismatched cfg was found.

## 7. Recommended audit accounting

| Row | Status | Proven boundary / remaining clauses |
|---|---|---|
| prose-2026-04-17-213 | **partial** | **Bounded** marker/closure correction: Mists execution plus source policy coverage for Wrath/Era/Anniversary; stat/cooldown controls show ordinary selected outputs. Not whole secret-system disablement, every Mainline-secret API, all four executed profiles, or blanket Midnight security parity. |
| prose-2026-04-17-214 | **partial** | **Bounded** Mists taint, one protected-action denial/recovery, duration lifecycle, numeric/color curves. Chat/guild restrictions, combat-log compatibility, every restricted action, and all associated object APIs remain unproven by this change. |
| prose-2026-04-17-215 | **pending** | The narrative says upstream docs still mention secrets and an upstream documentation/overall-validation pass is underway. This commit neither fixes those docs nor proves that pass/feedback workflow. The bounded runtime correction is credited to row213, not duplicate whole-row closure here. |

## Artifact verification

[EXIST] PASS — six changed files present; line counts respectively 445, 198, 918, 527, 224, 986.

[SUBSTANTIVE] PASS — exhaustive profile match, real marker/closure guards, consistent state storage, seven behavioral tests, seven expectation adaptations.

[WIRED] PASS — producer/consumer and generated test-harness references traced above.

[ANTI-PATTERN] PASS — added lines contain zero TODO/FIXME/HACK/XXX/`except: pass` markers; no new empty catch or commented-out implementation found.

**OVERALL: PASS for the bounded artifact review. ACCEPT WITH QUALIFICATIONS; no whole-row or all-profile execution acceptance.**
