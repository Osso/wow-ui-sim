# Action cooldown output restriction — exact Retail 12.0.5 row 233

`C_ActionBar.GetActionCooldown` must apply the bounded output restriction policy below to its existing cooldown DTO. Exact source: `data/patch-api/sources/12.0.5-api-changes.txt:233`, `SecretWhenActionCooldownRestricted -> SecretWhenCooldownsRestricted`. Batch64 implements only this output annotation after genuine compiled RED at `9ac29940debec17ce7ffe9bde6dbeb4839309122`; GREEN and acceptance remain main-owned and pending. [Lua API architecture](../wiki/systems/lua-api.md) provides subsystem context.

## What it must do

All requirements remain unchecked until producer GREEN and independent acceptance; pre-producer RED does not satisfy them.

### Output policy (explicit simulator inference)

- [ ] Return one ordinary, accessible public table; when the explicit live `cooldowns_restricted` input is true, its `startTime`, `duration` and `modRate` must independently be actual VM-owned secret numbers with unchanged numeric payloads, verified only through authenticated trusted-host inspection. Public numbers retain Lua `type == 'number'`; opaque secret values do not receive a nominal Lua-type parity claim. Trusted host production must use typed `wrap_host_secret_number`, not generic declassification, fake secret flags or caller/callback bypasses.
- [ ] Apply that numeric-field policy even to zero/empty cooldown intervals, including assigned non-cooldown spells and existing unassigned-positive-slot results. `modRate` remains numeric 1, secret under restriction, public otherwise.
- [ ] When unrestricted, return unchanged ordinary numeric values for spell-only, GCD-only, overlapping and empty state. This annotation changes output secrecy only, not interval selection or input acceptance.
- [ ] Keep `isEnabled` public true and preserve the existing profile fieldset: `isActive` exists only under `retail-12-1-0` in this test domain, with its current public active/empty truth; older 12.0.5 payloads retain four fields. Do not introduce `isOnGCD`, `activeCategory` or `timeUntilEndOfStartRecovery`.
- [ ] Read the explicit flag on each query; combat and unit-stat restriction must not substitute for it. `C_Secrets.ShouldCooldownsBeSecret` is only a state sanity control, never the expected-output oracle.

### Authentication, snapshots and lifecycle

- [ ] Public slot calls from secure and addon-tainted callers preserve caller context. Addon code may observe the public table, field secrecy and public booleans without numerically reading opaque fields.
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
- `src/c_api/c_action_bar.rs::get_action_cooldown`: after unchanged interval lookup, snapshots `super::charge_state::cooldowns_are_restricted` through an immutable borrow and releases it before VM allocation. Roots the ordinary result table immediately after creation, then loops over `startTime`, `duration`, `modRate` (1.0), using actual typed `wrap_host_secret_number` under restriction or unchanged public `Val::Num` otherwise. Zero intervals use the same inferred policy. Existing `table_set` roots each value during key allocation. Returns the already-rooted table once; `isEnabled` and profile-gated `isActive` remain public booleans computed from the raw interval. Permissive arguments, selector/defaults, field count, profile gates and wiring remain unchanged; helper false retains earlier-profile/Forever ordinary payloads without new execution proof.
- `src/lua_api/globals/action_bar_api.rs`: existing `spell_cooldown_times` latest-ending-active-interval selection; unchanged.
- `src/c_api/charge_state.rs`: existing `cooldowns_are_restricted` gates the explicit flag to `retail-12-0-5` plus Retail/PTR; unchanged. Charge output is precedent for inferred field wrapping, not proof of native ActionBar policy.
- `src/c_api/c_secrets.rs`: existing query sanity control; unchanged.

## Tests asserting this spec

`tests/action_cooldown_output_restriction.rs`, gated by `cfg(all(feature = "retail-12-0-5", any(feature = "profile-retail", feature = "client-ptr")))`:

| Cases | Test names | Proof |
|---|---|---|
| Public numeric intervals | `unrestricted_spell_interval_is_public_and_exact`, `unrestricted_gcd_only_interval_is_public_and_exact`, `unrestricted_overlap_keeps_latest_ending_gcd_selection`, `unrestricted_empty_and_unassigned_positive_slots_keep_zero_payloads` | Saved compiled RED: 4 PASS |
| Authentic restricted numeric intervals | `restricted_spell_interval_wraps_all_three_actual_numbers`, `restricted_gcd_only_interval_wraps_all_three_actual_numbers`, `restricted_overlap_keeps_latest_ending_gcd_selection`, `restricted_assigned_spell_without_cooldown_wraps_zero_interval`, `restricted_unassigned_positive_slot_keeps_secret_zero_payload` | Saved compiled RED: 5 first-secret-observation FAIL |
| Shape, current flag, callers | `booleans_and_fieldset_remain_public_for_active_and_empty_results`, `live_restriction_toggle_keeps_old_secrets_and_returns_fresh_public_numbers`, `explicit_flag_not_combat_or_stat_policy_controls_output`, `secure_and_tainted_public_callers_preserve_context_and_field_observation` | Saved compiled RED: 4 first-secret-observation FAIL |
| Denial, copy, isolation, lifetime | `tainted_arithmetic_denial_keeps_roots_context_and_secure_recovery`, `tainted_table_copy_preserves_actual_wrapper_identity`, `result_mutation_and_replacement_do_not_change_model_or_other_dtos`, `live_interval_changes_expiry_and_independent_envs_use_current_state`, `forced_gc_keeps_original_wrappers_roots_and_fresh_result_payloads` | Saved compiled RED: 5 first-secret-observation FAIL; downstream behavior unproved |

