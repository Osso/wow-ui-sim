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
| Finite event registration | `4deca63f1`, `725abfb8b`, `25c6ac778` | Adds finite source-documented Forever acceptance for additional startup events. `725abfb8b` adds ten generated-documentation `LiteralName` values, and `25c6ac778` adds later Discord/social/ping/pet-training names; invented names remain rejected. This is registration acceptance, not producer-state modeling. |
| Timed signal maps | `799389a6e` | Extracts a `timed-signal-maps` capability shared by PTR 12.1.5 and Forever. It exposes the existing `C_Timer.NewTimedSignalMap` state, scheduling, and `TimerUtil.lua` consumer without enabling a retail epoch. |
| Table utilities | `b6a6a7dad` | Reuses existing table extensions for Forever so `TableUtil.lua` can publish its compatibility aliases. The real-source fixture is committed; final verification remains pending. |
| `securecopy` | `0d7942e64` | Moves the existing cycle-safe deep-copy compatibility helper from PTR bootstrap scope into a shared temporary workaround for PTR and Forever. Its tests cover nested/cyclic table independence and userdata identity; no metatable or taint-copy claim. |
| Texture atlases | `5cd134dcd` | Generates and selects exact `1.60.1.69913` DB2/listfile atlas data with provenance: 19,203 entries and 20,684 elements. It records 2,306 skips (1,560 missing-path rows, 55 FDIDs, 746 duplicates), avoiding fabricated rectangles; no full texture-coverage claim. |
| Source-documented enums and gamepad constants | `5e2558264`, `b838d6917`, `7f6297984` | Publishes Forever `BattleNetFriendLevel`, `VisualAlertType`, and `CooldownViewerSound` with metadata, plus `Constants.GamepadActionBarConstants`. Vendor-consumer regressions cover enum values and real action-button initialization; this does not implement interactive gamepad input. |
| Input interface style | `a0e6df424` | Adds Forever `InputDeviceInterfaceType`, `C_InputInterfaceStyle.GetCurrentStyle()`, finite transition-event handling, and callback dispatch state. Real `InputUtil.lua` initialization/transition fixtures pass; no host-gamepad delivery claim. |
| Social enums | `916acf5e8` | Publishes source-backed `ClubStreamType`, `BattleNetFriendTag`, and `RecentAlliesInteractionCategoryFilter` values/metadata for Forever. Real Communities sorting and API-documentation value checks pass. |
| Edit Mode enums | `5955650e8` | Publishes finite source-backed preset/system enums and metadata through a shared C API registration path. Development proof is GREEN (`3/3`) for actual Mainline presets, Shared setting display, and source equality across 15 namespaces; no retail epoch is enabled. |
| Cooldown categories | `413fcd1ce` | Publishes documented Forever cooldown-viewer category values 0–8 and metadata. The actual settings data provider is GREEN (`1/1`); this does not establish cooldown population or interactive settings behavior. |
| Regional name policy | `6f9877555`, `df74659ea` | Models Forever regional unique-name availability through existing player-identity state with an explicit simulator default of `false`. Actual Camelot `NameUtil` proof is GREEN (`2/2`); default-profile exclusion remains pending. |
| Texture metatable | `7da2b6028`, `343c4624b`, `f4800730c` | Exposes the real texture metatable helper required by `UnitFrameUtil.lua`. The actual consumer went RED then GREEN (`1/1`) for `__index` identity, type, and mutation; default-profile exclusion remains pending. |
| Roleset methods | `e3dfe88a3`, `1532a3cc8` | Shares existing `AddRoleset`, `GetRolesetNames`, `RemoveRoleset`, and `SetRolesets` with Forever while leaving the retail-only security method group unchanged. RED (`0/1`) then warning-free GREEN (`1/1`) covers membership, replacement, removal, clear, isolation, and metatable publication. |
| Inventory slot methods | `68989ffcf`, `25442752c` | Publishes existing `C_PaperDollInfo` inventory-slot methods and loads the required equipment-flyout data for Forever; development fixtures cover real equipment-button and namespace slot contracts. |
| Finite UI constants | `db875f554` | Publishes source-backed finite Forever values used by Minimap/Ping/Transmog/Gamepad possess-bar consumers. Focused constants and Minimap construction tests pass; this does not complete all UI state models. |
| Build-specific global strings | `f954fc665` | Adds 27,262 build-specific `GlobalStrings.csv` entries for Forever with provenance and generator. Real GameplaySettingsGroup/string restoration fixtures pass. |
| LFG lair category | deferred | `Constants.lua:482` is instrumented as the exact missing key (`LFG_CATEGORY_LAIR`), but the Forever cache only consumes the symbol and provides no value assignment. Existing retail 12.1 value `8` is not Forever evidence; no fix is claimed. |
| Aura XML widgets | `c20f26dad` | Shares existing AuraContainer, ManagedAuraContainer, and AuraButton XML/factory support with Forever. Nested schema and widget-behavior fixtures are GREEN (`2/2`); this is not a full vendor UnitFrame load claim. |
| Gamepad stick scripts | `72fc53d24`, `5570fe263`, `d5ad14601` | Adds canonical `OnGamePadStick` and the Forever Lua `OnGamepadStick` alias across script APIs, with generic `(stick, x, y)` dispatch; follow-ups load those handlers from XML/templates and correct template-handler array length. It does not enable host-gamepad delivery or establish native alias validation. |

