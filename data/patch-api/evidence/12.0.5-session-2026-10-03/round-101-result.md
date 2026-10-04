# Round 101 result

Both staged parts landed. **393 targeted cases passed: 381 integration + 12 lib; zero final failures or ignored cases.** Startup remains unverified because the requested helper invocation rejected its flags.

## Commits and isolation

Worktree: `/home/osso-test/.worktrees/wow-ui-sim-round-101`, branch `round-101`, clean. Base: `74e8f4a9ca12bc23f9e3505e7dea89816395d7ab`.

| Commit | Change |
|---|---|
| `de427a66d2bebf9b7af203080666133e4a86545b` | Cached secure aura header fixture and two ordering tests |
| `a6c337eef4ad625bcc1a2c70c1b8b9d2356d7aa5` | Secure helper command delegation, shared completion, lifecycle tests, fixture adaptations and spec |

Both messages end with `Co-Authored-By: Claude Opus 5.5 (1M context) <noreply@anthropic.com>`. No push, merge, vendor/cache edits, canonical/other-worktree modifications, agents or model CLIs.

## Files changed

Paths relative to the worktree:

```text
NEW     docs/specs/secure-aura-header-helper-delegation.md
NEW     src/c_api/class_talent_commands.rs
EDIT    src/c_api/mod.rs
DELETE  src/iced_app/casting.rs
EDIT    src/iced_app/mod.rs
EDIT    src/iced_app/update.rs
NEW     src/lua_api/cast_completion.rs
EDIT    src/lua_api/globals/missing_surface/traits/class_talents.rs
EDIT    src/lua_api/mod.rs
EDIT    tests/admin_spec_talent_api.rs
EDIT    tests/click_targeting.rs
EDIT    tests/combat_verbs.rs
EDIT    tests/hero_talents.rs
EDIT    tests/hero_talents/rendering.rs
NEW     tests/secure_aura_header_helpers.rs
EDIT    tests/unit_auras_private.rs
```

Git represents old casting source → shared completion source as a rename: 15 diff entries, 793 insertions, 34 deletions across both commits.

## Integration and TDD

All **17 OLD anchors matched uniquely** at current master. Applied exact replacements; never copied staged snapshots over existing files. Verified the deleted casting source against staged SHA-256 `c13540f516501333dc99556e592487141a68376a84fb5d11edebaeae3b25f731`. No anchor adaptation was necessary; unrelated B99 changes were preserved.

Aura fixture extends the parsed restricted-environment TOC before its first load. Real cached Lua/XML supplies sorting and child lifecycle; no aura producer/comparator change. Probe construction and subsequent events must have no Lua errors; preceding whole-UI startup errors are isolated, not counted as startup proof.

Compilable RED included tests, five fixture adjustments, and refactor-only shared completion with its old behavior. Four command delegates, talent-state completion update, and the new specialization notification remained withheld. Aura tests passed 2/2; four helper tests failed because direct mutation bypassed vendor UI writes, command observers and pending casts. Rewritten hero lifecycle test failed 0/1.

Then enabled staged producers, formatted and committed before GREEN. All four commands traverse real cached helper callbacks through the secure event boundary. GUI and headless `WowLuaEnv` callers use the same `tick_casting` producer. Specialization completion retains STOP/SUCCEEDED ordering, synchronizes talent/config/hero state, then publishes both specialization notifications. Earlier-epoch registration/completion behavior remains epoch-selected, not a fallback.

Three mandated regression modules exposed stale current-master identity fixtures. Their tests failed before adaptation; no extra producer change was needed. Corrected observers retain their original behavioral guarantees, as detailed below.

## Exact proof commands and counts

All test invocations used this environment and worktree. Each integration filter below was a **separate invocation** of command A; substitute the table's exact filter for `<FILTER>`. Output was captured into logs under `SCRATCH`, with stdout and stderr retained and read.

```text
SCRATCH=/tmp/claude-1000/-home-osso-test-Projects-wow-wow-ui-sim/f86e1850-973d-4071-a472-f5757f1d7fdc/scratchpad
CARGO_BUILD_JOBS=6
BUILD_HOST_SCRIPTS=/home/osso-test/Projects/world-of-osso/game-engine/scripts
```

**A — integration argv:**

```text
python3 scripts/build-host.py --build-host local --test --target-dir /home/osso-test/.cache/wow-ui-sim-target-b101 --test integration <FILTER> -- --test-threads=1
```

RED ran A with exactly:

```text
secure_aura_header_helpers::
hero_talents::test_class_talents_switch_methods_update_seeded_spec_and_loadout_state
```

First RED compilation finished in 8m12s, without timeout. Logs: `round-101-red-helpers.log` and `round-101-red-hero.log`.

An initial attempt put both RED filters before `--`; Cargo rejected it before compilation: `error: unexpected argument 'hero_talents::test_class_talents_switch_methods_update_seeded_spec_and_loadout_state' found`. Log: `round-101-red.log`. Corrected to separate invocations.

