# Clicked first-install saved variables

Clicked's two `ClickCastHeader` errors are a first-profile initialization defect in the addon, not evidence that the simulator lacks the frame. The errors appear without a Clicked saved profile and disappear when the current ClickedDB data loads.

## Content

### Runtime evidence

- **VERIFIED (supplied Retail full-addon run with SavedVariables disabled):** Clicked emitted two unique errors / two occurrences at `ClickCastFrames.lua:96`, one event record and one top-level record.
- **VERIFIED (Retail simulator run with the same addon root and current ClickedDB):** Clicked emitted zero errors; three unrelated errors remained.
- **VERIFIED (instrumented simulator run):** AceAddon received `ADDON_LOADED("Clicked")` with Clique absent and enable state 0. Clicked's `OnInitialize` ran, then failed at `Blacklist.lua:125` before its private `Addon.ClickCastHeader` was set. The later global `ClickCastHeader` named `DandersFramesClickCastHeader` belonged to DandersFrames.

### Source explanation

- **INFERRED (bundled AceDB and Clicked source; not native-client evidence):** after registering `OnNewProfile` to `InitializeDatabase` (`Core/Clicked.lua:344`), Clicked calls `UpgradeDatabase`; the upgrade first reads `Addon.db.profile.version` (`Core/Upgrader.lua:304`). AceDB lazily creates a missing profile and fires `OnNewProfile` (`AceDB-3.0.lua:211-215`), so `InitializeDatabase` calls `BlacklistOptions:Refresh()` (`Core/Database.lua:173-199`) before `BlacklistOptions:Initialize` sets local `config` (`Config/Blacklist.lua:74-81`). The refresh at line 125 fails and aborts `OnInitialize` before Clicked creates its header.

No simulator workaround is warranted. Do not use the later DandersFrames global as proof that Clicked created its own header.

## Sources

- `Clicked/Core/Clicked.lua`, `Clicked/Core/Database.lua`, and `Clicked/Config/Blacklist.lua` — Clicked callback order and the unset blacklist configuration.
- `Clicked/Libs/AceDB-3.0/AceDB-3.0.lua` — profile creation callback.
- `DandersFrames/ClickCasting/Frames.lua` — creates and publishes the distinct `DandersFramesClickCastHeader` global.
- [Mythic+ map request reentrancy](mythic-plus-map-request-reentrancy.md) — same-root error comparison that first recorded the Clicked errors.

## See Also

- [[addon-load-order]] — addon startup and dependency ordering
- [[mythic-plus-map-request-reentrancy]] — Clicked errors in the broader startup comparison
