# Independent bounded Mists wrapper preservation report

## Overall: BLOCKED / verification failure (not a source or compiler failure claim)

The required check was spawned exactly once. The Pyrun evaluation boundary did not preserve its process handle or context. Its stdout/stderr pipes and exit status were not retrieved. A subsequent `ps -p 3596388` returned exit 1 with no process row; a complete process listing contained neither that PID nor the dedicated target-dir argument. No command was rerun. This is a verifier execution/capture failure, not evidence of a concrete compiler diagnostic. The required full-stream/exit proof is missing.

The subsequent offline/locked integration no-run build and three exact runtime filters were NOT RUN because the check handoff has no valid result. No integration executable was selected; binary hash and runtime selected counts are unknown (not zero-match evidence). Runtime executed cases: 0/3; passes: unproven, not 0/3 failures.

## Revision / scope

Initial and check-spawn HEAD: `41f1abbeb83d508b323f51d1f371a3bc506b63b5`.
Tracked tree: `f654c2271d7dd3a2aac7c1bfd3bca0138c16e65a`.
Final HEAD: `e14488aaaed19d49038d366630768cec3d4b23f6`.
Exact per-file SHA-256 hashes: `initial-scope.json`, `final-scope.json`; saved source bytes under `source/`. Code scope hashes unchanged across this attempt. `docs/specs/prefork-test-harness.md` changed through concurrent documentation commits: initial SHA-256 `a708859ac70cba53f1cc04d5e681aaab35ebad2a20b25ee5020d714dee2daa38`, final SHA-256 `c97737956813f586145d201d3214ad5b4799d4c7f27e5678f6b61679b5b05d3b`. Both epochs retained; `epoch-diff.stdout.log` records the documentation-only delta. This does not assert a successfully executed compile source revision: capture failure prevents that claim. Existing untracked `.code-index.db` retained; no source edits made.

## Static constructor/body preservation

| Wrapper source | Case | Original constructor retained | Body comparison |
|---|---|---|---|
| `tests/chat_frame.rs:398` | `test_chat_editbox_click_type_and_submit` | `setup_env()` | PASS |
| `tests/chat_frame.rs:468` | `test_chat_editbox_text_color_after_activation` | `setup_env()` | PASS |
| `tests/spell_casting.rs:475` | `cast_bar_respects_edit_mode_lock_setting_after_startup_fix` | `env_with_full_blizzard_ui()` | PASS |
| `tests/blizzard_player_spells_loads.rs:256` | `mainline_spellbook_keybind_opens_and_closes_without_runtime_errors` | `load_runtime_game_ui()` | PASS |

Comparison retains inner source bytes and removes only the former function/harness shell and constructor statement, trimming outside blank lines. All four bodies byte-match; `preservation-audit.json` retains hashes and wrapper text. Original full sources and function diffs retained under `originals/` and `*.diff`.

Chat originals are from `29e48f225^`; cast-bar and spellbook originals from `911c980fc^`. Chat `setup_env` differs only by `pub(crate)` visibility; chat startup event function byte-identical. Cast constructor, delegated constructor, and startup event functions byte-identical. Spellbook constructor differs only by `pub(crate)` visibility. Constructor bodies and assertions therefore unchanged in inspected source.

Ordinary integration ownership is SOURCE-proven, not executable-list-proven: `tests/integration.rs:1` includes the generated module tree; `build.rs:54-90` emits modules from top-level `.rs` paths, and `build.rs:484-499` excludes none of these three wrappers. Macro definitions in `tests/common/mod.rs:115-131` create a sibling module exposing `run`, while retained `#[test]` functions call that same body.

Chat module requires `gui` (`tests/chat_frame.rs:1`), present in requested args. Each of the three retained wrappers has `#[cfg(not(feature = "client-retail"))]`, true for the requested non-default Mists features, and uses `test_timeout!` with its original constructor. `tests/common/mod.rs:145-147` supplies the 120-second timeout, invoking the separate ordinary Linux timeout path (`:26-28`), not the prefork runner. Runtime timeout/cleanup not exercised here.

Mainline spellbook marker and wrapper require `retail-12-1-0` or `client-wowforever` (`tests/blizzard_player_spells_loads.rs:208,251-254`), neither enabled by `client-mists = []` (`Cargo.toml:152`). It is intentionally excluded in Mists. Source gate count: 3 retained ordinary cases; 1 excluded Mainline case. These are static counts, NOT libtest selection counts.

## Command ledger

Requested check argv, exact invocation:

```text
["cargo", "check", "--offline", "--locked", "--no-default-features", "--features", "sound,gui,casc,client-mists", "--tests", "--target-dir", "/home/osso/Projects/wow/wow-ui-sim/target/mists-wrapper-proof"]
```

Canonical cwd: `/home/osso/Projects/wow/wow-ui-sim`.
Dedicated target-dir: `/home/osso/Projects/wow/wow-ui-sim/target/mists-wrapper-proof`.
Environment overrides: none. Inherited environment KEY NAMES recorded in `ledger.json`; no secret values exposed.
Start UTC: `2026-10-09T17:39:37.544578+00:00`; actual completion time and exit unknown.
Full stdout/stderr: UNAVAILABLE (capture failure). No warnings suppressed, but warning count and typecheck result unproven. NOT zero-warnings proof.

Not-run next command would be `cargo test --offline --locked --no-default-features --features sound,gui,casc,client-mists --test integration --no-run --message-format=json --target-dir /home/osso/Projects/wow/wow-ui-sim/target/mists-wrapper-proof`; no retry or test execution authorized by this report.

Requested exact runtime filters, all NOT RUN: `chat_frame::test_chat_editbox_click_type_and_submit`, `chat_frame::test_chat_editbox_text_color_after_activation`, `spell_casting::cast_bar_respects_edit_mode_lock_setting_after_startup_fix`. Exact selected counts: UNKNOWN for each; no compiled binary/list evidence. Required budget remains 120 seconds per original test.

## Boundaries

Only static Mists wrapper preservation confirmed. Mists `--tests` typecheck and 3-wrapper execution remain OPEN. No PTR/Forever/Era/Wrath/Anniversary execution, broad/full suites, prefork execution, source/cache/vendor repairs, commits, pushes, deploys, network, or delegation. Cargo's permitted build attempt may have populated its own dedicated target. Parent acceptance and every broader non-Retail gate remain with main.
