//! Character stats, player state, world/instance state, and related helpers.

use std::collections::{HashMap, HashSet};

use crate::lua_api::game_data::AuraInfo;
use crate::lua_api::state_defaults::default_equipped_items;

use super::collections::{
    EquippedItem, GreatVaultActivity, HeirloomData, MailAttachment, MailMessage, MountData,
    PetData, PremadeListing, ToyData, TransmogAppearance, WarbandSceneData,
};

/// Computed character stats (base + gear).
#[derive(Debug, Clone, Default)]
pub struct CharacterStats {
    pub strength: f64,
    pub agility: f64,
    pub stamina: f64,
    pub intellect: f64,
    pub armor: i32,
    pub crit_rating: i32,
    pub haste_rating: i32,
    pub mastery_rating: i32,
    pub versatility_rating: i32,
    pub speed_rating: i32,
    pub avoidance_rating: i32,
    pub leech_rating: i32,
}

impl CharacterStats {
    /// Compute stats from base class stats + equipped item levels.
    /// Uses a simplified model: each equipped piece contributes stats proportional to ilvl.
    pub fn compute(equipped_items: &HashMap<i32, EquippedItem>, class_index: i32) -> Self {
        let mut stats = Self::base_stats();
        let total_ilvl = Self::total_equipped_ilvl(equipped_items);
        Self::apply_primary_stats(&mut stats, class_index, total_ilvl);
        Self::apply_secondary_and_armor(&mut stats, equipped_items, total_ilvl);
        stats
    }

    fn base_stats() -> Self {
        Self {
            strength: 120.0,
            agility: 100.0,
            stamina: 350.0,
            intellect: 100.0,
            armor: 1200,
            ..Self::default()
        }
    }

    fn total_equipped_ilvl(equipped_items: &HashMap<i32, EquippedItem>) -> f64 {
        equipped_items
            .values()
            .filter_map(|e| crate::items::get_item(e.item_id))
            .map(|item| item.item_level as f64)
            .sum()
    }

    /// Add primary stat from gear based on class (Str/Agi/Int).
    fn apply_primary_stats(stats: &mut Self, class_index: i32, total_ilvl: f64) {
        let primary_from_gear = total_ilvl * 1.2;
        let stamina_from_gear = total_ilvl * 1.8;
        match class_index {
            1 | 2 | 6 => stats.strength += primary_from_gear, // Warrior/Paladin/DK
            3 | 4 | 10 | 12 => stats.agility += primary_from_gear, // Hunter/Rogue/Monk/DH
            _ => stats.intellect += primary_from_gear,        // casters
        }
        stats.stamina += stamina_from_gear;
    }

    fn apply_secondary_and_armor(
        stats: &mut Self,
        equipped_items: &HashMap<i32, EquippedItem>,
        total_ilvl: f64,
    ) {
        let secondary = total_ilvl * 0.15;
        stats.crit_rating = secondary as i32;
        stats.haste_rating = (secondary * 0.9) as i32;
        stats.mastery_rating = (secondary * 1.1) as i32;
        stats.versatility_rating = (secondary * 0.7) as i32;
        let armor_slots = equipped_items
            .keys()
            .filter(|s| matches!(**s, 1 | 3 | 5 | 6 | 7 | 8 | 9 | 10))
            .count();
        stats.armor += (armor_slots as i32) * 4500;
    }

    pub fn crit_pct(&self) -> f64 {
        self.crit_rating as f64 / 180.0
    }
    pub fn haste_pct(&self) -> f64 {
        self.haste_rating as f64 / 170.0
    }
    pub fn mastery_pct(&self) -> f64 {
        self.mastery_rating as f64 / 130.0
    }
    pub fn versatility_pct(&self) -> f64 {
        self.versatility_rating as f64 / 205.0
    }
}

/// Player character state: identity, combat, power, health, buffs, spec.
#[derive(Debug, Clone, Copy, Default)]
pub struct SecondaryPowerState {
    pub current: i32,
    pub max: i32,
}

/// One map entry in a player's Mythic+ rating summary.
/// Mirrors `C_PlayerInfo.MythicPlusRatingMapSummary`.
#[derive(Debug, Clone)]
pub struct MythicPlusRatingMapSummary {
    pub challenge_mode_id: i32,
    pub map_score: f64,
    pub best_run_level: i32,
    pub best_run_duration_ms: i64,
    pub finished_success: bool,
}

