# BreakUpLargeNumbers — exact411

Next53 covers only `global api-Localization BreakUpLargeNumbers-411` in the cumulative mainline `retail-12-0-5` epoch. Source: `data/patch-api/sources/12.0.5-api-changes.txt:411`; retained Localization declaration adds `arg2 NeverSecret`, declares `largeNumber: number`, `natural: bool` with default false, and one string result. `AllowedWhenTainted` is not permission to decode a NeverSecret argument. See [Lua API architecture](../lua-api.md) for runtime integration.

**Chosen INFERRED simulator model, not native parity:** localized ICU Decimal grouping through the current global `GetLocale` provider; omitted/nil/false natural uses ordinary decimal, true truncates toward zero before formatting. The true branch is an explicit guess with intentionally observable behavior. Strict public types, finite-only input, public result, conservative secret arg1 rejection, and requiring the current GetLocale callback to return a public UTF-8 string are local policies; Lua callback errors, missing/wrong-type results, non-UTF-8 bytes and authentic VM-secret locale strings must error with public recovery after restoring a known provider. This locale input guard is explicit local policy, not native permission evidence; row411 does not establish arg1 permissions or result secrecy. Native probes unavailable; not a completion gate for this inferred model.

## What it must do

### Formatting and inputs

- [x] After `WowLuaEnv::new`, return exactly one public STRING: enUS `1234` → `1,234`, `1234.5` → `1,234.5`; omission, nil and false agree (INFERRED).
- [x] Natural true truncates toward zero: `1234.75` → `1,234`, `-1234.75` → `-1,234`, `0.75` → `0`, `-1.75` → `-1`; false retains fractions (EXPLICIT GUESS).
- [x] Format zero and group boundaries without abbreviation: `0`, `999`, `1,000`, `-1,000`, `1,000,000` (INFERRED).
- [x] Read the current global locale provider on each successful call; explicit fixture mutations enUS/deDE/frFR take immediate effect. `1234.5` gives `1,234.5` / `1.234,5` / `1\u202f234,5` (INFERRED; no native locale acquisition).
- [x] Leave provider identity, caller input tables and actual Frame identity/properties unchanged; environment fixtures remain isolated.
- [x] Require public finite NUMBER arg1 and optional public BOOL arg2, default false. Reject missing/nil arg1, numeric strings, booleans, tables, functions, actual Frames, NaN/infinities; reject nonbool public natural values without coercion. Fresh valid calls recover after errors (local strict policy).

### Security and lifetime

- [x] Reject authentic host-VM secret natural BOOL false and true even in secure callers, before payload inspection, locale acquisition or formatting. Rejection must not depend on hidden payload (NeverSecret boundary).
- [x] Reject actual secret NUMBER, STRING, Frame and table natural inputs, preserving secrecy, wrapper identity and original object state.
- [x] Forced GC preserves global, list and stack roots; later rejection and public recovery remain valid. Tainted secret-BOOL raw equality must assert VM denial, not success; secure Lua equality proves only boolean payload equality. Prove wrapper identity separately at the host boundary using pre-execution `Val`/`GcRef` identity plus live userdata allocation metadata, comparing global/list roots and exported stack-held wrappers after Lua returns without reading payloads or changing taint.
- [x] Secure and stamped-tainted calls preserve caller taint through secret rejection, public type errors and public recovery, including the inferred true branch.
- [x] Conservatively reject secret NUMBER arg1 in both caller contexts without declassification (local policy, **not source411 arg1 permission evidence**).

## How it works

- [Lua API architecture](../lua-api.md).
- [Client profiles](../wiki/systems/client-profiles.md).
- [NeverSecret space-limit precedent](string-util-space-limit-security.md).

## Implementation inventory

