//! Small admin-facing SimState sub-structs — separated from
//! `state.rs` so the main `SimState` definition + default/new machinery
//! stays readable.
//!
//! Everything here is a data holder owned by `SimState`: the public
//! API surface for each struct is defined by `A_Admin.Set*` setters
//! and the corresponding `C_*`/global probes in `lua_api::globals`.

mod archaeology;
mod arrow_callouts;
mod gardenweald;
mod quest_log;
mod viewed_artifact;
mod wow_labs;

pub use archaeology::{ArchaeologyArtifact, ArchaeologyRace, ArchaeologyState, SelectedArtifact};
pub use arrow_callouts::{ArrowCalloutInfo, ArrowCalloutState};
pub use gardenweald::GardenwealdState;
pub use quest_log::{QuestLogEntry, QuestLogState, QuestRewardCurrency, QuestRewardItem};
pub use viewed_artifact::{
    ArtifactAppearanceInfo, ArtifactAppearanceSetInfo, ArtifactArtInfo, ArtifactPowerInfo,
    ColorRgb, MetaPowerEntry, RelicSlotInfo, ViewedArtifactState,
};
pub use wow_labs::{
    WowLabsAreaInfo, WowLabsCircleInfo, WowLabsDataManagerState, WowLabsMatchmakingState,
    WowLabsPartyInvite, WowLabsPartyMember, WowLabsPoint, WowLabsState,
};

use std::collections::{HashMap, HashSet};

/// Simulated network statistics returned by `GetNetStats()`.
///
/// WoW's real `GetNetStats` returns `(bandwidthIn, bandwidthOut, latencyHome,
/// latencyWorld)` in (kB/s, kB/s, ms, ms). The sim has no socket, so these are
/// purely a state knob — tests set values via the admin API to drive UI code
/// that renders latency/bandwidth indicators.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct NetStats {
    pub bandwidth_in_kbps: f64,
    pub bandwidth_out_kbps: f64,
    pub latency_home_ms: f64,
    pub latency_world_ms: f64,
}

/// Modifier-key down state. `IsModifierKeyDown()` returns true iff any of
/// shift/control/alt is held — matches real WoW's inclusive-or semantic,
/// excluding the meta key (meta tests via the dedicated `IsMetaKeyDown`).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ModifierKeys {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub meta: bool,
}

impl ModifierKeys {
    /// True iff shift, control, or alt is currently down. Does not include
    /// meta — WoW keeps that on its own `IsMetaKeyDown()` probe.
    pub fn any_modifier(&self) -> bool {
        self.shift || self.control || self.alt
    }
}

/// Mouse-button down state backing `IsMouseButtonDown([button])`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct MouseButtons {
    pub left: bool,
    pub right: bool,
    pub middle: bool,
    pub button4: bool,
    pub button5: bool,
}

impl MouseButtons {
    pub fn any_down(&self) -> bool {
        self.left || self.right || self.middle || self.button4 || self.button5
    }

    pub fn is_down(&self, button: Option<&str>) -> bool {
        let Some(button) = button else {
            return self.any_down();
        };
        match button.to_ascii_lowercase().as_str() {
            "leftbutton" | "left" => self.left,
            "rightbutton" | "right" => self.right,
            "middlebutton" | "middle" => self.middle,
            "button4" => self.button4,
            "button5" => self.button5,
            _ => false,
        }
    }

    pub fn set_down(&mut self, button: &str, down: bool) -> bool {
        match button.to_ascii_lowercase().as_str() {
            "leftbutton" | "left" => self.left = down,
            "rightbutton" | "right" => self.right = down,
            "middlebutton" | "middle" => self.middle = down,
            "button4" => self.button4 = down,
            "button5" => self.button5 = down,
            _ => return false,
        }
        true
    }
}

/// Backing state for the `C_GameRules` namespace. WoW's retail client
/// exposes a handful of named game rules (`"DISABLE_DUELS"`,
/// `"ALLOW_PING_PARTY_MEMBERS"`, etc.) that the UI queries to decide which
/// features to surface. Each rule has a float / int / string representation;
/// we store all three on one entry so a single rule can satisfy all three
/// getter variants without round-tripping strings.
#[derive(Debug, Clone, PartialEq)]
pub struct GameRulesState {
    /// Currently-active game mode id. Matches `Enum.GameMode`:
    /// `0 = Standard`, `1 = Plunderstorm`, `2 = Delves`, etc. Tests that don't
    /// care treat nonzero as "some non-standard mode".
    pub active_game_mode: i32,
    /// Glue screen name the current game mode opens on at the login flow.
    /// Default `"CharacterSelect"` (Standard).
    pub glue_screen_name: String,
    /// Sparse rule store keyed by rule name. Missing key = inactive.
    pub rules: HashMap<String, GameRuleValue>,
}

