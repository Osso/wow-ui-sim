//! Paladin spellbook data for the WoW UI simulator.
//!
//! Organizes spells into skill lines matching WoW's spellbook structure:
//! - General (racial/class-neutral spells)
//! - Class (Paladin baseline spells)
//! - Protection (active spec)
//! - Holy (off-spec)
//! - Retribution (off-spec)

/// A single spell entry in the spellbook.
#[derive(Debug, Clone, Copy)]
pub struct SpellBookEntry {
    pub spell_id: u32,
    pub is_passive: bool,
}

/// A skill line (tab/group) in the spellbook.
#[derive(Debug, Clone)]
pub struct SkillLineData {
    pub name: &'static str,
    pub icon_id: u32,
    pub spec_id: Option<i32>,
    pub off_spec_id: Option<i32>,
    pub spells: &'static [SpellBookEntry],
}

const fn spell(spell_id: u32) -> SpellBookEntry {
    SpellBookEntry {
        spell_id,
        is_passive: false,
    }
}

const fn passive(spell_id: u32) -> SpellBookEntry {
    SpellBookEntry {
        spell_id,
        is_passive: true,
    }
}

/// General spells available to all classes.
static GENERAL_SPELLS: &[SpellBookEntry] = &[
    spell(6603), // Auto Attack
    spell(8690), // Hearthstone
    spell(7328), // Redemption
];

/// Paladin class baseline spells (shared across all specs).
static CLASS_SPELLS: &[SpellBookEntry] = &[
    spell(35395),   // Crusader Strike
    spell(19750),   // Flash of Light
    spell(85673),   // Word of Glory
    spell(853),     // Hammer of Justice
    spell(275779),  // Judgment
    spell(465),     // Devotion Aura
    spell(1022),    // Blessing of Protection
    spell(1044),    // Blessing of Freedom
    spell(642),     // Divine Shield
    spell(633),     // Lay on Hands
    spell(190784),  // Divine Steed
    spell(96231),   // Rebuke
    spell(10326),   // Turn Evil
    spell(213644),  // Cleanse Toxins
    spell(6940),    // Blessing of Sacrifice
    spell(31884),   // Avenging Wrath
    spell(375576),  // Divine Toll
    spell(115750),  // Blinding Light
    spell(32223),   // Crusader Aura
    spell(317920),  // Concentration Aura
    spell(183435),  // Retribution Aura
    spell(5502),    // Sense Undead
    spell(121183),  // Contemplation
    passive(137026), // Plate Specialization
    passive(385125), // Of Dusk and Dawn
];

/// Protection specialization spells.
static PROTECTION_SPELLS: &[SpellBookEntry] = &[
    spell(31935),   // Avenger's Shield
    spell(53595),   // Hammer of the Righteous
    spell(26573),   // Consecration
    spell(53600),   // Shield of the Righteous
    spell(31850),   // Ardent Defender
    spell(86659),   // Guardian of Ancient Kings
    spell(62124),   // Hand of Reckoning
    spell(498),     // Divine Protection
    spell(327193),  // Moment of Glory
    spell(378974),  // Bastion of Light
    spell(387174),  // Eye of Tyr
    spell(204019),  // Blessed Hammer
    passive(85043),  // Grand Crusader
    passive(152261), // Holy Shield
    passive(76671),  // Mastery: Divine Bulwark
    passive(280373), // Redoubt
];

/// Holy specialization spells (off-spec when Protection is active).
static HOLY_SPELLS: &[SpellBookEntry] = &[
    spell(20473),   // Holy Shock
    spell(82326),   // Holy Light
    spell(85222),   // Light of Dawn
    spell(4987),    // Cleanse
    spell(53563),   // Beacon of Light
    spell(105809),  // Holy Avenger
    spell(200652),  // Tyr's Deliverance
    passive(53576),  // Infusion of Light
    passive(183997), // Mastery: Lightbringer
];

/// Retribution specialization spells (off-spec when Protection is active).
static RETRIBUTION_SPELLS: &[SpellBookEntry] = &[
    spell(184575),  // Blade of Justice
    spell(85256),   // Templar's Verdict
    spell(255937),  // Wake of Ashes
    spell(184662),  // Shield of Vengeance
    spell(343527),  // Execution Sentence
    spell(343721),  // Final Reckoning
    spell(383185),  // Exorcism
    passive(267344), // Art of War
    passive(231832), // Blade of Wrath
    passive(269569), // Zeal
];

