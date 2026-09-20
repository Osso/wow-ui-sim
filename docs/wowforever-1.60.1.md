# WoW Forever 1.60.1.69913

Running compatibility report for the authenticated `wow_classic_beta` build with interface `16001`. This records committed work and measured evidence only; pending work is updated incrementally rather than inferred from source presence.

## Scope

- Profile: `client-wowforever`
- Source branch: `forever`
- Runtime cache: `~/.cache/wow-ui-sim/blizzard-ui/wowforever/AddOns`
- Product: `wow_classic_beta`
- Build: `1.60.1.69913`
- Interface: `16001`

## Committed changes

| Area | Commits | Committed behavior / current proof |
|---|---|---|
| Distinct profile | `abbc1272f` | Adds `WowForever`; it is not an Era or Anniversary alias. Selects the `wowforever` cache, `_classic_beta_` install path, Camelot game directory, and interface `16001`. |
| Source inventory | `f2619520c`, `d1ebc389e` | Adds the 4,398-file Forever manifest and maps `wowforever` to the Gethe `forever` source branch. |
| CASC mappings and cache sync | `65139f909` | Refreshing the community listfile added 592 generated path-to-FDID rows. Rebuilding completed sync of all 4,398 manifest files; the verifier recorded one existing CDN recovery, so this is not an all-local-CASC claim. |
| Loader-bound `require` | `be073174d`, `55c9b3d27`, `964358618`, `78cf08372`, `23928cfb7` | Forever-only `require` resolves completed addon Lua modules, retains values/provenance across GC, enforces direct TOC dependencies for disk callers, and never becomes filesystem or `package` loading. |
| Family TOC routing | `5e26960b6`, `5820b54e6` | Corrects `[Family]` from `Classic` to `Mainline`, then corrects accepted inline tags to `camelot`/`mainline`; `[Game]` remains `Camelot`. This selects base definitions and source-present mainline entries before Camelot overrides. |
| Math utilities | `0a354c0e0` | Reuses the simulator's existing math extensions for Forever so `MathUtil.lua` can publish `Round` and related aliases. A real `MathUtil.lua` fixture covers the aliases and extension behavior; no retail API epoch is enabled. |
| Finite event registration | `4deca63f1` | Adds source-documented Forever acceptance and ordinary frame dispatch for `PET_STATS_UPDATE`, `SHARD_TRANSFER`, `SHARD_TRANSFER_IMMINENT`, `GUILD_PREFERRED_PLAY_SETTINGS_UPDATED`, and `HIDDEN_GROUP_BUFFS_CHANGED`; invented names remain rejected. |
| Timed signal maps | `799389a6e` | Extracts a `timed-signal-maps` capability shared by PTR 12.1.5 and Forever. It exposes the existing `C_Timer.NewTimedSignalMap` state, scheduling, and `TimerUtil.lua` consumer without enabling a retail epoch. |
| Table utilities | `b6a6a7dad` | Reuses existing table extensions for Forever so `TableUtil.lua` can publish its compatibility aliases. The real-source fixture is committed; final verification remains pending. |
| `securecopy` | `0d7942e64` | Moves the existing cycle-safe deep-copy compatibility helper from PTR bootstrap scope into a shared temporary workaround for PTR and Forever. Its tests cover nested/cyclic table independence and userdata identity; no metatable or taint-copy claim. |
| Texture atlases | `5cd134dcd` | Generates and selects exact `1.60.1.69913` DB2/listfile atlas data with provenance: 19,203 entries and 20,684 elements. It records 2,306 skips (1,560 missing-path rows, 55 FDIDs, 746 duplicates), avoiding fabricated rectangles; no full texture-coverage claim. |
| Source-documented enums and gamepad constants | `5e2558264`, `b838d6917`, `7f6297984` | Publishes Forever `BattleNetFriendLevel`, `VisualAlertType`, and `CooldownViewerSound` with metadata, plus `Constants.GamepadActionBarConstants`. Vendor-consumer regressions cover enum values and real action-button initialization; this does not implement interactive gamepad input. |
| Input interface style | `a0e6df424` | Adds Forever `InputDeviceInterfaceType`, `C_InputInterfaceStyle.GetCurrentStyle()`, finite transition-event handling, and callback dispatch state. Real `InputUtil.lua` initialization/transition fixtures pass; no host-gamepad delivery claim. |
| Social enums | `916acf5e8` | Publishes source-backed `ClubStreamType`, `BattleNetFriendTag`, and `RecentAlliesInteractionCategoryFilter` values/metadata for Forever. Real Communities sorting and API-documentation value checks pass. |
| Edit Mode enums | `5955650e8` | Publishes source-backed Forever Edit Mode preset/system enums and metadata through a shared C API registration path; real preset-layout/display fixtures pass. |
| Cooldown categories | `413fcd1ce` | Publishes the documented Forever cooldown-viewer category enum/metadata used by cooldown data providers. |
| Regional name policy | `6f9877555` | Models Forever regional unique-name availability through existing player-identity state; fixture covers vendor name-policy consumer. |
| Aura XML widgets | `c20f26dad` | Shares the existing aura XML widget capability with Forever; real AuraContainer/ManagedAuraContainer/AuraButton XML fixtures pass. |
| Gamepad stick scripts | `72fc53d24`, `5570fe263`, `d5ad14601` | Adds canonical `OnGamePadStick` and the Forever Lua `OnGamepadStick` alias across script APIs, with generic `(stick, x, y)` dispatch; follow-ups load those handlers from XML/templates and correct template-handler array length. It does not enable host-gamepad delivery or establish native alias validation. |

