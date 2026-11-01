use crate::lua_api::state::{AlliedRaceInfo, AlliedRaceRacialAbility};
use std::collections::HashMap;

/// Canonical allied-race directory consumed by `C_AlliedRaces.GetRaceInfoByID`.
///
/// `raceID` matches the `RACE_DATA` table seeded for `C_CreatureInfo.GetRaceInfo`
/// in `runtime_surface_bootstrap.lua`. Model IDs are the live Mainline values
/// the `Blizzard_AlliedRacesUI/Blizzard_AlliedRacesFrameUI.lua` actor map uses
/// (`Actor_X_ModelID` table at the bottom of that file). `race_file_string`
/// is the lowercased token the consumer joins onto `RACE_INFO_` to look up
/// the race description.
pub fn default_allied_races() -> HashMap<i64, AlliedRaceInfo> {
    canonical_allied_race_seeds()
        .into_iter()
        .map(|info| (info.race_id, info))
        .collect()
}

fn canonical_allied_race_seeds() -> Vec<AlliedRaceInfo> {
    vec![
        allied_race_lightforged_draenei(),
        allied_race_dark_iron_dwarf(),
        allied_race_void_elf(),
        allied_race_mechagnome(),
        allied_race_vulpera(),
        allied_race_zandalari_troll(),
        allied_race_highmountain_tauren(),
        allied_race_nightborne(),
        allied_race_maghar_orc(),
        allied_race_earthen_dwarf(),
    ]
}

fn allied_race_lightforged_draenei() -> AlliedRaceInfo {
    AlliedRaceInfo {
        race_id: 28,
        male_model_id: 82_729,
        female_model_id: 82_730,
        achievement_ids: vec![12_245, 11_846],
        male_name: "Lightforged Draenei".to_string(),
        female_name: "Lightforged Draenei".to_string(),
        description: "Veterans of the Army of the Light.".to_string(),
        race_file_string: "lightforgeddraenei".to_string(),
        crest_atlas: "alliedraces-icon-lightforgeddraenei".to_string(),
        model_background_atlas: "alliedraces-background-lightforgeddraenei".to_string(),
        banner_color: (0.94, 0.83, 0.36),
        racial_abilities: lightforged_draenei_abilities(),
    }
}

fn allied_race_dark_iron_dwarf() -> AlliedRaceInfo {
    AlliedRaceInfo {
        race_id: 31,
        male_model_id: 87_992,
        female_model_id: 87_993,
        achievement_ids: vec![12_241, 12_843],
        male_name: "Dark Iron Dwarf".to_string(),
        female_name: "Dark Iron Dwarf".to_string(),
        description: "Fire-forged dwarves of Blackrock.".to_string(),
        race_file_string: "darkirondwarf".to_string(),
        crest_atlas: "alliedraces-icon-darkirondwarf".to_string(),
        model_background_atlas: "alliedraces-background-darkirondwarf".to_string(),
        banner_color: (0.65, 0.16, 0.12),
        racial_abilities: dark_iron_dwarf_abilities(),
    }
}

fn allied_race_void_elf() -> AlliedRaceInfo {
    AlliedRaceInfo {
        race_id: 27,
        male_model_id: 82_736,
        female_model_id: 82_735,
        achievement_ids: vec![12_243, 11_843],
        male_name: "Void Elf".to_string(),
        female_name: "Void Elf".to_string(),
        description: "Ren'dorei shaped by the Void.".to_string(),
        race_file_string: "voidelf".to_string(),
        crest_atlas: "alliedraces-icon-voidelf".to_string(),
        model_background_atlas: "alliedraces-background-voidelf".to_string(),
        banner_color: (0.41, 0.27, 0.62),
        racial_abilities: void_elf_abilities(),
    }
}

