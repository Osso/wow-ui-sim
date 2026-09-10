//! Collection / inventory / world-object data types.

pub use super::auction_house::{
    AuctionBrowseResult, AuctionItemClassFilter, AuctionReplicateItem, AuctionRowInfo,
    AuctionSellQuote, AuctionSellQuoteKind, AuctionSortSpec, BidAuction, BrowseQuery,
    CommodityPurchaseQuote, CommoditySearchResultInfo, CommoditySearchResults, ItemSearchKey,
    ItemSearchResultInfo, ItemSearchResults, OwnedAuction,
};

/// A Great Vault activity slot (one row/tier in the weekly rewards UI).
#[derive(Debug, Clone)]
pub struct GreatVaultActivity {
    /// WeeklyRewardChestThresholdType: 1=Activities, 2=Raid, 4=RankedPvP, 5=World.
    pub activity_type: i32,
    /// Slot index within the row (1-3).
    pub index: i32,
    /// Number of activities required to unlock this slot.
    pub threshold: i32,
    /// Current progress toward the threshold.
    pub progress: i32,
    /// Key level, boss difficulty, or rating.
    pub level: i32,
}

/// A mount in the mount journal.
#[derive(Debug, Clone)]
pub struct MountData {
    pub mount_id: u32,
    pub name: String,
    pub spell_id: u32,
    pub icon: u32,
    pub is_collected: bool,
    pub is_usable: bool,
    pub mount_type: u32,
}

/// A battle pet in the pet journal.
#[derive(Debug, Clone)]
pub struct PetData {
    pub pet_id: String,
    pub species_id: u32,
    pub name: String,
    pub icon: u32,
    pub pet_type: i32,
    pub level: i32,
    pub quality: i32,
    pub is_collected: bool,
}

/// A toy in the toy box.
#[derive(Debug, Clone)]
pub struct ToyData {
    pub item_id: u32,
    pub name: String,
    pub icon: u32,
    pub is_collected: bool,
    pub is_usable: bool,
}

/// A campsite/warband-scene entry in the collections journal.
#[derive(Debug, Clone)]
pub struct WarbandSceneData {
    pub warband_scene_id: u32,
    pub name: String,
    pub description: String,
    pub source: String,
    pub quality: i32,
    pub texture_kit: String,
    pub is_collected: bool,
    pub is_favorite: bool,
    pub has_fanfare: bool,
    pub source_type: i32,
}

/// An heirloom item in the collection.
#[derive(Debug, Clone)]
pub struct HeirloomData {
    pub item_id: u32,
    pub name: String,
    /// Equipment location string (e.g. "INVTYPE_HEAD", "INVTYPE_SHOULDER").
    pub equip_loc: String,
    /// Icon fileDataID.
    pub icon: u32,
    /// Current upgrade level (0 = base, max varies by expansion).
    pub upgrade_level: i32,
    /// Source description (e.g. "Vendor", "Achievement").
    pub source: String,
    /// Minimum effective level.
    pub min_level: i32,
    /// Maximum effective level at current upgrade.
    pub max_level: i32,
}

/// One chat bubble visible in the world (NPC speech, emote, say, etc.).
/// Drives `C_ChatBubbles.GetAllChatBubbles`. Since we don't render 3D
/// world chat bubbles the frame_id is advisory only — callers get a
/// lightweight table per bubble rather than a real Frame.
#[derive(Debug, Clone)]
pub struct ChatBubble {
    /// The message text shown in the bubble.
    pub message: String,
    /// Name of the unit speaking (e.g. "Thrall").
    pub sender: String,
    /// Chat type string (e.g. "SAY", "YELL", "EMOTE").
    pub chat_type: String,
    /// Frame id of the owning frame when one exists.
    pub frame_id: Option<u64>,
}

