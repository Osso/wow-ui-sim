# Forever documented event and enum surface

Forever 1.60.1.69913 must accept `DISCORD_SERVER_LIST_UPDATE` and
`LEGACY_FRIEND_SYSTEM_STATUS_UPDATED` while retaining finite event validation.
Registration and unregistration must work; invented events remain rejected.
The documented GuildControl events `DISCORD_GUILD_LOBBY_UPDATE` and
`DISCORD_GUILD_SETTINGS_UPDATE`, and FriendsListTemplates event
`SOCIAL_UI_FRIENDS_LIST_SYSTEM_STATUS_UPDATED`, must also register.
`NEW_MATCHMAKING_PARTY_INVITE`, consumed by FriendsFrame, already registers
through the common event surface and must not be duplicated in the Forever list.

Forever exposes `CustomAuraButtonDispelTypeTextureStyle`: Border=0,
BorderWithIcon=1, Icon=2, PreserveAsset=3, CustomAsset=4; metadata is
MinValue=0, MaxValue=4, NumValues=5. The unmodified
`Deprecated_12_1_0.lua` must construct its AuraButtonBorderStyle aliases.

Forever exposes CurioRarity Common=1, Uncommon=2, Rare=3, Epic=4,
EpicTier2=5, with metadata MinValue=1, MaxValue=5, NumValues=5.
Existing other-profile publication remains unchanged; no retail epoch is enabled.

Sources: matching cached Blizzard_APIDocumentationGenerated files
DiscordDocumentation.lua, FriendListDocumentation.lua,
AuraContainerSharedDocumentation.lua, and DelvesConstantsDocumentation.lua.

Proof scope: grouped integration tests in `tests/wowforever_batch5_surface.rs`.
These establish simulator publication and the actual deprecated alias consumer,
not native conformance or complete startup acceptance.
