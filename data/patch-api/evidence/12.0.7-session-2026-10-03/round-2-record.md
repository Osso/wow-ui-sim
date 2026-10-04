# 12.0.7 integration round 2 — record (reconstructed)

The original result file, proof ledger, run logs and both review reports lived in the session scratchpad under `/tmp` and were lost when the host ran out of memory and restarted on 2026-10-04. This record is rewritten by the main session from the reports as they were read before the crash. Counts are the integrator's; the logs no longer exist.

## Commits

| Master | Worktree original | Content |
|---|---|---|
| `e4fde2ceb` | `9140a06e8` | Gate retired 12.0.7 natives and prove cached forwarding (slice A: B31–B35) |
| `9d40163da` | `70fcc20c1` | Authenticate 12.0.7 assets and numeric invites; model delve titles (slice B: B01–B04 title) |
| `54d237a91` | — | INFERRED markers on three producers (review B finding F2) |
| `9ce93bb12` | — | Retired namespace members absent on ordinary lookup (review A blocking finding) |

## Integrator proof (worktree `p1207-r2`, default feature set only)

- A RED, producers withheld: `patch_12_0_7_removed_native_surface::` 12 pass / 18 fail. GREEN 30/0.
- B RED, producers withheld: `patch_12_0_7_b01_b04::` 3 pass / 10 fail (B03 bag total needed no producer change). GREEN 13/0.
- A controls GREEN: `spell_maw_powers::` 10, `group_verbs::` 11, `c_auto_complete_probes::` 1, `c_autocomplete::` 5, `startup_api_stubs::` 24, `global_frames::` 31, `c_map_api::` 51, `click_binding_spell_identifier::` 9, `blizzard_deprecated_loads::` 3, `admin_party_api::` 28, `click_targeting::` 22, `blizzard_deprecated_auto_complete_loads::` 3, `blizzard_auto_complete_realm_appending::` 1; lib `minimap_specialized::` 7, `auto_complete_defaults::` 3, `click_bindings_defaults::` 3, `spell_static_defaults::` 2.
- B controls GREEN: 34 narrow filters, including `bags::` 20, `c_item_api::` 100, `c_battle_net_probes::` 11, `delves_ui::` 1, `delves_api_inputs::` 9, `inventory_counts::` 15, `runtime_bootstrap_boundaries::` 46, `ui_file_assets::` 3.
- `cargo fmt --check` and `cargo check` exit 0 at `70fcc20c1`.
- Failing controls, unchanged from base `50d390688`: lib `startup_globals::test_patch_12_0_7_duration_objects_and_text_binding` (zero-span `HasExpired`), and three `c_api_surface::*_are_not_c_api_temporary_shims` source-substring assertions. Review A confirmed by reading the parent source that all four contradictions predate the round.
- Not run: strict 12.0.7 or older feature builds, so every `not(feature = "retail-12-0-7")` branch is unexecuted. Startup CLI was not run in the worktree.
- Excluded: world-tier difficulty candidate (row 030). Its none-default error would break cached `InstanceDifficulty.lua` / `DifficultyUtil_Shared.lua`.

## Review A — commit `e4fde2ceb` (GPT-6.1-sol, source read only): REJECT as submitted

- Blocking: `runtime_surface_bootstrap.lua` namespace `__index` fabricates a nil-returning function for any missing member, so `C_ClickBindings.MakeModifiers`, `C_ClickBindings.GetStringFromModifiers` and `C_Spell.GetMawPowerBorderAtlasBySpellID` stayed callable; the new tests checked only `rawget`. Rows 065–067 partial until fixed. **Fixed in `9ce93bb12`**: the three keys are recorded in `__wow_removed_keys`, and the absence helper now asserts ordinary lookup is nil, a call fails, and lookup publishes nothing.
- Cached forwarding tests execute the real, unmodified cached deprecation files; spies observe inputs, coercion and result tuples. No source-substring assertions.
- No previously live unrelated assertion was lost by epoch-splitting existing tests.
- Only six of the 17 global/namespace symbols had a producer to gate; the other eleven were already absent, so their historical availability on older epochs is not established.
- Rows 175–178 are forwarding statements, not removals; credit is for current cached forwarding only. Row 178 chronology (12.0.5 wrappers) unproved.
- No cached Blizzard consumer calls a removed native directly (static scan).

## Review B — commit `9d40163da` (GPT-6.1-sol, source read only): REJECT against the strict standard

- F1: `GetFileID` reads the bundled listfile catalog, and bags default to 16 slots, so neither row has an empty-default host state. **Not accepted by main as blocking**: the empty-default rule guards against fabricated data; the catalog is real data and the bag default predates the commit. Rows 049 and 028 are credited as bounded for "bundled catalog" and "seeded bag capacity" scope only.
- F2: INFERRED policies were marked in specs but not in three producers. **Fixed in `54d237a91`.**
- Confirmed: all arguments and extras are authenticated before validation or mutation in `GetFileID` and `InviteFriend`; no cached consumer depends on the old fabricated-friend behavior; the tests cannot pass against a constant; row 030 must stay pending.

## Row outcome to account after master confirmation

- Bounded: 064, 068–080, 121–126, 175–178 (default-build absence and current cached forwarding), 065–067 (after `9ce93bb12`), 049, 028, 029.
- Partial: `prose-undated-021`, 027 (request capture only; no delivery, rejection or events).
- Pending: 030.