/// Overall Mythic+ rating summary for the player.
/// Mirrors `C_PlayerInfo.MythicPlusRatingSummary`.
#[derive(Debug, Clone)]
pub struct MythicPlusRatingSummary {
    pub current_season_score: f64,
    pub runs: Vec<MythicPlusRatingMapSummary>,
}

/// Simulated player movement flags (all false = stationary).
#[derive(Debug, Clone, Default)]
pub struct MovementState {
    pub moving: bool,
    pub mounted: bool,
    pub flying: bool,
    pub falling: bool,
    pub swimming: bool,
}

/// Player character state: identity, combat, power, health, buffs, spec.
///
/// `Default` is derived: every field is its zero/empty value. The seeded
/// "level 70 paladin" preset lives in [`PlayerState::seeded`].
#[derive(Debug, Clone, Default)]
pub struct PlayerState {
    pub name: String,
    /// Simulator name policy; defaults false, not an inferred native default.
    pub regional_unique_names_enabled: bool,
    pub health: i32,
    pub health_max: i32,
    pub class_index: i32,
    pub race_index: usize,
    pub level: i32,
    pub sex: i32,
    pub power: i32,
    pub power_max: i32,
    pub power_type: i32,
    /// Secondary power pools keyed by Enum.PowerType value (e.g. Holy Power).
    pub secondary_powers: HashMap<i32, SecondaryPowerState>,
    pub in_combat: bool,
    pub is_resting: bool,
    pub money: i64,
    pub item_level: f32,
    pub equipped_items: HashMap<i32, EquippedItem>,
    pub stats: CharacterStats,
    pub pvp_enabled: bool,
    pub honor_level: i32,
    pub buffs: Vec<AuraInfo>,
    pub movement: MovementState,
    pub active_spec_index: i32,
    pub pending_spec_change: Option<i32>,
    /// Mail inbox.
    pub inbox: Vec<MailMessage>,
    /// Items attached to outgoing mail (12 slots max).
    pub send_mail_items: [Option<MailAttachment>; 12],
    /// Money attached to outgoing mail (copper).
    pub send_mail_money: u64,
    /// COD amount on outgoing mail (copper).
    pub send_mail_cod: u64,
    /// True while the send-mail tab is selected.
    pub send_mail_showing: bool,
    /// True while the Open All Mail helper is processing inbox contents.
    pub opening_all_mail: bool,
    /// Counter for generating unique mail IDs.
    pub next_mail_id: u64,
    /// Current experience within the player's level. Drives `UnitXP("player")`.
    pub xp: i64,
    /// Experience required to ding the next level. Drives `UnitXPMax("player")`.
    pub xp_max: i64,
    /// True when the player is currently in their alternate form (e.g. worgen, druid).
    pub is_alternate_form: bool,
    /// True when the alternate form is the default/native form.
    pub alternate_form_is_default: bool,
    /// True when the player is eligible for the New Player Experience.
    pub is_npe_eligible: bool,
    /// True when the player is restricted by NPE (starter zone limits).
    pub is_npe_restricted: bool,
    /// True when the player is currently in the Returning Player Experience.
    pub is_in_rpe: bool,
    /// Mythic+ rating summary for the player. None = no rating data.
    pub mythic_plus_rating_summary: Option<MythicPlusRatingSummary>,
    /// True when the player is occupying a vehicle/possess slot. Drives
    /// `MainMenuBarVehicleLeaveButton` visibility together with `on_taxi`.
    pub in_vehicle: bool,
    /// True when the player is the *controlling* occupant of the vehicle —
    /// the case `UnitControllingVehicle("player")` reports.
    pub controlling_vehicle: bool,
    /// True when the override action bar / vehicle UI is currently shown.
    /// Drives `UnitHasVehicleUI` and `PossessActionBarMixin:Update`.
    pub has_vehicle_ui: bool,
    /// True when the player is on a flight master taxi route.
    pub on_taxi: bool,
    /// Latched true after the player clicks the leave button while on taxi.
    /// Surfaced for tests to assert that `TaxiRequestEarlyLanding` fired.
    pub taxi_early_landing_requested: bool,
    /// Skin/style identifier for the player's current vehicle UI. `None`
    /// surfaces as the empty string — the sentinel `ActionBarController`
    /// uses to swap between `MainActionBar` and a skinned `OverrideActionBar`.
    pub vehicle_skin: Option<String>,
}