/// All skill lines for a Protection Paladin, in WoW skill line index order.
/// Index 1 = General, 2 = Class, 3 = MainSpec (Protection), 4+ = OffSpecs.
static SKILL_LINES: &[SkillLineData] = &[
    SkillLineData {
        name: "General",
        icon_id: 136243,
        spec_id: None,
        off_spec_id: None,
        spells: GENERAL_SPELLS,
    },
    SkillLineData {
        name: "Paladin",
        icon_id: 135920,
        spec_id: None,
        off_spec_id: None,
        spells: CLASS_SPELLS,
    },
    SkillLineData {
        name: "Protection",
        icon_id: 236264,
        spec_id: Some(66),
        off_spec_id: None,
        spells: PROTECTION_SPELLS,
    },
    SkillLineData {
        name: "Holy",
        icon_id: 135920,
        spec_id: Some(65),
        off_spec_id: Some(65),
        spells: HOLY_SPELLS,
    },
    SkillLineData {
        name: "Retribution",
        icon_id: 135873,
        spec_id: Some(70),
        off_spec_id: Some(70),
        spells: RETRIBUTION_SPELLS,
    },
];

/// Number of skill lines.
pub fn num_skill_lines() -> i32 {
    SKILL_LINES.len() as i32
}

/// Get skill line data by 1-based index.
pub fn get_skill_line(index: i32) -> Option<&'static SkillLineData> {
    let idx = (index - 1) as usize;
    SKILL_LINES.get(idx)
}

/// Convert a global spellbook slot index (1-based) to a spell entry.
/// Slots are contiguous across all skill lines.
pub fn get_spell_at_slot(
    slot: i32,
) -> Option<(i32, &'static SpellBookEntry, &'static SkillLineData)> {
    let mut offset = 0i32;
    for skill_line in SKILL_LINES {
        let count = skill_line.spells.len() as i32;
        if slot > offset && slot <= offset + count {
            let local_idx = (slot - offset - 1) as usize;
            let skill_line_index = (SKILL_LINES
                .iter()
                .position(|s| std::ptr::eq(s, skill_line))
                .unwrap()
                + 1) as i32;
            return Some((skill_line_index, &skill_line.spells[local_idx], skill_line));
        }
        offset += count;
    }
    None
}

/// Get the slot index offset for a skill line (0-based, for UI itemIndexOffset).
pub fn skill_line_offset(index: i32) -> i32 {
    let idx = (index - 1) as usize;
    let mut offset = 0i32;
    for (i, skill_line) in SKILL_LINES.iter().enumerate() {
        if i == idx {
            return offset;
        }
        offset += skill_line.spells.len() as i32;
    }
    offset
}

/// Check if a spell ID is known (exists in any non-offspec skill line).
pub fn is_spell_known(spell_id: u32) -> bool {
    for skill_line in SKILL_LINES {
        if skill_line.off_spec_id.is_some() {
            continue; // Off-spec spells are not "known"
        }
        for entry in skill_line.spells {
            if entry.spell_id == spell_id {
                return true;
            }
        }
    }
    false
}

/// Find a spell ID by name (case-insensitive).
pub fn find_spell_by_name(name: &str) -> Option<u32> {
    for skill_line in SKILL_LINES {
        for entry in skill_line.spells {
            if let Some(spell) = crate::spells::get_spell(entry.spell_id) {
                if spell.name.eq_ignore_ascii_case(name) {
                    return Some(entry.spell_id);
                }
            }
        }
    }
    None
}

/// Find the spellbook slot for a given spell ID.
/// Returns (slot_index, spell_bank) where spell_bank is 0 for player.
pub fn find_spell_slot(spell_id: u32) -> Option<(i32, i32)> {
    let mut offset = 0i32;
    for skill_line in SKILL_LINES {
        for (i, entry) in skill_line.spells.iter().enumerate() {
            if entry.spell_id == spell_id {
                return Some((offset + i as i32 + 1, 0));
            }
        }
        offset += skill_line.spells.len() as i32;
    }
    None
}