impl Default for GameRulesState {
    fn default() -> Self {
        Self {
            active_game_mode: 0,
            glue_screen_name: "CharacterSelect".into(),
            rules: HashMap::new(),
        }
    }
}

/// A single `C_GameRules` rule value. Stored as all three interpretations
/// (float/int/string) so each getter returns the "correct" form without a
/// parse step. Admin `A_Admin.SetGameRule(name, value)` fills the right
/// fields based on the Lua type passed in.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct GameRuleValue {
    pub as_float: f64,
    pub as_int: i64,
    pub as_string: String,
}

/// Pet-battle state backing `C_PetBattles.GetNumPets(owner)` /
/// `GetBattleState()`. WoW's `owner` argument is 1 (player) or 2 (enemy);
/// other values return 0. `battle_state` mirrors
/// `Enum.PetbattleState` — default 0 (`PVEInvitationSent` / "no active
/// battle"). Non-zero = some battle phase is active.
///
/// Extended fields back the full 15-method C_PetBattles surface: per-side
/// active-pet index, per-pet stat records, round timing, turn result, and
/// PvP matchmaking flag.
#[derive(Debug, Clone, PartialEq)]
pub struct PetBattleState {
    pub num_pets_player: i32,
    pub num_pets_enemy: i32,
    pub battle_state: i32,
    /// 1-based active pet slot for the player side.
    pub active_pet_player: i32,
    /// 1-based active pet slot for the enemy side.
    pub active_pet_enemy: i32,
    /// Pets on the player's side (up to 3).
    pub player_pets: Vec<PetBattlePet>,
    /// Pets on the enemy's side (up to 3).
    pub enemy_pets: Vec<PetBattlePet>,
    /// Milliseconds left in the current round (0 = no round active).
    pub round_time_left_ms: f64,
    /// Total duration of a round in milliseconds.
    pub round_time_ms: f64,
    /// Last-turn result code (0 = not set / in-progress).
    pub turn_result: i32,
    /// Whether StartPVPMatchmaking has been called without a cancellation.
    pub is_matchmaking: bool,
}

impl Default for PetBattleState {
    fn default() -> Self {
        Self {
            num_pets_player: 1,
            num_pets_enemy: 1,
            battle_state: 0,
            active_pet_player: 1,
            active_pet_enemy: 1,
            player_pets: vec![PetBattlePet::default_player()],
            enemy_pets: vec![PetBattlePet::default_enemy()],
            round_time_left_ms: 0.0,
            round_time_ms: 30_000.0,
            turn_result: 0,
            is_matchmaking: false,
        }
    }
}

/// Stats and ability list for a single pet in a pet battle.
#[derive(Debug, Clone, PartialEq)]
pub struct PetBattlePet {
    pub name: String,
    pub species_id: i32,
    pub level: i32,
    pub max_health: i32,
    pub current_health: i32,
    pub power: i32,
    pub speed: i32,
    /// Enum.BattlePetType — 1 = Humanoid, 2 = Dragonkin, …
    pub pet_type: i32,
    /// Ability IDs available to this pet (up to 6).
    pub ability_ids: Vec<i32>,
    /// Current XP for this pet.
    pub xp: i32,
    /// Max XP until next level.
    pub max_xp: i32,
}

impl PetBattlePet {
    pub fn default_player() -> Self {
        Self {
            name: "Squirrel".into(),
            species_id: 39,
            level: 1,
            max_health: 289,
            current_health: 289,
            power: 10,
            speed: 20,
            pet_type: 1,
            ability_ids: vec![110, 111, 112],
            xp: 0,
            max_xp: 100,
        }
    }

    pub fn default_enemy() -> Self {
        Self {
            name: "Rabbit".into(),
            species_id: 87,
            level: 1,
            max_health: 305,
            current_health: 305,
            power: 9,
            speed: 22,
            pet_type: 1,
            ability_ids: vec![120, 121, 122],
            xp: 0,
            max_xp: 100,
        }
    }
}