impl PlayerState {
    /// Seeded "level 70 paladin" preset used to bootstrap a fresh `Sim`.
    /// All other fields keep their `Default`-derived zero/empty values.
    pub fn seeded() -> Self {
        let equipped_items = default_equipped_items();
        let stats = CharacterStats::compute(&equipped_items, 2);
        Self {
            health: 100_000,
            health_max: 100_000,
            class_index: 2,
            level: 70,
            sex: 2,
            power: 100,
            power_max: 100,
            item_level: 615.0,
            equipped_items,
            stats,
            active_spec_index: 2,
            next_mail_id: 1,
            xp_max: 180_000,
            alternate_form_is_default: false,
            ..Self::default()
        }
    }
}

/// XP / honor / rest globals consumed by `Blizzard_ActionBar/Shared/ExpBar.lua`
/// and `Mainline/HonorBar.lua`. Default values mirror "level 70 paladin in a
/// rested inn, no Trial restrictions": no exhaustion XP, rest state 1
/// ("Rested", 1.5x multiplier), no honor banked, no trial cap. The basic
/// XP totals (`xp` / `xp_max`) live on [`PlayerState`] alongside health
/// because many existing readers (UnitXP / UnitXPMax / IsXPUserDisabled /
/// IsResting / UnitHonorLevel) already key off it.
#[derive(Debug, Clone)]
pub struct PlayerXpState {
    /// Rest XP banked at an inn, in raw XP. `None` = no rest XP, which
    /// matches `GetXPExhaustion()` returning nil rather than 0.
    pub exhaustion: Option<i64>,
    /// `GetRestState()` first return: 1 = Rested, 2 = Normal, etc.
    pub rest_state: i32,
    /// `GetRestState()` second return — localized state name.
    pub rest_state_name: String,
    /// `GetRestState()` third return — XP gain multiplier (1.5 while rested).
    pub rest_multiplier: f64,
    /// `IsPlayerAtEffectiveMaxLevel()`. Default false — the seeded paladin
    /// is at 70, not at the cap.
    pub is_max_level: bool,
    /// `UnitHonor("player")` — current honor within the level.
    pub honor: i32,
    /// `UnitHonorMax("player")` — honor required for the next level.
    pub honor_max: i32,
    /// `UnitTrialXP("player")` — capped XP shown while a trial-account
    /// level limit is active. Drives the `ExpBar:IsCapped` branch.
    pub trial_xp: i32,
    /// `UnitTrialBankedLevels("player")` — banked levels held for trial
    /// accounts that have not subscribed yet.
    pub trial_banked_levels: i32,
    /// `GetRestrictedAccountData()` first return — the level cap a
    /// limited account is locked at. 20 matches the historical Trial cap
    /// and the previous custom-stub return.
    pub restricted_level: i32,
    /// `GetRestrictedAccountData()` second return — copper cap.
    pub restricted_money: i64,
    /// `GetRestrictedAccountData()` third return — profession-skill cap.
    pub restricted_profession: i32,
    /// `GameLimitedMode_IsBankedXPActive()` — true while the trial cap is
    /// armed (the player has stopped earning XP toward the next level).
    pub banked_xp_active: bool,
    /// `GameLimitedMode_GetLevelLimit()` — the level the trial cap
    /// enforces. Defaults to the same value as `restricted_level`.
    pub level_limit: i32,
}

impl Default for PlayerXpState {
    fn default() -> Self {
        Self {
            exhaustion: None,
            rest_state: 1,
            rest_state_name: "Rested".to_string(),
            rest_multiplier: 1.5,
            is_max_level: false,
            honor: 0,
            honor_max: 0,
            trial_xp: 0,
            trial_banked_levels: 0,
            restricted_level: 20,
            restricted_money: 0,
            restricted_profession: 0,
            banked_xp_active: false,
            level_limit: 20,
        }
    }
}

