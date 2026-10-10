# Compiler epoch and six-case audit

Audit time (UTC): 2026-10-10T00:28:51.545768+00:00. Existing artifacts only; no test executable runs, builds, tests, checks, network, delegation, or repository edits. Reports written only to this epoch's `audit/`.

## Compiler: PASS, bounded provenance

Source commit: `40d2beb9ad719032b4d345455e5410f2971ded7a`. Recorded cargo exit **0**, outcome exit **0**; all **739** stdout lines parse as compiler JSON; terminal `build-finished.success=true`. Before/after source snapshots are equal across **3839** entries. Eleven scoped build/test/wiring files independently match commit-content SHA256 values; full commit-content equality across all 3839 entries is not claimed. Six manifest deprecation warnings remain; warning-free build is not claimed.

Command retained: `/usr/bin/cargo test --offline --locked --lib --test integration --no-run --message-format=json`. No-run compilation completed 2026-10-09T23:38:01.177239+00:00; runtime cases below ran October 10, 2026 UTC, not at compilation time.

Both test artifacts have identical compiler JSON feature sets: `aura-containers, aura-instance-enumeration, aura-xml-widgets, base-spell-relationships, casc, client-retail, default, forbidden-aspects, gui, native-duration-formatting, numeric-rule-formatters, on-update-modes, player-cast-durations, profile-retail, retail-12-0-0, retail-12-0-5, retail-12-0-7, retail-12-1-0, rodio, sound`. In particular `client-retail` and `retail-12-1-0` are enabled; `client-wowforever` is absent. The new Retail cooldown case is eligible; its other-profile alternative is not selected.

## Runnable artifacts and hashes

- **wow_ui_sim**: `/home/osso/Projects/wow/wow-ui-sim/target/debug/deps/wow_ui_sim-bcad23723a7b3b0d`
  - SHA256 `43d0063f52e052553ca42b83d1270adc770d96e83821316f5d0e4caf6fe7110e`; ELF and executable permission verified; compiler `fresh=false`, test profile.
- **integration**: `/home/osso/Projects/wow/wow-ui-sim/target/debug/deps/integration-1db16289b0998b2a`
  - SHA256 `1b9ecf472c4456118cb63e874c77a3dadef69228703644d71ef6a53592a1c652`; ELF and executable permission verified; compiler `fresh=false`, test profile.

Current binary hashes match each run's retained before/after hashes. Compiler JSON provides paths but no sealed compile-time hashes; this is a bounded local-receipt chain, not cryptographic proof of compiler provenance. `proof.json` pins compiler streams, snapshots, receipts, selector streams, runtime stdout/stderr, and binaries by path, byte count, and SHA256.

## Exact changed-case coverage

| Case | Exact selector | Change commit | Eligible in epoch | Retained result |
|---|---|---|---|---|
| case1 | `edit_mode_api::enums::unit_frame_edit_mode_setting_meta_includes_big_defensive_icon_size` | `21b513f3e` | YES | 1 pass / 0 fail; exit 0 |
| case2 | `patch_12_0_7_b23_b28::b28_registered_absent_file_and_unregistered_present_file_are_not_io_queries` | `21b513f3e` | YES | 1 pass / 0 fail; exit 0 |
| case3 | `toplevel_render_groups::native_controls_keep_unraised_and_raised_groups_in_owner_strata` | `21b513f3e` | YES | 1 pass / 0 fail; exit 0 |
| case4 | `toplevel_render_groups::screen_roots_do_not_capture_independent_render_groups` | `21b513f3e` | YES | 1 pass / 0 fail; exit 0 |
| case5 | `lua_api::workarounds::temporary::debug_environment_defaults::tests::installs_debug_environment_defaults` | `00a838a2e` | YES | 0 pass / 1 fail; exit 101 |
| case6 | `wowforever_cooldown_categories::forever_cooldown_categories_preserve_retail_12_1_0_epoch` | `40d2beb9a` | YES | 1 pass / 0 fail; exit 0 |

All six retained selectors list exactly one test, selector exit 0, empty selector stderr; all six executions run exactly one test. Runtime stream hashes recompute equal to outcome receipts, summary lines agree, and exit codes agree with counts. **5 passed, 1 failed**. Integrity PASS does not convert case5 into a test PASS. Cases1–4 cover the four requested 21b fixture checks, not every test sharing those helpers. Generated integration module wiring was observed; the retained executable selector lists independently establish compiled reachability.

## Marker diagnostic: FAIL retained, boundary observed

Case5 exit **101**, 0 passed / 1 failed. At `probe_start`, marker is nil and secretwrap a function. At `after_secure_delegate`, `after_button_metatable`, and `after_editbox_metatable`, both are functions with stable identities across those three phases. No `after_secretwrap` phase is present. Retained stderr reports `attempt to call a userdata value`; panic location: `src/lua_api/workarounds/temporary/debug_environment_defaults.rs:263:14` in compiled source. Source probe next evaluates `secretwrap(marker)()`. Evidence locates failure at that expression; it does not prove which nested call/interpreter operation produced the userdata failure. No repair or root-cause completion claimed. Raw identity addresses and payloads are not copied.

## Exclusions and current source

The later `0769f7b996e538adf776eaba86fc5ae0dcdfa494` explicit-charge fixture (`spell_api::test_spell_get_spell_charges`) is **not compiled in this epoch**. Compiled file SHA256 `405f7564d9068ea502e3c05b992cebd983c0a845e7f2d7922b3122fa118dc6e1` differs from later source SHA256 `d281fd3909e89dc70830d2cfc644aaed478ea377b8ca39c0cc5f66c6d6de9fc0`. Observed HEAD earlier in audit was 0769; current marker source also differs from epoch snapshot. This epoch is **not blanket current-HEAD proof**.

No previous-run recertification; only this epoch's case1..case6 receipts audited. No full-suite, other-profile, or real-WoW/native-client parity claim. External path dependencies, inherited environment, runtime cache/addons and untracked index are excluded by submission. Files are local unsigned receipts; compiler/metadata integrity hashes were established now, not compared with a previously signed baseline.

## Audit tooling incident

A read-only `git cat-file --batch` content comparison unexpectedly auto-backgrounded while the Pyrun argv helper was handling batch input/output; its reported log was absent. Exact blocking cause is unproven; full-source commit comparison remains incomplete. Own git child PID 410567 was cancelled and process absence verified. No batch evidence retained, no rerun. Eleven scoped `git show` reads supplied the successful content comparisons. No source or operational state changed, apart from cancelling the audit's own stuck process.