- `tests/break_up_large_numbers.rs`: 20 focused epoch-gated behavioral tests; host-VM secrets and explicit current-global locale providers, never replacing `BreakUpLargeNumbers`.
- `tests/integration.rs`: existing generated grouped integration entrypoint; parent owns dispatcher inclusion/compiled proof. No new Cargo target.
- [x] `src/lua_api/globals/real/break_up_large_numbers.rs`: implemented strict public finite NUMBER + optional public BOOL; VM secret authentication for both arguments precedes type/payload/locale access. One inferred public string, no unwrap or taint mutation. Saved corrected bounded GREEN observed; acceptance pending.
- [x] `src/lua_api/globals/register.rs::register_frame_foundation_globals`: invokes the real registrar under cumulative `retail-12-0-5`; `real/mod.rs` uses the same epoch gate. `env_init::init_lua_state` runs this before temporary bootstraps.
- [x] `src/lua_api/workarounds/temporary/formatting_utility_defaults.rs`: tostring provider compiled only without `retail-12-0-5`, not a competing enabled-epoch nil fallback. Startup/reapplication assertion expects `12,345` in the enabled epoch and retains `12345` earlier.
- [x] Current global `GetLocale` is read on every successful call through the locked VM's `LuaState::call_function` stack API (no method named `call` exists there). Getter/result remain stack-rooted until locale bytes are copied; temporary stack/frame state restores on success/error. The separate enUS fallback is installed before normal invocation. Four ASCII letters normalize generically from language+region to language_region; other identifiers pass through, with no fabricated catalog or widened locale helper.
- [x] `src/c_api/mod.rs` and `build/intl_native.rs`: existing native number backend and its shims compile from `retail-12-0-5` so the producer reuses `intl_native::format_number(..., NumberStyle::Decimal)` exactly. `C_Intl` registration/publication and Intl operations are unchanged; no new state. Existing ICU precision defaults remain the limit.

Saved corrected parent GREEN is bounded development evidence; unchecked behavioral requirements above are not acceptance claims.

## Tests asserting this spec

`tests/break_up_large_numbers.rs` contains 20 tests:

| Capability | Exact test names | Proof level |
|---|---|---|
| Public result/default, natural guess, boundaries | `startup_formatter_returns_one_public_grouped_string_with_default_agreement`, `inferred_natural_truncates_both_signs_toward_zero_not_floor`, `inferred_zero_and_group_boundaries_have_decimal_not_abbreviated_output` | First parent GREEN passed; unchanged |
| Locale mutation, read-only inputs, environment isolation | `current_wow_locale_provider_mutations_apply_immediately`, `formatter_keeps_provider_identity_and_caller_inputs_read_only`, `locale_and_input_fixtures_are_isolated_between_environments` | First parent GREEN passed; unchanged |
| Strict bad types/nonfinite and recovery | `required_public_number_rejects_missing_nil_and_wrong_types_then_recovers`, `nonfinite_numbers_reject_in_both_natural_modes_then_recover`, `optional_public_natural_rejects_nonbool_values_then_recovers` | First parent GREEN passed; unchanged |
| Authentic secret natural and GC | `authentic_secret_false_natural_rejects_even_in_secure_context`, `authentic_secret_true_natural_rejects_even_in_secure_context`, `secret_number_and_string_natural_reject_before_formatting`, `wrapped_actual_frame_and_table_natural_reject_without_representation_assumptions`, `gc_keeps_global_list_and_stack_secret_roots_identical_and_secret` | Corrected saved GREEN passed; host wrapper identity asserted |
| Taint and conservative arg1 | `secure_and_stamped_tainted_calls_keep_taint_across_rejection_and_recovery`, `secret_arg1_rejection_is_conservative_local_policy_not_row411_permission` | Historical invalid BOOL equality fixture failures retained; corrected saved GREEN passed |

Supplement53 adds four callback-boundary tests against the already-committed producer `6ed338813`; first parent GREEN observed 17 PASS / 3 FAIL, not full acceptance. Each runs with secure and stamped-tainted callers, checks unchanged original Frame/table identity and state, restores a known GetLocale provider, and issues multiple valid recovery queries. The GC provider collects inside the actual Lua callback before returning a dynamically assembled public locale; the secret-locale fixture wraps `enUS` with the actual host VM helper. Only GetLocale fixture inputs change, never BreakUpLargeNumbers. Corrected host-identity fixtures passed the saved targeted integration recheck; independent verifier443 remains pending. Unchanged lib/bootstrap, ICU controls, integration controls and startup proof remain reusable.

