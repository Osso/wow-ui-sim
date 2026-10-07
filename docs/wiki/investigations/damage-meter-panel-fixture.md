# DamageMeter panel fixture regression

The isolated panel test relied on synthetic runtime combat records removed by `1a9fcd1eca59d6744f3248955a13721261848e46` (October 1, 2026). Supply explicit test-owned aggregate and source-detail input; retain the empty runtime default and every original UI assertion.

## Bisect and root cause

`cargo test --test integration test_showuipanel_lod::damage_meter_loads_and_populates_primary_session_window` reproduces `entry_frame_count=0` at `c3a69fddd`. The suggested `d1a2a0250` baseline also fails alone against the same current retail cache, so it is not a good endpoint. Parent `97f7edd2d` passes.

A separate detached worktree ran `git bisect run` from `97f7edd2d` to `c3a69fddd`, building the integration target separately and running exactly the panel test. Build failures would return 125; no revisions needed skipping. Ten tested revisions failed with the same empty-entry boundary; first bad commit is `1a9fcd1ec`. Temporary worktree was reset and removed.

That commit deliberately replaced the synthetic Lua seed with empty-default `DamageMeterInput` and independent C API snapshots. The panel test never provided host input after that migration. This is stale fixture setup, not an API retirement, VM secrecy change or ScrollBox defect.

## Current Blizzard consumer and fix

Read-only current retail cache evidence:

- `Blizzard_DamageMeter/DamageMeterSessionWindow.lua:562–576`: type/ID getters select the session; `:603–621` consumes `combatSources`.
- `Blizzard_DamageMeter/DamageMeterSourceWindow.lua:156–166`: type/ID detail getters use source GUID/creature selectors; `:176–193` consumes `combatSpells`.

The repaired test explicitly supplies availability, Overall/Current bindings, a Player aggregate (52,000 total, 1,300/sec, 40 seconds), and separate spell-19750 details before loading Blizzard_DamageMeter. Existing visible-window, entry-count, identity, totals, rate and source-window assertions stay unchanged. No vendor/cache writes, Lua monkey-patches, runtime seed restoration, retired API restoration or patch ledger changes.

## Verification

RED and bisect logs live in the task scratchpad. Required post-commit checks: isolated panel test, `damage_meter`, `test_showuipanel_lod`, formatting and Mists tests compilation. No publication behavior changed; publication sweep is not affected. Execution results are recorded below after verification.

## Sources

- [Panel regression test](../../../tests/test_showuipanel_lod.rs)
- [Explicit DamageMeter input contract](../../specs/damage-meter-combat-source.md)
- [C API model](../../../src/c_api/c_damage_meter.rs)

## See Also

- [[patch-12-0-5-api-audit]] — explicit-input migration and bounded secrecy proof.
