# Cast duration lost through deadline rounding

The completion-duration flake comes from reconstructing a configured span by subtracting floating-point absolute timestamps, not GC or cross-test state.

## Evidence

At base `0856b624e`, eight full parallel lib runs did not reproduce the duration assertion. A temporary diagnostic assertion captured targeted run 3 failing with `total=0.99999999999999989 start=0.25440984300000002 end=1.2544098429999999 now=0.25443996000000002`. The cast input was exactly one second. `SetCasting` stored `end = start + duration`; `UnitCastingDuration` recovered `end - start`. Rounding at the addition makes that subtraction one ULP below one. Startup scheduling changes the start value; neither expiration nor an event callback caused this failure.

`unit_cast_duration_preserves_span_across_rounded_deadline` seeds the captured start and one-second span. It fails against the old model without load, sleeps, retries, or collection. It also checks the rounded end, delay extension, and retained snapshot independence. Temporary diagnostic instrumentation was removed; the original completion test and exact one-second assertion remain unchanged.

## Fix

`CastingState` stores `start_time` and `duration` instead of `start_time` and `end_time`. `end_time()` derives the absolute deadline for casting/channel tuples and completion. Cast, specialization, crafting, channel and empower producers preserve their input spans; delay/update inputs mutate duration. Duration snapshots read the span directly and add empower hold before constructing the snapshot. There is no second mutable deadline or duration cache to synchronize.

Existing fixtures switch from absolute deadlines to equivalent spans. Zero-duration completion fixtures still finish through the real completion boundary. Overflow validation stays before mutation.

## Verification — 2026-10-05

Code revision `703437add45266fb3e4661e3a6a4b76a483501c5`; subsequent documentation-only changes do not invalidate it. All builds/checks/tests used local host, default retail debug and the worktree's existing target directory. Raw outputs and ledger remain at `target/debug/cast-flake-proof/` (ignored build artifacts).

Command prefix: `python3 /home/osso/.worktrees/wow-ui-sim-cast-flake/scripts/build-host.py --build-host local`.

| Command tail / scope | Result |
|---|---|
| Base + diagnostics: `--test --lib`, eight runs | Duration test passed all eight; six known failures each time. First run additionally failed `gui_resolution_cache_preparation`: its cold child exceeded 60 seconds. No asset code changed; subsequent seven baseline and all three final runs pass that test. |
| Base + diagnostics: `--test --lib unit_cast_duration_clears_before_completion_callbacks`, three runs | Two pass; third captures the precise failing total above. |
| Base + deterministic fixture: `--test --lib unit_cast_duration_preserves_span_across_rounded_deadline` | RED, then GREEN 1/1 after the span-storage fix. |
| `--test --lib unit_cast_duration_clears_before_completion_callbacks`, three final runs | 1/1 pass each; original assertion unchanged. |
| `--test --lib lua_api::cast_completion::`, three final runs | 14/14 pass each, including delay, reentrancy, spec/crafting completion and forced-GC control. |
| `--test --lib`, three final parallel runs | Each 1,971 pass / six known fail (54.84s, 71.75s, 72.33s); both duration tests pass every run. |
| `--test --test integration <filter>` | 95/95 across `unit_cast_durations::` (7), `channel_lifecycle::` (4), `channel_reentrancy::` (3), `cast_bar_id::` (2), `spell_casting::` (19), `unit_spell_target_name::` (10), `c_vehicle_possession_globals::` (22), `c_spell_flyout_probes::` (14), `p1200_plain_globals::` (8), `secure_aura_header_helpers::` (6). Compilation also covers every migrated integration fixture. |
| `--check`; `cargo fmt --manifest-path /home/osso/.worktrees/wow-ui-sim-cast-flake/Cargo.toml -- --check` | Both exit 0. Changed-line readability review finds shallow explicit timing mutations, no duplicated mutable deadline or new warning suppression. |
| `--bin wow-sim`, then bounded `--no-build --run -- --no-addons --no-saved-vars lua-errors` | Both exit 0; startup `[]`, zero unique/total errors. Run wrapped in `timeout 90`, never the build. |

The six known lib failures are `test_patch_12_0_0_transmog_situation_enum_values`, `installs_debug_environment_defaults`, `installs_seeded_housing_catalog_surface`, and the three `apply_system_anchors` cases (player frame, nil singletons, compact unit frame). No new Rust warnings; six pre-existing iced manifest lint-name deprecations remain unchanged and unsuppressed. No vendor, canonical checkout, other worktree, or page-coverage ledger was edited; no push, merge or agents.

## Sources

- [Cast state](../../../src/lua_api/game_data.rs)
- [Duration queries](../../../src/lua_api/channeling/durations.rs)
- [Deterministic regression](../../../src/iced_app/casting/duration_tests.rs)
- [Duration contract](../../specs/unit-cast-durations.md)

## See Also

- [GC rooting audit](gc-rooting-audit.md) — independent GUID/event-payload lifetime fixes.