fn allied_race_mechagnome() -> AlliedRaceInfo {
    AlliedRaceInfo {
        race_id: 34,
        male_model_id: 94_370,
        female_model_id: 94_371,
        achievement_ids: vec![13_710, 14_013],
        male_name: "Mechagnome".to_string(),
        female_name: "Mechagnome".to_string(),
        description: "Tinkerers enhanced with machinery.".to_string(),
        race_file_string: "mechagnome".to_string(),
        crest_atlas: "alliedraces-icon-mechagnome".to_string(),
        model_background_atlas: "alliedraces-background-mechagnome".to_string(),
        banner_color: (0.84, 0.71, 0.27),
        racial_abilities: mechagnome_abilities(),
    }
}

fn allied_race_vulpera() -> AlliedRaceInfo {
    AlliedRaceInfo {
        race_id: 35,
        male_model_id: 94_257,
        female_model_id: 94_256,
        achievement_ids: vec![13_435, 14_012],
        male_name: "Vulpera".to_string(),
        female_name: "Vulpera".to_string(),
        description: "Resourceful nomads of Vol'dun.".to_string(),
        race_file_string: "vulpera".to_string(),
        crest_atlas: "alliedraces-icon-vulpera".to_string(),
        model_background_atlas: "alliedraces-background-vulpera".to_string(),
        banner_color: (0.74, 0.40, 0.16),
        racial_abilities: vulpera_abilities(),
    }
}

fn allied_race_zandalari_troll() -> AlliedRaceInfo {
    AlliedRaceInfo {
        race_id: 29,
        male_model_id: 89_631,
        female_model_id: 89_632,
        achievement_ids: vec![12_244, 12_775],
        male_name: "Zandalari Troll".to_string(),
        female_name: "Zandalari Troll".to_string(),
        description: "Ancient kings of troll empires.".to_string(),
        race_file_string: "zandalaritroll".to_string(),
        crest_atlas: "alliedraces-icon-zandalaritroll".to_string(),
        model_background_atlas: "alliedraces-background-zandalaritroll".to_string(),
        banner_color: (0.85, 0.18, 0.18),
        racial_abilities: zandalari_troll_abilities(),
    }
}

fn allied_race_highmountain_tauren() -> AlliedRaceInfo {
    AlliedRaceInfo {
        race_id: 26,
        male_model_id: 82_733,
        female_model_id: 82_731,
        achievement_ids: vec![12_242, 11_851],
        male_name: "Highmountain Tauren".to_string(),
        female_name: "Highmountain Tauren".to_string(),
        description: "Descendants of Huln Highmountain.".to_string(),
        race_file_string: "highmountaintauren".to_string(),
        crest_atlas: "alliedraces-icon-highmountaintauren".to_string(),
        model_background_atlas: "alliedraces-background-highmountaintauren".to_string(),
        banner_color: (0.43, 0.29, 0.20),
        racial_abilities: highmountain_tauren_abilities(),
    }
}

fn allied_race_nightborne() -> AlliedRaceInfo {
    AlliedRaceInfo {
        race_id: 25,
        male_model_id: 82_708,
        female_model_id: 82_709,
        achievement_ids: vec![12_240, 11_842],
        male_name: "Nightborne".to_string(),
        female_name: "Nightborne".to_string(),
        description: "Arcwine-fueled children of Suramar.".to_string(),
        race_file_string: "nightborne".to_string(),
        crest_atlas: "alliedraces-icon-nightborne".to_string(),
        model_background_atlas: "alliedraces-background-nightborne".to_string(),
        banner_color: (0.62, 0.39, 0.85),
        racial_abilities: nightborne_abilities(),
    }
}

fn allied_race_maghar_orc() -> AlliedRaceInfo {
    AlliedRaceInfo {
        race_id: 32,
        male_model_id: 86_343,
        female_model_id: 86_342,
        achievement_ids: vec![12_246, 12_512],
        male_name: "Mag'har Orc".to_string(),
        female_name: "Mag'har Orc".to_string(),
        description: "Uncorrupted clans from alternate Draenor.".to_string(),
        race_file_string: "magharorc".to_string(),
        crest_atlas: "alliedraces-icon-magharorc".to_string(),
        model_background_atlas: "alliedraces-background-magharorc".to_string(),
        banner_color: (0.65, 0.40, 0.27),
        racial_abilities: maghar_orc_abilities(),
    }
}

