# Forever Login Startup

The Windows Forever login screen exposed missing profile data, missing API surfaces,
and loader ordering/provenance bugs. The initial rebuilt `wowf` 1.60.1.70634 capture
contained 38 unique error records and 58 occurrences, including duplicate error-handler
reports. These captures use the actual profile cache with the explicitly allowed UI
version mismatch; the configured source target remains 1.60.1.70532.

## Causes and fixes

| Failure | Simulator correction |
| --- | --- |
| SocialUI `math.max(nil, 0.8)` | Add `userFontScaleGlue` to the Forever CVar catalog, using the existing modeled scale of `1.0`; original TextSizeManager continues to read the CVar store. |
| Rejected glue events | Accept the exact event names used by selected Forever glue consumers, without synthesizing event dispatch. |
| Missing regional-naming query | Project the existing player naming policy through `C_CharacterCreation.AreRegionalUniqueNamesEnabled`. |
| PTR feedback globals missing | Normalize explicitly included sibling Lua files into their physical addon module identity instead of rejecting their TOC paths. |
| Customization hooks/parent missing | Load CharacterCreate before PTRFeedbackGlue; its declared dependencies load CharacterCustomize and its original OnLoad attaches the real parent. Frames remain hidden on Login. |
| Missing realm/game-mode display queries | Use the approved temporary offline presentation defaults described below. |
| Missing credits and graphics capabilities | Use the approved offline catalog and unsupported-renderer capability defaults below. |
| Credits viewer sees an empty text-data array | Return the documented numeric `HTMLContentNode` rows from `SimpleHTML:GetTextData`, using the existing parser and retaining existing metadata fields. |
| SD/HD toggle and seen-choice queries missing | Expose the existing game-rule state and offered-choice seen tracker to Forever. |
| Credits close calls a missing cursor producer | Expose `ShowCursor` and `HideCursor` through the existing Forever cursor visibility model. This verifies Lua state, not OS cursor rendering. |

The prior `C_Login.GetState` correction makes Forever use the existing table-shaped
login state. The older scalar-return contract remains for profiles whose consumers
use it.

## Approved offline policy

The user explicitly approved offline simulator defaults after being told that native
catalog/capability inputs were unavailable. These are not native-default measurements:

- No cheat realm button, subregions, or superdistrict records; no special game-mode
  display record.
- High-resolution texture-pack availability defaults to false independently of the
  installed/enabled CVar. PBR water, outlines, secondary lighting and native world
  linear-lighting/Bloom mode are unsupported by the 2D renderer.
- One explicit offline credits entry and notice text, using a sentinel file ID and
  existing Classic artwork; no invented Blizzard credits.
- An empty preview-choice cleanup operation until a character preview model exists.

Defaults are Forever-only, raw-guarded temporary workarounds with retirement notes.
They should be replaced by real catalog/capability providers when those systems are
modeled. Ordinary graphics settings and the real naming/CVar backing remain active.

## Borrowed TOC file identity

The module registry normalizes the TOC owner plus relative file components from the
AddOns root. A leading `..` may select a sibling addon, but traversal above AddOns
and paths without an addon plus Lua filename remain rejected. Completed values and
delayed relative imports use the normalized source identity. Execution still keeps
the TOC owner's addon arguments, private table, taint, environment and dependency
metadata. Logical import checks are unchanged. Borrowed-source identity is simulator
policy, not native import-conformance evidence.

## Verification

The original isolated Windows Forever login capture returned `[]` (zero Lua
errors), down from 38 unique errors / 58 occurrences, with persisted CVars unchanged.

After review, the branch based on master `3d286838c` builds with the committed
dependencies using Cargo/rustc 1.99.1 and `sound,gui,casc,client-wowforever` (default
features disabled). All 55 focused integration tests and 14 loader-selection unit
tests pass, including complete zero-error Login startup, credits open/close,
settings, focus, screen transitions and the enabled high-resolution settings path.
The persisted CVar file's before/after hash, size and modification time matched.

Other profiles, the game-screen baseline and direct simulator GUI comparison were
not covered.

## Review follow-up

The high-resolution texture test originally stopped at an error sentinel before
the original graphics consumer could call
`C_BattleNet.CanToggleHighResTexturesWithoutClientReload`. The enabled capability
path therefore still had a missing Forever API. Expose the existing CVar-backed
producer to Forever and exercise the complete original setting registration,
including the returned proxy value and both toggle-capability states.

## Sources

- [Glue regressions](../../../tests/glue_login.rs) — complete Login startup and screen-state transitions.
- [Module import implementation](../../../src/loader/addon_modules.rs) and [tests](../../../tests/addon_require.rs) — sibling-file loading and import boundaries.
- [Startup dependency implementation](../../../src/loader/mod.rs) — real customization initialization before feedback hooks.
- [Temporary defaults](../../../src/lua_api/workarounds/temporary/forever_glue_display_defaults.rs) and [credits catalog](../../../src/lua_api/workarounds/temporary/offline_glue_credits.rs).
- [HTML content nodes](../../../src/lua_api/frame/methods/text_attribute_event/text/simple_html.rs), [cursor state](../../../src/lua_api/globals/real/mouse_probes.rs), and [catalog consumer tests](../../../tests/forever_login_catalog_defaults.rs).
- [Texture toggle capability](../../../src/c_api/c_battle_net.rs) — reuse the existing CVar-backed producer for the complete Forever graphics consumer.

## See Also

- [[addon-module-imports]] — completed-module registry and logical import policy.
- [[client-profiles]] — profile-scoped sources and return-contract selection.
- [[forever-startup-gaps]] — separate game-screen startup investigation.
