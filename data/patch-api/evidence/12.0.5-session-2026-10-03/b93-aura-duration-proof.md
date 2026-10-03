# B93 aura-duration independent proof

Pinned producer: `012cf889a`; inputs: `b9b9eeec8`. Read-only inspection; verification underway.

Initial evidence: scoped producer-to-HEAD diff empty. Input spec explicitly labels permanent/invalid-instance policies inferred; RED/GREEN logs found.

Independent prebuilt tests (scoped diff empty; binary mtime 2026-10-03T20:47:52.097543+00:00):
- `aura_duration_object::`: `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 10082 filtered out; finished in 1.18s`
- `aura_expiration_time::`: `test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 10079 filtered out; finished in 2.09s`
- `aura_refresh_duration::`: `test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 10070 filtered out; finished in 6.48s`

Inspection: both unwrap_secret calls precede validation; no competing GetAuraDuration registration in pinned src. Cached Lua mentions only API documentation and private AuraButton methods. Probe nil/missing tests install their own namespace, not simulator behavior.

## Verdict: ACCEPT WITH QUALIFICATIONS

Accept bounded simulator implementation and argument-policy delta at `012cf889a`, not native WoW parity. Merge risk: low for inspected existing consumers; mixed-zero aura records remain semantically ambiguous, access restrictions remain unmodeled, and numeric edge cases lack producer tests.

### 1. Source/declaration/register — PASS

- Pinned source `data/patch-api/sources/12.0.5-api-changes.txt:379–380`: `C_UnitAuras.GetAuraDuration`, then `# SecretArguments AllowedWhenTainted -> AllowedWhenUntainted`.
- Pinned register `12.0.5-register.json` record `global api-C_UnitAuras-GetAuraDuration-380`: `"kind": "delta"`, `"source_lines": [380]`, same policy change, `"status": "consolidated-delta"`. This is the source-accounting register, not a native behavior test.
- Cached retail `Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:265–282`: `RequiresUnitAuraAccess = true`, `RequiresValidUnitAuraInstance = true`, `SecretArguments = "AllowedWhenUntainted"`; both arguments and the single `LuaDurationObject` return have `Nilable = false`.

### 2. Producer/security/numeric semantics — PASS, qualified

- `git show 012cf889a` adds only the helper import, registration and producer in `src/c_api/aura_duration.rs` (13 insertions); its other change updates the spec proof note. Registration is `table_set_rust_fn_static(state, namespace, "GetAuraDuration", get_aura_duration)?`; `globals/register.rs:93` calls this module, gated by `retail-12-0-5`.
- `aura_duration.rs:64–74`: `read_expiration_arguments(state)?` precedes aura lookup, then `push_timed_duration_object(state, aura.expiration_time - aura.duration, aura.duration)`.
- Reader `aura_duration.rs:93–115`: `let unit = unwrap_secret(state, stack_val(state, 1))?;` and `let instance_id = unwrap_secret(state, stack_val(state, 2))?;` both precede string/finite-number validation. A denied first argument aborts immediately; otherwise second authentication precedes either validation. No lookup occurs earlier.
- Nil unit is read as `None`, but the new producer's `and_then(...).ok_or_else(...)` raises `"C_UnitAuras.GetAuraDuration: no such aura instance"`; it cannot silently return nil. Unknown/cross-unit instance follows the same error path.
- `lua_duration_object.rs:132–155` creates a new object and invokes its validated `SetTimeFromStart`, passing rate `1.0`; returns `Ok(1)`.
- `lua_duration_object/core.rs:169–221` requires finite arguments, nonnegative finite base, positive finite rate, finite start/span/end. Negative start is allowed. Invalid/overflowing endpoints error rather than exposing infinity.
- Zero span is accepted, not normalized: start/end retain the supplied start. For `(duration, expiration) = (0,0)`, both endpoints are zero and `IsZero()` is true. At Retail 12.0.5, zero spans also report expired, not active, and elapsed-percent 1 (`core.rs:302–338`); these extra methods are not asserted by B93.
- For finite positive `d` and `expiration < d`, start is negative; this is a valid interval, not necessarily nonsensical. For `(d>0, expiration=0)`, object spans `[-d,0]`: positive total but already expired on the default nonnegative clock. For `(d=0, expiration=e>0)`, object is zero-span at `e`, and zero-span expiry semantics apply even before `e`. Neither mixed-zero case is rejected or classified as permanent. This differs from refresh eligibility, which returns `None` if either field is zero (`aura_duration.rs:33–34`). Native meaning is unknown; spec should explicitly bound these cases.
- Clock alignment PASS by source: object default `core.rs:148–155`, `GetTime` in `globals/register.rs:347–353`, and refresh calculation in `aura_duration.rs` all use `sim.start_time.elapsed().as_secs_f64()`. Admin aura creation uses `st.start_time.elapsed().as_secs_f64() + duration` (`admin.rs:646–649`). Target fixtures instead use fixed numeric expiry offsets; this is an acknowledged fixture limitation. B93 does not behaviorally test moving remaining time.

