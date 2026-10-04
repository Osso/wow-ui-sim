//! 12.1.0 aura filter strings: `!` negation, DISPELLABLE, IMPORTANT and
//! RAID_PLAYER_DISPELLABLE on enemies, through the public C_UnitAuras getters.
#![cfg(feature = "retail-12-1-0")]

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::AuraInfo;

/// (instance, spell, helpful, own, raid, stealable, dispel type)
type AuraSeed = (i32, i32, bool, bool, bool, bool, Option<&'static str>);

const PLAYER_AURAS: &[AuraSeed] = &[
    (41, 9041, true, true, true, false, None),
    (42, 9042, true, false, false, false, Some("Magic")),
    (43, 9043, false, false, false, false, Some("Curse")),
    (44, 9044, false, false, false, false, Some("Poison")),
    (45, 9045, false, false, false, false, None),
];

const TARGET_AURAS: &[AuraSeed] = &[
    (61, 9061, true, false, false, false, Some("Magic")),
    (62, 9062, true, false, false, true, None),
    (63, 9063, true, false, false, false, Some("Enrage")),
    (64, 9064, false, true, false, false, Some("Curse")),
];

fn seed_auras(template: &AuraInfo, seeds: &[AuraSeed]) -> Vec<AuraInfo> {
    seeds
        .iter()
        .map(|&(id, spell, helpful, own, raid, stealable, dispel)| {
            let mut aura = template.clone();
            aura.aura_instance_id = id;
            aura.spell_id = spell;
            aura.name = format!("Aura {id}");
            aura.is_helpful = helpful;
            aura.is_from_player_or_player_pet = own;
            aura.is_raid = raid;
            aura.is_stealable = stealable;
            aura.is_nameplate_only = false;
            aura.dispel_type = dispel.map(str::to_owned);
            aura
        })
        .collect()
}

fn seeded_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.exec("TargetUnit('party1')").unwrap();
    {
        let mut state = env.state().borrow_mut();
        let template = state.player.buffs[0].clone();
        state.player.buffs = seed_auras(&template, PLAYER_AURAS);
        state.target_auras = seed_auras(&template, TARGET_AURAS);
        let target = state.current_target.as_mut().unwrap();
        target.is_enemy = true;
        target.reaction = 2;
        let facts = &mut state.aura_filter_facts;
        facts.important_spell_ids.extend([9042, 9045]);
        facts.raid_defensive_dispel_types.insert("Curse".into());
        facts.raid_offensive_dispel_types.insert("Magic".into());
        facts.raid_can_spellsteal = true;
    }
    env.exec(
        r#"
        function Ids(unit, filter)
            return table.concat(C_UnitAuras.GetUnitAuraInstanceIDs(unit, filter), ',')
        end
        "#,
    )
    .unwrap();
    env
}

/// (unit, filter, expected instance IDs)
const FILTER_CASES: &[(&str, &str, &str)] = &[
    ("player", "HELPFUL", "41,42"),
    ("player", "HARMFUL", "43,44,45"),
    ("player", "HELPFUL|PLAYER", "41"),
    ("player", "HELPFUL|!PLAYER", "42"),
    ("player", "!HELPFUL", "43,44,45"),
    ("player", "!HARMFUL", "41,42"),
    ("player", "HELPFUL|RAID", "41"),
    ("player", "helpful | !raid", "42"),
    ("player", "HARMFUL|DISPELLABLE", "43,44"),
    ("player", "HARMFUL|!DISPELLABLE", "45"),
    ("player", "HELPFUL|DISPELLABLE", "42"),
    ("player", "HELPFUL|IMPORTANT", "42"),
    ("player", "HARMFUL|IMPORTANT", "45"),
    ("player", "HARMFUL|!IMPORTANT", "43,44"),
    ("player", "HARMFUL|RAID_PLAYER_DISPELLABLE", "43"),
    ("player", "HARMFUL|!RAID_PLAYER_DISPELLABLE", "44,45"),
    ("player", "HELPFUL|RAID_PLAYER_DISPELLABLE", ""),
    ("player", "HELPFUL|MAW", ""),
    ("player", "HELPFUL|!MAW", ""),
    ("target", "HELPFUL|RAID_PLAYER_DISPELLABLE", "61,62"),
    ("target", "HELPFUL|!RAID_PLAYER_DISPELLABLE", "63"),
    ("target", "HARMFUL|RAID_PLAYER_DISPELLABLE", ""),
    ("target", "HELPFUL|DISPELLABLE", "61,63"),
    ("target", "HARMFUL|PLAYER|DISPELLABLE", "64"),
];

#[test]
fn aura_filter_strings_select_instance_ids() {
    let env = seeded_env();
    for (unit, filter, expected) in FILTER_CASES {
        let actual: String = env
            .eval(&format!("return Ids('{unit}', '{filter}')"))
            .unwrap();
        assert_eq!(&actual, expected, "{unit} {filter}");
    }
}

#[test]
fn raid_player_dispellable_follows_raid_capabilities_and_hostility() {
    let env = seeded_env();
    let ids = |unit: &str, filter: &str| -> String {
        env.eval(&format!("return Ids('{unit}', '{filter}')"))
            .unwrap()
    };
    env.state().borrow_mut().aura_filter_facts.raid_can_spellsteal = false;
    assert_eq!(ids("target", "HELPFUL|RAID_PLAYER_DISPELLABLE"), "61");
    env.state()
        .borrow_mut()
        .aura_filter_facts
        .raid_offensive_dispel_types
        .insert("Enrage".into());
    assert_eq!(ids("target", "HELPFUL|RAID_PLAYER_DISPELLABLE"), "61,63");
    {
        let mut state = env.state().borrow_mut();
        let target = state.current_target.as_mut().unwrap();
        target.is_enemy = false;
        target.reaction = 5;
    }
    assert_eq!(ids("target", "HELPFUL|RAID_PLAYER_DISPELLABLE"), "");
    env.state()
        .borrow_mut()
        .aura_filter_facts
        .raid_defensive_dispel_types
        .clear();
    assert_eq!(ids("player", "HARMFUL|RAID_PLAYER_DISPELLABLE"), "");
}

#[test]
fn filtered_out_query_applies_the_same_filter_strings() {
    let env = seeded_env();
    env.exec(
        r#"
        local filtered = C_UnitAuras.IsAuraFilteredOutByInstanceID
        assert(filtered('player', 43, 'HARMFUL|RAID_PLAYER_DISPELLABLE') == false)
        assert(filtered('player', 44, 'HARMFUL|RAID_PLAYER_DISPELLABLE') == true)
        assert(filtered('player', 45, 'HARMFUL|!DISPELLABLE') == false)
        assert(filtered('player', 43, 'HARMFUL|!DISPELLABLE') == true)
        assert(filtered('player', 42, 'HELPFUL|!IMPORTANT') == true)
        assert(filtered('player', 41, 'HELPFUL|!IMPORTANT') == false)
        assert(filtered('target', 62, 'HELPFUL|RAID_PLAYER_DISPELLABLE') == false)
        assert(filtered('target', 63, 'HELPFUL|RAID_PLAYER_DISPELLABLE') == true)
        "#,
    )
    .unwrap();
}
