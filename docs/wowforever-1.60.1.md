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

| Area | Commits | Verified result |
|---|---|---|
| Distinct profile | `abbc1272f` | Adds `WowForever`; it is not an Era or Anniversary alias. Selects the `wowforever` cache, `_classic_beta_` install path, Camelot game directory, and interface `16001`. |
| Source inventory | `f2619520c`, `d1ebc389e` | Adds the 4,398-file Forever manifest and maps `wowforever` to the Gethe `forever` source branch. |
| CASC mappings and cache sync | `65139f909` | Refreshing the community listfile added 592 generated path-to-FDID rows. Rebuilding then extracted all 4,398 manifest files from local `wow_classic_beta` CASC. |
| Loader-bound `require` | `be073174d`, `55c9b3d27`, `964358618`, `78cf08372`, `23928cfb7` | Forever-only `require` resolves completed addon Lua modules, retains values/provenance across GC, enforces direct TOC dependencies for disk callers, and never becomes filesystem or `package` loading. |
| Family TOC routing | `5e26960b6` | Corrects `[Family]` from `Classic` to `Mainline`; `[Game]` remains `Camelot`. This loads base `NineSliceLayouts`, `InputUtil`, and shared-panel definitions before Camelot overrides, while still excluding generic `mainline`-annotated TOC entries. |

## Proof

- The initial independent `require` verifier recorded formatting, default and Forever checks, ten module-loader tests, six Forever sandbox tests, and six default sandbox tests passing. See [addon module imports](specs/addon-module-imports.md).
- At `65139f909`, the rebuilt Forever sync completed `4,398/4,398` extraction. This proves manifest mapping and local CASC extraction only.
- The first `--no-addons --no-saved-vars lua-errors` capture after sync exited `1`, with 422 distinct records and 514 occurrences. It is preserved in [the Forever error baseline](baselines/wowforever-lua-errors.json).
- That baseline predates `5e26960b6`; it is failure evidence, not a current compatibility result.

## Pending fixes

These source-backed items were identified from the initial baseline but are not recorded here as completed compatibility:

- expose existing Forever-needed math and table utility registrations without enabling a retail API epoch;
- expose the existing timed-signal-map implementation for `C_Timer.NewTimedSignalMap`;
- publish source-documented Forever enums and five finite registerable events;
- expose the existing cycle-safe `securecopy` helper for Forever;
- rerun startup after each causal group, then classify remaining loader, API, template, widget, atlas, and input-handler gaps.

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
