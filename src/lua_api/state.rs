//! Shared state types for the WoW Lua API.

use crate::cvars::CVarStorage;
use crate::event::{EventQueue, ScriptRegistry};
use crate::lua_api::animation::AnimGroupState;
use crate::lua_api::message_frame::MessageFrameData;
use crate::lua_api::simple_html::SimpleHtmlData;
use crate::lua_api::tooltip::TooltipData;
use crate::sound::SoundManager;
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
    /// When the member died (for auto-rez after 30s).
    pub dead_since: Option<std::time::Instant>,
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

/// Class display names (index 0 = class_index 1, etc.).
pub const CLASS_LABELS: &[&str] = &[
    "Warrior", "Paladin", "Hunter", "Rogue", "Priest",
    "Death Knight", "Shaman", "Mage", "Warlock", "Monk",
    "Druid", "Demon Hunter", "Evoker",
];

/// Race data: (display_name, file_name, faction).
pub const RACE_DATA: &[(&str, &str, &str)] = &[
    ("Human", "Human", "Alliance"),
    ("Orc", "Orc", "Horde"),
    ("Dwarf", "Dwarf", "Alliance"),
    ("Night Elf", "NightElf", "Alliance"),
    ("Undead", "Scourge", "Horde"),
    ("Tauren", "Tauren", "Horde"),
    ("Gnome", "Gnome", "Alliance"),
    ("Troll", "Troll", "Horde"),
    ("Blood Elf", "BloodElf", "Horde"),
    ("Draenei", "Draenei", "Alliance"),
    ("Worgen", "Worgen", "Alliance"),
    ("Goblin", "Goblin", "Horde"),
    ("Pandaren", "Pandaren", "Neutral"),
    ("Dracthyr", "Dracthyr", "Neutral"),
    ("Earthen", "Earthen", "Neutral"),
];

/// A simulated aura (buff or debuff).
#[derive(Clone)]
pub struct AuraInfo {
    pub name: &'static str,
    pub spell_id: i32,
    pub icon: i32,
    /// Total duration in seconds (0 = permanent/no duration).
    pub duration: f64,
    /// Absolute GetTime() value at which this aura expires (0 = permanent).
    pub expiration_time: f64,
    /// Stack count.
    pub applications: i32,
    pub source_unit: &'static str,
    pub is_helpful: bool,
    pub is_stealable: bool,
    pub can_apply_aura: bool,
    pub is_from_player_or_player_pet: bool,
    /// Unique instance ID for this aura.
    pub aura_instance_id: i32,
}

/// Rot damage intensity levels: (label, percentage of max HP per tick).
pub const ROT_DAMAGE_LEVELS: &[(&str, f64)] = &[
    ("Light (1%)", 0.01),
    ("Medium (3%)", 0.03),
    ("Heavy (5%)", 0.05),
    ("Brutal (10%)", 0.10),
];