| Supplement capability | Exact test name | Proof level |
|---|---|---|
| Lua callback error propagation and recovery | `locale_provider_lua_error_preserves_callers_and_multiple_query_recovery` | First parent GREEN passed; unchanged |
| Missing/wrong public result and invalid UTF-8 rejection | `locale_provider_wrong_missing_and_non_utf8_results_reject_then_recover` | First parent GREEN passed; unchanged |
| Authentic secret locale rejection under explicit public input policy | `authentic_secret_locale_result_rejects_under_local_public_input_policy` | Corrected saved GREEN passed; host wrapper identity asserted |
| GC inside callback, live roots and public recovery | `locale_provider_collects_before_returning_public_locale_with_live_roots` | Historical invalid BOOL equality fixture failure retained; corrected saved GREEN passed |

Source fixture maps: `/tmp/patch-12.0.5-break-up-large-numbers-model-map.md`, `/tmp/patch-12.0.5-break-up-intl-locale-map.md`, `/tmp/patch-12.0.5-break-up-producer-fixture-boundary.md`. Requested `large-numbers-model.md` was absent; existing model-map used. The chosen inferred true behavior fills the map's previously unresolved branch; it is an assistant-selected guess under the permitted inference policy, not a user-selected native contract.

## Known gaps (current cycle)

- [x] Parent compiled RED at `77af6ad9a8723401298574b1a307796667312332`: build exit0 / 390.341632021009s (includes blocking Cargo lock; stderr reports finished381s); focused run exit101 / 10.068309725029394s, 0 PASS / 16 genuine FAIL. Startup and actual secret VM/Frame/GC fixture setup succeeded before strict-rejection/localized-decimal failures. Artifacts: `/tmp/patch-12.0.5-batch53-red-build-result.json`, `/tmp/patch-12.0.5-batch53-red-run.json` and their full outputs. Dirty-source-bound evidence includes preserved unowned duration diff SHA256 `6967f0b47312d926c2359bd29bc1c26d4d1d523522abf8d87e067104da170a1a`, not clean-revision proof.
- [x] First parent GREEN at `eb52b92c034818eb1b09017230b2c8f90f0f93a8`: build completed in 897.557s; focused run exit101 / 3.38025370601099s, 17 PASS / 3 FAIL. Failures: `locale_provider_collects_before_returning_public_locale_with_live_roots`, `secret_arg1_rejection_is_conservative_local_policy_not_row411_permission`, `secure_and_stamped_tainted_calls_keep_taint_across_rejection_and_recovery`. Artifacts: `/tmp/patch-12.0.5-batch53-green-run-0.stdout`, `/tmp/patch-12.0.5-batch53-green-runs.json`.
- [x] Main diagnosis: fixtures incorrectly demanded tainted `rawequal` success on authentic secret BOOLs. Pinned rilua `6044544` routes `lua_rawequal` through `checked_secret_equality`; `checked_secret_bool` requires a secure caller before unwrap. Exact denial: `table security operation requires an untainted caller`. `BUContexts` invokes the already-created probe; rejected closure-inheritance hypothesis has no source evidence. Existing-binary diagnostic `/tmp/patch-12.0.5-break-up-secret-bool-diagnostic.lua` and its stdout/stderr establish denial, actual secret arg2 rejection, public recovery, preserved taint and restored secure caller. NeverSecret contract remains unchanged.
Corrected fixtures assert tainted BOOL equality denial/secrecy/taint and compare host wrapper handles before execution and after Lua returns. Applied to all nine existing secret/root tests, including the three failures and secure BOOL/GC controls; no new tests or VM types. Host snapshots add no VM roots, inspect no payloads, and do not clear taint. Saved corrected focused run at `70da8fd09`: 20 PASS, exit0. Fixture-only correction strengthens host metadata wrapper identity and tainted-comparison denial without relaxing security or changing production.
- [x] Source-unchanged parent proof reused: lib/bootstrap 1 PASS, ICU 20 PASS, integration controls 69 PASS, startup zero errors (`[]`). No expensive lib rebuild needed for fixture-only correction.
- [ ] Independent verifier443 and acceptance pending (predicted424 superseded; documentation updater444). No builds/tests/checks/readability/coverage/gates run during this docs reconciliation. Source411, source accounting and unchecked behavioral requirements remain uncredited.

## Reconciled batch53 saved parent GREEN — 2026-10-02

### Provenance and failure boundary

