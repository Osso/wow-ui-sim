# Action cooldown output restriction — exact Retail 12.0.5 row 233

`C_ActionBar.GetActionCooldown` must apply the bounded output restriction policy below to its existing cooldown DTO. Exact source: `data/patch-api/sources/12.0.5-api-changes.txt:233`, `SecretWhenActionCooldownRestricted -> SecretWhenCooldownsRestricted`. This slice adds tests/spec only; producer changes, compiled RED, GREEN and acceptance remain pending. [Lua API architecture](../wiki/systems/lua-api.md) provides subsystem context.

## What it must do

All requirements remain unchecked until compiled behavioral evidence exists.

### Output policy (explicit simulator inference)

- [ ] Return one ordinary, accessible public table; when the explicit live `cooldowns_restricted` input is true, its `startTime`, `duration` and `modRate` must independently be actual VM-owned secret numbers with unchanged numeric payloads and Lua `type == 'number'`. Trusted host production must use typed `wrap_host_secret_number`, not generic declassification, fake secret flags or caller/callback bypasses.
- [ ] Apply that numeric-field policy even to zero/empty cooldown intervals, including assigned non-cooldown spells and existing unassigned-positive-slot results. `modRate` remains numeric 1, secret under restriction, public otherwise.
- [ ] When unrestricted, return unchanged ordinary numeric values for spell-only, GCD-only, overlapping and empty state. This annotation changes output secrecy only, not interval selection or input acceptance.
- [ ] Keep `isEnabled` public true and preserve the existing profile fieldset: `isActive` exists only under `retail-12-1-0` in this test domain, with its current public active/empty truth; older 12.0.5 payloads retain four fields. Do not introduce `isOnGCD`, `activeCategory` or `timeUntilEndOfStartRecovery`.
- [ ] Read the explicit flag on each query; combat and unit-stat restriction must not substitute for it. `C_Secrets.ShouldCooldownsBeSecret` is only a state sanity control, never the expected-output oracle.

### Authentication, snapshots and lifecycle

- [ ] Public slot calls from secure and addon-tainted callers preserve caller context. Addon code may observe table/field types, secrecy and public booleans without numerically reading opaque fields.
- [ ] Addon arithmetic on each authentic secret numeric field fails through the existing VM authentication boundary, with a nonempty error and no tested private interval payload disclosure; recovery restores a secure host entry without declassification. Error wording is not native-verified.
- [ ] Field copying through an ordinary table preserves original secret wrapper identity and secrecy. Secure host-only inspection authenticates and verifies the three exact numeric payloads independently; never compare secret booleans in tainted Lua.
- [ ] Turning restriction off produces a fresh public-number result without changing old rooted secret wrappers. Mutating or replacing a returned table must not change other DTO snapshots, spell/GCD inputs, action mappings, charge inputs or the flag.
- [ ] Later queries reflect live interval replacement, expiry and GCD removal. Independent environments retain their own inputs and output policy.
- [ ] Original rooted wrappers retain VM identity, allocation sequence and root-list membership across authentication denial, copying, fresh queries, allocation churn and forced GC. Fresh result fields retain correct secrecy and payloads after GC.

## How it works

- [Lua API architecture](../wiki/systems/lua-api.md)
- [Existing ignore-GCD duration contract](action-cooldown-duration.md)
- [Existing explicit cooldown-restriction contract](cooldown-restriction.md)

## Implementation inventory

- `tests/action_cooldown_output_restriction.rs`: 18 grouped behavioral tests, discovered by existing `build.rs` into `tests/integration.rs`; no new Cargo target or harness changes.
- `src/c_api/c_action_bar.rs`: existing `get_action_cooldown` producer and permissive `read_action_cooldown`; read-only in this slice. Currently publishes ordinary numbers, `isEnabled`, and the profile-gated `isActive`.
- `src/lua_api/globals/action_bar_api.rs`: existing `spell_cooldown_times` latest-ending-active-interval selection; unchanged.
- `src/c_api/charge_state.rs`: existing `cooldowns_are_restricted` gates the explicit flag to `retail-12-0-5` plus Retail/PTR; unchanged. Charge output is precedent for inferred field wrapping, not proof of native ActionBar policy.
- `src/c_api/c_secrets.rs`: existing query sanity control; unchanged.

## Tests asserting this spec

`tests/action_cooldown_output_restriction.rs`, gated by `cfg(all(feature = "retail-12-0-5", any(feature = "profile-retail", feature = "client-ptr")))`:

| Cases | Test names | Proof |
|---|---|---|
| Public numeric intervals | `unrestricted_spell_interval_is_public_and_exact`, `unrestricted_gcd_only_interval_is_public_and_exact`, `unrestricted_overlap_keeps_latest_ending_gcd_selection`, `unrestricted_empty_and_unassigned_positive_slots_keep_zero_payloads` | Authored only |
| Authentic restricted numeric intervals | `restricted_spell_interval_wraps_all_three_actual_numbers`, `restricted_gcd_only_interval_wraps_all_three_actual_numbers`, `restricted_overlap_keeps_latest_ending_gcd_selection`, `restricted_assigned_spell_without_cooldown_wraps_zero_interval`, `restricted_unassigned_positive_slot_keeps_secret_zero_payload` | Authored only |
| Shape, current flag, callers | `booleans_and_fieldset_remain_public_for_active_and_empty_results`, `live_restriction_toggle_keeps_old_secrets_and_returns_fresh_public_numbers`, `explicit_flag_not_combat_or_stat_policy_controls_output`, `secure_and_tainted_public_callers_preserve_context_and_field_observation` | Authored only |
| Denial, copy, isolation, lifetime | `tainted_arithmetic_denial_keeps_roots_context_and_secure_recovery`, `tainted_table_copy_preserves_actual_wrapper_identity`, `result_mutation_and_replacement_do_not_change_model_or_other_dtos`, `live_interval_changes_expiry_and_independent_envs_use_current_state`, `forced_gc_keeps_original_wrappers_roots_and_fresh_result_payloads` | Authored only |

Fixtures assign positive slot 17 to actual spell ID 19750, with clock anchored at 335 seconds, spell `(312,237)` ending at 549 and GCD `(330,300)` ending at 630. Thus overlap selects GCD, consistent with the existing latest-ending selector. Slot 18 is explicitly unassigned. Expiry moves only the existing clock anchor to 900; no sleeps, new input model or fabricated charge data. These intervals have ample active margins for bounded GC tests, not an unlimited wall-clock guarantee.

## Known gaps (current cycle)

- [ ] Parent-owned compiled RED on the new grouped tests; compilation errors do not count as RED. No build, test, check, lint, readability or acceptance execution occurs in this slice.
- [ ] Producer implementation, targeted GREEN and independent gates remain pending. Expected secret-output failures are hypotheses until compiled RED; downstream denial/copy/GC assertions are not proven by an earlier failure.
- [ ] Accounting stays unchanged: user-provided current checkpoint is 197 pending, 150 bounded, 14 partial, 1 metadata = 362 IDs, 69 capabilities. Row 233 remains pending; existing 40-stat/charge partial capability is not upgraded. No accounting artifact edits or capability credit.
- [ ] Main's separate row295 producer/independent502 work and active batch63 files remain untouched; their execution/proof is not evidence for this slice.
- [ ] Future optional native probe: record slot 0 behavior separately, public unrestricted/restricted results for valid assigned slots, active spell versus active GCD selection, empty intervals, tainted public-slot queries, ordinary table field copies, numeric Lua types and `isEnabled`/`isActive`/optional `isOnGCD` truth and secrecy exceptions. Probe work is not a completion gate here.

## Source grounding and inference limits

Cached full declaration read at `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ActionBarFrameDocumentation.lua:155–170`: `GetActionCooldown` requires `actionID` (`luaIndex`, non-nil), declares `RequiresValidActionSlot = true`, `SecretWhenCooldownsRestricted = true`, `SecretArguments = "AllowedWhenUntainted"`, and returns non-nil `SpellCooldownInfo`.

Cached `SpellSharedDocumentation.lua:19–31` defines numeric `startTime`, `duration`, `modRate` without `NeverSecret`; boolean `isEnabled` and `isActive` are `NeverSecret`; optional `isOnGCD` is also `NeverSecret`. The cache also lists optional category/recovery fields not currently modeled here. The annotations support selecting numeric rather than boolean fields, but do not prove ordinary-table versus whole-table secrecy, all zero-result cases, exact native exceptions, slot validity/access behavior or result parity. Numeric-field/zero/table policies are explicitly chosen simulator inferences based on these declarations and existing charge wrapping, not native findings.

The temporary recommendation `/tmp/patch-12.0.5-next-actionbar-security-boundary.md` is historical context only. Its inaccurate accounting, proposed existing-test destination, Forever helper claim and stop-if-unknown-metadata instruction do not constrain this authorized slice. Actual source and explicit user requirements govern.

## Out of scope

Input guards, `AllowedWhenUntainted` implementation/credit, NeverSecret argument credit, slot ranges or parser changes; duration objects and consumers; production edits or input scaffolding; Forever policy and earlier-profile execution; rows231/237/239/241 or other annotation rows; new DTO fields; callback replacement/security relaxation; protected `aura_duration` inspection; native parity and probes; Cargo changes/delegation/operations/push. Existing profile payload and selector controls do not claim these excluded capabilities.