### 3. Existing queries/shadowing — PASS

- Producer diff does not edit `DoesAuraHaveExpirationTime`, `GetAuraBaseDuration`, `GetRefreshExtendedDuration`, either argument reader, refresh metadata or lookup. Existing nil/false policies remain unchanged.
- Pinned `git grep GetAuraDuration -- src` finds only the new registration and its error text in `aura_duration.rs`; no named stale Lua default or competing registration. Generic lazy namespace behavior is not a second named implementation; successful object tests demonstrate the new registered function is reached.
- Independent existing-query tests: 9 expiration-time and 18 refresh-duration tests pass, including nil-unit behavior, secrets, blocked records, metadata and clock cases. This is bounded regression proof, not full-suite proof.

### 4. Six spec bullets mapped to tests — PASS, bounded

| Spec bullet | Assertions / test | Unasserted clauses or limits |
|---|---|---|
| One new object; stored start/span; rate 1; player helpful/harmful and party | `timed_auras_return_one_duration_object_spanning_stored_times`: IDs 301/303/402; helper checks `select('#', ...) == 1`, table, total/start/end/rate/zero | General fresh identity is tested on two live snapshots, not every polarity; no moving clock or numeric edge fixture |
| Permanent `(0,0)` zero span | `permanent_aura_yields_a_zero_span_object`: helper checks total/start/end 0, rate 1, `IsZero()` | Other zero-span methods and mixed-zero fields |
| Live independent snapshot; mutation isolation | `results_are_independent_snapshots_of_live_state`: mutate Rust fields to `(20,90)`, expect `(70,20)`; `not rawequal`; old start/total unchanged; mutate result, re-query; Rust fields unchanged | Old clock/all methods and unrelated state not exhaustively checked |
| Invalid/missing inputs error, never nil | `unknown_or_malformed_arguments_error_and_recover`: unknown/cross-unit IDs, nil/missing unit/ID, string/NaN ID, numeric/table unit, then valid recovery | Infinity ID not explicit (finite guard covers source); malformed aura fields/blocked/target not tested here |
| Untainted authentic secrets allowed | `untainted_query_accepts_each_authentic_secret_argument_and_combination`: host-secret string/number, either/both, permanent ID; unknown secret unit errors; input secrecy/untainted stack retained | No secret malformed-ID or GC-lifetime exercise in B93 |
| Tainted either-secret denial before validation/lookup; unchanged taint/secrecy; public recovery | `tainted_query_denies_each_secret_before_validation_or_lookup`: secret unit/ID combinations, unknown units, numeric/boolean public unit plus secret ID; taint checks, failed secret unwrap, public success, untainted recovery | Does not directly observe lookup side effects; no first-secret plus malformed second-public-ID case |

- None of the six tests is vacuous against the prior nil-returning placeholder: object checks fail on nil, rejection checks fail because placeholder succeeds, and tainted ordering cannot establish distinct denial/invalid/malformed errors. Supplied RED log confirms all six fail, including `"duration object"` failures. `type(query) == 'function'` alone would not distinguish a lazy placeholder, but the remaining assertions do.
- `err == denial` is sound for these fixtures: calls use `pcall` on the same function, denial is compared with distinct public lookup/type errors, and candidate calls require `not ok`. This checks observed error precedence without requiring native wording. It does not prove lookup absence universally; source ordering supplies that evidence. Baseline `pcall` booleans are not explicitly asserted, but candidates and distinct error comparisons plus source inspection avoid a vacuous pass here.