fn allied_race_earthen_dwarf() -> AlliedRaceInfo {
    AlliedRaceInfo {
        race_id: 37,
        male_model_id: 121_634,
        female_model_id: 121_635,
        achievement_ids: vec![19_017],
        male_name: "Earthen".to_string(),
        female_name: "Earthen".to_string(),
        description: "Titan-forged explorers of the deep places.".to_string(),
        race_file_string: "earthendwarf".to_string(),
        crest_atlas: "alliedraces-icon-earthen".to_string(),
        model_background_atlas: "alliedraces-background-earthen".to_string(),
        banner_color: (0.56, 0.46, 0.36),
        racial_abilities: earthen_dwarf_abilities(),
    }
}

fn lightforged_draenei_abilities() -> Vec<AlliedRaceRacialAbility> {
    vec![
        ability(
            "Demonbane",
            "Increased experience from killing demons.",
            1_103_069,
        ),
        ability(
            "Light's Judgment",
            "Call down a strike of Holy energy on a target location.",
            1_500_852,
        ),
        ability(
            "Forge of Light",
            "Allows the use of a draenei forge to create or repair items.",
            1_500_853,
        ),
        ability(
            "Holy Resistance",
            "Reduces all incoming Holy damage taken.",
            1_500_854,
        ),
    ]
}

fn dark_iron_dwarf_abilities() -> Vec<AlliedRaceRacialAbility> {
    vec![
        ability(
            "Dungeon Delver",
            "Increases your movement speed while in dungeons.",
            1_604_167,
        ),
        ability(
            "Forged in Flames",
            "Reduces damage taken from Physical attacks.",
            1_604_168,
        ),
        ability(
            "Fireblood",
            "Removes all poison, disease, curse, magic, and bleed effects.",
            1_604_166,
        ),
        ability(
            "Mole Machine",
            "Drill an underground tunnel to a Dark Iron meeting stone.",
            1_604_165,
        ),
    ]
}

fn void_elf_abilities() -> Vec<AlliedRaceRacialAbility> {
    vec![
        ability(
            "Spatial Rift",
            "Open a rift in space to teleport to its location.",
            1_500_866,
        ),
        ability(
            "Chill of Night",
            "Reduces damage taken from Shadow attacks.",
            1_500_868,
        ),
        ability(
            "Preternatural Calm",
            "Casting time of all spells is reduced.",
            1_500_869,
        ),
        ability(
            "Entropic Embrace",
            "Your damage and healing have a chance to be empowered by the Void.",
            1_500_867,
        ),
    ]
}

fn mechagnome_abilities() -> Vec<AlliedRaceRacialAbility> {
    vec![
        ability(
            "Combat Analysis",
            "Periodically analyze your enemies, increasing your damage and healing done to them.",
            3_528_286,
        ),
        ability(
            "Skeleton Pinkie",
            "Acts as a skeleton key, allowing you to open locks.",
            3_528_287,
        ),
        ability(
            "Hyper Organic Light Originator",
            "Reduces all incoming Holy damage taken.",
            3_528_288,
        ),
        ability(
            "Emergency Failsafe",
            "When you fall below 20% health, you are healed for a moderate amount.",
            3_528_289,
        ),
    ]
}

fn vulpera_abilities() -> Vec<AlliedRaceRacialAbility> {
    vec![
        ability(
            "Bag of Tricks",
            "Pull out an interesting trinket and use it on yourself or an enemy.",
            3_528_303,
        ),
        ability(
            "Make Camp",
            "Make camp at your current location.",
            3_528_304,
        ),
        ability(
            "Nose for Trouble",
            "Damage taken is reduced when above 50% health.",
            3_528_305,
        ),
        ability(
            "Vulpera Cunning",
            "Your starting reputation with all Horde factions is increased.",
            3_528_306,
        ),
    ]
}