/// Hunter / warlock pet state — drives the four legacy pet-stat
/// probes (`GetPetExperience` / `GetPetHappiness` / `GetPetLoyalty` /
/// `GetPetTimeInCombat`).
///
/// Modern WoW no longer exposes happiness / loyalty for hunter pets
/// (they were removed in Cataclysm), so the sim defaults all four
/// fields to 0 / empty. Addons that still query these probes receive
/// retail's modern shape unless a test seeds the struct.
#[derive(Debug, Default, Clone)]
pub struct PetState {
    /// Current XP within the pet's level. Drives `GetPetExperience`
    /// (first return).
    pub xp: i32,
    /// XP required for the pet to ding. Drives `GetPetExperience`
    /// (second return).
    pub xp_max: i32,
    /// Classic-era happiness level (1 = Unhappy, 2 = Content,
    /// 3 = Happy). Drives `GetPetHappiness` (first return). 0 = no pet.
    pub happiness: i32,
    /// Classic-era damage percentage (75/100/125 depending on
    /// happiness). Drives `GetPetHappiness` (second return).
    pub damage_percent: i32,
    /// Classic-era loyalty gain rate (internal counter). Drives
    /// `GetPetHappiness` (third return).
    pub loyalty_rate: i32,
    /// Classic-era loyalty level as a localized string (e.g.
    /// `"Loyal"`). Drives `GetPetLoyalty`. Empty when no pet.
    pub loyalty_label: String,
    /// Seconds the pet has been in combat. Drives `GetPetTimeInCombat`.
    pub time_in_combat: i32,
}

/// A single chat / addon message entry in the simulator's outbound log.
/// `kind` is `"chat"` for `SendChatMessage` or `"addon"` for
/// `SendAddonMessage`.
#[derive(Debug, Clone)]
pub struct MessageLogEntry {
    pub kind: String,
    pub prefix: String,
    pub message: String,
    pub channel: String,
    pub target: String,
}

/// A member inside a simulated voice-chat channel.
#[derive(Debug, Clone)]
pub struct VoiceMember {
    pub member_id: i32,
    pub name: String,
    pub is_active_speaker: bool,
    pub volume: f64,
}

/// A simulated voice-chat channel with its member list.
#[derive(Debug, Clone)]
pub struct VoiceChannel {
    pub channel_id: i32,
    pub name: String,
    pub channel_type: i32,
    pub members: Vec<VoiceMember>,
}

/// Voice chat presentation state: volume sliders + mute/deafen flags.
/// Volumes are `[0.0, 1.0]`. `headset_mode` reflects whether the
/// headset-check confirmation dialog has been accepted.
#[derive(Debug, Clone)]
pub struct VoiceChatState {
    pub microphone_volume: f32,
    pub output_volume: f32,
    pub muted: bool,
    pub deafened: bool,
    pub headset_mode: bool,
    /// Whether voice chat is enabled in settings. Drives `IsVoiceEnabled`.
    /// Default true — retail ships with voice chat enabled by default.
    pub enabled: bool,
    /// Whether the player is actively using voice chat (connected and
    /// in a channel). Drives `IsUsingVoiceChat`. Default false.
    pub using: bool,
    /// Whether the voice-chat client is currently establishing a
    /// connection. Drives `VoiceChat_IsConnecting`.
    pub connecting: bool,
    /// Whether the local player is currently transmitting voice.
    /// Drives `VoiceChat_IsTalking`.
    pub talking: bool,
    /// Simulated channel list. Drives `GetChannels`, `GetChannel`, etc.
    pub channels: Vec<VoiceChannel>,
    /// ID of the currently active channel, or None. Drives `GetActiveChannelID`.
    pub active_channel_id: Option<i32>,
    /// Connection status code (0=Disconnected, 1=Connecting, 2=Connected).
    /// Drives `GetCurrentVoiceChatConnectionStatusCode`.
    pub connection_status: i32,
    /// Master volume scale `[0.0, 1.0]`. Drives `GetMasterVolumeScale`.
    pub master_volume_scale: f64,
    /// Whether parental controls have disabled voice chat.
    /// Drives `IsParentalDisabled`.
    pub is_parental_disabled: bool,
}