Fixtures assign positive slot 17 to actual spell ID 19750, with clock anchored at 335 seconds, spell `(312,237)` ending at 549 and GCD `(330,300)` ending at 630. Thus overlap selects GCD, consistent with the existing latest-ending selector. Slot 18 is explicitly unassigned. Expiry moves only the existing clock anchor to 900; no sleeps, new input model or fabricated charge data. These intervals have ample active margins for bounded GC tests, not an unlimited wall-clock guarantee.

## Saved pre-producer RED — 2026-10-02

Proof ledger: `/tmp/patch-12.0.5-batch64-red-build-result.json` records `cargo test --test integration --no-run --message-format=json` at `9ac29940debec17ce7ffe9bde6dbeb4839309122`, dirty-combined provenance, exit 0, **126.86125784402248 seconds**, zero compiler diagnostics. Build stderr is `/tmp/patch-12.0.5-batch64-red-build.stderr`. Integration binary `target/debug/deps/integration-d36739a1204f0133` has saved SHA256 `f1e13ffcdbe8dd4dd37fc7b0c3ba0dea45a08775a21532e1386a272ab77fe0d9`; protected source is neither inspected nor rehashed here.

`/tmp/patch-12.0.5-batch64-red-run.json` records `timeout 90 <integration-binary> action_cooldown_output_restriction:: --nocapture --test-threads=1`: exit **101**, **4.229881642968394 seconds** wall time (harness reports 3.55 seconds). Saved stdout/stderr `/tmp/patch-12.0.5-batch64-red-run.stdout` and `.stderr` contain **18 tests: 4 unrestricted PASS, 14 genuine FAIL**. The four controls establish existing numeric types/payloads, fieldset and meaningful spell/GCD/empty selection setup. Nine failures reach the initial numeric secrecy assertion, three reach actual wrapper metadata inspection, one reaches a preserved DTO's field-secrecy observation after mutation/replacement, and one reaches the tainted caller's first field-secrecy observation. Those first failures are missing output secrecy, not compile/setup defects. Later denial/copy/GC assertions are not RED proof until GREEN reaches them.

This proof covers only the pre-producer snapshot; the new producer invalidates its applicability as current passing evidence. Compile and execution costs are separate, no rerun or fresh proof is claimed, and the short run is bounded development evidence rather than whole-goal acceptance.

## Post-producer fixture correction — 2026-10-02

Saved `batch64-green-focused-run` is **not GREEN**:4PASS/14FAIL, exit101,3.0324058649130166s. Failures now reach the helper's combined type/secrecy check. Namespace registration imports `c_api::c_action_bar::get_action_cooldown`; the legacy four-return global is a separate function in `globals/cooldown_probes.rs`. Read-only511's wrong-provider diagnosis is rejected against those exact imports and bindings; no second producer or registration change is justified.

Pinned rilua604 `wrap_host_secret_number` allocates `Userdata::secret(Val::Num(value))`; builtin `type` classifies the Val tag without secret unwrapping. Requiring opaque fields to report Lua `number` was an unsupported fixture assumption, not part of source233's predicate rename. Corrected tests retain unrestricted numeric types, authentic `issecretvalue`, trusted-host exact `Val::Num` payloads, caller denial/copy/root/GC checks and public booleans. They make no secret nominal-type parity claim. No VM type override or declassification is added. Record native `type(secretField)` separately in a future probe; actual native primitive parity remains unresolved.

First corrected focused execution is still not fully GREEN:17PASS/1FAIL at `add2d0a84`, exit101,3.3528383200755343s. The remaining copy fixture repeated the same unsupported private `type == number` check. That assertion is now corrected too; a full pattern scan of both cooldown fixture files leaves nominal numeric-type checks only for unrestricted values. Copy metadata/identity and denial checks remain intact. Final corrected execution and independent acceptance remain pending. Producer unchanged; initial RED proves missing secrecy, not the later type expectation. Full source trace/corrections live in `/tmp/patch-12.0.5-action-cooldown-publication-failure.md`.

