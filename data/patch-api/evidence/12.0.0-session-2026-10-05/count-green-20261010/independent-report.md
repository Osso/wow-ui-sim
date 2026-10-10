# Final count receipts audit

Date: 2026-10-10. **OVERALL: PASS for the two specified retained receipt scopes. FAIL: none. PENDING: none within those scopes.** Not a warning-free result.

Read `/home/osso/AgentConfig/skills/verify/SKILL.md`; performed verifier-role artifact audit directly, without delegation. No builds, tests, checks, process/service operations, source edits, or commits. Only this requested report created. Read complete compiler JSON streams, selected stdout/stderr, check stdout/stderr, receipts, and source manifests; independently parsed counts and streamed SHA-256 over sealed executables and current manifest-listed files. No executable run.

## Evidence roots and historical boundary

- **A:** `/home/osso/.local/state/wow-ui-sim/verification/retail-1200-count-green-current/20261010T170526Z`
- **B:** `/home/osso/.local/state/wow-ui-sim/verification/count125-controls-current/20261010T170731Z`

Prior source audit: `/home/osso/.local/state/wow-ui-sim/verification/retail-1200-count-green-current/independent-report.md`. Prior runtime snapshot: [count-runtime-gate-resumed.md](count-runtime-gate-resumed.md). Their historical PENDING findings are superseded only for A/B's completed, specified receipts. Earlier resource-blocked epoch and preserved RED remain historical evidence, not passed executions.

## Actual execution coverage

Every row independently agrees across `execution-results.json`, named test lines, `running N tests`, and terminal `test result: ok` output. Every process exit is 0; each receipt records `artifact_unchanged: true`. All runs used `/usr/bin/timeout 90`, sealed executable, `--nocapture --test-threads=1`.

| Scope / stdout | Exact selector | Actual passed | Failed / ignored / measured | Filtered out | Seconds | Verdict |
| --- | --- | ---: | --- | ---: | ---: | --- |
| A / prose.stdout | `patch_12_0_0_prose::` | 3 | 0 / 0 / 0 | 7818 | 0.46 | PASS |
| A / early-policy.stdout | `spell_cast_count_early_policy::` | 1 | 0 / 0 / 0 | 7820 | 0.15 | PASS |
| A / bare-retirement.stdout | `patch_12_0_0_deprecated::patch_12_0_0_deprecated_native_retirement` | 1 | 0 / 0 / 0 | 7820 | 0.12 | PASS |
| B / spell-counts.stdout | `spell_count_outputs::` | 30 | 0 / 0 / 0 | 10670 | 2.63 | PASS |
| B / max-applications.stdout | `spell_max_cumulative_aura_applications::` | 8 | 0 / 0 / 0 | 10692 | 0.76 | PASS |
| B / metadata-defaults.stdout | `spell_metadata_defaults::tests::` | 2 | 0 / 0 / 0 | 1995 | 0.21 | PASS |

**A: 5/5. B: 40/40 (38 integration tests + 2 library units). Total: 45/45 selected tests, zero failures.** Not whole-binary or whole-suite counts.

A's three named prose tests are callback-event mechanism, combat-log registration, and live explicit spell-cast inputs. Its policy case is `supplied_cast_count_stays_ordinary_when_cooldowns_are_restricted`. Its retirement case is one bare-environment test asserting absence for 21 data rows, not 21 Rust tests.

**Separate scope:** `tests/patch_12_0_0_deprecated.rs` retains the `prefork_full_ui_case!` case `patch_12_0_0_deprecated_retirement_and_successors`, which checks cached absence/loaded aliases and successor probes. Neither selected receipt credits that original cached21 acceptance case. Its verification remains PENDING / outside this audit; no cached21 completion claimed. The word `native` in the bare test's name means the simulator's bare API surface, not evidence from native WoW.

## Compilation and checks

| Receipt | Command / result | Verdict |
| --- | --- | --- |
| A / compile-result.json | `cargo test --no-run --test integration --offline --locked --no-default-features --features profile-retail,retail-12-0-0 --message-format=json --timings -j 12`; exit 0, source_equal true; compiler `build-finished.success: true` | PASS |
| B / compile-result.json + cargo-result.json | `cargo test --no-run --lib --test integration --offline --locked --message-format=json --timings -j 12`; both exits 0, source_equal true, stream_errors empty; compiler `build-finished.success: true` | PASS |
| A / checks/fmt.json | `cargo fmt --check`; exit 0, source_equal true; stdout/stderr empty | PASS |
| A / checks/exact-check.json | `cargo check --offline --locked --no-default-features --features profile-retail,retail-12-0-0 -j 12`; exit 0, source_equal true; terminal Finished dev profile, 37.47s | PASS with warnings |
| B / checks/fmt.json | `cargo fmt --check`; exit 0, source_equal true; stdout/stderr empty | PASS |
| B / checks/default-check.json | `cargo check --offline --locked -j 12`; exit 0, source_equal true; terminal Finished dev profile, 13.05s | PASS with warnings |