/// A mirror timer (breath / exhaustion / feign death) currently active
/// on the player. Drives `GetMirrorTimerInfo` /
/// `GetMirrorTimerProgress`.
#[derive(Debug, Default, Clone)]
pub struct MirrorTimer {
    /// Timer name token ("BREATH", "EXHAUSTION", "FEIGNDEATH").
    pub name: String,
    /// Starting value when the timer was last reset.
    pub start_value: f64,
    /// Maximum value this timer counts toward.
    pub max_value: f64,
    /// Rate of change per second (negative = counting down).
    pub scale: f64,
    /// 0 when ticking, 1 when paused.
    pub paused: i32,
    /// Localised label shown on the HUD.
    pub label: String,
    /// Spell id associated with the timer (0 when none).
    pub spell_id: i32,
    /// Current progress reading. Drives `GetMirrorTimerProgress`.
    pub progress: f64,
}

/// World/instance state: zone, guild, collections, vault, loot.
///
/// `Default::default()` produces a fully-empty / zero-valued state. The
/// sim's seeded defaults (Stormwind zone, "Heroes of Azeroth" guild,
/// populated collections) live in `seeded_world_state` and are applied
/// by `SimState::Default`.
#[derive(Debug, Default, Clone)]
pub struct WorldState {
    pub zone_name: String,
    pub zone_id: i32,
    pub sub_zone_name: String,
    pub instance_name: String,
    pub instance_type: String,
    pub instance_difficulty: i32,
    pub instance_max_players: i32,
    pub in_instance: bool,
    /// Difficulty display string (e.g. "Normal", "Heroic", "Mythic").
    /// Drives the 4th return of `GetInstanceInfo`. Empty outside instances.
    pub instance_difficulty_name: String,
    /// `dynamicDifficulty` from `GetInstanceInfo` (6th return). `0` when
    /// the instance does not have dynamic scaling.
    pub instance_dynamic_difficulty: i32,
    /// Whether the instance dynamically scales its difficulty. 7th return
    /// of `GetInstanceInfo`.
    pub instance_is_dynamic: bool,
    /// Instance map id (mapID). 8th return of `GetInstanceInfo`.
    pub instance_id: i32,
    /// Current group size inside the instance. 9th return of
    /// `GetInstanceInfo`. Usually matches `instance_max_players` or 0.
    pub instance_group_size: i32,
    /// LFG dungeon id for the current instance, if queued via the Group
    /// Finder. 10th return of `GetInstanceInfo` (nilable).
    pub instance_lfg_dungeon_id: Option<i32>,
    /// Mirror timers (underwater breath, exhaustion, feign death).
    /// `GetMirrorTimerInfo(index)` reads by 1-based index;
    /// `GetMirrorTimerProgress(name)` reads by name.
    pub mirror_timers: Vec<MirrorTimer>,
    /// Whether an encounter is currently in progress (boss pull active).
    /// Drives `IsEncounterInProgress`.
    pub encounter_in_progress: bool,
    /// Whether the current area allows flying. Drives `IsFlyableArea`.
    pub flyable_area: bool,
    /// Whether the current instance is an arena battleground. Drives
    /// `IsBattlefieldArena`.
    pub battlefield_arena: bool,
    pub guild_name: Option<String>,
    pub guild_rank: Option<String>,
    pub guild_num_members: i32,
    pub great_vault_activities: Vec<GreatVaultActivity>,
    pub great_vault_has_rewards: bool,
    pub great_vault_can_claim: bool,
    pub loot_rolls: HashMap<i32, LootRollInfo>,
    pub collected_transmogs: HashSet<i32>,
    pub transmog_appearances: Vec<TransmogAppearance>,
    pub transmog_collected_shown: bool,
    pub transmog_uncollected_shown: bool,
    pub transmog_all_factions_shown: bool,
    pub transmog_all_races_shown: bool,
    pub transmog_class_filter: i32,
    pub transmog_source_type_filters: HashSet<i32>,
    pub transmog_search_text: HashMap<i32, String>,
    /// Applied transmog per equipment slot: slotID → sourceID.
    pub applied_transmog_slots: HashMap<i32, i32>,
    pub collected_mounts: HashSet<i32>,
    pub mounts: Vec<MountData>,
    pub favorite_mounts: HashSet<u32>,
    pub active_mount_id: Option<u32>,
    /// Current mount journal search text. Drives displayed mount APIs.
    pub mount_search_text: String,
    pub collected_pets: HashSet<i32>,
    pub pets: Vec<PetData>,
    pub collected_toys: HashSet<i32>,
    pub toys: Vec<ToyData>,
    pub warband_scenes: Vec<WarbandSceneData>,
    pub favorite_toys: HashSet<u32>,
    pub heirlooms: Vec<HeirloomData>,
    pub collected_heirlooms: HashSet<u32>,
    pub earned_achievements: HashSet<i32>,
    pub premade_listings: Vec<PremadeListing>,
    /// Seeded PvP battleground info returned by `C_PvP.GetWorldPVPAreaInfo()`.
    pub world_pvp_areas: Vec<WorldPvpBattlegroundInfo>,
    /// Seeded holiday battleground info returned by `C_PvP.GetHolidayBGInfo()`.
    pub holiday_bg_info: Option<RandomBGInfo>,
    /// Seeded random battleground info returned by `C_PvP.GetRandomBGInfo()`.
    pub random_bg_info: Option<RandomBGInfo>,
    /// Seeded random epic battleground info returned by `C_PvP.GetRandomEpicBGInfo()`.
    pub random_epic_bg_info: Option<RandomBGInfo>,
    /// Player-managed battleground locklist returned by `C_PvP.GetLocklistMap()`.
    pub locklist_maps: Vec<u32>,
    /// Current zone's PvP type, returned by `C_PvP.GetZonePVPInfo()` as its
    /// first value. Canonical WoW tokens: `"contested"`, `"sanctuary"`,
    /// `"arena"`, `"friendly"`, `"hostile"`, `"combat"`. Default `"contested"`.
    pub pvp_type: String,
    /// Whether the current sub-zone applies its own PvP rules (e.g. a PvP
    /// district inside a contested zone). Returned as the second value.
    pub is_sub_zone_pvp: bool,
    /// For faction-locked zones, the faction whose PvP rules apply
    /// (`"Alliance"` / `"Horde"`); `None` on neutral zones. Third return.
    pub pvp_faction_name: Option<String>,
    /// Guild tabard crest — `GetGuildLogoInfo()` returns nine colour
    /// channels (background RGB, border RGB, emblem RGB) plus the emblem
    /// texture filename. All zeros + empty filename when no guild.
    pub guild_logo: GuildLogo,
    /// Guild ranks in display order (index 0 = rank 1). Empty when no
    /// guild. Drives `GuildControlGetNumRanks` / `GuildControlGetRankName` /
    /// `GuildControlGetRankFlags`.
    pub guild_ranks: Vec<GuildRank>,
    /// 1-based rank index currently "selected" by `GuildControlSetRank`.
    /// `GuildControlGetRankName()` / `GetRankFlags()` without an explicit
    /// index return the selected rank's fields. `0` = nothing selected.
    pub guild_selected_rank: i32,
    /// Club id exposed by `C_GuildInfo.GetClubId()`. WoW returns a string
    /// (Battle.net community id) or nil when the player has no guild.
    pub guild_club_id: Option<String>,
    /// Whether the player holds officer rank in their guild. Drives
    /// `C_GuildInfo.IsGuildOfficer()`. Default false (no guild).
    pub guild_is_officer: bool,
    /// Whether the player can speak in guild chat. `C_GuildInfo.CanSpeakInGuildChat()`.
    /// Default true — matches retail's "no explicit mute" baseline so addons
    /// that gate chat input on this probe don't silence themselves on startup.
    pub guild_can_speak_in_chat: bool,
    /// Guild Message of the Day. Empty string when not set. Drives
    /// `C_GuildInfo.GetMOTD()` and `GetGuildRosterMOTD()`.
    pub guild_motd: String,
    /// Guild long-form info / description text editable by officers via
    /// `C_GuildInfo.SetInfoText`. Drives `C_GuildInfo.GetInfoText()`.
    pub guild_info_text: String,
    /// Active weekly guild challenges. Drives `GetNumGuildChallenges()` /
    /// `GetGuildChallengeInfo(orderIndex)`. Each entry corresponds to one
    /// challenge type id (`1`=Dungeon, `2`=Raid, `3`=Rated BG, `4`=Scenario,
    /// `5`=Mythic+). The Communities Guild Info panel iterates
    /// `GUILD_CHALLENGE_ORDER = {1,4,2,3}` and looks up rows by type id.
    pub guild_challenges: Vec<GuildChallenge>,
    /// Guild members (names + 1-based rank indices + online state). Populated by
    /// `GuildInvite` / `GuildUninvite` / `GuildKick` / `GuildPromote`.
    /// Empty when the player has no guild.
    pub guild_members: Vec<GuildMember>,
    /// Guild chat messages sent at runtime via `C_Club.SendMessage`. Appended
    /// after the static seed messages exposed by `C_Club.GetMessagesBefore`.
    pub guild_chat_messages: Vec<GuildChatMessage>,
    /// Guild event log entries surfaced by `GetNumGuildEvents` /
    /// `GetGuildEventInfo`. Newest entries last; the Communities Guild Log
    /// panel iterates from the end. Empty when the player has no guild.
    pub guild_events: Vec<GuildEvent>,
}