Producer `6ed338813`; initial fixtures `533271095`, callback supplement `eb52b92c034818eb1b09017230b2c8f90f0f93a8`; correction `70da8fd09c1625ce81d9e9007a90288afed89285` changes fixtures/spec only, not production. RED revision `77af6ad9a8723401298574b1a307796667312332`: 16 genuine FAIL. First attempt remains 17 PASS / 3 FAIL, not native evidence or a producer defect. Main-behavior diagnostic confirms intentional pinned rilua `6044544` tainted secret-BOOL equality denial; agent421 closure-inheritance hypothesis rejected. Corrected root-append taint checks and host `Val`/`GcRef` plus live allocation metadata establish wrapper identity independently of BOOL payload equality. No payload inspection, extra GC roots, declassification or security relaxation.

Both GREEN builds include saved dirty-source diff SHA256 `6967f0b47312d926c2359bd29bc1c26d4d1d523522abf8d87e067104da170a1a`; these are actual combined revision-plus-dirty-scope observations, never clean revision proof. Unowned duration body was not accessed during reconciliation.

Original unified compile `cargo test --lib --test integration --no-run --message-format=json`: exit0, **897.5574265570613s**; expected auto-backgrounding completed successfully, not an aborted build. Corrected integration-only compile `cargo test --test integration --no-run --message-format=json`: exit0, **103.28732768201735s**. Compiler JSONL records were read/parsed in full, including terminal build results; build stderr and complete run outputs retained below. Compile cost is separate from runtime.

### Selected capabilities and observed proof

All runtime commands use `timeout 90 <executable> <filter> --test-threads=1`; exact argv/revision/hash records live in the named JSON artifacts. Corrected formatter replaces original failed index0; indices1–8 reuse unchanged production/control scope without reruns. This is **110 distinct selected PASS**, not 127 by counting the superseded 17 again.

| Selected scope | Count | Artifact / filter | Wall seconds | Result |
|---|---:|---|---:|---|
| Corrected formatter, including callback failure/GC and all nine identity/root fixtures | 20 | `green-fixed-run.json`: `break_up_large_numbers::` | 2.6977940219221637 | exit0 / 20 PASS |
| Reused control index1 | 1 | `green-runs.json`: `formatting_utility_defaults::tests::installs_formatting_utility_defaults` | 0.3189654710004106 | exit0 / 1 PASS |
| Reused control index2 | 20 | `green-runs.json`: `c_api::intl_native::` | 0.07439741806592792 | exit0 / 20 PASS |
| Reused control index3 | 17 | `green-runs.json`: `action_spell_slot_identifiers::` | 2.957461864920333 | exit0 / 17 PASS |
| Reused control index4 | 10 | `green-runs.json`: `c_action_bar_slot_mutation::` | 1.7992813929449767 | exit0 / 10 PASS |
| Reused control index5 | 25 | `green-runs.json`: `inventory_verbs::` | 4.320322371902876 | exit0 / 25 PASS |
| Reused control index6 | 5 | `green-runs.json`: `action_text::` | 0.9086085100425407 | exit0 / 5 PASS |
| Reused control index7 | 8 | `green-runs.json`: `string_util_space_limit_security::` | 1.457009183941409 | exit0 / 8 PASS |
| Reused control index8 | 4 | `green-runs.json`: `character_stats::stat_restriction::` | 0.8640741010894999 | exit0 / 4 PASS |

Artifact stems above mean `/tmp/patch-12.0.5-batch53-`. Original lib executable `target/debug/deps/wow_ui_sim-aea4a6c4f2f83086` SHA256 `61e6dedbb6e667f8f7671effee200d0dbae41ae9f4544a4107627cf57d23b398`; original integration executable `target/debug/deps/integration-a11e89d240f9bd0c` SHA256 `f40591e7e73256837796fddba1dd77b09fadf7530db91bfe7fdbe05bbbc1c80e`.
Corrected integration executable uses the same path, SHA256 `32595636c4eb7ded173c9b1da9ad2fae8e8926c3257e283767cb364af4095358`. RED integration SHA256 `8936a1a53e35764532e5d0531c36c146bfa4bb744bbc1a7f8fd57beda1bdc8d0`. Paths are rooted at `/syncthing/Sync/Projects/wow/wow-ui-sim/`.

