//! Game simulation data types, constants, and helpers.
//!
//! Contains player/party/target/aura definitions and the default data
//! generators used by `SimState::default()`.

use std::collections::HashMap;

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

/// Rot damage intensity levels: (label, percentage of max HP per tick).
pub const ROT_DAMAGE_LEVELS: &[(&str, f64)] = &[
    ("Light (1%)", 0.01),
    ("Medium (3%)", 0.03),
    ("Heavy (5%)", 0.05),
    ("Brutal (10%)", 0.10),
];

/// Pick a random WoW-style player name using the current time as a seed.
pub fn random_player_name() -> String {
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
pub fn default_party() -> Vec<PartyMember> {
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
pub fn build_target_info(unit_id: &str, state: &super::state::SimState) -> Option<TargetInfo> {
    match unit_id {
        "player" => Some(build_player_target(state)),
        u if u.starts_with("party") => build_party_target(u, state),
        "enemy1" => Some(build_enemy_target()),
        _ => None,
    }
}

fn build_player_target(state: &super::state::SimState) -> TargetInfo {
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

fn build_party_target(unit_id: &str, state: &super::state::SimState) -> Option<TargetInfo> {
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
