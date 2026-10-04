# Retail 12.0.7 named CVar additions

Six additions in the retained [12.0.7 API excerpt](../../data/patch-api/sources/12.0.7-api-changes.txt), lines 166–171, expose mutable CVar values through the public global and `C_CVar` APIs. Source IDs: `cvars-assistedCombatReduceHighlights-166`, `cvars-developerLogFilterDebug-167`, `cvars-developerLogFilterError-168`, `cvars-developerLogFilterFatal-169`, `cvars-developerLogFilterNormal-170`, `cvars-developerLogFilterSpam-171`. The excerpt names additions only; it does not establish native defaults, flags, validation, security or persistence. All defaults below are **INFERRED simulator choices**, not historical client evidence.

## What it must do

### Defaults and mutable values

- [ ] Before any write or runtime registration, return the chosen default through both `GetCVar` / `C_CVar.GetCVar` and `GetCVarDefault` / `C_CVar.GetCVarDefault`: `assistedCombatReduceHighlights = '1'`, `developerLogFilterDebug = '0'`, `developerLogFilterError = '1'`, `developerLogFilterFatal = '1'`, `developerLogFilterNormal = '1'`, `developerLogFilterSpam = '0'`. **INFERRED** default policy.
- [ ] For each name, global and namespace `SetCVar` return true and change the current value to the opposite `'0'` / `'1'` value, visible through both readers, without changing either default reader. Explicitly reset by writing the default read from either surface; verify the resulting value, not just setter success. Reset does not mean the no-op `ResetTestCvars` hook.
- [ ] **INFERRED** use case-insensitive lookup and mutation of the same key; uppercase writes must be visible through original- and uppercase global/namespace readers.
- [ ] **INFERRED** getters return exactly one value. Both bool getters track live values, not a fixed bool or a Lua string's truthiness.

### Coercion and isolation

- [ ] **INFERRED** both setters stringify boolean `true` / `false` to `'1'` / `'0'`, numeric `1` / `0` / `2` to `'1'` / `'0'` / `'2'`, and preserve string `'true'`, `'01'` and `''`. Both bool readers return true only for the stored string `'1'`; all other tested values return false. Defaults remain unchanged throughout and explicit reset restores the default bool.
- [ ] Writes to each of the six names leave all other named CVar values/defaults unchanged.
- [ ] In the integration harness's independent storage environments, mutate the first environment before creating the second. The second starts at chosen defaults, can be independently changed to `'2'`, and cannot change or reset the first; resetting the first cannot reset the second. This does not claim isolation between production environments sharing a persisted storage path.

- [ ] **INFERRED** an unknown control initially returns nil current/default values and false bool values on both surfaces. Setting it creates only an override: reads change but defaults stay nil, and the six named built-ins remain unchanged.

### Publication epoch

- [ ] Publish these six built-in defaults only with cumulative `retail-12-0-7` enabled (including later epochs). On a fresh strict older retail build without that feature, both string readers and default readers return one nil, both bool readers return false; an existing pre-12.0.7 CVar remains mutable as a positive control.
- [ ] Test the enabled side under strict `profile-retail,retail-12-0-7` and the disabled side under `profile-retail,retail-12-0-5`, without default features. `client-retail` enables `retail-12-1-0` and is not a disabled-gate proof.
- [ ] Scope the publication claim to built-in defaults in fresh environments: the existing general-purpose `SetCVar` and `RegisterCVar` can create arbitrary runtime names. No new prohibition on addon-created names or persisted older-profile overrides is inferred from this source.

## How it works

- [Lua API architecture](../lua-api.md)
- [Client profile and cumulative epoch contract](client-profiles.md)

## Implementation inventory

- `src/cvars.rs` — six explicit epoch-gated defaults, per-storage overrides, original-case names and bool reads; no production changes proposed for this batch.
- `src/cvars.yaml` — shared defaults; none of these six names occurs here.
- `src/lua_api/globals/set_cvar_verb.rs` — global and namespace providers routing to the same environment storage.
- `src/lua_api/state.rs` / `src/lua_api/state/sim_state.rs` — each environment owns its `CVarStorage`.
- `Cargo.toml` — cumulative retail epochs and explicit integration target; `build.rs` auto-includes top-level test modules into that target.

## Tests asserting this spec

`tests/patch_12_0_7_cvars.rs` — ten authored tests, execution blocked during integration: six per-name default/mutation/reset cases; both-surface coercion; environment isolation with second construction after first mutation; sibling isolation; unknown-CVar control. The first line gates this new module to `retail-12-0-7`. No provider replacement, defaults injection, runtime registration or presence-only credit.

`tests/set_cvar_global.rs` contains the surviving `patch_12_0_7_named_defaults_are_absent_before_publication` negative control outside the new feature-gated module. Both modules use the existing single auto-included `integration` binary; no Cargo or harness edits are required. The integration request forbids alternate feature sets, so strict enabled/disabled epoch execution remains unproved. Default-feature execution includes later cumulative epochs.

## Known gaps (current cycle)

- [ ] Obtain behavioral RED/GREEN and regression proof after the unrelated master compile blocker is resolved. On 2026-10-04, integration rebased onto `9ce93bb12`; RED temporarily withdrew the six existing default tuples and GREEN restored them unchanged. Both builds failed before tests: `src/c_api/c_spell_maw_powers.rs` uses `Val` with `retail-12-0-7` enabled but imports it only when disabled (`E0433`). The same blocker prevented all narrow CVar regression filters. No passing or behavioral-failure counts, bounded capability credit or checked requirements are claimed.
- [ ] Run strict enabled/disabled epoch cases when separately authorized; alternate feature sets are prohibited for this integration. Publication gating currently has source-inspection evidence only.
- [ ] Authenticate native defaults, CVar flags, restrictions, coercion/arity, errors, secret-value handling and persistence separately. Current source names alone cannot prove those policies.
- [ ] `source-context-172` states `(20 added, 5 removed; crawler excerpt did not include full list.)`. Only six additions are named; fourteen additions and all five removal names remain unidentified. Obtain the complete historical page/extract or authenticated build 67602 → 68182 CVar diff. Do not infer names from current simulator constants or invent nineteen rows.

## Out of scope

- Executing assisted-combat highlight reduction or developer-log filtering, log severity routing/output, native UI rendering: these rows name configuration additions, not those downstream behaviors.
- Persistence across production processes, profile migration and complete native flag/security semantics: not specified by retained excerpt and not proved by these fixtures.
- Other named simulator additions/removals, full CVar inventory reconciliation and vendor/cache edits: not authorized by B05.