/// Metadata for an LFG category (Dungeons, Raids, etc.). Drives
/// `C_LFGInfo.GetLFGCategoryInfo(categoryID)` and
/// `C_LFGList.GetLfgCategoryInfo(categoryID)`. Seeded with standard
/// retail categories: 2=Dungeons, 3=Raids, 4=Arenas, 6=Custom, 9=Battlegrounds.
#[derive(Debug, Clone)]
pub struct LfgCategoryInfo {
    /// Display name shown in the Group Finder tab.
    pub name: String,
    /// Sort order within the category picker.
    pub order: i32,
    /// Whether recommended groups are shown in a separate section.
    pub separate_recommended: bool,
    /// Whether to prefer activities in the player's current area.
    pub prefer_current_area: bool,
    /// Whether cross-faction groups are allowed.
    pub allow_cross_faction: bool,
    /// Whether to automatically select an activity when entering.
    pub auto_choose_activity: bool,
    /// Whether to show the playstyle dropdown.
    pub show_playstyle_dropdown: bool,
}

/// An activity group (sub-grouping within a category).
/// Drives `C_LFGList.GetAvailableActivityGroups` and `GetActivityGroupInfo`.
#[derive(Debug, Clone)]
pub struct LfgActivityGroupInfo {
    pub group_id: u32,
    pub category_id: i32,
    pub name: String,
    pub order_index: i32,
    /// Bitmask matching Enum.LFGListFilter values (PvE=1, PvP=2, etc.).
    pub filters: u32,
}

/// An individual activity within a group.
/// Drives `C_LFGList.GetAvailableActivities` and `GetActivityInfoTable`.
#[derive(Debug, Clone)]
pub struct LfgActivityInfo {
    pub activity_id: u32,
    pub group_id: u32,
    pub category_id: i32,
    pub full_name: String,
    pub short_name: String,
    pub min_level: i32,
    pub max_players: i32,
    pub item_level: i32,
    pub filters: u32,
    /// Enum.LFGListDisplayType (0=ClassEnumerate, 1=ItemLevel, etc.).
    pub display_type: i32,
    pub order_index: i32,
    pub use_honor_level: bool,
    pub difficulty_id: i32,
    pub allow_cross_faction: bool,
    pub is_current_raid_activity: bool,
}

/// A dungeon entry for the LFD (Looking for Dungeon) panel.
/// Drives `GetLFGDungeonInfo`, `GetLFDChoiceOrder`, etc.
#[derive(Debug, Clone)]
pub struct LfdDungeonInfo {
    /// Negative ids are header rows; positive ids are dungeons.
    pub dungeon_id: i32,
    pub name: String,
    /// 1=Dungeon, 2=HeroicDifficulty, 6=Random.
    pub type_id: i32,
    pub subtype_id: i32,
    pub min_level: i32,
    pub max_level: i32,
    pub rec_level: i32,
    pub min_rec_level: i32,
    pub max_rec_level: i32,
    pub expansion_level: i32,
    pub group_id: i32,
    pub texture_filename: String,
    pub difficulty: i32,
    pub max_players: i32,
    pub description: String,
    pub is_holiday: bool,
    /// Exact group-size requirement for special queues. Values <= 1 represent
    /// normal Dungeon Finder entries that can fill missing roles.
    pub min_players: i32,
    pub map_name: String,
    pub min_gear: i32,
    pub is_scaling_dungeon: bool,
    /// Whether this entry shows in the random-dungeon dropdown.
    pub is_random: bool,
    /// Whether `C_LFGInfo.IsLFGFollowerDungeon` returns true.
    pub is_follower_dungeon: bool,
    /// Explicit simulator classification; no native Training Grounds IDs are seeded.
    pub training_ground_kind: Option<crate::c_api::c_pvp::TrainingGroundKind>,
}

#[derive(Debug, Clone)]
pub struct LfgRoleSelection {
    pub leader: bool,
    pub tank: bool,
    pub healer: bool,
    pub dps: bool,
}

impl Default for LfgRoleSelection {
    fn default() -> Self {
        Self {
            leader: false,
            tank: false,
            healer: false,
            dps: true,
        }
    }
}

