# BreakUpLargeNumbers — exact411

Next53 covers only `global api-Localization BreakUpLargeNumbers-411` in the cumulative mainline `retail-12-0-5` epoch. Source: `data/patch-api/sources/12.0.5-api-changes.txt:411`; retained Localization declaration adds `arg2 NeverSecret`, declares `largeNumber: number`, `natural: bool` with default false, and one string result. `AllowedWhenTainted` is not permission to decode a NeverSecret argument. See [Lua API architecture](../lua-api.md) for runtime integration.

**Chosen INFERRED simulator model, not native parity:** localized ICU Decimal grouping through the current global `GetLocale` provider; omitted/nil/false natural uses ordinary decimal, true truncates toward zero before formatting. The true branch is an explicit guess with intentionally observable behavior. Strict public types, finite-only input, public result, and conservative secret arg1 rejection are local policies; row411 does not establish arg1 permissions or result secrecy. Native probes unavailable; not a completion gate for this inferred model.

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

- `tests/break_up_large_numbers.rs`: 16 focused epoch-gated behavioral tests; host-VM secrets and explicit current-global locale providers, never replacing `BreakUpLargeNumbers`.
- `tests/integration.rs`: existing generated grouped integration entrypoint; parent owns dispatcher inclusion/compiled proof. No new Cargo target.
- `src/lua_api/workarounds/temporary/formatting_utility_defaults.rs`: current temporary tostring fallback and startup assertion `12345 -> 12345`; unchanged in this tests/spec slice. Later mainline producer must retire that competing fallback and update the relevant startup assertion while preserving earlier-profile behavior.
- `src/c_api/intl_native.rs`: existing ICU Decimal precision inherited by the future producer; no broader precision guarantees or Intl API change.

Future real-global registration dispatcher remains parent-owned. No producer/runtime/registration/state changes authorized before parent compiled RED.

## Tests asserting this spec

`tests/break_up_large_numbers.rs` contains 16 tests:

| Capability | Exact test names | Proof level |
|---|---|---|
| Public result/default, natural guess, boundaries | `startup_formatter_returns_one_public_grouped_string_with_default_agreement`, `inferred_natural_truncates_both_signs_toward_zero_not_floor`, `inferred_zero_and_group_boundaries_have_decimal_not_abbreviated_output` | Written; uncompiled |
| Locale mutation, read-only inputs, environment isolation | `current_wow_locale_provider_mutations_apply_immediately`, `formatter_keeps_provider_identity_and_caller_inputs_read_only`, `locale_and_input_fixtures_are_isolated_between_environments` | Written; uncompiled |
| Strict bad types/nonfinite and recovery | `required_public_number_rejects_missing_nil_and_wrong_types_then_recovers`, `nonfinite_numbers_reject_in_both_natural_modes_then_recover`, `optional_public_natural_rejects_nonbool_values_then_recovers` | Written; uncompiled |
| Authentic secret natural and GC | `authentic_secret_false_natural_rejects_even_in_secure_context`, `authentic_secret_true_natural_rejects_even_in_secure_context`, `secret_number_and_string_natural_reject_before_formatting`, `wrapped_actual_frame_and_table_natural_reject_without_representation_assumptions`, `gc_keeps_global_list_and_stack_secret_roots_identical_and_secret` | Written; uncompiled |
| Taint and conservative arg1 | `secure_and_stamped_tainted_calls_keep_taint_across_rejection_and_recovery`, `secret_arg1_rejection_is_conservative_local_policy_not_row411_permission` | Written; uncompiled |

Source fixture maps: `/tmp/patch-12.0.5-break-up-large-numbers-model-map.md`, `/tmp/patch-12.0.5-break-up-intl-locale-map.md`, `/tmp/patch-12.0.5-break-up-producer-fixture-boundary.md`. Requested `large-numbers-model.md` was absent; existing model-map used. The chosen inferred true behavior fills the map's previously unresolved branch; it is an assistant-selected guess under the permitted inference policy, not a user-selected native contract.

## Known gaps (current cycle)

- [ ] Parent compiled RED, future producer GREEN and independent acceptance pending. No compiler/tests/checks/gates run in this slice; source411 remains uncredited.
- [ ] Parent must integrate the fixture through the existing grouped target and establish actual current-tostring failures; written expectations alone are not RED evidence.
- [ ] Later producer must use existing/generic locale normalization for WoW identifiers, not an invented locale catalog, while leaving Intl API unchanged.
- [ ] Parent must replace mainline fallback ownership/startup expectation, retain earlier profiles, and verify startup with the meaningful formatter.

## Out of scope

- Native parity, native exact errors, native natural semantics, arg1 secret permissions and result secrecy: unknown; explicit inferences above do not resolve them.
- Precision beyond the existing ICU Decimal defaults; broad locale/catalog acquisition or new models.
- Earlier-profile behavior changes, Intl API changes, producer implementation and registration/state changes in this tests/spec slice.
- Source411 credit, accounting changes, broad acceptance, unavailable native probes, and other agents' files.