## Proof

- The initial independent `require` verifier recorded formatting, default and Forever checks, ten module-loader tests, six Forever sandbox tests, and six default sandbox tests passing. See [addon module imports](specs/addon-module-imports.md).
- At `65139f909`, the rebuilt Forever sync completed `4,398/4,398` extraction. This proves manifest mapping and cache synchronization; one file used existing CDN recovery.
- The first `--no-addons --no-saved-vars lua-errors` capture after sync exited `1`, with 422 distinct records and 514 occurrences. It is preserved in [the Forever error baseline](baselines/wowforever-lua-errors.json).
- That baseline predates `5e26960b6`; it is failure evidence, not a current compatibility result. Fresh startup captures fell from 334/496 to 248/344 after input-style/social fixes, then to **161 records / 183 occurrences** after Edit Mode, cooldown, name-policy, and Aura XML work. The latest run still exits `1`; its remaining roots include unit/equipment slot inputs, rolesets, social/ping state, cooldown secure state, gamepad override state, and downstream event/update initialization.
- `5820b54e6` annotation routing proof is GREEN (`1/1` loader, `6/6` profile). `5cd134dcd` atlas proof is GREEN (`1/1` real Lua cycle query and `1/1` generator fixture). `a0e6df424` input-style proof is GREEN (`2/2`), `916acf5e8` social-enum proof covers three documentation tables plus Communities sorting, `5955650e8` Edit Mode fixtures pass, `413fcd1ce` cooldown-category fixtures pass, `6f9877555` name-policy fixture passes, and `c20f26dad` Aura XML fixtures pass. `0a354c0e0` math and `b6a6a7dad` table development proof each went RED (`0/1`) then GREEN (`1/1`) against real Forever consumers, with no warnings. `799389a6e` timed-signal proof went RED (`0/6`) then GREEN (`6/6`) including the real Forever `TimerUtil.lua` wrapper, with no warnings. `5e2558264`/`7f6297984` enum proof is GREEN (`3/3`) against API documentation plus AccountUtil/sound consumers. `0d7942e64` securecopy proof is GREEN (`2/2`) without warnings for graph structure and real AuraShared; consumer RED was not cleanly observable because of unrelated compile/prerequisite blockers. `5e2558264`/`b838d6917` gamepad-constants proof went RED (`0/2`) then GREEN (`2/2`) against real `InitActionButtons`; it does not cover hardware input. `72fc53d24`/`5570fe263`/`d5ad14601` gamepad-stick script binding and generic dispatch proof is GREEN (`2/2`) without warnings; it also does not cover hardware input. `4deca63f1` event proof was RED on an unregistered documented event, then GREEN (`2/2`); its concurrent-build warnings prevent treating it as final integration proof. `a0e6df424` input-style proof is GREEN (`2/2`) through actual `InputUtil.lua` initialization and transition callbacks. `916acf5e8` social-enum proof is GREEN (`2/2`) through API-documentation value checks and actual Communities sorting. The latest recorded full startup before these commits remains exit `1`, 334 records / 496 occurrences; neither focused result establishes its post-commit count. Final independent verification remains pending.

## Pending fixes

These source-backed items remain open:

- Resolve remaining unit/equipment slot inputs, rolesets, social/ping state, cooldown secure state, gamepad override state, and downstream event/update initialization by causal family; do not patch downstream nils individually.
- Reconcile uncommitted generated/event/profile changes before treating the 161/183 capture as clean-HEAD acceptance.
- Rerun startup after each causal group and retain exact records/occurrences in the proof ledger.
- Validate remaining gamepad behavior against real UI paths; generic stick dispatch does not establish host-gamepad input support.

## Limitations

- No native WoW execution or native-conformance claim.
- A synchronized cache does not establish clean startup, panel interaction, rendering parity, gamepad input dispatch, or full UI compatibility.
- `GetBuildInfo()` date and trailing return fields retain existing temporary defaults.
- `require` preserves only the Warcraft Wiki contract covered by its [canonical spec](specs/addon-module-imports.md); unspecified native edge cases remain unmodeled.

## Canonical documents

- [Client profiles](specs/client-profiles.md) — supported profile contract and feature isolation.
- [Addon module imports](specs/addon-module-imports.md) — `require` behavior and exclusions.
- [Updating Blizzard UI](updating-blizzard-ui-to-a-new-patch.md) — manifest/listfile refresh and residual-miss workflow.
- [Profile baselines](baselines/README.md) — startup-baseline interpretation.
- [Client profile system](wiki/systems/client-profiles.md) — loader/cache architecture.