Receipt commands use `/usr/bin/cargo`; submissions name canonical cwd `/home/osso/Projects/wow/wow-ui-sim`. Submission/check revision is `095081101f04254270dee6805ac6b8cbb8e9ea3b`, matching independently read current HEAD.

## Source equality and current relevant bytes

**PASS:** A `source-before.json` equals `source-after.json`, 23,691 entries; independently hashed current files match all 23,691, zero missing/mismatched. B before equals after, 3,853 entries; independently hashed current files match all 3,853, zero missing/mismatched. Compile, outcome, and check receipts also record source_equal true. Scope is the manifests, not every possible build/runtime input.

**PASS:** Independently inspected Git diff `84dcf2723..095081101`: only retained RED evidence under `data/patch-api/evidence/12.0.0-session-2026-10-05/count-red-20261010/` and six documentation paths changed. No implementation, tests, Cargo manifests/lock/config, or build script changed. Thus the completed receipts cover the relevant implementation bytes audited at `84dcf27231db5f06d8bf39cb6ba2ddaaf91fdc7c`, with later documentation/evidence commit `095081101`.

Current Git status observed `.code-index.db` and `verification/` untracked; neither treated as captured source provenance. No tracked changes observed.

## Sealed hashes and profile flags

**PASS:** Independently streamed current sealed files; all three match recorded SHA-256. Each has exactly one matching executable compiler-artifact record, whose feature array exactly equals its artifact receipt.

| Sealed executable | SHA-256 |
| --- | --- |
| A / integration-sealed | `29f97173f44e45618b0309178c8f613f6820243dab4f7faa0ed7c7da3f146edf` |
| B / integration-sealed | `b7cbaecd7918e79f0096bfe182368298748ff56ef58556ea99607a23112285a0` |
| B / wow_ui_sim-sealed | `c0bfe59abae51c726fe75287610a09987e07cd7cfcd0eb228583f5534c9df29e` |

A features exactly: `aura-instance-enumeration`, `profile-retail`, `retail-12-0-0`. No `client-retail`, GUI, default, or 125 feature in this executable.

B features, identical for integration and library test executable: `aura-containers`, `aura-instance-enumeration`, `aura-xml-widgets`, `base-spell-relationships`, `casc`, `client-retail`, `default`, `forbidden-aspects`, `gui`, `native-duration-formatting`, `numeric-rule-formatters`, `on-update-modes`, `player-cast-durations`, `prefork-full-ui`, `profile-retail`, `retail-12-0-0`, `retail-12-0-5`, `retail-12-0-7`, `retail-12-1-0`, `rodio`, `sound`. B is current default retail with 125 enabled, not an isolated exact125 build.

All matching compiler-artifact records have test=true, opt_level=1, debuginfo=line-tables-only, debug_assertions=true, overflow_checks=true. Hash association is retained artifact consistency, not hermetic source-to-artifact provenance.

## Warnings

**Warning-free claim: FAIL.** Both compile stderr files and both cargo-check stderr files retain six `iced-wgpu-patched/Cargo.toml` deprecated hyphenated Clippy lint names: large-enum-variant, map-entry, match-wildcard-for-single-variants, redundant-closure-for-method-calls, trivially-copy-pass-by-ref, type-complexity. Each stream also has the manifest's six-warning summary; do not count that summary as another diagnostic.

A compiler JSON retains **10 project warning diagnostics**; A exact-check stderr retains the same nine library warnings plus one binary warning:

| Location | Warning |
| --- | --- |
| `src/c_api/item_spell/mod.rs:20` | unused import `c_item::location_item` |
| `src/blizzard_ui_sync.rs:28,180,663` | unused PROVENANCE_SCHEMA, CacheProvenance::new, remove_missing_marker (three diagnostics) |
| `src/c_api/c_transmog_collection.rs:246`; `src/casc_asset_fallback.rs:93` | unused read_custom_set_items; ensure_known_asset_cached |
| `src/render/font.rs:44`; `src/lua_api/globals/auras.rs:251` | unread encoding_key_hex; unused aura_matches_filter_string |
| `src/widget/frame.rs:601,610`; `src/bin/wow_sim/main.rs:20` | unused two cooldown methods (one diagnostic); unused SavedVariablesManager import |

B compiler JSON has zero compiler-message diagnostics; B default-check stderr has only the six manifest deprecations. Selected runtime stderr contains startup/timing output, no warning/error/panic/FAILED diagnostic matches. Success exits are not warning-free proof; no remediation performed.

## Exclusions

No native WoW parity, all-profile, full-suite, cached/prefork21 acceptance, hermetic build provenance, external dependency provenance, inherited-environment isolation, or uncaptured runtime assets claim. A excludes GUI-gated source. Manifest/current-byte equality and recorded source_equal do not establish those excluded properties. Original reports and receipts unchanged.