### 5. Consumers/regressions — PASS for searched scope

- Pinned Git search and working-tree Python scan of `tests`, `Interface/AddOns`, `docs/addons` found B93 tests, the private button test, and ApiContractProbe producer/tests. No simulator-facing consumer found requiring absent/nil API behavior.
- Probe test `docs/addons/ApiContractProbe/tests/aura_time.lua:7–12` installs its own namespace (`C_UnitAuras = ns`). Its missing-API and nil-return assertions intentionally replace the query (`ns[missing] = nil`, `ns.GetAuraDuration = function() return nil end` at 82/90). They test observation preservation, not simulator defaults.
- Cached retail Lua/XML scan finds only the API declaration and private AuraButton methods/calls. `AuraButtonPrivateMixin:GetAuraDuration()` (`Blizzard_AuraButton.lua:121–123`) returns `self.auraDuration`; it does not call `C_UnitAuras.GetAuraDuration`.
- `tests/duration_text_binding_tick.rs:232–236` calls `private:GetAuraDuration():SetClock(Clock)` on a button after `SetAuraInstance`; unrelated to the namespace producer. No cached feature check for `C_UnitAuras.GetAuraDuration` found.

### 6. Spec honesty — PASS with missing edge limit

- Inputs spec labels permanent zero-span and invalid-instance errors `Inferred policy` / `Inferred from ...`; cached annotations are explicitly `Contract context, not native execution evidence`. Producer comment also says `INFERRED`.
- Spec explicitly excludes `RequiresUnitAuraAccess`, restricted/secret output, blocked-record and target assertions, rate modifiers/haste, native error wording and invalid-instance native behavior. Blocked omission matches `collect_visible_unit_auras(...).filter(|a| !is_blocked_aura(...))` (`auras.rs:319–339`). Output is public even for accepted secret inputs; access annotations are not fully implemented.
- Missing limit: mixed-zero fields and malformed numeric aura data are not named. Tests only cover coherent finite nonnegative records. Registration must not be advertised as full native API parity.
- Input commit's stale `Inputs only` proof note is updated in producer commit; it is not an implementation gap at `012cf889a`. Six unchecked requirement boxes are not evidence of failure.

## Proof ledger and limits

- Authorized binary command per filter: `timeout 90 /home/osso-test/Projects/wow/wow-ui-sim/target/debug/deps/integration-8ea324359263a4d2 <filter> --test-threads=1`, explicit repo cwd. Exact independently observed result lines appear above. Total 33/33.
- Supplied historical RED: `test result: FAILED. 0 passed; 6 failed; 0 ignored; 0 measured; 10082 filtered out; finished in 1.72s`.
- Supplied historical GREEN: `test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 10055 filtered out; finished in 5.52s`.
- Supplied startup stdout: `[]`. Reviewed artifact only; did not independently rerun startup.
- Before independent binary execution, required producer-to-HEAD scoped diff was empty. Later working-tree-to-producer scoped diff was also empty. Observed HEAD: `ffd50c5d10d0027d43823a4b43afe8b3c1d381fd`. Findings remain pinned to producer, not unrelated HEAD changes.
- Binary mtime recorded above; no rebuild or cryptographic build-to-commit provenance established. Test execution corroborates the supplied source proof but mtime alone does not certify provenance.
- Not verified: native WoW behavior, full addon runtime interactions, full suite/CI, alternate profiles, builds/checks/formatting, GC lifetime of this new producer, mixed-zero/numeric-edge behavior by execution, blocked/target B93 behavior, access/restricted-output parity, dynamic consumers using constructed names or external addons outside searched roots.
- No repository/vendor edits, commits, builds, agents, model CLIs or Bash. Only authorized report created/appended. A missing `rg` executable was replaced with read-only Python scanning; no install performed.