/// Minimal area-POI metadata keyed by area poi id in
/// `SimState.area_pois`. Drives `C_AreaPoiInfo.GetAreaPOIInfo` and
/// `GetAreaPOISecondsLeft`. Only the subset of retail fields used by
/// Blizzard UI is carried; everything else is returned as nil /
/// default.
#[derive(Debug, Clone)]
pub struct AreaPoiInfo {
    pub area_poi_id: i32,
    pub name: String,
    /// UI-map id the POI is bound to. `None` when the POI is only
    /// looked up by id (the `uiMapID` arg to `GetAreaPOIInfo` is
    /// nilable too, so both sides can be unbound).
    pub ui_map_id: Option<i32>,
    /// Normalized 0..=1 screen position on the map.
    pub position: (f64, f64),
    pub atlas_name: Option<String>,
    pub description: Option<String>,
    pub faction_id: Option<i32>,
    pub icon_widget_set: Option<i32>,
    pub linked_ui_map_id: Option<i32>,
    pub is_current_event: bool,
    pub should_glow: bool,
    /// Seconds remaining until the POI expires. Drives
    /// `GetAreaPOISecondsLeft`. `None` for permanent POIs.
    pub seconds_left: Option<i32>,
}

/// Seeded achievement metadata keyed by achievement id in
/// `SimState.achievements`. Drives
/// `C_AchievementInfo.GetAchievementInfo`, `GetRewardItemID`, and
/// `IsValidAchievement`. Only the commonly-referenced few are
/// seeded; everything else returns `IsValidAchievement == false`.
///
/// `completed` / `was_earned_by_me` are derived from
/// `WorldState.earned_achievements` at read time, not stored on the
/// row, so `admin::mark_achievement_earned` flips the right bits
/// without having to touch this map.
#[derive(Debug, Clone)]
pub struct AchievementInfo {
    pub achievement_id: i32,
    pub name: String,
    pub points: i32,
    pub description: String,
    pub flags: i32,
    pub icon: i32,
    pub reward_text: String,
    pub is_guild: bool,
    pub is_statistic: bool,
    /// Item id rewarded when the achievement is earned; `None` when
    /// there is no item reward.
    pub reward_item_id: Option<i32>,
}

/// Snapshot of the comparison target's achievement progress, written
/// when `SetAchievementComparisonUnit` selects a friend/inspect target
/// and read by `GetAchievementComparisonInfo`,
/// `GetComparisonAchievementPoints`,
/// `GetComparisonCategoryNumAchievements`, and
/// `GetComparisonStatistic`. Empty by default — getters then return
/// "not earned" / 0 / nil so the comparison header desaturates and the
/// per-card friend column displays `INCOMPLETE`.
#[derive(Debug, Clone, Default)]
pub struct AchievementComparisonData {
    /// Achievement ids the comparison target has earned.
    pub earned: ::std::collections::HashSet<i32>,
    /// Per-achievement `(month, day, year)` completion timestamp the
    /// comparison frame renders via `FormatShortDate`.
    pub completion_dates: ::std::collections::HashMap<i32, (i32, i32, i32)>,
    /// Pre-formatted statistic display string per achievement id —
    /// powers `GetComparisonStatistic`.
    pub statistics: ::std::collections::HashMap<i32, String>,
}

/// Statistic display data keyed by achievement id in
/// `SimState.achievement_statistics`. Drives the legacy global
/// `GetStatistic(achievementID)`, which `AchievementFrameStats`
/// renders in the Summary statistic rows. Empty by default —
/// unseeded ids return `(nil, false)` and the tooltip's `"--"`
/// fallback kicks in client-side.
#[derive(Debug, Clone, Default)]
pub struct AchievementStatistic {
    /// Pre-formatted statistic display string (e.g. `"1234"` or
    /// `"1h 23m"`).
    pub quantity: String,
    /// True when the value is a numeric counter (vs. a derived
    /// time/string). Mirrors the second return of the legacy global.
    pub is_counter: bool,
}

/// Achievement search state in `SimState.achievement_search`. Drives
/// `SetAchievementSearchString` plus the four read-only getters
/// (`GetAchievementSearchProgress`, `GetAchievementSearchSize`,
/// `GetNumFilteredAchievements`, `GetFilteredAchievementID`) consumed
/// by `AchievementFrameSearchProgressBar_OnUpdate` and
/// `AchievementFrame_ShowSearchPreviewResults`. Default state is empty:
/// progress/size both 0 so the addon's `size > 0` guard short-circuits
/// the progress bar.
#[derive(Debug, Clone, Default)]
pub struct AchievementSearchState {
    /// Last query passed to `SetAchievementSearchString`.
    pub query: String,
    /// Server-side scan progress. Synchronous impl sets this equal to
    /// `size` so the progress bar fills immediately.
    pub progress: i32,
    /// Total work units the search needs to scan. Synchronous impl
    /// sets this to the result count.
    pub size: i32,
    /// Achievement ids matching the active query. Indexed 1..=N by
    /// `GetFilteredAchievementID(index)`.
    pub filtered_ids: Vec<i32>,
}