Reused startup: `timeout 90 /syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/wow-sim --no-addons --no-saved-vars lua-errors`, exit0, `[]`, 5.954047672916204s; executable SHA256 `9b02eb75da1f39bd7ceaec789dba97aec7018fab2b25e28568e65748b203390c` unchanged in corrected build artifacts. Selected test runtime = corrected **2.6977940219221637s** + original indices1–8 **12.700120313907974s** = **15.397914335830137s**; with startup **21.35196200874634s**. Below60 target retained as observed, no padding, reruns or final whole-page/goal claim.

Four callback tests cover Lua error propagation, missing/wrong/non-UTF-8 results, authentic secret locale rejection and GC inside actual locale callback, under secure/stamped-tainted callers with multiple valid recovery calls. Corrected 20 PASS includes those and host wrapper identity, original Frame/table state, taint preservation and conservative arg1 rejection. Behavioral requirements remain unchecked; independent verifier443 asynchronous acceptance pending. Source411/accounting unchanged. Natural truncation, locale bytes and strict input policies remain inferred/explicit guesses; native permissions, result secrecy, exact errors, acquisition/catalog, broader precision and other profiles remain unknown or excluded.

### Saved artifact integrity

SHA256 below binds saved records and full outputs; no new runtime proof was produced. Empty stderr SHA256 is `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` for RED run, corrected run and every original index0–8 run.

| `/tmp/` artifact | SHA256 |
|---|---|
| `patch-12.0.5-batch53-green-build-result.json` | `92aea100698de55b877bab516b378cbb8e939fd9ef56917df3bbbfbac4f0f022` |
| `patch-12.0.5-batch53-green-build.jsonl` | `61c339d11f0712363676bb20c1b8e4ff6c519c4d3dd7ace643e62a8f8021b74e` |
| `patch-12.0.5-batch53-green-build.stderr` | `5a74dc4c7989c9d4e5ede425a63a09ac504a2acc1ef4e1b074872a696ee12a33` |
| `patch-12.0.5-batch53-green-fixed-build-result.json` | `e5668bf7e4164f9fb6aa68da70fe24ed0fa637e4c4862eac4045de5b1e29e7cc` |
| `patch-12.0.5-batch53-green-fixed-build.jsonl` | `5ecdadf84cfd75899c3d2212faaa1fa1de44972069c1a1085fb405140d24eeb4` |
| `patch-12.0.5-batch53-green-fixed-build.stderr` | `ac6fe8ae7faeccb2674fdeeabe13fee8678011dee5f76527019dfb2df787cf79` |
| `patch-12.0.5-batch53-green-fixed-run.json` | `9d1787387a27db656c543ddb37f1f2b024a563d6a5b2a877e51201713a5a78b9` |
| `patch-12.0.5-batch53-green-fixed-run.stdout` | `2dbb96165fce2be176d58a4e8390bf8ded789a49e333802b49159784474e420a` |
| `patch-12.0.5-batch53-green-run-0.stdout` | `6df590276d4b91dac47b18b02d1871b9207f2270d6588943059432f220a75a33` |
| `patch-12.0.5-batch53-green-run-1.stdout` | `b5b1551dd2ebc2b144f369cdfec9dd23f8a4802d4a032db9a6bd9eaccf00096b` |
| `patch-12.0.5-batch53-green-run-2.stdout` | `5897fdb0c56d96c69cb4f5afdfe3fce88f155c6295c4b24f97e5bd1c7f435c2d` |
| `patch-12.0.5-batch53-green-run-3.stdout` | `6b64e6b4a934aec2d3c04b2a3d76c5ffb549402a243a03ca366700744e8c8cbd` |
| `patch-12.0.5-batch53-green-run-4.stdout` | `fe55044015de57b0068fe769e8d9a2798451f7db3fc10dfb968e5938976a1d3e` |
| `patch-12.0.5-batch53-green-run-5.stdout` | `2026f917b000d2e5f74c5fa8ac25f172a4127166926550f0643f30ecf7fccf45` |
| `patch-12.0.5-batch53-green-run-6.stdout` | `7fbb132b78a184a99265e7d239ef0e253f29025d00535a8922737325f784fc5b` |
| `patch-12.0.5-batch53-green-run-7.stdout` | `0ca9b3409c4c9c38e943752a4e180d4044a1352c16c462f25cb69b3dbad42d00` |
| `patch-12.0.5-batch53-green-run-8.stdout` | `935226a657e081e530a20fb89e71ee4315f1689b4b9b3b4eca40e5addb84f4b6` |
| `patch-12.0.5-batch53-green-runs.json` | `62d72df59c53742b1f38c9a6d432cf68e17c631acb709c8ba10533309e42fb6f` |
| `patch-12.0.5-batch53-green-startup-run.json` | `5825fe8a4cb87164f7fa27c51f0e1fe420ff351bec9f0eff4ac2c4459b5ed755` |
| `patch-12.0.5-batch53-green-startup.stderr` | `cea90136c96d941c9c2a162fef660f4cc9f7c9175b1b96f2eabbf3d23972bab6` |
| `patch-12.0.5-batch53-green-startup.stdout` | `37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570` |
| `patch-12.0.5-batch53-red-build-result.json` | `e83286c4e0db9fb89f0fe764b5593a0e73bca27da33c0212bde3e792bf721355` |
| `patch-12.0.5-batch53-red-build.jsonl` | `05fddce329ce43f994ec21b4e6d4adf918546b650b6f7127acae0a60538e8404` |
| `patch-12.0.5-batch53-red-build.stderr` | `69bee27e90d5aff7395345da757ef00b350d2a097afd1b9d5d80d97767888d87` |
| `patch-12.0.5-batch53-red-run.json` | `b2ae79f0c019895885f7c8c27afd3065a46ee9f3d18bdb5f0a89e00345030efd` |
| `patch-12.0.5-batch53-red-run.stdout` | `c76d0e4c9733fb692ba215f634f9deacbf465e3bb8e7aa12a23ffb4c13f29e93` |
| `patch-12.0.5-break-up-secret-bool-diagnostic.lua` | `52774bbcbfaf0960e004f27b44e492912dbcaa64f792c5c38081412d99c407b4` |
| `patch-12.0.5-break-up-secret-bool-diagnostic.stdout` | `20f4ce5e27e136cc0d7caf42994c7bc66934770a4d998784f38e3a15557faeaf` |
| `patch-12.0.5-break-up-secret-bool-diagnostic.stderr` | `fd92a8bb48c72153af4c40baf92655df228fbfde2e5a44248423b853c18d248a` |

