//! Shared state types for the WoW Lua API.

use crate::cvars::CVarStorage;
use crate::event::{EventQueue, ScriptRegistry};
use crate::lua_api::animation::AnimGroupState;
use crate::lua_api::message_frame::MessageFrameData;
use crate::lua_api::simple_html::SimpleHtmlData;
use crate::lua_api::tooltip::TooltipData;
use crate::widget::WidgetRegistry;
use mlua::RegistryKey;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::time::Instant;

/// Information about the current target.
#[derive(Clone)]
pub struct TargetInfo {
    pub unit_id: String,
    pub name: String,
    pub class_index: i32,
    pub level: i32,
    pub health: i32,
    pub health_max: i32,
    pub power: i32,
    pub power_max: i32,
    pub power_type: i32,
    pub power_type_name: &'static str,
    pub is_player: bool,
    pub is_enemy: bool,
    pub guid: String,
}

/// A simulated party member.
pub struct PartyMember {
    pub name: &'static str,
    /// 1-based class index into CLASS_DATA.
    pub class_index: i32,
    pub level: i32,
    pub health: i32,
    pub health_max: i32,
    pub power: i32,
    pub power_max: i32,
    /// 0=MANA, 1=RAGE, 2=FOCUS, 3=ENERGY.
    pub power_type: i32,
    pub power_type_name: &'static str,
    pub is_leader: bool,
}

/// A pending timer callback.
pub struct PendingTimer {
    /// Unique timer ID.
    pub id: u64,
    /// When this timer should fire.
    pub fire_at: Instant,
    /// Lua function to call (stored in registry).
    pub callback_key: RegistryKey,
    /// For tickers: interval between firings.
    pub interval: Option<std::time::Duration>,
    /// For tickers with limited iterations: remaining count.
    pub remaining: Option<i32>,
    /// Whether this timer has been cancelled.
    pub cancelled: bool,
    /// The timer/ticker handle table (stored in registry) to pass to callback.
    pub handle_key: Option<RegistryKey>,
}

/// Information about a loaded addon.
#[derive(Debug, Clone, Default)]
pub struct AddonInfo {
    /// Folder name (used as addon identifier).
    pub folder_name: String,
    /// Display title from TOC metadata.
    pub title: String,
    /// Notes/description from TOC metadata.
    pub notes: String,
    /// Whether the addon is currently enabled.
    pub enabled: bool,
    /// Whether the addon was successfully loaded.
    pub loaded: bool,
    /// Load on demand flag.
    pub load_on_demand: bool,
    /// Total load time in seconds (for profiler metrics).
    pub load_time_secs: f64,
}

/// Shared simulator state accessible from Lua.
pub struct SimState {
    pub widgets: WidgetRegistry,
    pub events: EventQueue,
    pub scripts: ScriptRegistry,
    /// Console output from Lua print() calls.
    pub console_output: Vec<String>,
    /// Pending timer callbacks.
    pub timers: VecDeque<PendingTimer>,
    /// Currently focused frame ID (for keyboard input).
    pub focused_frame_id: Option<u64>,
    /// Registered addons (includes all scanned addons, not just loaded ones).
    pub addons: Vec<AddonInfo>,
    /// Console variables (CVars).
    pub cvars: CVarStorage,
    /// Tooltip state for GameTooltip frames (keyed by frame ID).
    pub tooltips: HashMap<u64, TooltipData>,
    /// SimpleHTML state (keyed by frame ID).
    pub simple_htmls: HashMap<u64, SimpleHtmlData>,
    /// MessageFrame state (keyed by frame ID).
    pub message_frames: HashMap<u64, MessageFrameData>,
    /// Frame IDs with active OnUpdate script handlers.
    pub on_update_frames: HashSet<u64>,
    /// Animation groups keyed by unique group ID.
    pub animation_groups: HashMap<u64, AnimGroupState>,
    /// Counter for generating unique animation group IDs.
    pub next_anim_group_id: u64,
    /// Screen dimensions in UI coordinates.
    pub screen_width: f32,
    pub screen_height: f32,
    /// Action bar slots: slot (1-120) → spell ID.
    pub action_bars: HashMap<u32, u32>,
    /// Addon base paths for runtime on-demand loading (Blizzard UI + AddOns directories).
    pub addon_base_paths: Vec<PathBuf>,
    /// Current mouse position in UI coordinates (for ANCHOR_CURSOR tooltip positioning).
    pub mouse_position: Option<(f32, f32)>,
    /// Currently hovered frame ID (for IsMouseMotionFocus / GetMouseFocus).
    pub hovered_frame: Option<u64>,
    /// Simulated party members (empty = not in group).
    pub party_members: Vec<PartyMember>,
    /// Current target (None = no target).
    pub current_target: Option<TargetInfo>,
}

impl Default for SimState {
    fn default() -> Self {
        Self {
            widgets: WidgetRegistry::default(),
            events: EventQueue::default(),
            scripts: ScriptRegistry::default(),
            console_output: Vec::new(),
            timers: VecDeque::new(),
            focused_frame_id: None,
            addons: Vec::new(),
            cvars: CVarStorage::new(),
            tooltips: HashMap::new(),
            simple_htmls: HashMap::new(),
            message_frames: HashMap::new(),
            on_update_frames: HashSet::new(),
            animation_groups: HashMap::new(),
            next_anim_group_id: 1,
            screen_width: 1024.0,
            screen_height: 768.0,
            action_bars: default_action_bars(),
            addon_base_paths: Vec::new(),
            mouse_position: None,
            hovered_frame: None,
            party_members: default_party(),
            current_target: None,
        }
    }
}