impl Default for VoiceChatState {
    fn default() -> Self {
        Self {
            microphone_volume: 1.0,
            output_volume: 1.0,
            muted: false,
            deafened: false,
            headset_mode: false,
            enabled: true,
            using: false,
            connecting: false,
            talking: false,
            channels: seeded_voice_channels(),
            // No active voice channel: a client that is not in one shows the
            // speaker icon on the chat channel button, and IsLoggedIn answers nil.
            active_channel_id: None,
            connection_status: 2,
            master_volume_scale: 1.0,
            is_parental_disabled: false,
        }
    }
}

fn seeded_voice_channels() -> Vec<VoiceChannel> {
    vec![VoiceChannel {
        channel_id: 1,
        name: "Party".to_string(),
        channel_type: 1,
        members: seeded_voice_members(),
    }]
}

fn seeded_voice_members() -> Vec<VoiceMember> {
    vec![
        VoiceMember {
            member_id: 1,
            name: "Player1".to_string(),
            is_active_speaker: true,
            volume: 1.0,
        },
        VoiceMember {
            member_id: 2,
            name: "Player2".to_string(),
            is_active_speaker: false,
            volume: 0.8,
        },
    ]
}

/// Single gossip option row matching `C_GossipInfo.GossipOptionUIInfo`.
/// Only the fields most commonly used by retail addons are carried here;
/// `rewards` is omitted (always empty array in the sim).
#[derive(Debug, Default, Clone)]
pub struct GossipOption {
    pub gossip_option_id: u32,
    pub order_index: u32,
    pub name: String,
    pub flags: u32,
    pub icon: u32,
    pub spell_id: Option<u32>,
    pub select_option_when_only_option: bool,
}

/// Single quest row used by both `GetActiveQuests` and `GetAvailableQuests`.
/// Matches `C_GossipInfo.GossipQuestUIInfo`.
#[derive(Debug, Default, Clone)]
pub struct GossipQuestRow {
    pub quest_id: u32,
    pub quest_info_id: u32,
    pub quest_level: i32,
    pub title: String,
    pub is_important: bool,
    pub is_legendary: bool,
    pub is_meta: bool,
    pub is_trivial: bool,
    pub is_ignored: bool,
    pub frequency: Option<i32>,
    pub is_complete: Option<bool>,
    pub repeatable: Option<bool>,
}

/// Active gossip-dialog state. Retail fires `GOSSIP_SHOW` when a
/// gossip window opens against an NPC and `GOSSIP_CLOSED` when the
/// player walks away. The sim exposes three count knobs that probes
/// return + an `active` flag so listeners know whether a dialog is
/// up (the `open` half of the event pair).
#[derive(Debug, Default, Clone)]
pub struct GossipState {
    pub active: bool,
    pub text: String,
    pub num_options: i32,
    pub num_available_quests: i32,
    pub num_active_quests: i32,
    pub options: Vec<GossipOption>,
    pub active_quests: Vec<GossipQuestRow>,
    pub available_quests: Vec<GossipQuestRow>,
}

/// Torghast (Jailer's Tower) run state.  Drives
/// `IsOnGroundFloorInJailersTower()` — returns true when `active` and
/// `floor == 1`.  Both default to false/0 (no active run).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TorghastState {
    pub active: bool,
    pub floor: i32,
}

/// Single reputation-tab row. Drives `GetFactionInfoByID(id)` and the
/// broader reputation window. `standing` is the 1-8 reputation tier
/// (1 Hated .. 8 Exalted). `bottom` / `top` bracket the current
/// standing's value range; `earned` is how far through that bracket
/// the player is. `at_war` / `is_watched` / `is_header` /
/// `is_collapsed` carry the flag bits that addons destructure.
#[derive(Debug, Default, Clone)]
pub struct FactionEntry {
    pub faction_id: u32,
    pub name: String,
    pub description: String,
    pub standing: i32,
    pub bottom: i64,
    pub top: i64,
    pub earned: i64,
    pub at_war: bool,
    pub can_toggle_at_war: bool,
    pub is_header: bool,
    pub is_collapsed: bool,
    pub has_rep: bool,
    pub is_watched: bool,
    pub is_child: bool,
    pub has_bonus_rep_gain: bool,
    pub can_be_lfg_bonus: bool,
}

/// Party loot-method state — drives `GetLootMethod()` and
/// `GetMasterLooterThreshold()`. `method` is the retail token
/// (`"group"`, `"master"`, `"freeforall"`, `"roundrobin"`,
/// `"needbeforegreed"`, `"personalloot"`). `party_master_index` /
/// `raid_master_index` are 1-based loot-candidate indices that matter
/// only when `method == "master"`; 0 means "no master assigned".
/// `threshold` is the master-loot item-quality threshold: 0 Poor ..
/// 4 Epic.
#[derive(Debug, Clone)]
pub struct LootMethodState {
    pub method: String,
    pub party_master_index: i32,
    pub raid_master_index: i32,
    pub threshold: i32,
}