/// Reputation gating metadata keyed by achievement id in
/// `SimState.achievement_guild_rep`. Drives the legacy global
/// `GetAchievementGuildRep`, which the achievement tooltip calls to
/// decorate reputation-locked rows (e.g. Justicar) with the gating
/// faction's standing. The default `HashMap` is empty: ungated
/// achievements simply produce `(false, false, nil)`.
#[derive(Debug, Clone, Default)]
pub struct AchievementGuildRep {
    /// True when the achievement is gated behind a reputation level.
    pub requires_rep: bool,
    /// True when the player meets the gating threshold.
    pub has_rep: bool,
    /// Reputation level (`Standing` enum) required to unlock — `None`
    /// when `requires_rep == false`.
    pub rep_level: Option<i32>,
}

/// Per-map metadata keyed by ui-map id in `SimState.maps`. Drives
/// `C_Map.GetMapArtID`, `GetMapChildrenInfo`, and
/// `GetPlayerMapPosition`. Only the handful of ids commonly referenced
/// by Blizzard UI / addons are seeded (Azeroth world map, Eastern
/// Kingdoms continent, Stormwind City zone).
#[derive(Debug, Clone)]
pub struct MapData {
    pub ui_map_id: i32,
    pub name: String,
    /// `UIMapType` token (1=World, 2=Continent, 3=Zone, 4=Dungeon).
    pub map_type: i32,
    /// Parent map id in the hierarchy. `0` for the Cosmic root.
    pub parent_map_id: i32,
    /// Art tileset id returned by `GetMapArtID`. Non-zero for real
    /// zones; `0` for purely-logical maps (Cosmic).
    pub art_id: i32,
    /// `UIMapFlag` bitmask (0 = no flags).
    pub flags: i32,
    /// Direct children of this map in display order. Drives
    /// `GetMapChildrenInfo` (filtered by mapType when the caller
    /// passes one).
    pub child_map_ids: Vec<i32>,
    /// Normalized child rects in this map's coordinate space, used by
    /// `C_Map.GetMapInfoAtPosition` to resolve a point on this parent
    /// back to a leaf zone. Each rect's `(left, right, top, bottom)`
    /// uses Blizzard's UI convention where `top < bottom` (y grows
    /// downward, 0 = top edge). Empty for leaf maps.
    pub child_rects: Vec<MapChildRect>,
    /// Normalized rect describing where this map sits inside its
    /// `parent_map_id`'s coordinate space. `None` for root maps and
    /// for any map whose placement on its parent is not seeded.
    /// Drives `C_Map.GetMapRectOnMap`, which composes these rects up
    /// the parent chain to project the map onto a non-direct
    /// ancestor.
    pub rect_on_parent: Option<MapRect>,
}

/// One entry in `MapData.child_rects`. The child whose rect contains a
/// queried point is returned by `C_Map.GetMapInfoAtPosition`.
#[derive(Debug, Clone, Copy)]
pub struct MapChildRect {
    pub map_id: i32,
    pub left: f64,
    pub right: f64,
    pub top: f64,
    pub bottom: f64,
}

/// Normalized rect in some parent map's coordinate space. Used by
/// `MapData.rect_on_parent` to position a child within its direct
/// parent; `C_Map.GetMapRectOnMap` composes these up the chain.
/// Follows Blizzard's UI convention where `top < bottom` (y grows
/// downward, 0 = top edge).
#[derive(Debug, Clone, Copy)]
pub struct MapRect {
    pub left: f64,
    pub right: f64,
    pub top: f64,
    pub bottom: f64,
}

