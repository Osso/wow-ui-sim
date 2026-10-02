# BreakUpLargeNumbers — exact411

Next53 covers only `global api-Localization BreakUpLargeNumbers-411` in the cumulative mainline `retail-12-0-5` epoch. Source: `data/patch-api/sources/12.0.5-api-changes.txt:411`; retained Localization declaration adds `arg2 NeverSecret`, declares `largeNumber: number`, `natural: bool` with default false, and one string result. `AllowedWhenTainted` is not permission to decode a NeverSecret argument. See [Lua API architecture](../lua-api.md) for runtime integration.

**Chosen INFERRED simulator model, not native parity:** localized ICU Decimal grouping through the current global `GetLocale` provider; omitted/nil/false natural uses ordinary decimal, true truncates toward zero before formatting. The true branch is an explicit guess with intentionally observable behavior. Strict public types, finite-only input, public result, conservative secret arg1 rejection, and requiring the current GetLocale callback to return a public UTF-8 string are local policies; Lua callback errors, missing/wrong-type results, non-UTF-8 bytes and authentic VM-secret locale strings must error with public recovery after restoring a known provider. This locale input guard is explicit local policy, not native permission evidence; row411 does not establish arg1 permissions or result secrecy. Native probes unavailable; not a completion gate for this inferred model.

## What it must do

### Formatting and inputs

- [ ] After `WowLuaEnv::new`, return exactly one public STRING: enUS `1234` → `1,234`, `1234.5` → `1,234.5`; omission, nil and false agree (INFERRED).
- [ ] Natural true truncates toward zero: `1234.75` → `1,234`, `-1234.75` → `-1,234`, `0.75` → `0`, `-1.75` → `-1`; false retains fractions (EXPLICIT GUESS).
- [ ] Format zero and group boundaries without abbreviation: `0`, `999`, `1,000`, `-1,000`, `1,000,000` (INFERRED).
- [ ] Read the current global locale provider on each successful call; explicit fixture mutations enUS/deDE/frFR take immediate effect. `1234.5` gives `1,234.5` / `1.234,5` / `1\u202f234,5` (INFERRED; no native locale acquisition).
- [ ] Leave provider identity, caller input tables and actual Frame identity/properties unchanged; environment fixtures remain isolated.
- [ ] Require public finite NUMBER arg1 and optional public BOOL arg2, default false. Reject missing/nil arg1, numeric strings, booleans, tables, functions, actual Frames, NaN/infinities; reject nonbool public natural values without coercion. Fresh valid calls recover after errors (local strict policy).

### Security and lifetime

- [ ] Reject authentic host-VM secret natural BOOL false and true even in secure callers, before payload inspection, locale acquisition or formatting. Rejection must not depend on hidden payload (NeverSecret boundary).
- [ ] Reject actual secret NUMBER, STRING, Frame and table natural inputs, preserving secrecy, wrapper identity and original object state.
- [ ] Forced GC preserves global, list and stack roots; later rejection and public recovery remain valid.
- [ ] Secure and stamped-tainted calls preserve caller taint through secret rejection, public type errors and public recovery, including the inferred true branch.
- [ ] Conservatively reject secret NUMBER arg1 in both caller contexts without declassification (local policy, **not source411 arg1 permission evidence**).

## How it works

- [Lua API architecture](../lua-api.md).
- [Client profiles](../wiki/systems/client-profiles.md).
- [NeverSecret space-limit precedent](string-util-space-limit-security.md).

## Implementation inventory

- `tests/break_up_large_numbers.rs`: 20 focused epoch-gated behavioral tests; host-VM secrets and explicit current-global locale providers, never replacing `BreakUpLargeNumbers`.
- `tests/integration.rs`: existing generated grouped integration entrypoint; parent owns dispatcher inclusion/compiled proof. No new Cargo target.
- [x] `src/lua_api/globals/real/break_up_large_numbers.rs`: implemented strict public finite NUMBER + optional public BOOL; VM secret authentication for both arguments precedes type/payload/locale access. One inferred public string, no unwrap or taint mutation. GREEN pending.
- [x] `src/lua_api/globals/register.rs::register_frame_foundation_globals`: invokes the real registrar under cumulative `retail-12-0-5`; `real/mod.rs` uses the same epoch gate. `env_init::init_lua_state` runs this before temporary bootstraps.
- [x] `src/lua_api/workarounds/temporary/formatting_utility_defaults.rs`: tostring provider compiled only without `retail-12-0-5`, not a competing enabled-epoch nil fallback. Startup/reapplication assertion expects `12,345` in the enabled epoch and retains `12345` earlier.
- [x] Current global `GetLocale` is read on every successful call through the locked VM's `LuaState::call_function` stack API (no method named `call` exists there). Getter/result remain stack-rooted until locale bytes are copied; temporary stack/frame state restores on success/error. The separate enUS fallback is installed before normal invocation. Four ASCII letters normalize generically from language+region to language_region; other identifiers pass through, with no fabricated catalog or widened locale helper.
- [x] `src/c_api/mod.rs` and `build/intl_native.rs`: existing native number backend and its shims compile from `retail-12-0-5` so the producer reuses `intl_native::format_number(..., NumberStyle::Decimal)` exactly. `C_Intl` registration/publication and Intl operations are unchanged; no new state. Existing ICU precision defaults remain the limit.