## Proof

- The initial independent `require` verifier recorded formatting, default and Forever checks, ten module-loader tests, six Forever sandbox tests, and six default sandbox tests passing. See [addon module imports](specs/addon-module-imports.md).
- At `65139f909`, the rebuilt Forever sync completed `4,398/4,398` extraction. This proves manifest mapping and cache synchronization; one file used existing CDN recovery.
- The first `--no-addons --no-saved-vars lua-errors` capture after sync exited `1`, with 422 distinct records and 514 occurrences. It is preserved in [the Forever error baseline](baselines/wowforever-lua-errors.json).
- Pinned batch-two startup at `725abfb8b` rebuilt warning-free, then exited `1` with **141 records / 161 occurrences**. Artifacts: `/tmp/forever-ui-batch2-build.json`, `/tmp/forever-ui-batch2-startup.stdout`, and `/tmp/forever-ui-batch2-startup.stderr`. The isolated follow-up diagnostic identified the `Constants.lua` table-key failure as `LFG_CATEGORY_LAIR` at line 482, but no Forever source assignment has been found, so the attempted fix was reverted. Clean batch-three startup at `b8c9d99fd` rebuilt warning-free and fell to **137 records / 155 occurrences**; artifacts: `/tmp/forever-ui-batch3-build.stdout`, `/tmp/forever-ui-batch3-startup.stdout`, and `/tmp/forever-ui-batch3-startup.stderr`. It still exits `1`.
- The earlier 334 / 496 startup capture predates later causal fixes. A separate 222 / 290 diagnostic ran against a concurrently rebuilt, scope-unknown binary and remains non-acceptance evidence. New finite-constants and global-string focused tests are warning-free GREEN (`2/2` each); full startup has not yet been rerun after `f954fc665`/`db875f554`.
- `5820b54e6` annotation routing proof is GREEN (`1/1` loader, `6/6` profile). `725abfb8b` verifies ten event names and `25c6ac778` extends the finite list; `68989ffcf` has targeted inventory-slot proof. These focused results do not establish a fresh full-startup count; the prior 334 / 496 startup capture reproduced their rejection, and only `INPUT_DEVICE_INTERFACE_TRANSITION` has current actual-InputUtil consumer proof (`2/2`). The remaining nine event paths await final verification. `6f9877555`/`df74659ea` name-policy proof is GREEN (`2/2`) through actual Camelot `NameUtil`; its concurrent roleset warning means it is not warning-free final evidence. `7da2b6028`/`343c4624b`/`f4800730c` texture-metatable proof went RED then GREEN (`1/1`) through actual `UnitFrameUtil.lua`; it has the same concurrent roleset-warning caveat. `5cd134dcd` atlas proof is GREEN (`1/1` real Lua cycle query and `1/1` generator fixture). `a0e6df424` input-style proof is GREEN (`2/2`), `916acf5e8` social-enum proof covers three documentation tables plus Communities sorting, `5955650e8` Edit Mode fixtures pass, `413fcd1ce` cooldown-category fixtures pass, `6f9877555` name-policy fixture passes, and `c20f26dad` Aura XML fixtures pass. `0a354c0e0` math and `b6a6a7dad` table development proof each went RED (`0/1`) then GREEN (`1/1`) against real Forever consumers, with no warnings. `799389a6e` timed-signal proof went RED (`0/6`) then GREEN (`6/6`) including the real Forever `TimerUtil.lua` wrapper, with no warnings. `5e2558264`/`7f6297984` enum proof is GREEN (`3/3`) against API documentation plus AccountUtil/sound consumers. `0d7942e64` securecopy proof is GREEN (`2/2`) without warnings for graph structure and real AuraShared; consumer RED was not cleanly observable because of unrelated compile/prerequisite blockers. `5e2558264`/`b838d6917` gamepad-constants proof went RED (`0/2`) then GREEN (`2/2`) against real `InitActionButtons`; it does not cover hardware input. `72fc53d24`/`5570fe263`/`d5ad14601` gamepad-stick script binding and generic dispatch proof is GREEN (`2/2`) without warnings; it also does not cover hardware input. `4deca63f1` event proof was RED on an unregistered documented event, then GREEN (`2/2`); its concurrent-build warnings prevent treating it as final integration proof. `a0e6df424` input-style proof is GREEN (`2/2`) through actual `InputUtil.lua` initialization and transition callbacks. `916acf5e8` social-enum proof is GREEN (`2/2`) through API-documentation value checks and actual Communities sorting. The latest recorded full startup before these commits remains exit `1`, 334 records / 496 occurrences; neither focused result establishes its post-commit count. Final independent verification remains pending.

## Pending fixes

These source-backed items remain open:

- Classify the remaining 137 / 155 startup failures at `b8c9d99fd`, then fix their demonstrated roots without masking cascades.
- Do not use the 222 / 290 concurrent-binary diagnostic as clean-HEAD acceptance.
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