/// Active spell cast state (for cast bar display).
pub struct CastingState {
    pub spell_id: u32,
    pub spell_name: String,
    pub icon_path: String,
    /// GetTime() at cast start (seconds).
    pub start_time: f64,
    /// GetTime() at cast end (seconds).
    pub end_time: f64,
    pub cast_id: u32,
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
    /// Cached subset of `on_update_frames` whose ancestors are all visible.
    /// Invalidated when `WidgetRegistry::visibility_dirty` is set.
    pub visible_on_update_cache: Option<Vec<u64>>,
    /// Cached ancestor-visible IDs with effective alpha. Built lazily on first
    /// `draw()`, then updated eagerly by `set_frame_visible`.
    pub ancestor_visible_cache: Option<HashMap<u64, f32>>,
    /// Per-strata buckets of visible frame IDs. Index = FrameStrata as usize.
    /// Built alongside ancestor_visible_cache, updated eagerly by `set_frame_visible`.
    pub strata_buckets: Option<Vec<Vec<u64>>>,
    /// Persistent layout rect cache. Built lazily on first `draw()`, entries
    /// invalidated eagerly when layout-affecting properties change (anchors,
    /// size, scale, parent). Frames not in cache are recomputed on next rebuild.
    pub layout_rect_cache: Option<crate::iced_app::layout::LayoutCache>,
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
    /// Current focus target (None = no focus).
    pub current_focus: Option<TargetInfo>,
    /// Audio playback manager (None when no audio device or WOW_SIM_NO_SOUND=1).
    pub sound_manager: Option<SoundManager>,
    /// Player character name (randomly chosen on startup).
    pub player_name: String,
    /// Player current health.
    pub player_health: i32,
    /// Player maximum health.
    pub player_health_max: i32,
    /// Player class (1-based index matching CLASS_DATA in unit_api).
    pub player_class_index: i32,
    /// Player race (0-based index into RACE_DATA).
    pub player_race_index: usize,
    /// Rot damage intensity (index into ROT_DAMAGE_LEVELS).
    pub rot_damage_level: usize,
    /// Player buffs/debuffs (disabled by WOW_SIM_NO_BUFFS=1).
    pub player_buffs: Vec<AuraInfo>,
    /// Current framerate (FPS), updated by the app's FPS counter.
    pub fps: f32,
    /// Instant at which the UI started (used by GetTime and message timestamps).
    pub start_time: Instant,
    /// Active spell cast (None = not casting).
    pub casting: Option<CastingState>,
    /// Counter for generating unique cast IDs.
    pub next_cast_id: u32,
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
            visible_on_update_cache: None,
            ancestor_visible_cache: None,
            strata_buckets: None,
            layout_rect_cache: None,
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
            current_focus: None,
            sound_manager: None,
            player_name: random_player_name(),
            player_health: 100_000,
            player_health_max: 100_000,
            player_class_index: 2,  // Paladin
            player_race_index: 0,   // Human
            rot_damage_level: 0,    // Light
            player_buffs: default_player_buffs(),
            fps: 0.0,
            start_time: Instant::now(),
            casting: None,
            next_cast_id: 1,
        }
    }
}

impl SimState {
    /// Return the cached ancestor-visible map, building it on first call.
    /// Also builds strata_buckets as a side effect.
    pub fn get_ancestor_visible(&mut self) -> &HashMap<u64, f32> {
        if self.ancestor_visible_cache.is_none() {
            let visible = crate::iced_app::frame_collect::collect_ancestor_visible_ids(&self.widgets);
            let buckets = self.build_strata_buckets(&visible);
            self.strata_buckets = Some(buckets);
            self.ancestor_visible_cache = Some(visible);
        }
        self.ancestor_visible_cache.as_ref().unwrap()
    }

    /// Return the per-strata buckets (requires ancestor_visible to have been built).
    pub fn get_strata_buckets(&self) -> Option<&Vec<Vec<u64>>> {
        self.strata_buckets.as_ref()
    }

    /// Build per-strata ID buckets from the ancestor-visible map.
    fn build_strata_buckets(&self, visible: &HashMap<u64, f32>) -> Vec<Vec<u64>> {
        use crate::widget::FrameStrata;
        let mut buckets = vec![Vec::new(); FrameStrata::COUNT];
        for &id in visible.keys() {
            if let Some(f) = self.widgets.get(id) {
                buckets[f.frame_strata.as_index()].push(id);
            }
        }
        buckets
    }

    /// Take the persistent layout cache for use during quad rebuild.
    /// Returns the existing cache (or empty) for the caller to populate.
    /// Caller must return it via `set_layout_cache` after the rebuild.
    pub fn take_layout_cache(&mut self) -> crate::iced_app::layout::LayoutCache {
        self.layout_rect_cache.take().unwrap_or_default()
    }

    /// Store the layout cache back after a quad rebuild.
    pub fn set_layout_cache(&mut self, cache: crate::iced_app::layout::LayoutCache) {
        self.layout_rect_cache = Some(cache);
    }

    /// Invalidate cached layout for a frame and all its descendants.
    /// Called when layout-affecting properties change (anchors, size, scale, parent).
    pub fn invalidate_layout(&mut self, id: u64) {
        let Some(cache) = self.layout_rect_cache.as_mut() else { return };
        Self::remove_layout_subtree(&self.widgets, id, cache);
    }

