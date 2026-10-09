# Narrow Forever module gate repair — FAIL / BUILD BLOCKED

## Scope and preservation

Verified canonical commit `0b5c69d6fdac137e57f4cb9a9f16114773234e34`, copied as source-worktree commit `811a0491b8a1ba21e2ebb27f364f03a563c5300c`. All four verification commands below ran with explicit cwd `/home/osso/.worktrees/wow-ui-sim-p1601-source`, inherited environment, installed `/usr/bin/cargo`, own existing `target/`, and unchanged toolchain. Observed rustc: `1.99.0 (b940084d7 2026-09-28) (Arch Linux rust 1:1.99.0-1)`; no toolchain changes.

Read/followed verify skill as verifier; no delegation. Read prior RED report, cfg map, source assertions and registration/state gates. No source edits, target copying, Bash, network request, fallback profile/host, broad/full suite, push or deployment. No auto-backgrounding. An initial Pyrun setup evaluation failed with undefined `canonical` before invoking any Cargo command; the requested source test itself ran exactly once.

Source revision/status unchanged throughout: only intentionally dirty `PLAN.md`; its SHA-256 unchanged (`0e157b07dbd8d745cefdf23ca1d5d0f68de5bd52fff89ba4ec970bd433fc246f`). Canonical advanced externally to `c2481c4c50d04489b081f35fafccc929ba0b9b73` with integration conflicts at final observation; no proof claimed for that moving HEAD. Initial/final status snapshots retained.

`canonical-equivalence.json` proves byte equality against the exact original canonical commit for `src/c_api/{mod,registration,addon_messages,c_combat_log}.rs` and `src/lua_api/state/sim_state.rs`. `proof-scope-sha256.json` records post-command content hashes for tracked source, tests, patch-tests, build script and Cargo configuration. These are post-command hashes, not invented pre-command snapshots; unchanged source-worktree HEAD/status brackets command execution.

## Exact command ledger

All times UTC on October 9, 2026. Full stdout/stderr, exact argv/cwd/revision, inherited environment and start/end timestamps retained in `/tmp/forever-cfg-independent/<name>-result.json`; separate `.stdout`/`.stderr` files and pre-command `-start.json` also retained. Full streams read. `receipt-sha256.json` hashes artifacts.

| Receipt name | Start → end | Exit / observed result |
|---|---|---|
| source-model | 14:21:48.568108 → 14:22:12.063082 | 101; two E0609; zero tests executed |
| fmt | 14:23:08.421911 → 14:23:19.587895 | 0; no output |
| retail-check | 14:23:08.422279 → 14:23:41.224937 | 0; focused Retail library compiles |
| retail-behavior | 14:23:54.141734 → 14:25:56.015113 | 101; E0432 disabled iced; zero tests executed |

```text
/usr/bin/cargo test --offline --locked --no-default-features --features client-wowforever --test patch_1_60_1_source_model -- --nocapture
/usr/bin/cargo fmt --check
/usr/bin/cargo check --offline --locked --no-default-features --features client-retail --lib
/usr/bin/cargo test --offline --locked --no-default-features --features client-retail --test integration p1200_rest_policy::p1200_rest_policy_defaults -- --exact --nocapture
```

Older inspected Retail receipts (`p504-retail-check.log`, `p342-retail-publication-v2-result.json`, `p343-portable-gate-result.json`) do not establish the exact repaired-head focused check scope, so the required focused Retail check ran once. No repeated broad check or redundant Forever rebuild. Further Forever behavioral execution shares the failed library boundary; further headless aggregate Retail tests share the disabled-iced boundary. Neither was rerun.

## Failure boundary and literal source evidence

Previous exact invocation had E0433 for cfg-removed module names and zero executed tests. The two module declarations now use `cfg(any(retail-12-0-0, client-wowforever))`. E0433 is absent from the fresh exact invocation, but compilation still fails:

