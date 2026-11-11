# Forever Login Startup

The Windows Forever login screen exposed missing profile data, missing API surfaces,
and loader ordering/provenance bugs. The initial rebuilt `wowf` 1.60.1.70634 capture
contained 38 unique error records and 58 occurrences, including duplicate error-handler
reports. Those captures used the actual profile cache with an explicitly allowed UI
version mismatch. The configured source target at the time was 1.60.1.70532.

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

## Login submission model (lt-a667, 2026-11-11)

The [login contract](../../specs/glue-login-flow.md) extends startup coverage to
original Retail and Forever account/password controls. `C_Login.Login` accepts
the actual password EditBox and clears it synchronously without retaining its
text. Regression cases hit-test the controls, type through normal key dispatch,
and submit with Enter or the original login button.

Accepting arbitrary credentials without network authentication is explicit
offline simulator policy. CONNECTING is synchronous; later update ticks advance
the connection and deliver a host roster. This is not a native network-event or
frame-timing contract. Original GlueParent and CharacterSelect Lua choose the
next screen.

Login uses an empty-by-default host roster, including before submission on
Retail/PTR/Forever, preserving empty and unresolved slots rather than borrowing
the social-account fixture. Explicit direct-screen boots retain historical
fixtures. Capacity uses separately supplied readiness and a signed limit; no
realm limit is invented. The empty-roster regression supplies readiness and
spare capacity. Forever fixtures select the real experience-preset CVar.

R46 removes Login exclusions for CharacterCreate, CharacterCustomize and
TimerunningCharacterCreate. Eligible addons and dependencies use the existing
startup loader, without separate preload, replay, teardown or vendor patch.
Ordering against GlueParent OnLoad and the first login event remains UNKNOWN.
The old headless settling helper also forced a character-select refresh during
Login; with an empty roster this incorrectly chooses charcreate before input.
That simulator-only refresh must remain separate from the submitted login flow.

Native evidence covers Retail 70688, Forever 70707 and specified PTR 70668
contracts. Cached Retail is 70575 and PTR is 70493; cached-UI compatibility does
not establish build-specific native conformance. The spec records inferred
scope, supplied-input boundaries and remaining unknowns.

R52 background lookup uses explicit host class/race field maps until the separate
real-data task populates them. Missing rows and present-zero fields remain
distinct; the verified missing-final-row fallback is 651555. R54 automatic boost
uses two independent optional session values behind existing protected wrappers.
CancelLogout and an explicit host realm-disconnect notification clear only boost
type; Logout and screen boot preserve both values.

Final acceptance is pending. Historical startup results below do not establish
that submission and empty-roster paths are green.

## Historical startup verification

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

## MSAA capability

Forever 1.60.1.70707 graphics settings call `MultiSampleAntiAliasingSupported()`
during registration. The missing global interrupted initialization, and the
incomplete settings addon caused session finalization to fail at close.

The UI renderer uses single-sample targets. A guarded Forever capability
returns one `false`, allowing the original settings code to finish without
offering MSAA. The regression checks the return value and registration of
`PROXY_MSAA` and `PROXY_MSAA_ALPHA`. Replace the temporary capability when
multisample targets and mode enumeration are available.
[Graphics settings regression](../../../tests/forever_login_catalog_defaults.rs)

In an earlier rebuilt Windows login capture with the fix, the controls responded,
the Lua error list was empty, and closing returned exit code 0. The persisted
CVar file retained its bytes, size and modification time.

## Native client visual verification (2026-11-02)

The installed Forever 1.60.1.70634 client displayed its login controls before
sign-in with no visible error dialog.

Credits displayed real text and two catalog entries: `World of Warcraft`, then
`Forever` (selected). The simulator's single offline entry remains a placeholder.

This visual check did not measure native Lua-error counts or API return values.
Fully disconnected operation was not verified.

## R60 display and capability replacement (2026-11-11)

The shared [GameMode display model](../../../src/c_api/game_mode_display.rs)
replaces the Forever nil-display placeholder. Both getters select the bounded
host override or the real currentGameMode CVar; rows are empty-by-default host
inputs. CVar callbacks clear the override and notify the original backdrop
consumer. The display-only record0 exclusion is separate from the screen-name
getter. No selected-realm data or native DB rows are invented.

[High-resolution availability](../../../src/c_api/video_texture_capability.rs)
replaces the mutable Forever fixture: PTR and Forever expose a read-only
capability-byte projection in Game and Glue; Retail does not publish it.
The zero byte is verified image data, while the completed false result is an
inference. Tests check publication and Boolean type, not fixed false.

At 380e9c1a2, Retail glue is 261 passed/1 failed and Forever is 148 passed/11
failed, with zero compiler warnings. The adapted Forever graphics consumer
passes. PTR's refined shared-state tests pass 2/2; its earlier full R60 run is
134/3. PTR's plain-login chat source/policy/visibility checks pass, but its final
all-errors guard exposes GetCreditsDisplayInfo and IsPBRWaterSupported, reported
for native contracts. The two other PTR failures are confirmed baseline.
Retail chat remains UNKNOWN pending R62: the earlier stale-pin explanation was
retracted because the installed 70783 root also lacks the split Game-only file.
No forced hiding, override, or replay is added. Forever failures remain the
baseline eight plus three submissions blocked by coordinator-owned APIs.

## Sources

- [Login contract](../../specs/glue-login-flow.md) — native evidence, offline policy and remaining unknowns.
- [Login model](../../../src/c_api/c_login.rs), [submission](../../../src/c_api/c_login/submission.rs), and [host roster](../../../src/c_api/c_login/character_roster.rs) — credential handling and supplied state.
- [Startup selection regressions](../../../src/loader/tests/screen_selection.rs) — eligibility and dependency ordering.

- [Glue regressions](../../../tests/glue_login.rs) — complete Login startup and screen-state transitions.
- [Module import implementation](../../../src/loader/addon_modules.rs) and [tests](../../../tests/addon_require.rs) — sibling-file loading and import boundaries.
- [Startup dependency implementation](../../../src/loader/mod.rs) — real customization initialization before feedback hooks.
- [Temporary defaults](../../../src/lua_api/workarounds/temporary/forever_glue_display_defaults.rs) and [credits catalog](../../../src/lua_api/workarounds/temporary/offline_glue_credits.rs).
- [HTML content nodes](../../../src/lua_api/frame/methods/text_attribute_event/text/simple_html.rs), [cursor state](../../../src/lua_api/globals/real/mouse_probes.rs), and [catalog consumer tests](../../../tests/forever_login_catalog_defaults.rs).
- [Texture toggle capability](../../../src/c_api/c_battle_net.rs) — reuse the existing CVar-backed producer for the complete Forever graphics consumer.
- [Video capabilities](../../../src/lua_api/workarounds/temporary/video_options_state.rs) and [UI pipeline](../../../src/render/shader/pipeline.rs) — unsupported MSAA policy for the single-sample renderer.

## See Also

- [[addon-module-imports]] — completed-module registry and logical import policy.
- [[client-profiles]] — profile-scoped sources and return-contract selection.
- [[forever-startup-gaps]] — separate game-screen startup investigation.
