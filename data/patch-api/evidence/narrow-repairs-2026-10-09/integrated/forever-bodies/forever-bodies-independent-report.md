# Retail-only body gate followup — bounded PASS; API behavior unproven

## Revision and scope
Canonical `1044215d0386dd809c51cb8f7b47e08bbf4016a1` copied as `2218c0f4114b50bb33ab507917983892b30dbf4a` in `/home/osso/.worktrees/wow-ui-sim-p1601-source`. Exact four-file byte/hash equivalence retained in `canonical-equivalence.json`. Followup changes only ten `#[cfg(feature = "retail-12-0-0")]` attributes in two files; no state or registration changes. Initial/final source/config/test hashes identical. HEAD unchanged; only pre-existing dirty PLAN.md, hash unchanged `0e157b07dbd8d745cefdf23ca1d5d0f68de5bd52fff89ba4ec970bd433fc246f`.

Read/followed verify skill as verifier, no delegation. No source edits, cwd switch, push, deploy, model/toolchain change, target copy, network request, broad/full suite, aggregate integration invocation, or duplicate check. Initial setup evaluation failed with undefined `repo` before Cargo invocation; exact requested test invoked once. No auto-backgrounding.

## Command ledger
All commands explicit cwd above, inherited environment (full mapping retained), installed `/usr/bin/cargo`, existing own target. Exact argv/revision/status/start/end/exit and full separate stdout/stderr saved in named result JSON and stream files.

| Command | UTC start → end | Result |
|---|---|---|
| `/usr/bin/cargo test --offline --locked --no-default-features --features client-wowforever --test patch_1_60_1_source_model -- --nocapture` | 2026-10-09T14:29:10.999137+00:00 → 2026-10-09T14:30:26.334210+00:00 | exit 0; 1 passed, 0 failed, 0 ignored, 0 filtered |
| `/usr/bin/cargo fmt --check` | 2026-10-09T14:30:44.796563+00:00 → 2026-10-09T14:30:56.393544+00:00 | exit 0; empty stdout/stderr |

Exact behavioral stdout:
```text
running 1 test
test forever_source_player_name_reads_configured_state_before_login ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
```
The previous two E0609 errors are absent; compilation reached and executed the source-model test.

## Proof coverage
| Capability | Proof |
|---|---|
| Configured player name before login dispatch | Runtime PASS: UnitName('player') reads Ada then Grace; independent environment continues reading Linus. Test uses WowLuaEnv::new and direct host state mutations; no PLAYER_LOGIN dispatch or loaded Blizzard UI. |
| Gate/body correction | Static PASS: new Retail-only bodies/import/constants match existing Retail callers and Retail state fields. registration.rs:56-64 still separates Forever chat/publication from Retail restriction. c_battle_net.rs:64-72 still gates SendGameData and register_bnet_chat to Retail. sim_state.rs:143-144 and :592-593 remain Retail-only. |
| Retail library compilation | Prior valid receipt reused, NOT fresh execution. `/tmp/forever-cfg-independent/retail-check-result.json`: exact client-retail --lib command, revision 811a0491b, 14:23:08.422279–14:23:41.224937 UTC, exit 0. Prior/current scope hashes differ only two gated files (PLAN.md absent from prior scope but separately unchanged). Every new attribute evaluates true under client-retail (client-retail → retail-12-1-0 → retail-12-0-0); bodies and reachable Retail conditional scope unchanged. No duplicate check justified. Receipt hash retained. Not Retail API behavior evidence. |
| Forever chat/removal behavior | UNPROVEN: existing tests inspected, not executed. Cargo.toml package autotests=false; neither addon_messages nor c_namespace_noop_replacements is an explicit test target. Generated aggregate harness includes both and game_menu; tests/game_menu.rs:9 imports disabled iced in headless builds (prior E0432). No standalone focused target available; aggregate rerun prohibited and no harness/source edits authorized. |
| Native parity / real pre-login lifecycle / loaded UI | UNPROVEN; no native process, capture, UI loading, tuple/security/token/unavailable-name assertions. |

Chat test caveat: tests/addon_messages.rs:17-25 require_senders checks SendAddonMessage, SendAddonMessageLogged AND C_BattleNet.SendGameData. The latter remains Retail-only at c_battle_net.rs:64-70; therefore this helper is not evidence that chat tests would pass on Forever. No runtime failure claimed without execution. Combat removal tests at c_namespace_noop_replacements.rs:1823-1899 inspect public/legacy getter absence, meter predicate, deprecated publisher and internal fixture getters; none executed. IsCombatLogRestricted absence also lacks an explicit runtime assertion here.

## Formatting / readability
Fresh fmt PASS. Manually inspected every changed attribute plus enclosing definitions, imports/constants, registration callers and state declarations. Ten simple feature predicates; no function/body/control-flow changes, suppressions, TODO/FIXME/HACK additions, or new readability violations. Existing function design outside changed-line scope not modified.

## Warnings — retained, not suppressed
Fresh Forever test: 6 iced_wgpu manifest deprecation warnings; 7 library warning diagnostics; 1 binary warning. Five library warnings match prior Retail receipt: PROVENANCE_SCHEMA (blizzard_ui_sync.rs:28), CacheProvenance::new (:180), remove_missing_marker (:663), ensure_known_asset_cached (casc_asset_fallback.rs:93), encoding_key_hex (render/font.rs:44).

Newly observed versus prior Retail check: is_never_secret_aura (c_api/c_secrets.rs:110), cooldown_elapsed_since_start / cooldown_remaining_seconds (one diagnostic, widget/frame.rs:601/610). Unused SavedVariablesManager import (bin/wow_sim/main.rs:20) matches prior Retail test warning. Prior Forever build failed before producing this successful compilation warning set; these are newly observed profile warnings, not established regressions caused by the ten attributes. All exact diagnostics in source-model.stderr; no suppression applied.

## Conclusion
PASS for copied gate correction, exact source-model behavior (1/1), formatting and static changed-line readability. No full/API/native acceptance claim. Existing focused API tests remain unavailable without prohibited aggregate compilation or new harness changes. Receipts and hashes: `/tmp/forever-bodies-independent/`.