/// Per-currency info keyed by currency id in `SimState.currency_info`.
/// Drives `C_CurrencyInfo.GetCurrencyInfo`, `GetCurrencyInfoFromLink`,
/// and `GetCurrencyContainerInfo`. Matches the 25-field retail struct;
/// most fields default to 0 / false for simplicity.
#[derive(Debug, Clone)]
pub struct CurrencyInfo {
    pub currency_id: i32,
    pub name: String,
    pub description: String,
    pub icon_file_id: u32,
    pub quantity: i32,
    pub max_quantity: i32,
    pub quality: i32,
    pub is_header: bool,
    pub is_header_expanded: bool,
    pub is_show_in_backpack: bool,
    pub discovered: bool,
    pub can_earn_per_week: bool,
    pub max_weekly_quantity: i32,
    pub quantity_earned_this_week: i32,
    pub is_account_transferable: bool,
    pub is_account_wide: bool,
    pub is_tradeable: bool,
    pub is_type_unused: bool,
    pub currency_list_depth: i32,
    pub recharging_amount_per_cycle: i32,
    pub recharging_cycle_duration_ms: i32,
    pub total_earned: i32,
    pub tracked_quantity: i32,
    pub transfer_percentage: Option<f64>,
    pub use_total_earned_for_max_qty: bool,
}

impl Default for CurrencyInfo {
    fn default() -> Self {
        Self {
            currency_id: 0,
            name: String::new(),
            description: String::new(),
            icon_file_id: 0,
            quantity: 0,
            max_quantity: 0,
            quality: 0,
            is_header: false,
            is_header_expanded: false,
            is_show_in_backpack: false,
            discovered: true,
            can_earn_per_week: false,
            max_weekly_quantity: 0,
            quantity_earned_this_week: 0,
            is_account_transferable: false,
            is_account_wide: false,
            is_tradeable: false,
            is_type_unused: false,
            currency_list_depth: 0,
            recharging_amount_per_cycle: 0,
            recharging_cycle_duration_ms: 0,
            total_earned: 0,
            tracked_quantity: 0,
            transfer_percentage: None,
            use_total_earned_for_max_qty: false,
        }
    }
}

/// A transmog appearance source (one way to obtain a visual appearance).
///
/// WoW's transmog system has three levels:
/// - **Visual**: the actual look (shared across items with identical models)
/// - **Source**: a specific item that grants a visual (e.g. "Heroic Garrosh's Helmet")
/// - **Category**: equipment slot grouping (Head=1, Shoulder=2, ..., MainHand=12, etc.)
#[derive(Debug, Clone)]
pub struct TransmogAppearance {
    pub source_id: i32,
    pub visual_id: i32,
    pub category_id: i32,
    pub item_id: i32,
    pub is_collected: bool,
    /// Source type from Enum.TransmogSource (Boss Drop=1, Quest=2, Vendor=3, etc.)
    pub source_type: i32,
    /// Item modification ID (difficulty variant: Normal=0, Heroic=1, Mythic=3, etc.)
    pub item_mod_id: i32,
}

/// A premade group listing in the Group Finder.
///
/// Backs `C_LFGList.GetSearchResultInfo` and `GetSearchResultMemberCounts`.
/// The role counts (`tanks` + `healers` + `damagers` + `no_role`) should
/// equal `num_members` so `LFGListGroupDataDisplayEnumerate_Update` doesn't
/// run off the end of its icon array.
#[derive(Debug, Clone)]
pub struct PremadeListing {
    pub search_result_id: u32,
    pub name: String,
    pub comment: String,
    pub leader_name: String,
    pub activity_id: u32,
    pub num_members: i32,
    pub max_members: i32,
    /// VOIP server URL or short label; empty string when none.
    /// Blizzard's UI checks `voiceChat ~= ""` to decide whether to show
    /// the headphone icon, so a bool would never trigger the right path.
    pub voice_chat: String,
    pub auto_accept: bool,
    pub is_delisted: bool,
    /// Stable group identifier — looked up by the search panel's per-result
    /// decline cache (`LFGListFrame.declines[searchResultInfo.partyGUID]`).
    pub party_guid: String,
    pub tanks: i32,
    pub healers: i32,
    pub damagers: i32,
    pub no_role: i32,
    /// Per-role class breakdown: `classes_by_role["TANK"]["WARRIOR"] = 1`.
    /// Leave empty for activities that use the role-count display
    /// (`Enum.LFGListDisplayType.RoleCount`).
    pub classes_by_role: std::collections::HashMap<String, std::collections::HashMap<String, i32>>,
    /// `Enum.LFGEntryGeneralPlaystyle`: 0=None, 1=Learning, 2=FunRelaxed,
    /// 3=FunSerious, 4=Expert.
    pub general_playstyle: i32,
    pub cross_faction_listing: bool,
    /// `PLAYER_FACTION_GROUP` index of the leader: 0=Horde, 1=Alliance.
    pub leader_faction_group: i32,
    pub num_bnet_friends: i32,
    pub num_char_friends: i32,
    pub num_guild_mates: i32,
}