## Out of scope

- Native parity, native exact errors, native natural semantics, arg1 secret permissions and result secrecy: unknown; explicit inferences above do not resolve them.
- Precision beyond the existing ICU Decimal defaults; broad locale/catalog acquisition or new models.
- Earlier-profile behavior changes, Intl API changes, new state, and other producers.
- Source411 credit, accounting changes, broad acceptance, unavailable native probes, and other agents' files.

## Independent bounded acceptance — 2026-10-02

Parent accepted independent443 (`/tmp/patch-12.0.5-break-up-large-numbers-independent-proof.md`) plus446 format-only supplement (`/tmp/patch-12.0.5-break-up-format-followup.md`). Corrected20 formatter tests plus90 reusable controls =110 uniquePASS; startup0 `[]`. Current cargo check exit0/31.997453s without warnings; seven-file scoped rustfmt exit0/0.018397s after import-only `4fa4bd4b6`. Original443 scoped-format exit1 remains historical and only that result is superseded. Global cargo fmt still exit1/19.197842s on preserved unowned source; **all proof dirty-combined, not clean revision**.

No repeated builds/tests/startup/checks for import ordering. Original0PASS/16 genuineRED and17PASS/3 invalid tainted-BOOL equality failures retained. Corrected host wrapper identity/liveness snapshots and intentional tainted equality denial strengthen security evidence without changing VM/producer behavior. Three readability advisories deferred: argument-handler length, locale callback/restoration length and host-snapshot loop; no observed behavioral counterexample or authorized adjacent refactor.

Only411 and [exact-eight annotation rows](unit-stat-output-restriction.md#exact-eight-annotation-acceptance--2026-10-02) promote: **217 pending/131 bounded/14 partial =362 ordered IDs;62 capabilities**.353 unrelated rows and60 prior capabilities preserved; parent owns postcommit validation. Natural truncation remains an assistant-selected guess; ICU defaults, strict inputs/public locale/result policy and conservative arg1 rejection are simulator behavior, not native permission/precision/error/natural/locale-acquisition or all-profile parity. Whole-page/goal work remains open.
