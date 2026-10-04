//! BattleNet / social friend list + summon-request state.

/// Seeded local character identity used by in-game player APIs and glue
/// character-list data. Keeping these aligned prevents UI like AddOnList from
/// showing a random current-player name that is not present in the account's
/// character list.
pub const SEEDED_LOCAL_CHARACTER_NAME: &str = "Uther";
pub const SEEDED_LOCAL_CHARACTER_GUID: &str = "Player-1-00000001";

/// One BattleNet friend entry. Drives `C_BattleNet.GetAccountInfoByGUID`,
/// `GetFriendAccountInfo`, `GetFriendNumAccounts`, and `GetNumFriends`.
/// The `game_accounts` vec holds all WoW (and other Blizzard game)
/// accounts attached to this BNet account that are currently online or
/// were recently seen.
#[derive(Debug, Clone)]
pub struct BnetFriend {
    /// 1-based index in the friends list (display order).
    pub friend_index: i32,
    /// Unique GUID for the BNet account (format: "BNet-0-<id>").
    pub bnet_account_guid: String,
    /// Numeric BNet account id (maps to `bnetAccountID` in the retail struct).
    pub bnet_account_id: i32,
    /// BattleTag display name (e.g. "Thrall#1234").
    pub battle_tag: String,
    /// Account name shown in the BNet friends list.
    pub account_name: String,
    /// Custom note set on this friend.
    pub note: String,
    /// Best-effort 12.1 title-friend custom display name.
    pub custom_title_friend_name: Option<String>,
    /// Best-effort 12.1 title-friend tags, exposed as `friendTags`.
    pub friend_tags: Vec<String>,
    /// Custom away message ("customMessage" in the retail struct).
    pub custom_message: String,
    /// Timestamp of the custom message (0 when not set).
    pub custom_message_time: i32,
    /// Whether this friend appears offline intentionally.
    pub appear_offline: bool,
    /// Whether this is a BattleTag friend (vs. RealID).
    pub is_battle_tag_friend: bool,
    /// `Enum.BattleNetFriendLevel` (1 BattleTag, 2 RealID, 3 Title).
    pub friend_level: i32,
    /// Whether this entry represents a real friend vs. a pending request.
    pub is_friend: bool,
    /// Whether the friend is marked as a favorite.
    pub is_favorite: bool,
    /// Whether the friend has AFK status set.
    pub is_afk: bool,
    /// Whether the friend has DND status set.
    pub is_dnd: bool,
    /// Last-online timestamp (0 when currently online or unknown).
    pub last_online_time: i32,
    /// RAF link type (0 = none).
    pub raf_link_type: i32,
    /// Nested game-account list (one per Blizzard game logged in).
    pub game_accounts: Vec<BnetGameAccount>,
}

/// One pending Battle.net friend invite. Best-effort backing for 12.1
/// `C_BattleNet.GetFriendInviteInfo` and `SendVerifiedBattleNetFriendInvite`.
#[derive(Debug, Clone)]
pub struct BnetFriendInvite {
    pub invite_id: i32,
    pub battle_tag: String,
    pub account_name: String,
    pub friend_level: i32,
    pub creation_timestamp: i64,
}

/// One Blizzard game account (WoW character, D3 account, etc.) attached
/// to a `BnetFriend`. Drives `C_BattleNet.GetGameAccountInfoByGUID`.
#[derive(Debug, Clone)]
pub struct BnetGameAccount {
    /// Unique GUID for this WoW/game account (format: "Player-<realm>-<id>").
    pub wow_account_guid: String,
    /// Numeric game account id (maps to `gameAccountID` in the retail struct).
    pub game_account_id: i32,
    /// Character name, or empty string when not in WoW.
    pub character_name: String,
    /// Realm name as shown in the UI.
    pub realm_name: String,
    /// Realm display name (may differ from `realm_name` in cross-realm contexts).
    pub realm_display_name: String,
    /// Numeric realm id.
    pub realm_id: i32,
    /// Character class id (1=Warrior … 13=Evoker); 0 when not applicable.
    pub class_id: i32,
    /// Class name string (e.g. "Paladin").
    pub class_name: String,
    /// Character level; 0 when not in WoW.
    pub character_level: i32,
    /// Current zone name.
    pub area_name: String,
    /// Whether this account is currently online.
    pub is_online: bool,
    /// Whether the game client is currently AFK.
    pub is_game_afk: bool,
    /// Whether the game client has DND set.
    pub is_game_busy: bool,
    /// Blizzard client program identifier (e.g. "WoW", "D3", "S2").
    pub client_program: String,
    /// Faction string: "Alliance", "Horde", or "" for neutral/non-WoW.
    pub faction_name: String,
    /// Race name string (e.g. "Human").
    pub race_name: String,
    /// Rich-presence text (game/activity description).
    pub rich_presence: String,
    /// Whether the player can be summoned.
    pub can_summon: bool,
    /// Whether this account is in the current region.
    pub is_in_current_region: bool,
    /// Whether this account has game focus.
    pub has_focus: bool,
    /// WoW project id (e.g. 1 for retail, 2 for classic).
    pub wow_project_id: i32,
    /// Timerunning season id (0 when not in a timerunning season).
    pub timerunning_season_id: i32,
    /// Numeric region id.
    pub region_id: i32,
    /// Player GUID string (character GUID, distinct from bnet/game account GUIDs).
    pub player_guid: String,
}

/// One WoW friends-list entry. Drives `C_Social.GetFriendInfo`,
/// `C_Social.GetFriends`, and `C_FriendList.GetNumFriends`.
/// Maps to the `C_FriendList.FriendInfo` retail structure.
#[derive(Debug, Clone)]
pub struct SocialFriend {
    /// Display name (character name or BattleTag).
    pub name: String,
    /// Character level.
    pub level: i32,
    /// Current zone/area.
    pub area: String,
    /// Class name (e.g. "Paladin").
    pub class_name: String,
    /// Player note set on this friend.
    pub note: String,
    /// Whether the friend is currently online.
    pub is_online: bool,
    /// Player GUID string (format: "Player-<realm>-<id>").
    pub guid: String,
}

/// Pending summon-request state. Drives `C_SummonInfo.*` and
/// `C_IncomingSummon.*`. Defaults to inactive (no active summon).
#[derive(Debug, Clone, Default)]
pub struct SummonRequestState {
    /// Whether a summon request is currently active.
    pub active: bool,
    /// Numeric summon reason code (see `Enum.SummonReason`).
    pub reason: i32,
    /// Time remaining on the summon confirmation timer, in milliseconds.
    pub time_left_ms: i32,
    /// Whether the summon skips the start experience flow.
    pub skips_start_experience: bool,
    /// Name of the player who initiated the summon.
    pub target_name: String,
}