impl Default for LootMethodState {
    fn default() -> Self {
        Self {
            // Retail's modern default is personal loot; classic-era
            // addons still probe GetLootMethod, so keep the token there.
            method: "personalloot".into(),
            party_master_index: 0,
            raid_master_index: 0,
            threshold: 2, // Uncommon — retail's default master-loot threshold.
        }
    }
}

/// Active trade window state. `None` means no trade in progress.
#[derive(Debug, Default, Clone)]
pub struct TradeState {
    /// Opponent's display name.
    pub target: String,
    /// Item ids in the 7 player trade slots (retail has 7 including the
    /// non-tradable slot). `0` means empty.
    pub player_slots: [u32; 7],
    /// Item ids in the 7 opponent trade slots.
    pub target_slots: [u32; 7],
    /// Copper offered by the player.
    pub player_money: u64,
    /// Copper offered by the opponent.
    pub target_money: u64,
    /// Whether the player has pressed Accept.
    pub player_accepted: bool,
    /// Whether the opponent has pressed Accept.
    pub target_accepted: bool,
}

/// A chat window's presentation + subscription state. Keyed by the
/// window's 1-based chat-frame index (e.g. ChatFrame1 → index 1).
/// Drives the `SetChatWindow*` / `AddChatWindowChannel` /
/// `GetChatWindow*` verb family.
#[derive(Debug, Clone)]
pub struct ChatWindow {
    pub font_size: f32,
    pub alpha: f32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub locked: bool,
    pub uninteractable: bool,
    /// Channels subscribed to this window, in insertion order.
    pub channels: Vec<String>,
    /// Channel membership mirror for idempotent insertion without scanning.
    pub channel_names: HashSet<String>,
    /// Message-type names this window receives (e.g. `"SAY"`, `"YELL"`).
    pub messages: Vec<String>,
}

impl Default for ChatWindow {
    fn default() -> Self {
        Self {
            font_size: 12.0,
            alpha: 1.0,
            r: 1.0,
            g: 1.0,
            b: 1.0,
            locked: false,
            uninteractable: false,
            channels: Vec::new(),
            channel_names: HashSet::new(),
            messages: Vec::new(),
        }
    }
}

/// A joined chat channel with its membership, moderators, and ban list.
/// Matches the shape retail addons expect when iterating the player's
/// channel list.
#[derive(Debug, Default, Clone)]
pub struct ChatChannel {
    /// Display name (e.g. `"General"`, `"Trade"`, a custom name).
    pub name: String,
    /// Channel members by display name. The local player is implicit.
    pub members: ::std::collections::BTreeSet<String>,
    /// Names granted moderator status in this channel.
    pub moderators: ::std::collections::BTreeSet<String>,
    /// Names banned from this channel — banning also removes membership.
    pub banned: ::std::collections::BTreeSet<String>,
}

/// Battlefield / arena / LFG queue state. A single slot models the
/// active queue — retail supports multiple concurrent slots, the sim
/// does not need them yet. `index` is the 1-based slot id callers
/// reference (e.g. `GetBattlefieldStatus(i)`); `name` is the
/// human-readable queue name for display.
#[derive(Debug, Default, Clone)]
pub struct BattlefieldQueue {
    pub status: BattlefieldStatus,
    pub index: i32,
    pub name: String,
}

/// Battlefield-queue status machine. Mirrors retail's
/// `GetBattlefieldStatus(index)` return strings — callers compare to
/// `"none" / "queued" / "confirm" / "active"`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum BattlefieldStatus {
    #[default]
    None,
    Queued,
    Confirm,
    Active,
}

impl BattlefieldStatus {
    /// Canonical WoW status string returned by `GetBattlefieldStatus`.
    pub fn as_wow_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Queued => "queued",
            Self::Confirm => "confirm",
            Self::Active => "active",
        }
    }
}

/// LFG-list counts backing `C_LFGList.GetNumApplications()` and
/// `GetNumApplicants()`. Each returns `(total, viewed)` — shape matters
/// because callers destructure both values in one statement.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct LfgListCounts {
    pub applications_total: i32,
    pub applications_viewed: i32,
    pub applicants_total: i32,
    pub applicants_viewed: i32,
}