/// A guild member: display name, 1-based rank index, and online state.
/// Rank 1 is highest (Guild Master). Higher values = lower rank.
#[derive(Debug, Clone)]
pub struct GuildMember {
    pub name: String,
    pub rank_index: i32,
    pub online: bool,
}

/// One entry of the guild event log returned by `GetGuildEventInfo(i)`:
/// `(type, player1, player2, rank, year, month, day, hour)`. `event_type` is
/// one of `"invite"`, `"join"`, `"promote"`, `"demote"`, `"remove"`, `"quit"`.
/// `player2` is the actor (e.g. the inviter) for `invite` / `promote` /
/// `demote` / `remove`; nil for `join` / `quit`. `rank_name` is only used for
/// `promote` / `demote`. Year is years-since-2000 to match retail.
#[derive(Debug, Clone)]
pub struct GuildEvent {
    pub event_type: String,
    pub player1: String,
    pub player2: Option<String>,
    pub rank_name: Option<String>,
    pub year: i32,
    pub month: i32,
    pub day: i32,
    pub hour: i32,
}

/// One row of `GetGuildChallengeInfo(orderIndex)`: returns
/// `(challengeType, current, max, gold, maxGold)`. `challenge_type` matches
/// `GUILD_CHALLENGE_TYPE`*N* / `GUILD_CHALLENGE_LABEL`*N* in `GlobalStrings`.
#[derive(Debug, Clone, Default)]
pub struct GuildChallenge {
    pub challenge_type: i32,
    pub current: i32,
    pub max: i32,
    pub gold: i32,
    pub max_gold: i32,
}

