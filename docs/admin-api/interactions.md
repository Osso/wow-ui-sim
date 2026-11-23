# Interactions

These simulator-only helpers drive player-interaction state and its normal manager events. They do not simulate server responses.

## Guild rename (Retail and WoW Forever)

```lua
A_Admin.OpenGuildRename()
```

This activates `Enum.PlayerInteractionType.GuildRename` and fires `PLAYER_INTERACTION_MANAGER_FRAME_SHOW`. The Blizzard PlayerInteractionFrameManager then opens the registered GuildRename frame. Deliver server results separately with `GUILD_RENAME_STATUS_UPDATE` when a test needs them.

## Mailbox

- `A_Admin.OpenMailbox()` activates `MailInfo` and fires the profile's mailbox-open event.
- `A_Admin.CloseMailbox()` clears `MailInfo` and fires the matching close event when the interaction was active.
