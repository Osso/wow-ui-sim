# Exact startup migration independent proof — final bounded result

## PASS on repaired revision `3de87465828db7cc7f6f900d4b63e24f1b399825`

Requested migrations `29e48f225` / `911c980fc` initially failed to compile. Caller then explicitly authorized reproof after `3de874658` replaced the macro emitter's absolute `$crate::` invocation with its imported unqualified scope. Both original RED streams remain unchanged; detailed original failure and preservation inspection are in `report-red.md`.

Canonical cwd: `/home/osso/Projects/wow/wow-ui-sim`. Initial HEAD `911c980fc0eceece074b8ce0c3167a49381fc57c`; concurrent SOURCE merge observed at `e119ffa3256a71e0c86f9727d1c108769e29ac93`; repaired proof began and ended at `3de87465828db7cc7f6f900d4b63e24f1b399825`. Recorded runtime scope hashes match before/after the repaired proof. The unrelated client-era-only Cargo test addition does not affect default Retail target selection. No fixture/body/gate changes accompanied repair.

## Runtime capability matrix

| Exact stable case | Actual postfork result | Original body preservation | Prefork full list count | Retail integration count |
|---|---|---|---|---|
| `chat_frame::test_chat_editbox_click_type_and_submit` | PASS | Exact text match | 1 | 0 |
| `chat_frame::test_chat_editbox_text_color_after_activation` | PASS | Exact text match | 1 | 0 |
| `spell_casting::cast_bar_respects_edit_mode_lock_setting_after_startup_fix` | PASS | Exact text match | 1 | 0 |
| `blizzard_player_spells_loads::mainline_spellbook_keybind_opens_and_closes_without_runtime_errors` | PASS | Exact text match | 1 | 0 |

**4/4 original assertion bodies pass**, through three fresh controller/fixture groups in the existing target. Each public target invocation runs all group dispatch paths; only the selected group has nonzero tests and initializes its parent. Chat listing established exactly the two intended names before selecting the `chat_frame::test_chat_editbox_` substring. Cast-bar and spellbook used separate exact-name invocations. No normalization, state reset or weakened assertions introduced by verification.

Direct captured stdout:

```text
test chat_frame::test_chat_editbox_click_type_and_submit ... ok
test chat_frame::test_chat_editbox_text_color_after_activation ... ok
test result: ok. 2 passed; 0 failed; 2 total

test spell_casting::cast_bar_respects_edit_mode_lock_setting_after_startup_fix ... ok
test result: ok. 1 passed; 0 failed; 1 total

test blizzard_player_spells_loads::mainline_spellbook_keybind_opens_and_closes_without_runtime_errors ... ok
test result: ok. 1 passed; 0 failed; 1 total
```

Conformance: **21/21 pass**, exit 0, including `conformance::exact_fixture_groups_list_each_case_once`, timeout/tree cleanup, multithreaded-fork rejection, bounded workers, read-only bytecode-cache child and parent-bypass cache. Expected deliberate timeout/rejection/panic fixtures are covered by successful conformance; no unexpected case timeout, thread/cache setup error or candidate failure appears in full candidate stdout/stderr. Final process snapshot contains no prefork executable processes or prefork zombies. Successful controller exits plus explicit case output and inspected fork runner wiring prove case execution occurred after fork, not just compile/listing success.

Full binary `--list` exits 0 and lists **2,325** cases; all four names occur exactly once. Ordinary `cargo nextest list --offline --locked --test integration` exits 0 and lists **10,681** names; all four names are absent. Complete streams retained, parsed without truncated-output inference. Listing observations: `listing-observations.json`.

## Exact execution and timings

All CLI argv/cwd/environment overrides, start/end timezone timestamps and exit statuses live in `commands.json`; full inherited environment in `environment.json`. All commands use argv-style `cli.command(...).cwd(repo)`. No shell or cwd mutation.

| Proof | argv (binary = path below) | Wall time | Exit |
|---|---|---:|---:|
| Repaired build | `cargo test --offline --locked --test prefork_full_ui --no-run --message-format=json` | 28.32s | 0 |
| Full prefork listing | `binary --list` | 0.018s | 0 |
| Runner conformance | `timeout 90 binary`, env `PREFORK_CONFORMANCE_SUITE=1` | 1.60s | 0 |
| Chat pair | `timeout 90 binary chat_frame::test_chat_editbox_ --test-threads=2` | 4.33s | 0 |
| Cast-bar | `timeout 90 binary spell_casting::cast_bar_respects_edit_mode_lock_setting_after_startup_fix --exact --test-threads=1` | 4.34s | 0 |
| Spellbook | `timeout 90 binary blizzard_player_spells_loads::mainline_spellbook_keybind_opens_and_closes_without_runtime_errors --exact --test-threads=1` | 4.73s | 0 |
| Retail integration listing | `cargo nextest list --offline --locked --test integration` | 24.44s | 0 |

Binary: `/home/osso/Projects/wow/wow-ui-sim/target/debug/deps/prefork_full_ui-21dd5c807fe7a7b6`, produced by repaired default-profile build. Artifact provenance/features: `prefork-artifacts-repair.json`. Runtime streams: `chat-repair.*`, `cast-bar-repair.*`, `spellbook-repair.*`; conformance: `conformance-repair.*`; listings: `prefork-list-repair.*`, `integration-list-repair.*`.

`cargo fmt --check` ran exactly once before repair, exit 0, 9.52s. Repair changed only macro qualification, not layout; no redundant format invocation. Therefore direct format command evidence covers pre-repair migration, with repair formatting inspected rather than separately command-verified. Six existing iced_wgpu manifest deprecation warnings remain; no Rust compiler warnings/errors emitted after repaired compilation and no lint suppression added.

## Static preservation / readability / unsupported execution

All four full original behavioral body ranges byte-match pre-migration source. All three constructor/startup ranges match exactly aside from added visibility. `preservation.json` and `.old-body`/`.new-body` hashes document comparisons. Chat retains its manual sequence without UPDATE_CHAT_WINDOWS and original SAY/white behavior; cast-bar retains original parent and anchor transitions; spellbook retains original profile-specific fixture and S key dispatch. Existing spellbook error clearing is unchanged, not an added reset. Source gates/wrappers stay unchanged through repaired revision.

Five Rust files are actually changed across the requested commits, not six; build.rs was also inspected as the sixth Rust boundary dependency. Commit file inventory and detailed source/wiring evidence retained in `report-red.md`. Changed-Rust readability skill applied to migration lines and repaired macro line: no separate readability violations found; compile defect independently demonstrated RED then repaired GREEN without suppression.

Non-Retail wrappers: source applicability/constructor preservation inspected; execution unsupported/unverified in this bounded Retail proof. No aggregate Forever iced compile attempted. No broad suite/check/reviewer, network, delegation, repo edits/commits or operational actions performed. The caller's broader asynchronous migration gates are not completed by these bounded passes.