**Counts are passed/failed.** “—” means no RED invocation for that module. “Pre-adaptation” identifies existing identity-fixture failures, not withheld-helper RED.

| Integration filter for A | RED / pre-adaptation | Final GREEN |
|---|---:|---:|
| `secure_aura_header_helpers::` | 2/4 | 6/0 |
| `hero_talents::` | 0/1, lifecycle test only | 16/0 |
| `admin_spec_talent_api::` | — | 23/0 |
| `secure_group_headers::` | — | 2/0 |
| `key_dispatch::` | — | 28/0 |
| `blizzard_unit_frame_loads::` | — | 10/0 |
| `channel_lifecycle::` | — | 4/0 |
| `channel_reentrancy::` | — | 3/0 |
| `unit_cast_durations::` | — | 7/0 |
| `spell_casting::` | — | 19/0 |
| `unit_auras_private::` | 4/1 pre-adaptation | 5/0 |
| `cast_events_identity::` | — | 10/0 |
| `test_crafting::` | — | 23/0 |
| `blizzard_ui_blizzard_actionbar::surface_events::` | — | 6/0 |
| `channel_blizzard::` | — | 3/0 |
| `spell_state_probes::` | — | 16/0 |
| `unit_spell_target_name::` | — | 10/0 |
| `c_action_bar_input_probes::` | — | 11/0 |
| `action_button_input_dispatch::` | — | 3/0 |
| `admin_combat_api::` | — | 17/0 |
| `combat_verbs::` | 15/1 pre-adaptation | 16/0 |
| `unit_api::` | — | 83/0 |
| `click_targeting::` | 21/1 pre-adaptation | 22/0 |
| `rilua_admin_split_smoke::` | — | 7/0 |
| `workarounds_professions::` | — | 2/0 |
| `unit_empowered_stage_percentages::` | — | 2/0 |
| `c_spell_flyout_probes::` | — | 14/0 |
| `admin_spell_effects_api::` | — | 10/0 |
| `cast_bar_id::` | — | 2/0 |
| `blizzard_ui_blizzard_actionbar::behavior_keybind_dispatch::` | — | 1/0 |

Discovery covered the requested `pending_spec_change`, `UNIT_SPELLCAST_`, `SetSpecialization`, and `PLAYER_SPECIALIZATION_CHANGED` patterns, then expanded to casting state, cast initiation/query, effects and specialization setup. The Mists-only talent/glyph panel is disabled by its `client-mists` cfg in this Retail build; no other-profile proof claimed. Character-stat `casting` fields are haste data, not cast lifecycle.

**B — relocated lib tests, same environment:**

```text
python3 scripts/build-host.py --build-host local --test --target-dir /home/osso-test/.cache/wow-ui-sim-target-b101 --lib lua_api::cast_completion:: -- --test-threads=1
```

| Lib submodule / original retained test file | GREEN |
|---|---:|
| `duration_tests` / `src/iced_app/casting/duration_tests.rs` | 1/0 |
| `input_tests` / `src/iced_app/casting/input_tests.rs` | 5/0 |
| `interrupted_tests` / `src/iced_app/casting/interrupted_tests.rs` | 3/0 |
| `tests` / `src/iced_app/casting/tests.rs` | 3/0 |

Path attributes in shared completion reference those unchanged files. Log: `round-101-green-lib-cast-completion.log`.

Other GREEN logs use `round-101-green-<module>.log`; repaired modules use `round-101-green-fixed-<module>.log`; actionbar surface log is `round-101-green-actionbar-surface-events.log`. Complete argv, environment, revisions, exits and artifact paths are in `round-101-proof-ledger.json`.

Actual Cargo runners used `/home/osso-test/.cache/wow-ui-sim-target-b101`, despite the helper banner printing the worktree's default target path. No unfiltered suite ran.

### Final checks and proof validity

```text
cargo fmt
cargo fmt --check
CARGO_BUILD_JOBS=6 cargo check --target-dir /home/osso-test/.cache/wow-ui-sim-target-b101 --jobs 6
```

Formatting ran before each code commit. Final fmt check and cargo check exited 0; no warning diagnostics. Changed Rust also received a manual readability audit; no agents/models used.

Primary/lib/unchanged-regression passing logs cover `e69f4ae2b29e9f1b9625d61b8eee7a4553527bdb`. Repaired regression logs cover `c22ad136dccabc5b1dfaeb906d453ecde5a9d184`. Final check/fmt cover `f1050f550436e932cf362bff0e665526e4313636`. Final amended commit differs from those scopes only by the three separately re-proven test fixtures and/or spec documentation. Production Rust stayed identical; existing proof remains valid without redundant reruns.

## Existing tests: old → new meaning

No behavioral assertions were weakened or removed.

