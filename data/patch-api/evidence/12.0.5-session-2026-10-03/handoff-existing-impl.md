# Existing implementation audit — Retail 12.0.5

Scope: prose-2026-03-25-094, prose-2026-03-25-115, prose-2026-03-31-161. Repo read-only; no cargo, git mutations, agents, model CLIs, or production changes. Findings below are incremental; execution evidence will be appended.

## Initial findings

- 094: real shallow table.freeze/table.isfrozen implementation, not placeholder. Existing library tests cover read-only behavior; inspect integration addon boundary before proposing duplicate tests.
- 115 / 161: same formatter-support sentence (planned / added). Real private rooted attachment, trusted typed dispatch, engine tick and renderer text consumption exist. Public configuration tests alone do not prove countdown rendering. Existing library renderer tests cover that boundary; audit them and existing ledger scope before proposing credit.

Initial inspection only; final decisions and proof follow.

## Final decisions

| Row | Classification | Proposed accounting |
|---|---|---|
| `prose-2026-03-25-094` | **(c)** Historical-epoch publication incomplete; existing behavior is real. One additional authenticated-addon regression staged. | New `table-freeze-addon-read-only`; hold strict 12.0.5 credit until registration correction and historical-epoch execution. Existing current-retail behavior qualifies only with an explicit later-epoch limitation. |
| `prose-2026-03-25-115` | **(a)** Existing attachment and actual countdown-consumer tests prove bounded support. | New `cooldown-countdown-formatter`, linked to both formatter prose rows; no duplicate implementation/tests. |
| `prose-2026-03-31-161` | **(a)** Same support statement, now described as added; same tests. | Same capability/evidence, not a second implementation or doubled test count. |

Exact source sentences (`data/patch-api/sources/12.0.5-api-changes.txt`):

- **94:** “Addons can now use the table.freeze/table.isfrozen APIs to make their own tables read-only.”
- **115:** “We are adding numeric formatter support for Cooldown frame fontstrings.”
- **161:** “Added numeric formatter support for Cooldown frame fontstrings.”

All three source rows currently have empty `capabilities`, `audit-pending`, and “behavioral applicability audit not completed.” No coverage JSON was modified.

## 094 — real implementation, wrong earliest epoch

Producer: `src/lua_api/globals/real/table_freeze.rs:11–45` installs both methods, marks only the supplied table read-only, queries actual VM state, and returns zero results before 12.1.5 / one identical table at 12.1.5. `ensure_mutable` guards table insert/remove/sort before mutation. Module and helper declarations are available without a 12.1.0 module gate.

**Root cause:** `src/lua_api/globals/register.rs:145–146` installs this producer only under `retail-12-1-0`. `Cargo.toml:119–122` makes that feature depend on 12.0.5, not vice versa. Default `client-retail` enables 12.1.0 (`Cargo.toml:149`), hiding the gap. A strict `profile-retail,retail-12-0-5` build does not install it. Source scan finds no alternate simulator publication; pinned rilua standard library has no freeze/isfrozen registration. This epoch failure is established by source/feature inspection, not an executed historical build.

Existing exact behavior assertions:

- `src/loader/tests/wow_api_globals/patch_12_1_5_table_freeze.rs:4`, `table_freeze_profile_returns_and_reads`: publication, `isfrozen` false→true, one query result, epoch-dependent freeze arity/idempotence, `t[1]==42`, label unchanged, `#t==1`, concat `42`, pairs count2.
- Same file `:33`, `table_freeze_rejects_writes_and_mutators`: assignment, new-key assignment, rawset, insert/remove/sort, wipe/tinsert/tremove must fail; array remains exactly `{3,1,2}` after each denial. Mutable controls still work.
- Same file `:71`, `table_freeze_only_marks_root_not_children_keys_or_metatables`: root frozen; child/key/metatable/inherited table mutable; child value8 and inherited value6 remain readable after full GC.
- Same file `:101`, `table_freeze_rejects_non_table_arguments`: missing/nil/non-table inputs fail, with table-expected errors for sampled values. These are validation controls, not proof of the complete prose sentence.
- `tests/table_util.rs:11`, `frozen_addon_namespace_does_not_freeze_callback_state_or_later_addons`: loads addon A, freezes its real namespace, loads addon B, dispatches both ADDON_LOADED events, checks exact event order, pending cleanup, callback result `child-alive`, later global `loaded-after-freeze`, string.match results and post-GC child/closure survival; no recorded Lua errors. Fresh prebuilt execution: **1 PASS**. It does not itself assert mutation denial or `isfrozen`; library assertions supply those axes. Library tests are enabled only for client-retail/client-ptr, so they do not independently test the strict historical profile.

### Smallest complete producer correction