/// Player-side filter state for the Group Finder search panel,
/// returned by `C_LFGList.GetAdvancedFilter`.
///
/// The default (all-false / zero / empty) is permissive: every helper
/// the panel calls (`...DifficultyNoneChecked`, `...PlaystyleNoneChecked`)
/// returns true, so `EntryStillSatisfiesFilters` passes every search
/// result. Tests / admin verbs may flip individual fields to model a
/// player who has narrowed their search.
#[derive(Debug, Clone, Default)]
pub struct LfgAdvancedFilter {
    pub needs_tank: bool,
    pub needs_healer: bool,
    pub needs_damage: bool,
    pub needs_my_class: bool,
    pub has_tank: bool,
    pub has_healer: bool,
    pub minimum_rating: i32,
    /// `groupFinderActivityGroupID` allow-list. Empty means "any".
    pub activities: Vec<u32>,
    pub difficulty_normal: bool,
    pub difficulty_heroic: bool,
    pub difficulty_mythic: bool,
    pub difficulty_mythic_plus: bool,
    /// `Enum.LFGEntryGeneralPlaystyle` flags 1..4 (Learning, FunRelaxed,
    /// FunSerious, Expert).
    pub general_playstyle1: bool,
    pub general_playstyle2: bool,
    pub general_playstyle3: bool,
    pub general_playstyle4: bool,
}

/// A pending application the player has submitted to a premade listing.
///
/// Backs `C_LFGList.GetApplications` and `GetApplicationInfo`. Created by
/// `ApplyToGroup`. `start_time` is captured at submission so the panel's
/// `appDuration` countdown (`GetTime() + appDuration`) animates correctly.
#[derive(Debug, Clone)]
pub struct LfgApplication {
    pub application_id: u64,
    pub search_result_id: u32,
    /// Application status. Canonical values used by the panel:
    /// `"none"`, `"applied"`, `"invited"`, `"inviteaccepted"`,
    /// `"invitedeclined"`, `"declined"`, `"declined_full"`,
    /// `"declined_delisted"`, `"timedout"`, `"cancelled"`, `"failed"`.
    pub status: String,
    pub pending_status: Option<String>,
    pub start_time: f64,
    pub duration: f64,
    /// Selected role: `"TANK"`, `"HEALER"`, `"DAMAGER"`, or empty when
    /// none chosen.
    pub role: String,
}

/// An item attachment in a mail message.
#[derive(Debug, Clone)]
pub struct MailAttachment {
    pub item_id: u32,
    pub count: i32,
    pub quality: i32,
}

/// A mail message in the player's inbox.
#[derive(Debug, Clone)]
pub struct MailMessage {
    pub id: u64,
    pub sender: String,
    pub subject: String,
    pub body: String,
    pub money: u64,
    pub cod_amount: u64,
    pub items: Vec<MailAttachment>,
    pub days_left: f32,
    pub was_read: bool,
    pub was_returned: bool,
    pub can_reply: bool,
    pub is_gm: bool,
    pub stationery_icon: u32,
}

/// An item in a bag slot.
#[derive(Debug, Clone)]
pub struct BagItem {
    pub item_id: u32,
    pub stack_count: i32,
    pub hyperlink: Option<String>,
}

/// An equipped item in an inventory slot.
#[derive(Debug, Clone)]
pub struct EquippedItem {
    pub item_id: u32,
    pub enchant_id: u32,
    pub gem_ids: [u32; 3],
}
