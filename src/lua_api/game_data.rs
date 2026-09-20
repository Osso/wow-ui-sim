//! Game simulation data types, constants, and helpers.
//!
//! Contains player/party/target/aura definitions and the default data
//! generators used by `SimState::default()`.

mod allied_races;
mod empower;

pub use allied_races::{default_allied_races, default_model_scenes};
pub use empower::EmpowerTiming;

use crate::lua_api::state::{MajorFactionData, RenownLevelInfo};
use std::collections::HashMap;

/// Server-provided interaction capabilities for a resolved world object.
#[derive(Clone, Copy, Debug, Default)]
pub struct UnitInteraction {
    pub is_game_object: bool,
    pub has_loot: bool,
    pub interactable: bool,
    pub in_range: bool,
}

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
    pub power_type_name: String,
    pub is_player: bool,
    pub is_enemy: bool,
    pub guid: String,
    /// "normal", "elite", "rare", "rareelite", "worldboss", "trivial", "minus".
    pub classification: String,
    /// "Humanoid", "Beast", "Demon", "Undead", "Elemental", "Dragonkin", etc.
    pub creature_type: String,
    /// 1-8: 1=Hostile, 4=Neutral, 5=Friendly (relative to player).
    pub reaction: i32,
    pub interaction: UnitInteraction,
}

/// A simulated party member.
#[derive(Clone)]
pub struct PartyMember {
    pub name: String,
    /// 1-based class index into CLASS_DATA.
    pub class_index: i32,
    pub level: i32,
    pub health: i32,
    pub health_max: i32,
    pub power: i32,
    pub power_max: i32,
    /// 0=MANA, 1=RAGE, 2=FOCUS, 3=ENERGY.
    pub power_type: i32,
    pub power_type_name: String,
    pub is_leader: bool,
    /// When the member died (for auto-rez after 30s).
    pub dead_since: Option<std::time::Instant>,
    /// Active buffs (helpful auras) on this party member.
    pub buffs: Vec<AuraInfo>,
    /// Active debuffs (harmful auras) on this party member.
    pub debuffs: Vec<AuraInfo>,
}

/// A simulated aura (buff or debuff).
#[derive(Debug, Clone)]
pub struct AuraInfo {
    pub name: String,
    pub spell_id: i32,
    pub icon: i32,
    /// Total duration in seconds (0 = permanent/no duration).
    pub duration: f64,
    /// Absolute GetTime() value at which this aura expires (0 = permanent).
    pub expiration_time: f64,
    /// Stack count.
    pub applications: i32,
    pub source_unit: String,
    pub is_helpful: bool,
    pub is_stealable: bool,
    pub can_apply_aura: bool,
    pub is_from_player_or_player_pet: bool,
    /// Magic, Curse, Disease, or Poison for dispellable debuffs.
    pub dispel_type: Option<String>,
    /// Unique instance ID for this aura.
    pub aura_instance_id: i32,
}

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
    pub empower: Option<EmpowerTiming>,
    /// Cumulative simulator delay input, in seconds.
    pub delay_time: f64,
}

/// Per-spell cooldown tracking.
#[derive(Clone, Debug)]
pub struct SpellCooldownState {
    /// GetTime() at cooldown start.
    pub start: f64,
    /// Cooldown duration in seconds.
    pub duration: f64,
}

pub enum SpellEffectResult {
    UnitHealthChanged(String),
    PlayerAurasChanged,
}

/// Spell target type: determines which units a spell can be cast on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellTargetType {
    /// Damage/CC spells — require a hostile target.
    Harmful,
    /// Healing/buff spells — require a friendly target (auto-target self if none).
    Helpful,
    /// Self-only spells (defensives, auras) — always cast regardless of target.
    SelfOnly,
}

fn is_paladin_aura_spell(spell_id: u32) -> bool {
    matches!(spell_id, 465 | 32223 | 317920 | 183435)
}

/// Classify a spell by its target type using the `implicit_target` field
/// from SpellEffect.db2 (auto-generated into `data/spells.rs`).
///
/// ImplicitTarget_0 values (from first effect, DifficultyID=0):
///   1=CASTER, 6=TARGET_ENEMY, 18=DEST_CASTER, 21=TARGET_ALLY,
///   22=SRC_CASTER, 24=CONE_ENEMY, 25=TARGET_ANY, 53=TARGET_ENEMY_OR_ALLY,
///   57=TARGET_ALLY_OR_RAID, 59=DEST_AREA_ALLY, 87=DEST_CASTER_GROUND,
///   104=CONE_TO_DEST_ENEMY
pub fn spell_target_type(spell_id: u32) -> SpellTargetType {
    let target = crate::spells::get_spell(spell_id)
        .map(|s| s.implicit_target)
        .unwrap_or(0);
    implicit_target_to_type(target)
}

