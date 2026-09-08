# PlayerSpells runtime load

## Summary

Retail `PlayerSpellsUtil.ToggleSpellBookFrame()` and `ToggleClassTalentFrame()` can demand-load `Blizzard_PlayerSpells` from keybindings. The simulator must preserve the active rilua call frame while `C_AddOns.LoadAddOn()` runs nested addon loads and `ADDON_LOADED`; otherwise the caller sees a bare `not a function` even though the addon finished loading. A live retail 12.1.0 capture (build 69497, interface 120100) also confirms that opening PlayerSpells replaces CharacterFrame through both direct and real toggle paths.

## Findings

- `C_AddOns.LoadAddOn("Blizzard_PlayerSpells")` reached `event Blizzard_PlayerSpells` but still raised after returning to the original Lua call. Preserving `top`, `base`, and `ci` around runtime addon loading keeps the native function ABI stable before pushing `LoadAddOn` return values.
- `PlayerSpellsFrame_LoadUI` is needed before the real addon is loaded, and the fallback should prefer modeled `C_AddOns.LoadAddOn` over `UIParentLoadAddOn`.
- The talents tab can show before some child OnLoad work has run. The temporary PlayerSpells backfill seeds ModelScene camera tables and PvP talent slot indices before showing the ClassTalents tab.
- PvP talent slot defaults must leave `selectedTalentID` nil for empty slots. Lua treats `0` as truthy, causing Blizzard code to query talent ID 0 and index a nil talent info record.

## Live retail panel contract (2026-08-28)

The live client capture targets WoW 12.1.0, build 69497, interface 120100. Both panel paths replace CharacterFrame with PlayerSpells:

- `ShowUIPanel(CharacterFrame)` followed by `ShowUIPanel(PlayerSpellsFrame)` leaves CharacterFrame closed and PlayerSpells open.
- `ToggleCharacter("PaperDollFrame")` followed by `PlayerSpellsUtil.ToggleSpellBookFrame()` leaves CharacterFrame closed and PlayerSpells open.
- Every recorded call in both scenarios, including reset hides, completed successfully (`ok = true`).
- CharacterFrame starts at width 338, expands to 540 when opened, and returns to width 338 when PlayerSpells replaces it.
- Captured non-nil `UIPanelWindows`/`UIPanelLayout-*` attributes: CharacterFrame `whileDead=1`, `pushable=3`; PlayerSpellsFrame `area="centerOrLeft"`, `pushable=3`, `whileDead=1`, `allowOtherPanels=1`, `checkFit=1`, `yoffset=75`.
- The probe recorded empty panel-slot tables. This is inconclusive: `GetUIPanel` belongs to the private `FramePositionDelegate`, not `UIParent`, so the probe's `UIParent:GetUIPanel(...)` lookup cannot establish active slot occupancy.

The production simulator panel behavior was unchanged. Commit `38bc75892` changed the test fixture to load `Blizzard_PlayerSpells` through dependency-aware `C_AddOns.LoadAddOn` rather than directly loading its TOC.

## Verification

- `keybind_n_loads_blizzard_player_spells_and_shows_talents`
- `raw_toggle_spellbook_frame_loads_blizzard_player_spells_and_shows_spellbook`
- `installs_pvp_talent_default_shapes`
- `installs_playerspells_util_bootstrap_defaults`

## 2026-08-27 SpellBook fallback completion

The global `ToggleSpellBook()` path must not treat a successful Lua call as proof that `PlayerSpellsUtil.ToggleSpellBookFrame()` handled the toggle. The temporary bootstrap wrapper can return `nil` without error while a failed `Blizzard_PlayerSpells` load leaves a partially initialized, hidden `PlayerSpellsFrame`; treating that return as handled skipped the simulator fallback and left `SimState.open_panels["SpellBook"]` closed in a bare environment.

The fix checks whether `PlayerSpellsFrame` reached the expected toggled visibility after the helper call. If not, `ToggleSpellBook()` uses its existing `open_panels`/frame-visibility fallback. Real full-UI helper behavior remains Blizzard-owned; the fallback covers only the helper no-op or failure path.

## Addon-configured Escape viewport (2026-09-08)

The user accepted the wider simulator window as the Escape acceptance viewport. With the real retail addon path and SavedVariables, resizing the owned Niri tile to 1920 pixels produced a 1906-unit canvas. `PlayerSpellsFrame` then fit the center panel (`1294.4` panel width), remained registered, and one native GUI `ESCAPE` IPC request left PlayerSpells, specialization, and GameMenu all hidden.

At the narrower 1266-unit canvas, PlayerSpells exceeds the 1186-unit center capacity. Blizzard `UpdateUIPanelPositions` clears the center slot at `UIParentPanelManager.lua:663`; Escape consequently has no panel to close and opens GameMenu. This remains unfixed. The observed callbacks include BlizzMove and EnhanceQoLMover scale-fit hooks; evidence does not establish either as the sole cause. No addon settings or vendor Lua were changed. Separate headless actual-addon startup returns `[]`, exit 0; the GUI interaction proof above is a distinct run. Three loader warnings remain, so this is not a zero-warning claim.

Final bounded verification passed `cargo fmt --check`, default `cargo check`, 63 focused default integration tests, two legacy forbidden-aspect library tests, and 26 PTR integration tests with one existing ignored snapshot-regeneration helper. A separate PTR no-addon/no-SavedVariables `lua-errors` run returned `[]`, exit 0. These checks preserve the relevant PTR boundaries; they do not prove PTR personal-addon startup or complete unsupported aura, secrecy, formatter, and curve semantics.

## Sources

- `/tmp/CharacterPlayerSpellsProbe-live-2026-08-28.lua` — live retail SavedVariables capture; SHA-256 `40dcf028acd5605d675810abbfc9eb8aa63147425ed5f8bb8b3b54b78c997595`
- `/tmp/pi-gui-accepted-panels.json` — current actual addon/SavedVariables GUI proof at the accepted wider viewport: native Escape closes specialization and SpellBook has 43 visible rows with 41 valid spells
- `/tmp/pi-gui-escape-fit-hook.json` — narrow-canvas fit and callback diagnostics
- `/tmp/pi-final-addon-verification.md` — final bounded default/PTR check matrix and limits
- `/tmp/pi-final-ptr-startup.*` — no-addon/no-SavedVariables PTR startup `[]`, exit 0
- [test_showuipanel_toggles.rs](../../../tests/test_showuipanel_toggles.rs) — commit `38bc75892` fixture using dependency-aware `C_AddOns.LoadAddOn`
- [panel_toggle_verbs.rs](../../../src/lua_api/globals/panel_toggle_verbs.rs) — SpellBook helper/fallback decision
- [panel_toggle_verbs.rs tests](../../../tests/panel_toggle_verbs.rs) — bare-environment regression coverage
- [player_spells_onload_backfill.rs](../../../src/lua_api/workarounds/temporary/player_spells_onload_backfill.rs) — temporary helper bootstrap

## See Also

- [[layout-system]] — canvas dimensions affect Blizzard panel fit
- [[lua-api]] — runtime panel and input surface