1. `src/c_api/addon_messages.rs:107:34`: E0609, no `bnet_custom_message` field on `RefMut<SimState>`. Ungated `set_custom_message` compiles with the newly enabled module, while the field remains Retail-only at `src/lua_api/state/sim_state.rs:592-593`.
2. `src/c_api/c_combat_log.rs:14:68`: E0609, no `combat_log_restricted` field on `Ref<SimState>`. Ungated `register_restriction` body compiles even though its registration call is Retail-only. Field remains Retail-only at `src/lua_api/state/sim_state.rs:143-144`.

Exact terminal result: `error: could not compile wow-ui-sim (lib) due to 2 previous errors` (Cargo renders the package name with backticks).

Thus the prior map's claim that no backing-state gate needed consideration is not supported by fresh compilation. No repair applied by verifier.

Separate Retail integration attempt fails at `tests/game_menu.rs:9:5`, unresolved import `iced`, because the aggregate includes this file with GUI disabled. This is not a failure of a runtime assertion or of the successful Retail `--lib` check. No profile expansion or alternative harness used to conceal it.

## Capability / proof matrix

| Claim | Inspected behavior and proof level |
|---|---|
| Two missing module names repaired | Declaration union and byte-equivalent copy confirmed; E0433 absent in fresh compilation. Entire Forever build NOT repaired. |
| Configured UnitName before test-dispatched login | Source test expects Ada then Grace, with independent Linus reads. Zero harness execution; all values UNPROVEN. No native/pre-login lifecycle/loaded-UI parity credit. |
| Forever addon senders | `tests/addon_messages.rs:100-139` asserts both senders return Success, concrete ordered `addon`/`addon_logged` records, default PARTY and explicit whisper routing, and no inbound-event echo. `:143-194` asserts concrete rejection codes, unchanged logs/events, group/guild state. Source inspected only; runtime UNPROVEN due library failure. |
| Forever removed combat-log getter | `tests/c_namespace_noop_replacements.rs:1823-1868` asserts raw/public `C_CombatLog.GetCurrentEventInfo` and legacy global absent, capability predicate, and preservation across deprecated publisher/post-load workarounds. Source inspected only; runtime UNPROVEN. |
| Forever Retail restriction API not newly exposed | Registration call stays Retail-only at `src/c_api/registration.rs:60-64`; static call-site evidence only. Inspected Forever tests do not explicitly assert absence of `IsCombatLogRestricted`; no runtime absence proof. |
| Existing Retail behavior | `tests/p1200_rest_policy.rs:5-31` checks restriction false→true and other modeled policy state. Attempted exact filtered test is build-blocked, zero assertions observed. `tests/p1200_rest.rs:6-35` covers BNet success/rejection, custom-message state and concrete message records; inspected only, not separately run after aggregate blocker. |
| Retail library | Focused client-retail `--lib` check exit 0, not test or API behavior proof. |
| Formatting | `cargo fmt --check` exit 0 at source revision. Not compilation proof. |

## Warnings (preserved, not suppressed)

Every Cargo compile/check reports the six previously reported iced-wgpu manifest deprecations: `large-enum-variant`, `map-entry`, `match-wildcard-for-single-variants`, `redundant-closure-for-method-calls`, `trivially-copy-pass-by-ref`, `type-complexity`.

Fresh headless Retail library check additionally reports five library dead-code warnings: `PROVENANCE_SCHEMA` (`src/blizzard_ui_sync.rs:28`), `CacheProvenance::new` (`:180`), `remove_missing_marker` (`:663`), `ensure_known_asset_cached` (`src/casc_asset_fallback.rs:93`), `encoding_key_hex` (`src/render/font.rs:44`). Retail test compilation repeats those and adds unused `SavedVariablesManager` import (`src/bin/wow_sim/main.rs:20`). These are newly observed in this verification scope, not proven new regressions caused by the two-line gate repair. The repair commit changes only two module attributes and does not edit these warning sites.

## Overall

**FAIL**: narrow gate correction removes the original unresolved-module boundary but reveals two missing-state-field compile errors. Zero Forever tests and zero Retail runtime tests executed. Formatting and focused Retail library check PASS only. Main received exact next-boundary errors/receipts and retains repair/integration ownership. Report and receipts do not grant native parity or final acceptance.