/// Map an ImplicitTarget_0 value to a SpellTargetType.
fn implicit_target_to_type(target: u8) -> SpellTargetType {
    match target {
        // Hostile: TARGET_ENEMY, CONE_ENEMY variants
        6 | 24 | 104 => SpellTargetType::Harmful,
        // Friendly: TARGET_ALLY, ALLY_OR_RAID, DEST_AREA_ALLY
        21 | 57 | 59 => SpellTargetType::Helpful,
        // Self/ground/any/unknown: CASTER, DEST_CASTER, SRC_CASTER,
        // DEST_CASTER_GROUND, TARGET_ANY, and everything else
        _ => SpellTargetType::SelfOnly,
    }
}

/// Check whether a spell can be cast given the current target.
/// Hardcoded spell effect amounts (damage or healing).
pub fn spell_effect_amount(spell_id: u32) -> i32 {
    match spell_id {
        35395 => 15_000, // Crusader Strike (instant, harmful)
        31935 => 25_000, // Avenger's Shield (instant, harmful)
        53600 => 20_000, // Shield of the Righteous (instant, harmful)
        24275 => 30_000, // Hammer of Wrath (instant, harmful)
        62124 => 5_000,  // Hand of Reckoning (instant, harmful)
        853 => 0,        // Hammer of Justice (stun only)
        19750 => 20_000, // Flash of Light (cast-time, helpful)
        82326 => 35_000, // Holy Light (cast-time, helpful)
        85673 => 20_000, // Word of Glory (instant, helpful)
        _ => 10_000,     // Default fallback
    }
}

/// Apply spell effect to state: damage enemy or heal friendly/self.
/// Returns the changed state kind so callers can fire the matching UI event.
pub fn apply_spell_to_state(
    state: &std::rc::Rc<std::cell::RefCell<super::state::SimState>>,
    spell_id: u32,
) -> Option<SpellEffectResult> {
    if apply_player_aura_spell(state, spell_id) {
        return Some(SpellEffectResult::PlayerAurasChanged);
    }

    let amount = spell_effect_amount(spell_id);
    if amount == 0 {
        return None;
    }
    match spell_target_type(spell_id) {
        SpellTargetType::Harmful => {
            apply_damage_to_target(state, amount).map(SpellEffectResult::UnitHealthChanged)
        }
        SpellTargetType::Helpful => {
            apply_heal_to_target(state, amount).map(SpellEffectResult::UnitHealthChanged)
        }
        SpellTargetType::SelfOnly => None,
    }
}

fn apply_player_aura_spell(
    state: &std::rc::Rc<std::cell::RefCell<super::state::SimState>>,
    spell_id: u32,
) -> bool {
    if !is_paladin_aura_spell(spell_id) {
        return false;
    }
    let Some(spell) = crate::spell_lookup::get_spell(spell_id) else {
        return false;
    };

    let mut sim = state.borrow_mut();
    let aura_instance_id = existing_or_next_player_aura_instance_id(&sim, spell_id);
    sim.player
        .buffs
        .retain(|aura| !is_paladin_aura_spell(aura.spell_id as u32));
    sim.player.buffs.push(AuraInfo {
        name: spell.name.to_string(),
        spell_id: spell_id as i32,
        icon: spell.icon_file_data_id as i32,
        duration: 0.0,
        expiration_time: 0.0,
        applications: 0,
        source_unit: "player".to_string(),
        is_helpful: true,
        is_stealable: false,
        can_apply_aura: true,
        is_from_player_or_player_pet: true,
        dispel_type: None,
        aura_instance_id,
    });
    true
}

fn existing_or_next_player_aura_instance_id(sim: &super::state::SimState, spell_id: u32) -> i32 {
    sim.player
        .buffs
        .iter()
        .find(|aura| aura.spell_id == spell_id as i32)
        .map(|aura| aura.aura_instance_id)
        .unwrap_or_else(|| next_player_aura_instance_id(sim))
}