/// Default party member definitions: (name, class_index, health_max, power, power_max, power_type, power_type_name).
const DEFAULT_PARTY_MEMBERS: &[(&str, i32, i32, i32, i32, i32, &str)] = &[
    ("Thrynn",   2, 120_000, 80_000, 80_000, 0, "MANA"),  // Paladin
    ("Kazzara",  1, 180_000,      0,    100, 1, "RAGE"),   // Warrior
    ("Sylvanas", 3, 100_000,    100,    100, 2, "FOCUS"),  // Hunter
    ("Jaina",    8,  90_000, 64_000, 80_000, 0, "MANA"),   // Mage
];

/// Default 4-member party (disabled by WOW_SIM_NO_PARTY=1).
fn default_party() -> Vec<PartyMember> {
    if std::env::var("WOW_SIM_NO_PARTY").is_ok() {
        return Vec::new();
    }
    DEFAULT_PARTY_MEMBERS
        .iter()
        .map(|&(name, class_index, health_max, power, power_max, power_type, power_type_name)| {
            PartyMember {
                name,
                class_index,
                level: 70,
                health: health_max,
                health_max,
                power,
                power_max,
                power_type,
                power_type_name,
                is_leader: false,
            }
        })
        .collect()
}

/// Enemy NPC definition: (name, class_index, level, health, health_max, power, power_max, power_type_name).
const ENEMY_NPC: (&str, i32, i32, i32, i32, i32, i32, &str) =
    ("Hogger", 1, 11, 45_000, 45_000, 0, 0, "MANA");

/// Build a TargetInfo from a unit ID string.
pub fn build_target_info(unit_id: &str, state: &SimState) -> Option<TargetInfo> {
    match unit_id {
        "player" => Some(build_player_target()),
        u if u.starts_with("party") => build_party_target(u, state),
        "enemy1" => Some(build_enemy_target()),
        _ => None,
    }
}

fn build_player_target() -> TargetInfo {
    TargetInfo {
        unit_id: "player".into(),
        name: "SimPlayer".into(),
        class_index: 2, // Paladin (matches existing player defaults)
        level: 70,
        health: 100_000,
        health_max: 100_000,
        power: 50_000,
        power_max: 100_000,
        power_type: 0,
        power_type_name: "MANA",
        is_player: true,
        is_enemy: false,
        guid: "Player-0000-00000001".into(),
    }
}

fn build_party_target(unit_id: &str, state: &SimState) -> Option<TargetInfo> {
    let idx = unit_id.strip_prefix("party")?
        .parse::<usize>().ok()
        .filter(|&n| n >= 1)
        .map(|n| n - 1)?;
    let m = state.party_members.get(idx)?;
    Some(TargetInfo {
        unit_id: unit_id.into(),
        name: m.name.into(),
        class_index: m.class_index,
        level: m.level,
        health: m.health,
        health_max: m.health_max,
        power: m.power,
        power_max: m.power_max,
        power_type: m.power_type,
        power_type_name: m.power_type_name,
        is_player: true,
        is_enemy: false,
        guid: format!("Player-0000-0000000{}", idx + 2),
    })
}

fn build_enemy_target() -> TargetInfo {
    let (name, class_index, level, health, health_max, power, power_max, power_type_name) =
        ENEMY_NPC;
    TargetInfo {
        unit_id: "enemy1".into(),
        name: name.into(),
        class_index,
        level,
        health,
        health_max,
        power,
        power_max,
        power_type: 0,
        power_type_name,
        is_player: false,
        is_enemy: true,
        guid: "Creature-0000-00000099".into(),
    }
}

/// Randomly adjust party member health by ±5%, clamped to [1, max].
///
/// Returns the 1-based indices of members whose health changed (for firing UNIT_HEALTH).
pub fn tick_party_health(members: &mut [PartyMember]) -> Vec<usize> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Simple deterministic-ish RNG seeded from current time nanos.
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    let mut changed = Vec::new();
    for (i, m) in members.iter_mut().enumerate() {
        let mut hasher = DefaultHasher::new();
        (nanos, i).hash(&mut hasher);
        let hash = hasher.finish();
        // Map hash to [-5%, +5%] of max health.
        let max5 = (m.health_max as f64 * 0.05) as i64;
        if max5 == 0 { continue; }
        let delta = (hash % (max5 as u64 * 2 + 1)) as i64 - max5;
        let new_hp = (m.health as i64 + delta).clamp(1, m.health_max as i64) as i32;
        if new_hp != m.health {
            m.health = new_hp;
            changed.push(i + 1); // 1-based party index
        }
    }
    changed
}

/// Pre-populate main action bar (slots 1-12) with Protection Paladin spells.
fn default_action_bars() -> HashMap<u32, u32> {
    let prot_paladin_bar: &[(u32, u32)] = &[
        (1, 19750),  // Flash of Light (1.5s cast)
        (2, 53595),  // Hammer of the Righteous
        (3, 275779), // Judgment
        (4, 26573),  // Consecration
        (5, 53600),  // Shield of the Righteous
        (6, 85673),  // Word of Glory
        (7, 62124),  // Hand of Reckoning (Taunt)
        (8, 31850),  // Ardent Defender
        (9, 86659),  // Guardian of Ancient Kings
        (10, 642),   // Divine Shield
        (11, 633),   // Lay on Hands
        (12, 1022),  // Blessing of Protection
    ];
    prot_paladin_bar.iter().copied().collect()
}
