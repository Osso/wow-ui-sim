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
| Source-documented enums and gamepad constants | `5e2558264`, `b838d6917` | Publishes Forever `BattleNetFriendLevel`, `VisualAlertType`, and `CooldownViewerSound` with metadata, plus `Constants.GamepadActionBarConstants`; the second commit exposes the new C API module. Vendor-consumer regressions are committed but targeted GREEN evidence is pending. This does not implement interactive gamepad input. |
| Gamepad stick scripts | `72fc53d24`, `5570fe263`, `d5ad14601` | Adds canonical `OnGamePadStick` and the Forever Lua `OnGamepadStick` alias across script APIs, with generic `(stick, x, y)` dispatch; follow-ups load those handlers from XML/templates and correct template-handler array length. It does not enable host-gamepad delivery or establish native alias validation. |

## Proof

- The initial independent `require` verifier recorded formatting, default and Forever checks, ten module-loader tests, six Forever sandbox tests, and six default sandbox tests passing. See [addon module imports](specs/addon-module-imports.md).
- At `65139f909`, the rebuilt Forever sync completed `4,398/4,398` extraction. This proves manifest mapping and cache synchronization; one file used existing CDN recovery.
- The first `--no-addons --no-saved-vars lua-errors` capture after sync exited `1`, with 422 distinct records and 514 occurrences. It is preserved in [the Forever error baseline](baselines/wowforever-lua-errors.json).
- That baseline predates `5e26960b6`; it is failure evidence, not a current compatibility result.
- `0a354c0e0` math and `b6a6a7dad` table development proof each went RED (`0/1`) then GREEN (`1/1`) against real Forever consumers, with no warnings. `799389a6e` timed-signal proof went RED (`0/6`) then GREEN (`6/6`) including the real Forever `TimerUtil.lua` wrapper, with no warnings. `5e2558264`/`7f6297984` enum proof is GREEN (`3/3`) against API documentation plus AccountUtil/sound consumers. `0d7942e64` securecopy proof is GREEN (`2/2`) without warnings for graph structure and real AuraShared; consumer RED was not cleanly observable because of unrelated compile/prerequisite blockers. `5e2558264`/`b838d6917` gamepad-constants proof went RED (`0/2`) then GREEN (`2/2`) against real `InitActionButtons`; it does not cover hardware input. `72fc53d24`/`5570fe263`/`d5ad14601` gamepad-stick script binding and generic dispatch proof is GREEN (`2/2`) without warnings; it also does not cover hardware input. `4deca63f1` event proof was RED on an unregistered documented event, then GREEN (`2/2`); its concurrent-build warnings prevent treating it as final integration proof. Final independent verification remains pending.

## Pending fixes

These source-backed items were identified from the initial baseline but are not recorded here as completed compatibility:

- rerun startup after each causal group, then classify remaining loader, API, template, widget, atlas, and input-handler gaps;
- validate remaining gamepad behavior against real UI paths; generic stick dispatch does not establish host-gamepad input support.

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
