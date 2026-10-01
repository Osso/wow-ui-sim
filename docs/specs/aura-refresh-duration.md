# Aura refresh duration

Retail 12.0.5 row394, `C_UnitAuras.GetRefreshExtendedDuration`, and its consumer prerequisite `C_UnitAuras.GetAuraBaseDuration` query explicit per-environment recast metadata independently of current aura duration. Input types live in `src/c_api/aura_duration.rs`; architecture references are under [How it works](#how-it-works). The producer registers both immutable Lua getters at `retail-12-0-5`; parent GREEN and bounded acceptance remain pending.

## What it must do

### Input and nullable output

- [ ] Both getters accept `(unit, auraInstanceID, optionalSpellIdentifier)` and return exactly one nullable number. Omitted/nil identifier uses the matched aura's `spell_id` directly. Explicit numeric/string identifiers use the existing C_Spell resolver, including seeded case-normalized aliases and numeric alias precedence. Resolver reuse is **INFERRED**; labels in tests are fixtures, not native spell names/links.
- [ ] Inputs are an environment-local `HashMap<u32, SpellAuraDuration>` with explicit `base_duration_seconds` and `max_carryover_seconds`. Default map is empty. Never seed invented production durations or infer base from `AuraInfo.duration`, remaining time, spell icons/names, or another environment.
- [ ] **INFERRED unknown policy:** unknown alias, missing metadata, unit or instance yields nil. Explicit unknown overrides never fall back to the aura's own spell. Metadata alone cannot create an aura or cross unit/instance identity.
- [ ] Resolve only existing public, unblocked helpful/harmful collector records, including modeled party stores. Blocking is scoped to exact unit and instance; a blocked instance remains inaccessible with an explicit known override. No new generic target store or native visibility enforcement.

### Duration policy — explicitly INFERRED, not native-verified

- [ ] Base getter returns configured base duration, not current duration. Refresh returns `base + min(max(expiration - now, 0), max_carryover_seconds)`, with `now` from the environment's existing `start_time.elapsed()` clock. The cap is explicit metadata, not an invented fixed percentage.
- [ ] Saturated and expired timed cases return exact cap/base results. Unsaturated tests bound elapsed time around the call with tolerance; do not introduce a time override framework.
- [ ] Permanent active aura (`duration == 0` or `expiration_time == 0`) yields nil for refresh, even with known recast metadata. Base getter still returns known metadata. This eligibility rule is an **informed guess**, not documented native behavior.
- [ ] Nonfinite or negative metadata in either field yields nil from both getters. Zero metadata remains valid for a timed active aura. Nonfinite refresh sums yield nil, never public infinity. Both metadata fields are validated before either getter's public output.

### Validation and query immutability — explicitly INFERRED

- [ ] Unit accepts a public string or nil (nil means unknown); required instance accepts a public finite number. Missing instance, wrong types and nonfinite numeric arguments error with nonempty diagnostics, including absent-unit queries. Optional spell accepts nil or a public finite number/string; no new spell parser or additional numeric range policy.
- [ ] Conservative rejection of real secret unit/instance/spell arguments precedes absent-unit results, in secure and addon-tainted callers. Full GC preserves secret identity; rejection neither declassifies inputs nor changes caller taint. Later public queries still work. This is **not** `SecretArguments = "AllowedWhenTainted"` or native secret-output parity.
- [ ] Both getters leave aura identity/current duration/expiration, metadata, aliases and event queue unchanged. Metadata and alias mutations remain environment-local.

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

Grouped filter: `aura_refresh_duration::` with `retail-12-0-5` enabled. Parent owns compilation and GREEN execution. Requirement checkboxes remain unverified until post-producer proof; tests are unchanged.

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

- [ ] Parent GREEN and independent bounded acceptance remain pending. Producer formatting only; no build/test/check/lint/readability/delegation performed in this slice.
- [ ] Native formula, eligibility, unknown/error/security parity remain unproven.
- [ ] Row394/page/accounting remain pending and unchanged; no status promotion authorized here.

## Saved pre-producer RED — 2026-10-01

Input commit `dac4314c6` is included in parent revision `ea7d67237882cac6c43cfb8f3c5b8f6235e8160a`. Parent compiled the grouped integration target with exit **0**, **1168.921s**; bounded `aura_refresh_duration::` runtime exit **101**, **26.570s**, **18 selected: 1 PASS / 17 FAIL**. Full diagnostics were read before implementation. The sole empty-metadata PASS exercises nullable defaults, not a meaningful getter producer. The other seventeen failures assert outputs, validation/security and immutable-query contracts; the elapsed-clock case reports numeric result expected, got nil.

Proof artifacts: `/tmp/patch-12.0.5-batch38-red-build-result.json`, `/tmp/patch-12.0.5-batch38-red-run.json`, `/tmp/patch-12.0.5-batch38-red-run.log`. This RED scope does not prove the new producer. Parent owns subsequent costly compilation/runtime and acceptance; no new runtime result or native parity is claimed.

## Out of scope

- Native formula/data, visibility, secret taint/output and error parity: unavailable evidence, no native-client probes required.
- Production duration records, inferred base from current aura state, alternate providers or fallback compatibility: explicitly excluded.
- Aura refresh mutation/history, cast/cooldown/charge simulation, new clock framework, legacy or existing spell-ID query changes: not needed for these immutable getters.
- Whole-page/row acceptance and audit accounting, unrelated docs311/verifier310 work: parent/concurrent ownership preserved.