    fn remove_layout_subtree(
        widgets: &crate::widget::WidgetRegistry,
        id: u64,
        cache: &mut crate::iced_app::layout::LayoutCache,
    ) {
        cache.remove(&id);
        let children: Vec<u64> = widgets.get(id)
            .map(|f| f.children.clone()).unwrap_or_default();
        for child_id in children {
            Self::remove_layout_subtree(widgets, child_id, cache);
        }
    }

    /// Set a frame's visibility and eagerly update both caches.
    ///
    /// When hiding: remove the frame and all descendants from both caches.
    /// When showing: add the frame and any descendants that are now
    /// ancestor-visible.
    pub fn set_frame_visible(&mut self, id: u64, visible: bool) {
        let was_visible = self.widgets.get(id).map(|f| f.visible).unwrap_or(false);
        self.widgets.set_visible(id, visible);
        if was_visible == visible {
            return;
        }
        self.update_on_update_cache(id, visible);
        self.update_ancestor_visible_cache(id, visible);
    }

    fn update_on_update_cache(&mut self, id: u64, visible: bool) {
        let Some(mut cache) = self.visible_on_update_cache.take() else {
            return;
        };
        if visible {
            self.add_on_update_descendants(id, &mut cache);
        } else {
            self.remove_on_update_descendants(id, &mut cache);
        }
        self.visible_on_update_cache = Some(cache);
    }

    fn update_ancestor_visible_cache(&mut self, id: u64, visible: bool) {
        let Some(mut cache) = self.ancestor_visible_cache.take() else {
            return;
        };
        let mut buckets = self.strata_buckets.take();
        if visible {
            self.add_visible_descendants(id, &mut cache, &mut buckets);
        } else {
            self.remove_visible_descendants(id, &mut cache, &mut buckets);
        }
        self.ancestor_visible_cache = Some(cache);
        self.strata_buckets = buckets;
    }

    /// Add `id` and its descendants to cache if they have OnUpdate and are ancestor-visible.
    fn add_on_update_descendants(&self, id: u64, cache: &mut Vec<u64>) {
        if self.on_update_frames.contains(&id) && self.widgets.is_ancestor_visible(id) {
            if !cache.contains(&id) {
                cache.push(id);
            }
        }
        let children: Vec<u64> = self.widgets.get(id)
            .map(|f| f.children.clone()).unwrap_or_default();
        for child_id in children {
            if self.widgets.get(child_id).is_some_and(|f| f.visible) {
                self.add_on_update_descendants(child_id, cache);
            }
        }
    }

    /// Remove `id` and all its descendants from cache (hidden ancestor = all hidden).
    fn remove_on_update_descendants(&self, id: u64, cache: &mut Vec<u64>) {
        cache.retain(|&cached_id| cached_id != id);
        let children: Vec<u64> = self.widgets.get(id)
            .map(|f| f.children.clone()).unwrap_or_default();
        for child_id in children {
            self.remove_on_update_descendants(child_id, cache);
        }
    }

    /// Add `id` and visible descendants to the ancestor-visible cache with alpha.
    fn add_visible_descendants(
        &self, id: u64, cache: &mut HashMap<u64, f32>,
        buckets: &mut Option<Vec<Vec<u64>>>,
    ) {
        let Some(f) = self.widgets.get(id) else { return };
        let parent_alpha = f.parent_id
            .and_then(|pid| cache.get(&pid).copied())
            .unwrap_or(1.0);
        if !f.visible {
            if is_button_state_texture(f, id, &self.widgets) {
                cache.insert(id, parent_alpha * f.alpha);
                if let Some(b) = buckets.as_mut() {
                    b[f.frame_strata.as_index()].push(id);
                }
            }
            return;
        }
        let eff = parent_alpha * f.alpha;
        cache.insert(id, eff);
        if let Some(b) = buckets.as_mut() {
            b[f.frame_strata.as_index()].push(id);
        }
        if f.widget_type == crate::widget::WidgetType::GameTooltip {
            return;
        }
        let children: Vec<u64> = f.children.clone();
        for child_id in children {
            self.add_visible_descendants(child_id, cache, buckets);
        }
    }

