//! Spell lookup facade over generated DB2 exports plus simulator-required
//! supplement rows.
//!
//! Blizzard UI does not define spell names in Lua. Retail resolves these
//! through client spell data, while the simulator keeps a compact generated
//! spell table. Supplement rows cover valid client spell IDs that addon Lua
//! references but the compact table does not currently include.

use crate::spells::{self, SpellInfo};

const SUPPLEMENTAL_SPELLS: &[(u32, SpellInfo)] = &[
    (430, spell("Drink", 136175, 11)),
    (1064, spell("Chain Heal", 136042, 1)),
    (2006, spell("Resurrection", 135955, 5)),
    (2008, spell("Ancestral Spirit", 136077, 5)),
    (7328, spell("Redemption", 135955, 5)),
    (20484, spell("Rebirth", 136080, 5)),
    (20707, spell("Soulstone", 136210, 5)),
    (52042, spell("Healing Stream Totem", 136243, 13)),
    (73920, spell("Healing Rain", 136037, 1)),
    (108280, spell("Healing Tide Totem", 538569, 13)),
    (114083, spell("Restorative Mists", 237590, 1)),
    (114911, spell("Ancestral Guidance", 538564, 1)),
    (50769, spell("Revive", 132132, 5)),
    (61999, spell("Raise Ally", 136143, 5)),
    (115178, spell("Resuscitate", 132132, 5)),
    (132403, spell("Shield of the Righteous", 236265, 2)),
    (132404, spell("Shield Block", 236171, 34)),
    (167152, spell("Refreshment", 589068, 0)),
    (170906, spell("Food & Drink", 132161, 7)),
    (43182, spell("Drink", 132989, 13)),
    (172786, spell("Drink", 132805, 1)),
    (192081, spell("Ironfur", 1378702, 1)),
    (195181, spell("Bone Shield", 458717, 1)),
    (197995, spell("Wellspring", 893778, 1)),
    (203819, spell("Demon Spikes", 1344645, 1)),
    (207778, spell("Downpour", 1698701, 6)),
    (215479, spell("Shuffle", 642416, 1)),
    (212036, spell("Mass Resurrection", 413586, 6)),
    (212040, spell("Revitalize", 132125, 6)),
    (212048, spell("Ancestral Vision", 237576, 6)),
    (212051, spell("Reawaken", 1056569, 6)),
    (212056, spell("Absolution", 1030102, 6)),
    (308433, spell("Food & Drink", 132805, 1)),
    (322118, spell("Invoke Yu'lon, the Jade Serpent", 877514, 1)),
    (369162, spell("Drink", 136243, 0)),
    (377509, spell("Dream Projection", 136243, 34)),
    (382311, spell("Ancestral Awakening", 237571, 5)),
    (361178, spell("Mass Return", 4622473, 6)),
    (361227, spell("Return", 4622472, 5)),
    (391054, spell("Intercession", 4726195, 5)),
    (
        395296,
        SpellInfo {
            name: "Ebon Might",
            subtext: "Black",
            icon_file_data_id: 5061347,
            school_mask: 12,
            implicit_target: 1,
        },
    ),
    (456574, spell("Cinder Nectar", 132805, 1)),
    (461063, spell("Quiet Contemplation", 1499566, 1)),
    (
        1247917,
        SpellInfo {
            name: "Clear Current Transmogrifications",
            subtext: "",
            icon_file_data_id: 7539422,
            school_mask: 1,
            implicit_target: 1,
        },
    ),
];

const fn spell(name: &'static str, icon_file_data_id: u32, implicit_target: u8) -> SpellInfo {
    SpellInfo {
        name,
        subtext: "",
        icon_file_data_id,
        school_mask: 0,
        implicit_target,
    }
}

pub fn get_spell(id: u32) -> Option<&'static SpellInfo> {
    spells::get_spell(id).or_else(|| supplemental_spell(id))
}

fn supplemental_spell(id: u32) -> Option<&'static SpellInfo> {
    SUPPLEMENTAL_SPELLS
        .iter()
        .find_map(|(spell_id, spell)| (*spell_id == id).then_some(spell))
}