Anchor: immediately after `table_extensions::register_all` and before `string_extensions::register_all`, replace only this two-line region:

```rust
    #[cfg(feature = "retail-12-1-0")]
    super::real::table_freeze::register_all(lua)?;
```

with:

```rust
    #[cfg(feature = "retail-12-0-5")]
    super::real::table_freeze::register_all(lua)?;
```

Full change staged as `staging/existing-impl/src/lua_api/globals/register.rs.patch`. No producer rewrite, new VM machinery, fallback, PTR return-policy change, or numeric-rule feature change needed. Future integration should update `docs/specs/table-freeze.md` earliest publication requirement from 12.1.0 to 12.0.5 in the same change; not staged here because repository documentation remains read-only.

Additional test staged as `staging/existing-impl/tests/retail_12_0_5_addon_table_freeze.rs` (one auto-included module through `build.rs:55–94`). First line is exactly `#![cfg(feature = "retail-12-0-5")]`. Test `retail_12_0_5_addon_can_freeze_its_own_table_without_freezing_children` uses an actual VM-tainted addon closure, not a no-op shim: checks publication, false→true state, six atomic mutation denials, unchanged concrete array/child identity, readable concat, mutable child, GC survival, preserved addon taint and restored outer security. Distinct missing axis: read-only operations under authenticated addon execution in the 12.0.5 feature-gated integration module.

Expected historical RED: first publication assertion. Expected GREEN after one-line producer correction. Neither was run: cargo/builds forbidden. Rustfmt parsed/formatted the staged file successfully; compile/execution remain explicitly unverified. Rust API use is the existing `WowLuaEnv::new`/`exec` pattern; no unsupported eval result type or raw-string delimiter collision. Do not accept this staged test merely because it is present.

## 115 / 161 — reuse exact existing formatter tests

Public producer: `cooldown.rs:25–57` validates typed formatter before changing attachment, retains it in private GC roots, and exposes setter/getter. Tick producer: `cooldown/countdown_formatter.rs:13–126` rereads each live root, computes remaining seconds from simulator monotonic time/modRate, invokes trusted shared numeric formatting, then stores actual formatted text. `src/lua_api/on_update.rs:68` calls that producer after frame update handlers. `src/iced_app/quad_builders_cooldown.rs:126–181` consumes the configured text through the actual countdown fontstring glyph-emission path. Not a placeholder.

Existing public integration assertions (`tests/cooldown_countdown_formatter.rs`):

| Test | Exact observable assertions |
|---|---|
| `countdown_formatter_retains_configured_abbreviated_object_identity` (:30) | Retained handle is original; 1234→`1.2 charges`; clear breakpoints changes it to `1234`; another Cooldown remains nil. |
| `countdown_formatter_retains_configured_numeric_rule_object_identity` (:52) | Original handle; 1234→`1234 ticks`; live rule mutation→`1234.0 turns`. Feature-gated subtype, not strict 12.0.5 proof. |
| `countdown_formatter_retains_configured_native_seconds_object_identity` (:68) | Original handle; 1234→`20m 34s`; desired-unit mutation→`20m`. |
| `countdown_formatter_defaults_to_one_nil_result` (:18) / `clearing_countdown_formatter_resets_attachment_without_mutating_objects` (:85) | Default/cleared getter has one nil; replace/reattach retains identity; first and second formatter outputs unchanged; thresholds90/5 preserved. |
| `invalid_countdown_formatter_inputs_leave_existing_attachment_unchanged` (:112), `untainted_countdown_formatter_setter_accepts_typed_secret_object_and_nil` (:143), `tainted_countdown_formatter_setter_rejects_secrets_atomically_but_accepts_plain_values` (:166) | Invalid/tainted-secret inputs fail atomically; impostor callbacks never run; untainted typed secret attachment succeeds; plain addon writes work and retain caller taint. |

Those eight tests freshly passed in the allowed prebuilt binary. **They prove configuration, not countdown rendering.** Renderer proof comes from `src/iced_app/quad_builders_cooldown/countdown_formatter_tests.rs`, whose `tick_at` changes simulator clock, executes actual `fire_on_update`, and calls the same library countdown text consumer used by glyph emission:

| Test | Exact countdown-consumer assertions |
|---|---|
| `configured_numeric_rule_renderer_ticks_mod_rate_and_live_rules` (:231) | Cooldown10/modRate2 at1.1→`8 ticks`, at2.1→`6 ticks`; OnUpdate changes rules→`6 turns`. Optional numeric-rule subtype. |
| `configured_abbreviated_renderer_uses_live_breakpoints` (:255) | Remaining1234.25→`1.2 charges`; breakpoint mutation→`1.2 stacks`. |
| `configured_native_seconds_renderer_ticks_and_live_unit_configuration` (:281) | Tick0.25→`20m 34s`; tick60.25→`19m 34s`; live desired-unit change→`19m`. |
| `configured_renderer_clear_restores_existing_default_thresholds` (:298) | Custom `8s`; clear→default `9` and later `2.4`; reattach→`8s`. |
| `configured_renderer_preserves_hide_minimum_and_expiry_gates` (:320) | Active `8s`; hide/minimum suppress; restore→`8s`; expiry/Clear suppress. |
| `configured_renderer_roots_handle_after_caller_release_and_collection` (:345) | Caller drops handle and collects; getter and countdown yield `20m 34s`; mutate/collect again→`20m`. |
| `configured_renderer_curve_replaces_another_attachment_during_collection` (:186) | Collecting curve replaces peer; peer produces `20m 34s`, Lua error list empty. |
| `configured_renderer_ignores_public_frame_formatter_callbacks_for_secret_timing` (:365) | Curve receives actual secret timing while tainted and cannot unwrap; engine produces `8s`; impostor callbacks0; child GetText remains nil; reading callback observation retains taint. |

Saved independently inspected batch36 run selects **20 PASS**, including these eight configured cases and threshold controls. Available provenance: `/tmp/patch-12.0.5-batch36-green-runs.json`, `...green-run-0.log`, and `/tmp/patch-12.0.5-cooldown-abbreviation-independent-proof.md`; compiled revision `053d7ed4d35743002646593d139a5e36f2e189d3`, library executable SHA256 `49fc123f065a0512af91c9f040ccbf08f76bca656edc9e9d89d3affaa5c1f985`. Current renderer, renderer tests, frame gates, cooldown methods, formatter handoff and Cargo.lock SHA256 match saved source identity. Cargo.toml differs; saved evidence is source-scoped historical proof, not current complete-build acceptance. The separate batch8 proof link in the spec is absent on this host; no reliance on that unavailable file.

Existing capability `duration-common-formatters` covers DurationObject methods only. Existing `cooldown-numeric-unit-consumer` covers three numeric-unit/threshold APIs only, despite referencing the same renderer test file. Neither scope already credits formatter attachment. Propose **new `cooldown-countdown-formatter` capability** with setter/getter, engine tick→configured countdown text, live changes/GC/clear and secret-preservation scope, linked to both prose115 and prose161. No duplicate tests needed.

Boundaries: child FontString GetText is intentionally not populated with configured plaintext; tested support is actual renderer text selection, not child Lua text consistency. No GPU rasterization, native-client parity, whole-page completion, arbitrary Lua formatter support, or all-profile claim. `NumericFormatter` means the shared accepted formatter contract, not exclusively the later feature-gated NumericRuleFormatter subtype; do not enable that later subtype in 12.0.5 just to count its test.

## Proof ledger / merge risk

| Invocation / evidence | Scope and result | Validity |
|---|---|---|
| `target/debug/deps/integration-8ea324359263a4d2 cooldown_countdown_formatter:: --test-threads=1` | 8 selected, 8 PASS, exit0, 3.33s; stderr empty | Executed prebuilt artifact only; compiled source revision unknown. |
| Same binary, filter `table_util::frozen_addon_namespace` | 1 selected, 1 PASS, exit0, 1.25s; stderr empty | Current/default artifact only; not historical 12.0.5 proof. |
| Saved batch36 renderer log and metadata, inspected without rerunning | 20 selected, 20 PASS; source identity matches relevant producers/tests | Historical compiled revision above; Cargo.toml changed since then. |
| Direct `rustfmt --edition 2024 <staged-test>` | exit0, no diagnostics | Staged Rust syntax/formatting only, not typechecking or Lua execution. |

Concurrent rebuild replaced the allowed binary: SHA256 before both fresh runs `b102a4e2c80d36c5efa6bc4a5b0c94d16cd7a3850f9f056a9f214f739581b390`, later `458a466d028a9dc3e0a5122c4ed3fa49e76cddd70425ca95500097c604ccab7a`. Both invocations started and completed successfully; no retry or redundant rerun. Do not bind either run to a specific current commit/hash. Manual changed-Rust readability audit found no issues in the staged test/one-line gate correction.

Risk of integrating now: formatter accounting is bounded and backed by real existing behavior; strict historical/current-build acceptance remains unexecuted. Table-freeze correction/test is staged but uncompiled/unexecuted; credit before RED/GREEN would overstate proof. A test over a placeholder earns nothing—none of these producers is a placeholder, but an unavailable epoch or unexecuted staged test still earns no execution credit.

No repository files or coverage statuses changed. Final staged artifacts: one integration test and one minimal producer patch, both under `staging/existing-impl/`.