fn next_player_aura_instance_id(sim: &super::state::SimState) -> i32 {
    sim.player
        .buffs
        .iter()
        .map(|aura| aura.aura_instance_id)
        .max()
        .unwrap_or(0)
        + 1
}

fn apply_damage_to_target(
    state: &std::rc::Rc<std::cell::RefCell<super::state::SimState>>,
    amount: i32,
) -> Option<String> {
    let mut s = state.borrow_mut();
    let t = s.current_target.as_mut()?;
    if !t.is_enemy || t.health <= 0 {
        return None;
    }
    t.health = (t.health - amount).max(0);
    Some(t.unit_id.clone())
}

fn apply_heal_to_target(
    state: &std::rc::Rc<std::cell::RefCell<super::state::SimState>>,
    amount: i32,
) -> Option<String> {
    let mut s = state.borrow_mut();
    if let Some(ref mut t) = s.current_target {
        if !t.is_enemy {
            if t.health <= 0 {
                return None;
            }
            t.health = (t.health + amount).min(t.health_max);
            let healed = t.health;
            let unit_id = t.unit_id.clone();
            if let Some(idx) = super::globals::unit_api::parse_party_index(&unit_id) {
                if let Some(m) = s.party_members.get_mut(idx) {
                    m.health = healed;
                }
            }
            Some(unit_id)
        } else {
            heal_player(&mut s, amount)
        }
    } else {
        heal_player(&mut s, amount)
    }
}

fn heal_player(s: &mut super::state::SimState, amount: i32) -> Option<String> {
    if s.player.health <= 0 {
        return None;
    }
    s.player.health = (s.player.health + amount).min(s.player.health_max);
    Some("player".to_string())
}

/// Class display names (index 0 = class_index 1, etc.).
pub const CLASS_LABELS: &[&str] = &[
    "Warrior",
    "Paladin",
    "Hunter",
    "Rogue",
    "Priest",
    "Death Knight",
    "Shaman",
    "Mage",
    "Warlock",
    "Monk",
    "Druid",
    "Demon Hunter",
    "Evoker",
];

/// Class file tokens (index 0 = class_index 1, etc.).
pub const CLASS_FILES: &[&str] = &[
    "WARRIOR",
    "PALADIN",
    "HUNTER",
    "ROGUE",
    "PRIEST",
    "DEATHKNIGHT",
    "SHAMAN",
    "MAGE",
    "WARLOCK",
    "MONK",
    "DRUID",
    "DEMONHUNTER",
    "EVOKER",
];

/// Shared class lookup used by both global `GetClassInfo` and
/// `C_CreatureInfo.GetClassInfo`.
pub fn class_info_by_index(class_index: i32) -> (&'static str, &'static str, i32) {
    match usize::try_from(class_index.saturating_sub(1)) {
        Ok(idx) if idx < CLASS_LABELS.len() => (CLASS_LABELS[idx], CLASS_FILES[idx], class_index),
        _ => ("Unknown", "UNKNOWN", class_index.max(1)),
    }
}

