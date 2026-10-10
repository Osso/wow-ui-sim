# Existing-artifact audit

Recorded revision: `2d6d314bd7f5a94d5cf6527a376b75f6d9eff21d`. Overall four-case result: **FAIL — 3 passed, 1 failed**.

Compiler receipt: exit 0; argv/CWD corroborated; 739 JSON records, 0 parse errors. Build-finished records: `[{"reason": "build-finished", "success": true}]`. Six manifest deprecation warnings; not warning-free.

Source-before/source-after: 3839 entries each; parsed and byte equality: True/True. Individual git-show comparison: 10/10 listed text files match recorded revision. **Not a full-commit comparison**: remaining 3829 snapshot entries were not compared to commit. No current-HEAD claim.

| Case | Exact selector | Selected | Passed / failed | Exit |
|---|---|---:|---:|---:|
| 1 | `spell_api::test_spell_get_spell_charges` | 1 | 1 / 0 | 0 |
| 2 | `c_spell_static_fallbacks::test_spell_override_maw_epoch_and_explicit_charge_state` | 1 | 1 / 0 | 0 |
| 3 | `lua_api::workarounds::temporary::debug_environment_defaults::tests::installs_debug_environment_defaults` | 1 | 1 / 0 | 0 |
| 4 | `tooltip_text_layout::test_tooltip_layout_is_clamped_to_viewport_edges` | 1 | 0 / 1 | 101 |

## Exact wrapped-tooltip failure

Recorded source explicitly passes `true` as AddLine wrap argument at `tests/tooltip_text_layout.rs:162`. Case 4 failed the existing right-edge assertion, not an unwrapped control. No fixture/assertion changes were made.

```text
thread 'tooltip_text_layout::test_tooltip_layout_is_clamped_to_viewport_edges' (518635) panicked at /home/osso/Projects/wow/wow-ui-sim/tests/tooltip_text_layout.rs:191:5:
Tooltip right edge is outside viewport: rect=LayoutRect { x: 0.0, y: 0.0, width: 416.7, height: 58.0 }, screen=400x300, right=416.7
```

Actual right edge 416.7 exceeds viewport width 400 (assertion tolerance 0.1). This proves the wrapped fixture failed its viewport-bound condition; it does not establish the underlying cause. Bottom-edge assertion was not reached.

## Integrity and limits

All 34 epoch input files hashed before/after audit: unchanged = True. Compiler records corroborate both saved executable records. Per-case stdout/stderr hashes, summaries, aggregate receipts, selectors and run artifact hashes are checked in `proof.json`.

- `lib` recorded SHA256 `6d2e6db638d72595b7a6f6cdc29155dbe406b3b07430e1622c973e7c064a6b37`; audit before/after `6d2e6db638d72595b7a6f6cdc29155dbe406b3b07430e1622c973e7c064a6b37` / `6d2e6db638d72595b7a6f6cdc29155dbe406b3b07430e1622c973e7c064a6b37`; matches recorded = True.
- `integration` recorded SHA256 `2824b6994960d09eb4a7b8990b8987b8b9fe21ed9cc804672bfb042a389a4dc2`; audit before/after `2824b6994960d09eb4a7b8990b8987b8b9fe21ed9cc804672bfb042a389a4dc2` / `2824b6994960d09eb4a7b8990b8987b8b9fe21ed9cc804672bfb042a389a4dc2`; matches recorded = True.

Historical receipts record identical artifact hashes before/after each case; audit-time executable hashing independently checks current bytes, not past execution. Compiler argv includes `--no-run`; case argv invokes compiled simulator test binaries with `--exact`. Neither constitutes native WoW-client proof.

Excluded: external path dependencies, uncaptured data/runtime caches/addons, inherited environment, untracked index. No full suite/all-profile/current-HEAD/native-client PASS claim.

Full saved streams were read locally; report exposes only selected test/diagnostic evidence. No commands from receipts were rerun. No builds, checks, network, operations, delegation or repository edits.