/// A guild chat message authored at runtime (typically by the player via
/// `C_Club.SendMessage`). The simulator assigns positions/epochs based on
/// the message's index in `WorldState::guild_chat_messages`.
#[derive(Debug, Clone)]
pub struct GuildChatMessage {
    pub author_member_id: i64,
    pub content: String,
}

/// Seeded `C_PvP.GetWorldPVPAreaInfo()` row.
#[derive(Debug, Clone, Default)]
pub struct WorldPvpBattlegroundInfo {
    pub bg_id: i32,
    pub can_enter: bool,
    pub can_queue: bool,
    pub is_active: bool,
    pub max_level: i32,
    pub min_level: i32,
    pub name: String,
    pub start_time: i32,
}

/// Seeded `C_PvP.GetHolidayBGInfo()` row.
#[derive(Debug, Clone, Default)]
pub struct RandomBGInfo {
    pub bg_id: i32,
    pub bg_index: i32,
    pub can_queue: bool,
    pub has_random_win_today: bool,
    pub max_level: i32,
    pub min_level: i32,
    pub name: String,
}

/// A macro slot. Matches the `/macro` addon view: name, icon, body text.
/// `icon` is the texture path passed to `EditMacro` (retail uses a mix
/// of texture ids and paths; the sim stores whatever the caller provides).
#[derive(Debug, Default, Clone)]
pub struct MacroInfo {
    pub name: String,
    pub icon: String,
    pub body: String,
}

/// A single guild rank row. `name` is the display name; `flags` is a bag of
/// arbitrary permission booleans — callers iterate and index by flag name
/// (WoW's real API returns a dense numeric-keyed table of flag values).
#[derive(Debug, Default, Clone, PartialEq)]
pub struct GuildRank {
    pub name: String,
    pub flags: Vec<bool>,
}

/// Guild tabard crest data returned by `GetGuildLogoInfo()`.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct GuildLogo {
    pub background: (f64, f64, f64),
    pub border: (f64, f64, f64),
    pub emblem: (f64, f64, f64),
    pub emblem_filename: String,
}

