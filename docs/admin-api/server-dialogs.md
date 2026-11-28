# Server dialogs

These inputs stand in for the server packets that open interaction dialogs. The
documented client requests answer them, change the modeled dialog and fire the
event the original Blizzard consumer listens to. Implementation:
`src/lua_api/globals/real/interaction_dialogs/` (feature `interaction-dialogs`).

### A_Admin.OpenItemText(item, material, creator, pages, isFullPage) (Retail)

Opens readable item text and fires `ITEM_TEXT_BEGIN` then `ITEM_TEXT_READY`.
`item` is a nonempty string. `material` and `creator` are strings or nil
(`nil` material uses the Parchment page). `pages` is a nonempty array of
strings. Answers: `ItemTextGetItem`, `ItemTextGetMaterial`, `ItemTextGetCreator`,
`ItemTextGetText`, `ItemTextGetPage`, `ItemTextHasNextPage`, `ItemTextIsFullPage`.
`ItemTextNextPage` / `ItemTextPrevPage` move one page and fire `ITEM_TEXT_READY`.
`CloseItemText` ends the session and fires `ITEM_TEXT_CLOSED`.

### A_Admin.OpenPetition(petitionType, title, bodyText, minSignatures, maxSignatures, originator, isOriginator, canSign, signatures) (Retail)

Opens a `"guild"` or `"arena"` charter and fires `PETITION_SHOW`. `signatures`
is an array of signer names. Answers: `GetPetitionInfo`, `GetNumPetitionNames`,
`GetPetitionNameInfo(index)`, `CanSignPetition`. `ClosePetition` ends the
session and fires `PETITION_CLOSED`.

### A_Admin.RequestDuel(challengerName) (Retail)

Stores the incoming request (shared with `AcceptDuel`) and fires
`DUEL_REQUESTED(challengerName)`. `CancelDuel` declines it and fires
`DUEL_FINISHED`. Without a request, `CancelDuel` does nothing.

### `A_Admin.SetSecureTransferTradeOfferWarning(showWarning)` (Retail and WoW Forever)

Set the Boolean host input read by
`C_SecureTransfer.ShouldShowTradeOfferWarning` in the selected secure trade
 dialog. It reads `false` until supplied. This setter changes simulator state
only; it does not start a trade or fire an event.

The original `Blizzard_SecureTransferUI` test supplies distinct partners and
warning values. This does not establish native defaults.

## Instance countdowns (Retail and WoW Forever)

Countdowns run on the simulator clock (`A_Admin.SetManualClock` /
`A_Admin.AdvanceTime`). An expired or absent countdown reads `0`.

### A_Admin.SetInstanceBoot(seconds|nil)

`seconds` (finite, `>= 0`) starts the boot countdown and fires
`INSTANCE_BOOT_START`; `nil` clears it and fires `INSTANCE_BOOT_STOP`.
Answers `GetInstanceBootTimeRemaining()`.

### A_Admin.SetInstanceLock(seconds|nil, extending, enforce, encounters)

Starts the instance-lock countdown. `encounters` is an array of
`{ name = string, texture = string?, killed = bool?, ineligible = bool? }`
(`name` nonempty). `enforce = true` fires `INSTANCE_LOCK_START` (the timed
lock), `false` fires `INSTANCE_LOCK_WARNING`. `nil` seconds clears the lock and
fires `INSTANCE_LOCK_STOP`. Answers `GetInstanceLockTimeRemaining()`
(`timeLeft, extending, #encounters, #killed`) and
`GetInstanceLockTimeRemainingEncounter(index)` (`name, texture, killed,
ineligible`; nothing outside the list).
