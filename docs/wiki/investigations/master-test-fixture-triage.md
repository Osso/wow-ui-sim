# Master test fixture and profile triage

This investigation separates test-fixture failures from simulator behavior gaps in the exact master failures listed by lt-4d82. **VERIFIED (R88ak, Mists Classic 5.5.4.70631):** the current-mode getter contract is known. **INFERRED (Mists 5.5.4.70268):** applying it to the selected cache does not establish native parity.

## Content

### Retail original-consumer fixtures

- **VERIFIED (Retail 12.1.5.70783):** `Blizzard_AchievementUI.toc:6` selects the Load-on-Demand bootstrap, which defines `ToggleAchievementFrame`; the partial panel harness must load that original bootstrap before calling the global. The add-on body stays lazy until the consumer requests it. See [[showuipanel-toggle-fixture-owners]].
- **VERIFIED (Retail 12.1.5.70783):** `GameMenuFrame.lua:36` reads `GameMenuEscPriority`, supplied by `Blizzard_GameMenuEsc`; `Blizzard_AddOnList/AddonList.lua:32` reads `UIPanelWindows`, supplied by `Blizzard_UIParentPanelManager`. The AddOnList fixture must load these owners and `Blizzard_ManagedFrameSystem` before it loads AddOnList.
- **UNKNOWN (Retail 12.1.5.70783 native input):** the AddOnList consumer says C++ supplies a Group value for each row and calls `group:lower()`. A forced nil value probes the simulator's defensive source patch, not a verified native input; keep this stress probe optional.
- **VERIFIED (Retail 12.1.5.70783):** `Blizzard_SharedXMLBase.toc` includes `SecureTypes.lua`; `Blizzard_ChatFrameBase/Shared/ChatFrameFilters.lua:6-8` creates a secure array through it. A bare `WowLuaEnv::new()` does not load this Blizzard producer/consumer closure.

### Profile-specific Glue consumers

- **VERIFIED (Mists 5.5.4.70268 cache):** the selected Glue TOC loads `Classic/GlueGameLogo.lua`, whose startup and update handlers call `C_GameRules.GetCurrentGameModeRecordID()`.
- **VERIFIED (R88ak, Mists Classic 5.5.4.70631):** `GetCurrentGameModeRecordID` returns one value: the numeric `currentGameMode`, or one nil at -1; its startup default is -1. `AutoConnectToGameModeRealm` sets a mode-ID override, and a current-mode update clears it before a zero-payload `GAME_MODE_DISPLAY_INFO_UPDATED` event.
- **UNKNOWN (R88ak, Mists Classic 5.5.4.70631):** the runtime writer for `currentGameMode` was not identified.
- **INFERRED (Mists 5.5.4.70268 from 5.5.4.70631):** the branch registers the getter for Mists and reads the existing `currentGameMode` CVar host input, with -1 as the empty default. The Mists callback emits the no-payload event; the test exercises the selected Classic `GlueGameLogo.lua` consumer.
- **VERIFIED (simulator source):** no `AutoConnectToGameModeRealm` producer is registered, so the Mists getter does not use the modern-profile temporary override. The new original-consumer test's RED at commit `96087b880` reaches `GlueGameLogo.lua:6` and fails on the missing method.
- **VERIFIED (Mists 5.5.4.70268 cache):** the selected character-create XML uses `CharCreateBackButton`; its Classic CharacterSelect XML uses `CharacterSelectCharacterFrame` with a ScrollBar. Mainline tests that require `BackButton` or a ScrollBox do not describe these selected Mists consumers.
- **VERIFIED (Mists 5.5.4.70268 cache):** the cache has no `Blizzard_UIPanels_Game/Mainline/ReputationFrame.xml`; the Mainline reputation-dropdown test does not apply to Mists.
- **VERIFIED (Forever 1.60.1.70707):** `IsNameReservationOnly` reads a feature-status byte, and the selected Mainline CharacterSelect consumer checks it before allowing entry. The simulator's `deliver_status` input must update the byte read by the Forever getter.

### StoreGlue test scope

- **VERIFIED (Retail 12.1.5.70688):** R88q defines the public StoreGlue getters and request behavior; it does not establish a Lua-visible `_state` table.
- **UNKNOWN (Mists 5.5.4.70268):** the legacy `_state` bootstrap probe is not native evidence and stays optional; do not infer Mists StoreGlue parity from Retail R88q.

## Sources

- [Glue login contract](../../specs/glue-login-flow.md) — Forever status and the Mists current-mode record boundary.
- [Store Glue contract](../../specs/store-glue-r88q.md) — modeled public methods and fallback scope.
- [Panel fixture investigation](showuipanel-toggle-fixture-owners.md) — original bootstrap ownership.
- [Panel tests](../../../tests/addon_coverage.rs), [AddonList tests](../../../tests/addon_list_scroll.rs), [Shared UI tests](../../../tests/blizzard_shared.rs), [Glue consumer tests](../../../tests/glue_character_select.rs), and [secret-value tests](../../../tests/secret_value_security.rs).

## See Also

- [[consumer-selection-and-red-proof]] — prove original consumer selection before accepting a RED.
- [[showuipanel-toggle-fixture-owners]] — original achievement and housing panel owners.
