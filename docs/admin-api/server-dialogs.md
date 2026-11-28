# Server dialogs (Retail and WoW Forever)

These inputs stand in for the server packets that open interaction dialogs. The
documented client requests answer them, change the modeled dialog and fire the
event the original Blizzard consumer listens to. Implementation:
`src/lua_api/globals/real/interaction_dialogs/` (feature `interaction-dialogs`).

### A_Admin.OpenItemText(item, material, creator, pages, isFullPage)

Opens readable item text and fires `ITEM_TEXT_BEGIN` then `ITEM_TEXT_READY`.
`item` is a nonempty string. `material` and `creator` are strings or nil
(`nil` material uses the Parchment page). `pages` is a nonempty array of
strings. Answers: `ItemTextGetItem`, `ItemTextGetMaterial`, `ItemTextGetCreator`,
`ItemTextGetText`, `ItemTextGetPage`, `ItemTextHasNextPage`, `ItemTextIsFullPage`.
`ItemTextNextPage` / `ItemTextPrevPage` move one page and fire `ITEM_TEXT_READY`.
`CloseItemText` ends the session and fires `ITEM_TEXT_CLOSED`.

### A_Admin.ReceiveGuildInvite(inviter, guildName, achievementPoints, oldGuildName, isNewGuild, joinRank, numMembers)

Fires `GUILD_INVITE_REQUEST(inviter, guildName, achievementPoints, oldGuildName,
isNewGuild)`. `joinRank` and `numMembers` are the membership the server grants.
`AcceptGuild` joins that guild and fires `PLAYER_GUILD_UPDATE`. `DeclineGuild`
drops the invite. Without an invite, both requests do nothing.

### A_Admin.OpenPetition(petitionType, title, bodyText, minSignatures, maxSignatures, originator, isOriginator, canSign, signatures)

Opens a `"guild"` or `"arena"` charter and fires `PETITION_SHOW`. `signatures`
is an array of signer names. Answers: `GetPetitionInfo`, `GetNumPetitionNames`,
`GetPetitionNameInfo(index)`, `CanSignPetition`. `ClosePetition` ends the
session and fires `PETITION_CLOSED`.

### A_Admin.RequestDuel(challengerName)

Stores the incoming request (shared with `AcceptDuel`) and fires
`DUEL_REQUESTED(challengerName)`. `CancelDuel` declines it and fires
`DUEL_FINISHED`. Without a request, `CancelDuel` does nothing.

### A_Admin.SetAreaSpiritHealer(secondsUntilWave, cemeterySelection)

A number puts the player in range of an area spirit healer whose next
resurrection wave is `secondsUntilWave` seconds away on the simulator clock, and
fires `AREA_SPIRIT_HEALER_IN_RANGE`. `nil` leaves range and fires
`AREA_SPIRIT_HEALER_OUT_OF_RANGE`. `AcceptAreaSpiritHeal` queues the player,
`CancelAreaSpiritHeal` leaves the queue. `GetAreaSpiritHealerTime` returns the
seconds left while queued and 0 otherwise. `IsCemeterySelectionAvailable`
returns `cemeterySelection` while in range.