| Existing test | Old → new meaning |
|---|---|
| `hero_talents::test_class_talents_switch_methods_update_seeded_spec_and_loadout_state` | Four bare-environment commands immediately mutated state → actual cached helpers complete Protection loadout 202, Holy activation/default 101, Holy loadout 102, and Retribution activation/default 301. Retains config names, last-selected IDs and hero-subtree checks; adds pre-deadline unchanged state, cast/event identity, coherent notification observations and no duplicate completion. Earlier-epoch test preserved. |
| `admin_spec_talent_api::test_trait_config_mapping_tracks_active_loadout` | UI command used as bare-env setup → existing `LoadConfig` provider establishes second Protection config and asserts Ready. Every mapping assertion retained. |
| `hero_talents::test_non_selectable_hero_nodes_do_not_show_selectable_glow` | Immediate specialization command used as setup → coherent Protection player/talent state seeded before UI load. Glow assertions unchanged. |
| `hero_talents::hero_talents_rendering::test_class_talent_edges_render_below_visible_talent_buttons` | Same immediate command setup → pre-load coherent Protection fixture. Layer assertions unchanged. |
| `hero_talents::hero_talents_rendering::test_button_frame_level_change_relevels_connected_edges_on_update` | Same immediate command setup → pre-load coherent Protection fixture. Relevel assertions unchanged. |
| `hero_talents::hero_talents_rendering::test_hero_spec_content_spec_image_anchors_to_spec_name` | Same immediate command setup → pre-load coherent Protection fixture. Anchor assertions unchanged. |
| `unit_auras_private::native_unit_event_dispatch_respects_unit_filter` | Obsolete `(unit, spellID)` event observer accidentally read GUID as spell ID → full `(unit, castGUID, spellID, castBarID)` observer. Exact event counts, unit filtering and spell 200749 retained; GUID/bar-ID agreement with live query added. |
| `combat_verbs::successive_casts_advance_cast_id` | Numeric decoding of now-string query slot 7 → numeric castBarID slot 10 for the new Retail tuple, slot 7 for earlier epochs. Strict monotonic identity assertion retained. |
| `click_targeting::cast_spell_book_item_blocked_while_casting` | Same obsolete numeric slot 7 decoding → same epoch-correct numeric identity selection. Exact equality across blocked second cast retained. |

The three additional adaptations were required by the requested casting-module GREEN scope and grounded in already-passing `cast_bar_id` / event-identity contracts. Their initial errors were payload-slot mismatch / `expected numeric result, got string`, not helper regressions.

## Startup attempt

Attempted exactly once, with the same test environment plus `WOW_SIM_CASC=0`:

```text
python3 scripts/build-host.py --build-host local --run --target-dir /home/osso-test/.cache/wow-ui-sim-target-b101 -- --no-addons --no-saved-vars lua-errors
```

Exit **2**, before build or runtime execution. Exact error:

```text
build-host.py: error: unrecognized arguments: --target-dir /home/osso-test/.cache/wow-ui-sim-target-b101
```

Log: `round-101-startup.log`. No alternative startup invocation, no runtime `[]` claim.

## Four prose rows: bounded proof vs gaps

Source: `data/patch-api/sources/12.0.5-register.json`.

| Row | Proven | Not proven |
|---|---|---|
| `03-25-112` | Proposed “short < long < permanent” order: real cached template, TIME ascending, shuffled input, finite→permanent update, removal/readdition, child reuse/hiding. | Other sort directions, filters/grouping/consolidation, permanent ties, automatic Retail header exposure and native-client comparison. |
| `03-31-151` | Same finalized ordering clause, same two actual cached-header tests; no simulator comparator change. | Same bounds; no full-row/native acceptance claim. |
| `03-25-120` | All four cached helper commands perform nontrivial real UI/state transitions from tainted addon callers. Specialization UI and player/overlay castbar fields stay clean through initiation/completion; loadout Ready-path talent fields stay clean. Caller taint restored on success/error; addon observer taint retained. | Delayed loadout commit/castbar flow, cold lazy loading, combat/secret arguments, every possible UI field and native-client comparison. |
| `03-31-177` | Same finalized neutrality clause, with real callback writes rather than unchanged-state success. Coherent specialization/talent/config/hero completion and notifications additionally tested. | Loadout castbar neutrality under `LoadInProgress` remains unmodeled/uncredited; same other bounds. |

No staged slice omitted. Existing Ready-plus-instant loadout provider retained; no invented cast duration, changed result enum, vendor patch or fallback. Automatic headless CLI ticking, other client profiles, live GUI proof, full suite and independent acceptance were not added or claimed. Added refresh-event chronology is simulator policy, not proven native chronology. Audit row classifications remain unchanged.

**Merge risk:** bounded implementation and targeted regressions are proven; startup is still unverified because of helper parsing, and delayed loadout/native edge cases remain outside this slice.