fn zandalari_troll_abilities() -> Vec<AlliedRaceRacialAbility> {
    vec![
        ability(
            "Embrace of Pa'ku",
            "Pa'ku grants you increased critical strike chance.",
            2_192_734,
        ),
        ability(
            "City of Gold",
            "Increases gold gained from quest rewards.",
            2_192_730,
        ),
        ability(
            "Pterrordax Swoop",
            "Use a Pterrordax to swoop down and attack a target.",
            2_192_733,
        ),
        ability(
            "Regeneratin'",
            "Channel to recover health over time.",
            2_192_735,
        ),
    ]
}

fn highmountain_tauren_abilities() -> Vec<AlliedRaceRacialAbility> {
    vec![
        ability(
            "Mountaineer",
            "Reduces all damage taken by a small amount.",
            1_500_859,
        ),
        ability(
            "Bull Rush",
            "Charge forward, knocking enemies down.",
            1_500_858,
        ),
        ability(
            "Pride of Ironhorn",
            "Mining skill is increased and you can mine faster.",
            1_500_860,
        ),
        ability(
            "Rugged Tenacity",
            "Reduces damage taken when below 50% health.",
            1_500_861,
        ),
    ]
}

fn nightborne_abilities() -> Vec<AlliedRaceRacialAbility> {
    vec![
        ability(
            "Cantrips",
            "Conjure a Mana Pudding for yourself or your party.",
            1_500_862,
        ),
        ability(
            "Magical Affinity",
            "Increases all magical damage dealt by a small amount.",
            1_500_863,
        ),
        ability(
            "Arcane Pulse",
            "Release a pulse of arcane energy, damaging nearby enemies.",
            1_500_864,
        ),
        ability(
            "Ancient History",
            "Inscription skill is increased and you can mill herbs faster.",
            1_500_865,
        ),
    ]
}

fn maghar_orc_abilities() -> Vec<AlliedRaceRacialAbility> {
    vec![
        ability("Open Skies", "Mounted speed is increased.", 1_604_169),
        ability(
            "Ancestral Call",
            "Invoke the spirits of your ancestors, granting you a random bonus stat.",
            1_604_170,
        ),
        ability(
            "Sympathetic Vigor",
            "Your pets gain extra health.",
            1_604_171,
        ),
        ability(
            "Savage Blood",
            "Reduces the duration of incoming poisons, diseases, and curses.",
            1_604_172,
        ),
    ]
}

fn earthen_dwarf_abilities() -> Vec<AlliedRaceRacialAbility> {
    vec![
        ability(
            "Azerite Surge",
            "Channel the residual energy of Azerite to deal damage to nearby enemies.",
            5_899_363,
        ),
        ability(
            "Ingest Mineral",
            "Eat a mineral to recover health over time.",
            5_899_364,
        ),
        ability(
            "Stoneskin",
            "Toughen your skin, reducing damage taken from physical attacks.",
            5_899_365,
        ),
        ability(
            "Hold Your Ground",
            "Reduces the effect of knockback effects.",
            5_899_366,
        ),
    ]
}

fn ability(name: &str, description: &str, icon: i64) -> AlliedRaceRacialAbility {
    AlliedRaceRacialAbility {
        name: name.to_string(),
        description: description.to_string(),
        icon,
    }
}

/// Script tags for the generated model scenes; 3D rendering remains unsupported.
pub fn default_model_scenes() -> HashMap<i64, Vec<String>> {
    let mut scenes: HashMap<i64, Vec<String>> = HashMap::new();
    for &(_, scene, tag) in sim_data::model_scenes::ACTORS {
        scenes.entry(scene).or_default().push(tag.to_owned());
    }
    scenes
}
