# Aura refresh duration

Retail 12.0.5 row394, `C_UnitAuras.GetRefreshExtendedDuration`, and its consumer prerequisite `C_UnitAuras.GetAuraBaseDuration` query explicit per-environment recast metadata independently of current aura duration. Input types live in `src/c_api/aura_duration.rs`; architecture references are under [How it works](#how-it-works). The producer registers both immutable Lua getters at `retail-12-0-5`; parent GREEN and independent bounded acceptance are recorded below.

## What it must do

### Input and nullable output

- [x] Both getters accept `(unit, auraInstanceID, optionalSpellIdentifier)` and return exactly one nullable number. Omitted/nil identifier uses the matched aura's `spell_id` directly. Explicit numeric/string identifiers use the existing C_Spell resolver, including seeded case-normalized aliases and numeric alias precedence. Resolver reuse is **INFERRED**; labels in tests are fixtures, not native spell names/links.
- [x] Inputs are an environment-local `HashMap<u32, SpellAuraDuration>` with explicit `base_duration_seconds` and `max_carryover_seconds`. Default map is empty. Never seed invented production durations or infer base from `AuraInfo.duration`, remaining time, spell icons/names, or another environment.
- [x] **INFERRED unknown policy:** unknown alias, missing metadata, unit or instance yields nil. Explicit unknown overrides never fall back to the aura's own spell. Metadata alone cannot create an aura or cross unit/instance identity.
- [x] Resolve only existing public, unblocked helpful/harmful collector records, including modeled party stores. Blocking is scoped to exact unit and instance; a blocked instance remains inaccessible with an explicit known override. No new generic target store or native visibility enforcement.

### Duration policy — explicitly INFERRED, not native-verified

- [x] Base getter returns configured base duration, not current duration. Refresh returns `base + min(max(expiration - now, 0), max_carryover_seconds)`, with `now` from the environment's existing `start_time.elapsed()` clock. The cap is explicit metadata, not an invented fixed percentage.
- [x] Saturated and expired timed cases return exact cap/base results. Unsaturated tests bound elapsed time around the call with tolerance; do not introduce a time override framework.
- [x] Permanent active aura (`duration == 0` or `expiration_time == 0`) yields nil for refresh, even with known recast metadata. Base getter still returns known metadata. This eligibility rule is an **informed guess**, not documented native behavior.
- [x] Nonfinite or negative metadata in either field yields nil from both getters. Zero metadata remains valid for a timed active aura. Nonfinite refresh sums yield nil, never public infinity. Both metadata fields are validated before either getter's public output.

### Validation and query immutability — explicitly INFERRED

- [x] Unit accepts a public string or nil (nil means unknown); required instance accepts a public finite number. Missing instance, wrong types and nonfinite numeric arguments error with nonempty diagnostics, including absent-unit queries. Optional spell accepts nil or a public finite number/string; no new spell parser or additional numeric range policy.
- [x] Conservative rejection of real secret unit/instance/spell arguments precedes absent-unit results, in secure and addon-tainted callers. Full GC preserves secret identity; rejection neither declassifies inputs nor changes caller taint. Later public queries still work. This is **not** `SecretArguments = "AllowedWhenTainted"` or native secret-output parity.
- [x] Both getters leave aura identity/current duration/expiration, metadata, aliases and event queue unchanged. Metadata and alias mutations remain environment-local.

### Evidence boundary

Observed cached source on **2026-10-01**, under `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/`:

- `Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:392–410` declares `(auraInstanceUnit: UnitToken, auraInstanceID: number, spellID: SpellIdentifier?) -> newDuration: number?`. It describes client-predicted new aura duration if cast again now and optional spell ID otherwise derived from the aura. Annotations include `RequiresUnitAuraAccess`, `RequiresValidUnitAuraInstance`, `SecretWhenUnitAuraRestricted`, and `SecretArguments = "AllowedWhenTainted"`; these do not prove simulator nil/error/security policies.
- `Blizzard_AuraContainer/Blizzard_CustomAuraButton.lua:612–627` calls both getters with optional identifier omitted; `Blizzard_CooldownViewer/CooldownViewer.lua:531–552` supplies an explicit spell identifier to both. Consumers use the positive difference between extended and base durations as carryover. They establish total-duration interpretation, not the formula or cap.
- Retained [12.0.5 source register](../../data/patch-api/sources/12.0.5-register.json), row `global api-C_UnitAuras-GetRefreshExtendedDuration-394`, documents only arg3's number-to-SpellIdentifier migration. Source accounting remains unchanged.

**Correction to prior integration notes:** `src/lua_api/globals/auras.rs::find_aura_by_instance_id` does **not** filter blocked records; `collect_filtered_unit_auras` does. Duration producers must use the latter's public unblocked scope rather than treating the former as access-filtered. The separate [spell-ID query contract](aura-spell-identifier.md) intentionally preserves unblocked lookup compatibility; this duration slice must not alter it.

No native probes establish formula, caps, permanence, unknown/error handling or secret parity. All chosen policies above are user-permitted informed guesses, not native facts.

## How it works

- [Lua API and environment architecture](../lua-api.md)
- [Existing unit-aura enumeration](unit-aura-instance-enumeration.md)
- [Existing explicit spell identifier query contract](aura-spell-identifier.md)

## Implementation inventory

- `src/c_api/aura_duration.rs` — `SpellAuraDuration`, immutable getter producers, shared argument/secret validation, public blocked-filtered helpful/harmful instance lookup, metadata validation and inferred capped formula.
- `src/c_api/mod.rs` — exports the module at `retail-12-0-5`.
- `src/lua_api/globals/register.rs` — authoritative registration after existing stubs/aura surface. Inspected static namespace stubs and `runtime_surface_bootstrap.lua`: neither contains a literal duration provider; the latter's generic namespace `__index` synthesizes one-nil functions. No alternate literal provider is retained.
- `src/lua_api/state/sim_state.rs` — environment-local explicit metadata map at the same epoch.
- `src/lua_api/state.rs` — initializes map empty.
- `tests/aura_refresh_duration.rs` — 18 focused contract cases auto-discovered by `build.rs` into the existing `integration` target; no additional Cargo target or harness edit.

## Tests asserting this spec

Grouped filter: `aura_refresh_duration::` with `retail-12-0-5` enabled. Parent saved compilation and GREEN execution below. Requirement checkboxes record bounded simulator acceptance only; tests are unchanged.

| Test | Contract |
|---|---|
| `empty_metadata_never_infers_base_from_current_aura_duration` | Empty defaults, no current-duration fallback |
| `omitted_and_nil_identifier_use_matched_aura_and_return_one_number` | Caller omission/nil forms, single number |
| `explicit_numeric_and_seeded_alias_override_duration_without_matching_aura_spell` | Explicit override forms |
| `numeric_alias_applies_only_to_explicit_identifier_not_omitted_aura_spell` | Alias-first explicit resolver, direct omitted spell |
| `unknown_alias_or_metadata_returns_one_nil_without_falling_back_to_aura_spell` | Unknowns, single nil |
| `unsaturated_refresh_uses_environment_elapsed_clock_with_moving_time_bounds` | Elapsed-clock unsaturated carryover |
| `saturated_refresh_uses_explicit_cap_not_thirty_percent_or_current_duration` | Explicit cap, current duration independence |
| `expired_timed_aura_returns_exact_base_without_mutating_expiration` | Expired timed case |
| `permanent_duration_or_expiration_returns_nil_refresh_but_known_base` | Inferred permanent eligibility |
| `party_polarities_and_blocking_are_unit_and_instance_isolated` | Helpful/harmful, exact blocking and unit isolation |
| `unknown_units_and_instances_stay_nil_even_with_known_override` | No fabricated or cross-unit instances |
| `metadata_and_alias_mutations_are_environment_local` | Independent environment inputs |
| `invalid_metadata_never_exposes_negative_or_nonfinite_public_numbers` | Reject invalid metadata |
| `finite_metadata_sum_overflow_never_becomes_public_infinity` | Reject nonfinite refresh output |
| `zero_base_and_zero_cap_are_valid_explicit_metadata` | Zero input boundaries |
| `malformed_arguments_error_before_missing_unit_short_circuit` | Wrong types, required args and nonfinite values |
| `actual_secret_arguments_are_rejected_without_declassification_or_taint_changes` | Real secret numbers, GC, taint and recovery |
| `getters_leave_aura_metadata_aliases_and_events_unchanged_across_gc` | Immutable queries |

## Known gaps (current cycle)

- [ ] Native formula, eligibility, unknown/error/security parity remain unproven.
- [ ] Production metadata and Blizzard visual/all-profile parity remain unverified.

## Saved pre-producer RED — 2026-10-01

Input commit `dac4314c6` is included in parent revision `ea7d67237882cac6c43cfb8f3c5b8f6235e8160a`. Parent compiled the grouped integration target with exit **0**, **1168.921s**; bounded `aura_refresh_duration::` runtime exit **101**, **26.570s**, **18 selected: 1 PASS / 17 FAIL**. Full diagnostics were read before implementation. The sole empty-metadata PASS exercises nullable defaults, not a meaningful getter producer. The other seventeen failures assert outputs, validation/security and immutable-query contracts; the elapsed-clock case reports numeric result expected, got nil.

Proof artifacts: `/tmp/patch-12.0.5-batch38-red-build-result.json`, `/tmp/patch-12.0.5-batch38-red-run.json`, `/tmp/patch-12.0.5-batch38-red-run.log`. This RED scope does not prove the new producer. Parent owns subsequent costly compilation/runtime and acceptance; no new runtime result or native parity is claimed.

## Reconciled batch38 parent proof — 2026-10-01

Inputs `dac4314c6`; producer `0acd750afae92f3c6448baeae9ded210abd01bb5`. Saved GREEN compile exit **0**, **799.340s**, using `cargo test --test integration --no-run --message-format=json`. Build manifest `/tmp/patch-12.0.5-batch38-green-build-result.json` binds integration SHA256 `27764c8e2e96cb91512e438c074aa9c57919a7934647f7525f87971fa73ee9a6` and wow-sim SHA256 `319c5408e94a61a2f43868904d9cdf5f98218e05d777b970ff4bad560c7dc8ff`.

`/tmp/patch-12.0.5-batch38-green-runs.json` records exact revision, binary hash, argv and logs. Each invocation uses `timeout 90`, its named filter, `--nocapture --test-threads=1`; all exit **0**:

| Filter | Selected PASS | Saved log |
|---|---:|---|
| `aura_refresh_duration::` | 18 | `/tmp/patch-12.0.5-batch38-green-run-0.log` |
| `aura_spell_identifier::` | 12 | `/tmp/patch-12.0.5-batch38-green-run-1.log` |
| `aura_api::` | 29 | `/tmp/patch-12.0.5-batch38-green-run-2.log` |
| `admin_buff_api::` | 18 | `/tmp/patch-12.0.5-batch38-green-run-3.log` |
| `c_spell_flyout_probes::` | 14 | `/tmp/patch-12.0.5-batch38-green-run-4.log` |

**91 distinct selected PASS**, no zero-selection or overlapping test credit. Startup manifest `/tmp/patch-12.0.5-batch38-green-startup-run.json` binds the saved wow-sim hash and `timeout 90 … --no-addons --no-saved-vars lua-errors`: exit **0**, **27.721s**, stdout `[]` in `green-startup.json`, diagnostics in `green-startup.log` under the same batch38 prefix.

This proves bounded simulator getter behavior and named controls only. Formula/cap, permanence eligibility, nil/unknown and strict secret policies remain **INFERRED**, not native-verified. Metadata defaults remain empty; fixtures are explicit inputs, not invented production metadata. Duration reads filter blocked records; earlier unblocked spell-query compatibility is preserved, not silently replaced by duration access rules.

## Independent bounded acceptance — 2026-10-01

Parent accepts `/tmp/patch-12.0.5-aura-refresh-duration-independent-proof.md` after source/contracts/diagnostics review: **18 duration + 73 controls = 91 distinct PASS**, saved startup exit0 `[]`, fresh fmt/check exit0 with unchanged relevant Rust hashes. Proof remains bound to the recorded revisions/binaries; this docs-only change invalidates no Rust proof and runs no checks/build/tests.

Only `global api-C_UnitAuras-GetRefreshExtendedDuration-394` promotes to bounded coverage, capability `aura-refresh-duration`, including consumer `GetAuraBaseDuration` prerequisite and explicit unknown metadata: **257/91/14 → 256 pending / 92 bounded / 14 partial = 362**. Before/after artifact: `/tmp/patch-12.0.5-aura-refresh-duration-accounting-before-after.json`. IDs/register/source SHA and all unrelated rows preserved.

**Two nonblocking readability suggestions, not zero issues:** LENGTH `read_public_arguments` (36 body lines); COMPLEX_COND `SpellAuraDuration::is_valid` (four validity checks). Deferred: no observed behavior issue; adjacent restructuring not authorized.

Formula/cap/permanence, strict secret rejection, nil/unknown and seeded aliases remain informed policies, not native semantics. Production metadata stays empty: duration queries return nil until explicit metadata. No Blizzard visual/native/whole-page/all-profile acceptance.

## Out of scope

- Native formula/data, visibility, secret taint/output and error parity: unavailable evidence, no native-client probes required.
- Production duration records, inferred base from current aura state, alternate providers or fallback compatibility: explicitly excluded.
- Aura refresh mutation/history, cast/cooldown/charge simulation, new clock framework, legacy or existing spell-ID query changes: not needed for these immutable getters.
- Whole-page acceptance, unrelated source rows and adjacent restructuring remain excluded.