    /// Remove `id` and all descendants from ancestor-visible cache.
    fn remove_visible_descendants(
        &self, id: u64, cache: &mut HashMap<u64, f32>,
        buckets: &mut Option<Vec<Vec<u64>>>,
    ) {
        if cache.remove(&id).is_some() {
            if let Some(b) = buckets.as_mut() {
                if let Some(f) = self.widgets.get(id) {
                    let bucket = &mut b[f.frame_strata.as_index()];
                    if let Some(pos) = bucket.iter().position(|&x| x == id) {
                        bucket.swap_remove(pos);
                    }
                }
            }
        }
        let children: Vec<u64> = self.widgets.get(id)
            .map(|f| f.children.clone()).unwrap_or_default();
        for child_id in children {
            self.remove_visible_descendants(child_id, cache, buckets);
        }
    }
}

/// Check if a frame is a button state texture (NormalTexture, PushedTexture, etc.).
fn is_button_state_texture(
    f: &crate::widget::Frame,
    id: u64,
    registry: &crate::widget::WidgetRegistry,
) -> bool {
    use crate::widget::WidgetType;
    if !matches!(f.widget_type, WidgetType::Texture) {
        return false;
    }
    let Some(parent_id) = f.parent_id else { return false };
    let Some(parent) = registry.get(parent_id) else { return false };
    if !matches!(parent.widget_type, WidgetType::Button | WidgetType::CheckButton) {
        return false;
    }
    ["NormalTexture", "PushedTexture", "HighlightTexture", "DisabledTexture"]
        .iter()
        .any(|key| parent.children_keys.get(*key) == Some(&id))
}

/// Pick a random WoW-style player name using the current time as a seed.
fn random_player_name() -> String {
    const NAMES: &[&str] = &[
        "Arthas", "Jaina", "Thrall", "Varian", "Anduin",
        "Garrosh", "Tyrande", "Malfurion", "Illidan", "Khadgar",
        "Genn", "Baine", "Rokhan", "Thalyssra", "Alleria",
        "Turalyon", "Calia", "Lothraxion", "Velen", "Yrel",
    ];
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as usize)
        .unwrap_or(0);
    NAMES[nanos % NAMES.len()].to_string()
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
                level: 80,
                health: health_max,
                health_max,
                power,
                power_max,
                power_type,
                power_type_name,
                is_leader: false,
                dead_since: None,
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
        "player" => Some(build_player_target(state)),
        u if u.starts_with("party") => build_party_target(u, state),
        "enemy1" => Some(build_enemy_target()),
        _ => None,
    }
}

