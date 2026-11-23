# Kiosk session state

Retail and WoW Forever expose the native `Kiosk` namespace. The simulator now models the three Kiosk members used by selected FrameXML consumers from host-supplied session input; native kiosk creation and credential policy remain unknown.

## Content

- **VERIFIED (Retail and Forever pinned caches):** `Blizzard_APIDocumentationGenerated` contains no `Kiosk` or `C_Kiosk` declaration.
- **VERIFIED (Retail and Forever pinned caches):** `Blizzard_Kiosk/Kiosk.lua` assigns `ExpirationWarningSoundKit` and `CharacterData` to `Kiosk`; it does not create the namespace.
- **VERIFIED (Retail pinned TOC and Lua):** `Blizzard_GroupFinder_Mainline.toc` selects `Mainline/PVEFrame.lua` for `mainline`; `ToggleLFDParentFrame` returns without opening the LFD panel when `Kiosk.IsEnabled()` is true.
- **INFERRED (R88fd (native evidence pending)):** absent host input means `IsEnabled()` and `IsCompetitiveModeEnabled()` return false, and `GetKioskLoginInfo()` returns no values.
- **INFERRED (R88fd (native evidence pending)):** `A_Admin.SetKioskSession` supplies the two flags and an optional all-or-none set of three login strings. This is simulator scenario input, not native session creation.

The test [stub_o_navigation.rs](../../../tests/stub_o_navigation.rs) runs the original Retail Group Finder consumer with enabled and disabled supplied sessions and checks the resulting `PVEFrame` visibility. The default Kiosk namespace shim remains enabled only on profiles other than Retail and WoW Forever.

## Sources

- [Kiosk model](../../../src/c_api/kiosk.rs)
- [admin API contract](../../admin-api/kiosk.md)
- [Client profiles](../../specs/client-profiles.md) — profile-scoped runtime cache selection
- Pinned Retail and Forever cache: `Blizzard_Kiosk/Kiosk.lua` and `Blizzard_APIDocumentationGenerated`
- Pinned Retail cache: `Blizzard_GroupFinder_Mainline.toc` and `Blizzard_GroupFinder/Mainline/PVEFrame.lua`
- [Kiosk probe](../../../tests/stub_o_navigation.rs)

## See Also

- [[lua-api]] — runtime API and temporary workaround inventory
- [[addon-loading]] — TOC game-type selection