/// An active loot roll (group loot item pending a player decision).
#[derive(Debug, Clone)]
pub struct LootRollInfo {
    /// Unique roll identifier.
    pub roll_id: i32,
    /// Duration in seconds for the roll timer.
    pub roll_time: f64,
    /// Item texture path/ID.
    pub texture: String,
    /// Item display name.
    pub name: String,
    /// Stack count.
    pub count: i32,
    /// Item quality (0=Poor..4=Epic).
    pub quality: i32,
    /// Whether the item binds on pickup.
    pub bind_on_pickup: bool,
    /// Whether need roll is allowed.
    pub can_need: bool,
    /// Whether greed roll is allowed.
    pub can_greed: bool,
    /// Whether disenchant roll is allowed.
    pub can_disenchant: bool,
    /// Disenchant required skill level.
    pub disenchant_level: i32,
    /// Item level.
    pub item_level: i32,
    /// Item link string.
    pub item_link: String,
}

/// A saved equipment ("gear") set surfaced through `C_EquipmentSet`.
/// Mirrors the retail concept of a named loadout: a snapshot of which
/// item lives in each inventory slot, optional spec assignment, and a
/// per-slot "ignore on save" mask used by the EquipmentManager pane to
/// preserve currently-equipped items in slots the user wants left alone.
#[derive(Debug, Clone)]
pub struct EquipmentSet {
    /// Stable id assigned by `CreateEquipmentSet` and used as the
    /// `setID` return of `GetEquipmentSetInfo` and the argument to
    /// every `*EquipmentSet*` call. 1-based, monotonically increasing
    /// per `EquipmentManagerState.next_id`.
    pub id: i32,
    pub name: String,
    /// Texture path supplied to `CreateEquipmentSet` / `ModifyEquipmentSet`
    /// (e.g. `Interface\\Icons\\INV_Misc_QuestionMark`). Stored as the
    /// raw input string so we can return it verbatim — `SetTexture`
    /// accepts both string paths and numeric file ids.
    pub icon: String,
    /// 1-based talent spec index assigned via
    /// `AssignSpecToEquipmentSet`; `None` when unassigned.
    pub spec_index: Option<i32>,
    /// Inventory slot ids whose contents should be preserved on save
    /// (`IgnoreSlotForSave`). Stored on the set itself so the saved
    /// mask survives across selections — the global "pending save"
    /// scratch lives separately on `EquipmentManagerState`.
    pub ignored_slots: HashSet<i32>,
    /// Inventory-slot → packed item location for items captured by
    /// `SaveEquipmentSet`. Empty for freshly-created sets that
    /// haven't snapshotted yet.
    pub item_locations: HashMap<i32, i64>,
    /// Inventory-slot → item id snapshot consumed by `GetItemIDs`.
    pub item_ids: HashMap<i32, u32>,
}

/// Equipment-manager scratchpad backing the full `C_EquipmentSet`
/// surface. Ownership of the canonical set list, the global "pending
/// save" ignore mask, and the most-recently-used set id all live
/// here so `SimState` only needs one field.
#[derive(Debug, Clone, Default)]
pub struct EquipmentManagerState {
    pub sets: Vec<EquipmentSet>,
    /// Next id handed out by `CreateEquipmentSet`. Starts at 1.
    pub next_id: i32,
    /// Slots flagged via `IgnoreSlotForSave` for the *next* save. The
    /// EquipmentManager pane primes this from the selected set's
    /// stored mask via `IgnoreSlotsForSet`, then `SaveEquipmentSet`
    /// folds it back onto the set.
    pub ignored_slots_pending_save: HashSet<i32>,
    /// Last set id passed to `UseEquipmentSet`. Drives the
    /// `isEquipped` flag on `GetEquipmentSetInfo`.
    pub last_used_set_id: Option<i32>,
}

impl EquipmentManagerState {
    pub fn new() -> Self {
        Self {
            sets: Vec::new(),
            next_id: 1,
            ignored_slots_pending_save: HashSet::new(),
            last_used_set_id: None,
        }
    }
}

#[cfg(test)]
#[path = "character_world_tests.rs"]
mod tests;

#[path = "character_world_defaults.rs"]
mod defaults;

pub use defaults::seeded_world_state;