fn build_player_target(state: &SimState) -> TargetInfo {
    TargetInfo {
        unit_id: "player".into(),
        name: state.player_name.clone(),
        class_index: state.player_class_index,
        level: 80,
        health: state.player_health,
        health_max: state.player_health_max,
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

/// Randomly damage party members, auto-resurrect after 30s dead.
///
/// `damage_pct` controls the intensity (fraction of max HP per tick).
/// Returns the 1-based indices of members whose health changed (for firing UNIT_HEALTH).
pub fn tick_party_health(members: &mut [PartyMember], damage_pct: f64) -> Vec<usize> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let now = std::time::Instant::now();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    let mut changed = Vec::new();
    for (i, m) in members.iter_mut().enumerate() {
        // Auto-rez after 30s dead.
        if let Some(died_at) = m.dead_since {
            if now.duration_since(died_at).as_secs() >= 30 {
                m.health = m.health_max;
                m.dead_since = None;
                changed.push(i + 1);
            }
            continue;
        }

        let mut hasher = DefaultHasher::new();
        (nanos, i).hash(&mut hasher);
        let hash = hasher.finish();
        let max_delta = (m.health_max as f64 * damage_pct) as i64;
        if max_delta == 0 { continue; }
        let delta = -((hash % (max_delta as u64 + 1)) as i64);
        let new_hp = (m.health as i64 + delta).clamp(0, m.health_max as i64) as i32;
        if new_hp != m.health {
            m.health = new_hp;
            if new_hp == 0 {
                m.dead_since = Some(now);
            }
            changed.push(i + 1);
        }
    }
    changed
}

/// Buff pool: (name, spell_id, icon_file_id, duration_secs, source_unit, can_apply_aura).
const BUFF_POOL: &[(&str, i32, i32, f64, &str, bool)] = &[
    ("Power Word: Fortitude", 21562, 135987, 3600.0, "player", true),
    ("Arcane Intellect", 1459, 135932, 3600.0, "party2", true),
    ("Mark of the Wild", 1126, 136078, 3600.0, "party3", true),
    ("Battle Shout", 6673, 132333, 3600.0, "party1", true),
    ("Retribution Aura", 183435, 135889, 0.0, "player", false),
    ("Devotion Aura", 465, 135893, 0.0, "player", false),
    ("Blessing of the Bronze", 381748, 4622449, 3600.0, "party4", true),
    ("Well Fed", 104280, 136000, 3600.0, "player", false),
];

/// Pick random buffs from the pool (disabled by WOW_SIM_NO_BUFFS=1).
fn default_player_buffs() -> Vec<AuraInfo> {
    if std::env::var("WOW_SIM_NO_BUFFS").is_ok() {
        return Vec::new();
    }
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as usize)
        .unwrap_or(0);
    let count = 4 + (nanos % 3); // 4, 5, or 6 buffs
    let mut indices: Vec<usize> = (0..BUFF_POOL.len()).collect();
    // Simple shuffle using nanos
    for i in (1..indices.len()).rev() {
        let j = (nanos.wrapping_mul(i + 7)) % (i + 1);
        indices.swap(i, j);
    }
    indices.truncate(count);
    indices.sort();
    build_auras_from_indices(&indices)
}

/// Build AuraInfo vec from selected pool indices.
///
/// `expiration_time` is the absolute GetTime() value when the buff expires.
/// Since GetTime() starts near 0 at startup, this equals the duration itself.
/// Permanent buffs (duration == 0) have expiration_time == 0.
fn build_auras_from_indices(indices: &[usize]) -> Vec<AuraInfo> {
    // GetTime() ≈ 0 at init, so expiration = 0 + duration = duration.
    let get_time = 0.0_f64;
    indices
        .iter()
        .enumerate()
        .map(|(i, &pool_idx)| {
            let (name, spell_id, icon, duration, source, can_apply) = BUFF_POOL[pool_idx];
            let expiration_time = if duration > 0.0 { get_time + duration } else { 0.0 };
            AuraInfo {
                name,
                spell_id,
                icon,
                duration,
                expiration_time,
                applications: 0,
                source_unit: source,
                is_helpful: true,
                is_stealable: false,
                can_apply_aura: can_apply,
                is_from_player_or_player_pet: source == "player",
                aura_instance_id: (i + 1) as i32,
            }
        })
        .collect()
}

/// Pre-populate main action bar (slots 1-12) with Protection Paladin spells.
fn default_action_bars() -> HashMap<u32, u32> {
    let prot_paladin_bar: &[(u32, u32)] = &[
        (1, 19750),  // Flash of Light (heal)
        (2, 31935),  // Avenger's Shield (pull/interrupt)
        (3, 275779), // Judgment (core rotational)
        (4, 26573),  // Consecration (ground AoE)
        (5, 53600),  // Shield of the Righteous (active mitigation)
        (6, 85673),  // Word of Glory (self-heal)
        (7, 62124),  // Hand of Reckoning (Taunt)
        (8, 853),    // Hammer of Justice (stun)
        (9, 375576), // Divine Toll (AoE ability)
        (10, 31850), // Ardent Defender (defensive CD)
        (11, 86659), // Guardian of Ancient Kings (defensive CD)
        (12, 642),   // Divine Shield (oh-shit button)
    ];
    prot_paladin_bar.iter().copied().collect()
}