/// Race data: (display_name, file_name, faction).
pub const RACE_DATA: &[(&'static str, &str, &str)] = &[
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

/// Rot damage intensity levels: (label, percentage of max HP per tick).
pub const ROT_DAMAGE_LEVELS: &[(&'static str, f64)] = &[
    ("Off", 0.0),
    ("Light (1%)", 0.01),
    ("Medium (3%)", 0.03),
    ("Heavy (5%)", 0.05),
    ("Brutal (10%)", 0.10),
];

/// XP bar levels: (label, fraction of XP bar filled). "Max Level" hides the bar entirely.
pub const XP_LEVELS: &[(&'static str, f64)] = &[
    ("Max Level", 0.0),
    ("33%", 0.33),
    ("66%", 0.66),
    ("100%", 1.0),
];

/// Default party member definitions: (name, class_index, health_max, power, power_max, power_type, power_type_name).
const DEFAULT_PARTY_MEMBERS: &[(&'static str, i32, i32, i32, i32, i32, &str)] = &[
    ("Thrynn", 2, 120_000, 80_000, 80_000, 0, "MANA"), // Paladin
    ("Kazzara", 1, 180_000, 0, 100, 1, "RAGE"),        // Warrior
    ("Sylvanas", 3, 100_000, 100, 100, 2, "FOCUS"),    // Hunter
    ("Jaina", 8, 90_000, 64_000, 80_000, 0, "MANA"),   // Mage
];

/// A simple buff AuraInfo for seeding party member aura lists.
fn make_party_buff(
    name: &str,
    spell_id: i32,
    icon: i32,
    source_unit: &str,
    aura_instance_id: i32,
) -> AuraInfo {
    AuraInfo {
        name: name.to_string(),
        spell_id,
        icon,
        duration: 3600.0,
        expiration_time: 3600.0,
        applications: 0,
        source_unit: source_unit.to_string(),
        is_helpful: true,
        is_stealable: false,
        can_apply_aura: true,
        is_from_player_or_player_pet: source_unit == "player",
        dispel_type: None,
        aura_instance_id,
    }
}

/// A simple debuff AuraInfo for seeding party member aura lists.
fn make_party_debuff(
    name: &str,
    spell_id: i32,
    icon: i32,
    source_unit: &str,
    aura_instance_id: i32,
) -> AuraInfo {
    AuraInfo {
        name: name.to_string(),
        spell_id,
        icon,
        duration: 30.0,
        expiration_time: 30.0,
        applications: 1,
        source_unit: source_unit.to_string(),
        is_helpful: false,
        is_stealable: false,
        can_apply_aura: false,
        is_from_player_or_player_pet: false,
        dispel_type: None,
        aura_instance_id,
    }
}

/// Default 4-member party (disabled by WOW_SIM_NO_PARTY=1).
///
/// Aura distribution:
/// - party1 (Thrynn):   buff only
/// - party2 (Kazzara):  debuff only
/// - party3 (Sylvanas): buff + debuff
/// - party4 (Jaina):    neither
pub fn default_party() -> Vec<PartyMember> {
    if std::env::var("WOW_SIM_NO_PARTY").is_ok() {
        return Vec::new();
    }
    DEFAULT_PARTY_MEMBERS
        .iter()
        .enumerate()
        .map(
            |(
                i,
                &(name, class_index, health_max, power, power_max, power_type, power_type_name),
            )| {
                let (buffs, debuffs) = default_party_auras(i);
                PartyMember {
                    name: name.to_string(),
                    class_index,
                    level: 80,
                    health: health_max,
                    health_max,
                    power,
                    power_max,
                    power_type,
                    power_type_name: power_type_name.to_string(),
                    is_leader: false,
                    dead_since: None,
                    buffs,
                    debuffs,
                }
            },
        )
        .collect()
}

/// Build (buffs, debuffs) for the i-th party member (0-based).
///
/// Distribution:
/// - 0 (party1): buff only
/// - 1 (party2): debuff only
/// - 2 (party3): buff + debuff
/// - 3+ (party4): neither
fn default_party_auras(i: usize) -> (Vec<AuraInfo>, Vec<AuraInfo>) {
    // Spell: Power Word: Fortitude (buff), Weakened Armor (debuff)
    let buff = make_party_buff("Power Word: Fortitude", 21562, 135987, "player", 1);
    let debuff = make_party_debuff("Weakened Armor", 113746, 136127, "target", 2);
    match i {
        0 => (vec![buff], vec![]),
        1 => (vec![], vec![debuff]),
        2 => (vec![buff], vec![debuff]),
        _ => (vec![], vec![]),
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
        if max_delta == 0 {
            continue;
        }
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
const BUFF_POOL: &[(&'static str, i32, i32, f64, &str, bool)] = &[
    (
        "Power Word: Fortitude",
        21562,
        135987,
        3600.0,
        "player",
        true,
    ),
    ("Arcane Intellect", 1459, 135932, 3600.0, "party2", true),
    ("Mark of the Wild", 1126, 136078, 3600.0, "party3", true),
    ("Battle Shout", 6673, 132333, 3600.0, "party1", true),
    ("Retribution Aura", 183435, 135889, 0.0, "player", false),
    ("Devotion Aura", 465, 135893, 0.0, "player", false),
    (
        "Blessing of the Bronze",
        381748,
        4622449,
        3600.0,
        "party4",
        true,
    ),
    ("Well Fed", 104280, 136000, 3600.0, "player", false),
];

/// Pick random buffs from the pool (disabled by WOW_SIM_NO_BUFFS=1).
pub fn default_player_buffs() -> Vec<AuraInfo> {
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
            let expiration_time = if duration > 0.0 {
                get_time + duration
            } else {
                0.0
            };
            AuraInfo {
                name: name.to_string(),
                spell_id,
                icon,
                duration,
                expiration_time,
                applications: 0,
                source_unit: source.to_string(),
                is_helpful: true,
                is_stealable: false,
                can_apply_aura: can_apply,
                is_from_player_or_player_pet: source == "player",
                dispel_type: None,
                aura_instance_id: (i + 1) as i32,
            }
        })
        .collect()
}

/// Major Factions for the modern Journeys panel. Drives the EncounterJournal
/// "Journeys" panel via `C_MajorFactions.GetMajorFactionIDs` and the per-id
/// `GetMajorFactionData` lookup. Faction ids match `MajorFactions.db2`:
///
/// Midnight (`expansion_filter = 11`):
/// - 2710 Silvermoon Court
/// - 2696 Amani Tribe
/// - 2704 Hara'ti
/// - 2699 The Singularity
///
/// The War Within (`expansion_filter = 10`):
/// - 2590 Council of Dornogal
/// - 2570 Hallowfall Arathi
/// - 2594 The Assembly of the Deeps
/// - 2600 The Severed Threads
pub fn default_major_factions() -> HashMap<i64, MajorFactionData> {
    let rows = default_major_faction_rows();
    rows.iter()
        .enumerate()
        .map(|(index, row)| {
            let priority = (rows.len() - index) as i32;
            (row.faction_id, row.to_data(priority))
        })
        .collect()
}

struct MajorFactionRow {
    faction_id: i64,
    name: &'static str,
    expansion_filter: i32,
    texture_kit: &'static str,
    faction_font_color: (f32, f32, f32),
}

impl MajorFactionRow {
    fn to_data(&self, ui_priority: i32) -> MajorFactionData {
        MajorFactionData {
            faction_id: self.faction_id,
            name: self.name.to_string(),
            expansion_filter: self.expansion_filter,
            max_level: 20,
            renown_level: 1,
            renown_reputation_earned: 0,
            renown_level_threshold: 2500,
            ui_priority,
            is_unlocked: true,
            unlock_description: None,
            celebration_sound_kit: 0,
            renown_fanfare_sound_kit_id: 0,
            texture_kit: self.texture_kit.to_string(),
            faction_font_color: self.faction_font_color,
        }
    }
}

fn default_major_faction_rows() -> Vec<MajorFactionRow> {
    vec![
        (2710, "Silvermoon Court", 11, "light", (1.00, 0.82, 0.36)),
        (2696, "Amani Tribe", 11, "sky", (0.50, 0.82, 0.36)),
        (2704, "Hara'ti", 11, "root", (0.86, 0.52, 0.35)),
        (2699, "The Singularity", 11, "origin", (0.56, 0.58, 1.00)),
        (2590, "Council of Dornogal", 10, "storm", (0.96, 0.78, 0.40)),
        (2570, "Hallowfall Arathi", 10, "flame", (0.99, 0.91, 0.62)),
        (
            2594,
            "The Assembly of the Deeps",
            10,
            "candle",
            (0.51, 0.78, 0.55),
        ),
        (2600, "The Severed Threads", 10, "web", (0.45, 0.78, 0.86)),
    ]
    .into_iter()
    .map(
        |(faction_id, name, expansion_filter, texture_kit, faction_font_color)| MajorFactionRow {
            faction_id,
            name,
            expansion_filter,
            texture_kit,
            faction_font_color,
        },
    )
    .collect()
}

/// Default Renown level table: levels 1..=20 per faction. The mixin uses the
/// last entry's `level` to clamp the bar (`GetMaxLevel`); milestone/capstone
/// flags are not yet driven by any panel we render.
pub fn default_major_faction_renown_levels() -> HashMap<i64, Vec<RenownLevelInfo>> {
    let mut map = HashMap::new();
    for &faction_id in &[2710i64, 2696, 2704, 2699, 2590, 2570, 2594, 2600] {
        let levels = (1..=20)
            .map(|level| RenownLevelInfo {
                faction_id,
                level,
                locked: false,
                is_milestone: false,
                is_capstone: level == 20,
            })
            .collect();
        map.insert(faction_id, levels);
    }
    map
}

/// Pre-populate main action bar (slots 1-12) with Protection Paladin spells.
pub fn default_action_bars() -> HashMap<u32, u32> {
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