Implementation precedes parent GREEN; unchecked behavioral requirements above are not acceptance claims.

## Tests asserting this spec

`tests/break_up_large_numbers.rs` contains 20 tests:

| Capability | Exact test names | Proof level |
|---|---|---|
| Public result/default, natural guess, boundaries | `startup_formatter_returns_one_public_grouped_string_with_default_agreement`, `inferred_natural_truncates_both_signs_toward_zero_not_floor`, `inferred_zero_and_group_boundaries_have_decimal_not_abbreviated_output` | Parent compiled RED; GREEN pending |
| Locale mutation, read-only inputs, environment isolation | `current_wow_locale_provider_mutations_apply_immediately`, `formatter_keeps_provider_identity_and_caller_inputs_read_only`, `locale_and_input_fixtures_are_isolated_between_environments` | Parent compiled RED; GREEN pending |
| Strict bad types/nonfinite and recovery | `required_public_number_rejects_missing_nil_and_wrong_types_then_recovers`, `nonfinite_numbers_reject_in_both_natural_modes_then_recover`, `optional_public_natural_rejects_nonbool_values_then_recovers` | Parent compiled RED; GREEN pending |
| Authentic secret natural and GC | `authentic_secret_false_natural_rejects_even_in_secure_context`, `authentic_secret_true_natural_rejects_even_in_secure_context`, `secret_number_and_string_natural_reject_before_formatting`, `wrapped_actual_frame_and_table_natural_reject_without_representation_assumptions`, `gc_keeps_global_list_and_stack_secret_roots_identical_and_secret` | Parent compiled RED; GREEN pending |
| Taint and conservative arg1 | `secure_and_stamped_tainted_calls_keep_taint_across_rejection_and_recovery`, `secret_arg1_rejection_is_conservative_local_policy_not_row411_permission` | Parent compiled RED; GREEN pending |

Supplement53 adds four callback-boundary tests against the already-committed producer `6ed338813`; no fabricated RED or GREEN claim. Each runs with secure and stamped-tainted callers, checks unchanged original Frame/table identity and state, restores a known GetLocale provider, and issues multiple valid recovery queries. The GC provider collects inside the actual Lua callback before returning a dynamically assembled public locale; the secret-locale fixture wraps `enUS` with the actual host VM helper. Only GetLocale fixture inputs change, never BreakUpLargeNumbers. Parent compiled integration/lib GREEN and independent gates remain pending.

| Supplement capability | Exact test name | Proof level |
|---|---|---|
| Lua callback error propagation and recovery | `locale_provider_lua_error_preserves_callers_and_multiple_query_recovery` | Added, uncompiled; parent GREEN pending |
| Missing/wrong public result and invalid UTF-8 rejection | `locale_provider_wrong_missing_and_non_utf8_results_reject_then_recover` | Added, uncompiled; parent GREEN pending |
| Authentic secret locale rejection under explicit public input policy | `authentic_secret_locale_result_rejects_under_local_public_input_policy` | Added, uncompiled; parent GREEN pending |
| GC inside callback, live roots and public recovery | `locale_provider_collects_before_returning_public_locale_with_live_roots` | Added, uncompiled; parent GREEN pending |

Source fixture maps: `/tmp/patch-12.0.5-break-up-large-numbers-model-map.md`, `/tmp/patch-12.0.5-break-up-intl-locale-map.md`, `/tmp/patch-12.0.5-break-up-producer-fixture-boundary.md`. Requested `large-numbers-model.md` was absent; existing model-map used. The chosen inferred true behavior fills the map's previously unresolved branch; it is an assistant-selected guess under the permitted inference policy, not a user-selected native contract.

## Known gaps (current cycle)

- [x] Parent compiled RED at `77af6ad9a8723401298574b1a307796667312332`: build exit0 / 390.341632021009s (includes blocking Cargo lock; stderr reports finished381s); focused run exit101 / 10.068309725029394s, 0 PASS / 16 genuine FAIL. Startup and actual secret VM/Frame/GC fixture setup succeeded before strict-rejection/localized-decimal failures. Artifacts: `/tmp/patch-12.0.5-batch53-red-build-result.json`, `/tmp/patch-12.0.5-batch53-red-run.json` and their full outputs. Dirty-source-bound evidence includes preserved unowned duration diff SHA256 `6967f0b47312d926c2359bd29bc1c26d4d1d523522abf8d87e067104da170a1a`, not clean-revision proof.
- [ ] Parent GREEN, startup reapplication, current checks and independent acceptance pending. No builds/tests/checks/readability/coverage/gates run by this implementer; supplement53 changes only the existing test fixture/test file and this inventory/policy, with no new Cargo target. Source411 and accounting remain uncredited.

## Out of scope

- Native parity, native exact errors, native natural semantics, arg1 secret permissions and result secrecy: unknown; explicit inferences above do not resolve them.
- Precision beyond the existing ICU Decimal defaults; broad locale/catalog acquisition or new models.
- Earlier-profile behavior changes, Intl API changes, new state, and other producers.
- Source411 credit, accounting changes, broad acceptance, unavailable native probes, and other agents' files.