/// Minimal keybinding store. In retail WoW the binding registry is
/// populated from `Bindings.xml` at load; here we only back the
/// *user-set* side (`SetBinding(key, action)` / `SetOverrideBinding`)
/// because the sim has no Bindings.xml to pre-register from. Overrides
/// shadow the base bindings during lookup, matching WoW's
/// `GetBindingAction(key, checkOverride=true)` semantics.
#[derive(Debug, Default, Clone)]
pub struct Keybindings {
    /// Insertion-ordered base bindings set by `SetBinding(key, action)`.
    /// Keyed by key name; the value is the bound action. Actions can be
    /// bound to at most 2 keys (WoW's documented limit).
    pub base: Vec<(String, String)>,
    /// Keys explicitly unbound by WTF `NONE` rows or `SetBinding(key, nil)`.
    /// These shadow simulator defaults during key dispatch.
    pub unbound: std::collections::HashSet<String>,
    /// Override bindings set by `SetOverrideBinding(owner, isPriority, key, action)`.
    /// Stored as a flat list so `ClearOverrideBindings` can drop the
    /// whole set.
    pub overrides: Vec<(String, String)>,
}

impl Keybindings {
    /// Return up to 2 keys currently bound to `action`. Overrides take
    /// precedence over base — an overridden key shadows its base entry.
    pub fn keys_for_action(&self, action: &str) -> (Option<String>, Option<String>) {
        let mut found: Vec<String> = self
            .overrides
            .iter()
            .filter(|(_, a)| a == action)
            .map(|(k, _)| k.clone())
            .collect();
        for (k, a) in &self.base {
            if a == action && self.overrides.iter().all(|(ok, _)| ok != k) {
                found.push(k.clone());
            }
        }
        let first = found.first().cloned();
        let second = found.get(1).cloned();
        (first, second)
    }

    /// Return the action bound to `key`, preferring an override. Empty
    /// string when unbound — WoW returns `""` (not nil) for
    /// `GetBindingAction`.
    pub fn action_for_key(&self, key: &str) -> String {
        if let Some((_, a)) = self.overrides.iter().rev().find(|(k, _)| k == key) {
            return a.clone();
        }
        self.base
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, a)| a.clone())
            .unwrap_or_default()
    }

    pub fn shadows_default_key(&self, key: &str) -> bool {
        self.unbound.contains(key) || self.base.iter().any(|(bound_key, _)| bound_key == key)
    }

    /// Bind `key` to `action`. An empty action unbinds `key`. A second
    /// key binding to the same action evicts the oldest key still bound
    /// (WoW's 2-keys-per-action limit).
    pub fn set(&mut self, key: &str, action: &str) {
        self.base.retain(|(k, _)| k != key);
        if action.is_empty() {
            self.unbound.insert(key.to_string());
            return;
        }
        self.unbound.remove(key);
        let bound: Vec<usize> = self
            .base
            .iter()
            .enumerate()
            .filter(|(_, (_, a))| a == action)
            .map(|(i, _)| i)
            .collect();
        if bound.len() >= 2 {
            self.base.remove(bound[0]);
        }
        self.base.push((key.to_string(), action.to_string()));
    }

    /// Install an override for `key` → `action`. Does NOT touch base.
    pub fn set_override(&mut self, key: &str, action: &str) {
        self.overrides.retain(|(k, _)| k != key);
        if !action.is_empty() {
            self.overrides.push((key.to_string(), action.to_string()));
        }
    }

    /// Drop every override; base bindings unaffected.
    pub fn clear_overrides(&mut self) {
        self.overrides.clear();
    }
}

/// Character-boost / trial service state.  Drives
/// `C_CharacterServices.GetActiveCharacterUpgradeBoostType` and
/// `C_CharacterServices.GetActiveClassTrialBoostType`.  Both default to
/// `None` (no active service).
#[derive(Debug, Default, Clone)]
pub struct CharacterServicesState {
    /// Active character-upgrade boost type id, or `None` when no boost
    /// purchase is pending.  Retail values: 5 = Level-60 boost, etc.
    pub active_upgrade_boost_type: Option<i32>,
    /// Active class-trial boost type id, or `None` when no trial is
    /// running.
    pub active_class_trial_boost_type: Option<i32>,
}