## Saved combined parent GREEN — 2026-10-02

At `cec856187c1c5bb2fc278d2aa0e268e6a20eb755`, default integration compilation exits0/zero diagnostics in186.0260727679124s. Full compiler output/hash manifest: `/tmp/patch-12.0.5-batch64-65-final-green-build.stdout.jsonl`, `.stderr`, `-result.json`. Integration SHA256 `7315587398811b5976e50819f5961983276070f4908b274685d7dc16c9e2668d` binds all runs.

`batch64-65-green-runs.json` records **117 distinct PASS**:18 row233 focused,22 row239 focused,42 cooldown controls,11 state globals,10 slot mutation and14 spell/flyout controls; six exits0, no overlap. Execution24.42142666503787s is separate from compilation, below60s bounded-development target, not padded full-goal acceptance. Startup separately returns `[]`, exit0,9.541190293966793s; full output/hash paths in `green-startup-run.json`.

All row233 opacity/copy/GC/root/caller checks now execute successfully without requiring opaque numeric fields to report Lua `number`. Producer remains fad6e780f; only unfounded fixture assumptions were corrected. Native nominal primitive parity remains a gap. Independent515 security/wiring/readability/scoped Rust checks and combined Forever preservation are pending; row233 receives no credit yet. Dirty-combined/globalfmt limits remain; no valid runtime/build replay solely for docs/accounting.

## Known gaps (current cycle)

- [ ] Main-owned targeted producer GREEN, startup, check, security/wiring, readability, independent acceptance and exact233 accounting remain pending. No delegation, build, test, check, lint, readability gate, operations or push occurs in this implementation slice; only the owned changed Rust file is formatted with child traversal disabled.
- [ ] Saved RED proves missing secret outputs, not downstream arithmetic denial, secure recovery, copying, isolation or GC. Producer implementation is committed before gates, not claimed passing. No fixture correction is justified or made.
- [ ] Accounting stays unchanged: user-provided current checkpoint is 197 pending, 150 bounded, 14 partial, 1 metadata = 362 IDs, 69 capabilities. Row 233 remains pending; existing 40-stat/charge partial capability is not upgraded. No accounting artifact edits or capability credit.
- [ ] Main's row295 independent505 Forever 3-PASS artifact and 20-test Retail refresh remain pending main acceptance; row239's 65 tests at `ed682bf3d` are only the next RED. Their source/tests/specs, accounting, wiki and PLAN remain untouched; neither slice proves row233.
- [ ] Future optional native probe: record slot 0 behavior separately, public unrestricted/restricted results for valid assigned slots, active spell versus active GCD selection, empty intervals, tainted public-slot queries, ordinary table field copies, numeric Lua types and `isEnabled`/`isActive`/optional `isOnGCD` truth and secrecy exceptions. Probe work is not a completion gate here.

## Source grounding and inference limits

Cached full declaration read at `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ActionBarFrameDocumentation.lua:155–170`: `GetActionCooldown` requires `actionID` (`luaIndex`, non-nil), declares `RequiresValidActionSlot = true`, `SecretWhenCooldownsRestricted = true`, `SecretArguments = "AllowedWhenUntainted"`, and returns non-nil `SpellCooldownInfo`.

Cached `SpellSharedDocumentation.lua:19–31` defines numeric `startTime`, `duration`, `modRate` without `NeverSecret`; boolean `isEnabled` and `isActive` are `NeverSecret`; optional `isOnGCD` is also `NeverSecret`. The cache also lists optional category/recovery fields not currently modeled here. The annotations support selecting numeric rather than boolean fields, but do not prove ordinary-table versus whole-table secrecy, all zero-result cases, exact native exceptions, slot validity/access behavior or result parity. Numeric-field/zero/table policies are explicitly chosen simulator inferences based on these declarations and existing charge wrapping, not native findings.

The temporary recommendation `/tmp/patch-12.0.5-next-actionbar-security-boundary.md` is historical context only. Its inaccurate accounting, proposed existing-test destination, Forever helper claim and stop-if-unknown-metadata instruction do not constrain this authorized slice. Actual source and explicit user requirements govern.

## Out of scope

Input guards, `AllowedWhenUntainted` implementation/credit, NeverSecret argument credit, slot ranges or parser changes; duration objects and consumers; production edits outside `get_action_cooldown` or input scaffolding; Forever policy and earlier-profile execution; rows231/237/239/241 or other annotation rows; new DTO fields; callback replacement/security relaxation; protected `aura_duration` inspection; native parity and probes; Cargo changes/delegation/operations/push. Existing profile payload and selector controls do not claim these excluded capabilities.
