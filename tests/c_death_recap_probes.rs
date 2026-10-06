//! Tests for `C_DeathRecap` probes backed by `SimState.death_recaps`.

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{DeathRecapEntry, KillingBlowInfo};

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

fn sample_death_recap() -> DeathRecapEntry {
    DeathRecapEntry {
        recap_id: 1,
        max_health: 100000,
        zone_name: "Icecrown Citadel".into(),
        killing_blows: vec![
            KillingBlowInfo {
                spell_id: 49560,
                ability_name: "Death Grip".into(),
                caster_name: "The Lich King".into(),
                amount: 98000,
                is_overkill: true,
            },
            KillingBlowInfo {
                spell_id: 70541,
                ability_name: "Infest".into(),
                caster_name: "The Lich King".into(),
                amount: 15000,
                is_overkill: false,
            },
        ],
    }
}

#[test]
fn get_killing_blows_returns_empty_array_when_no_deaths() {
    let env = env();
    let count: i32 = env.eval("return #C_DeathRecap.GetKillingBlows()").unwrap();
    assert_eq!(count, 0, "default state has no death recaps");
}

#[test]
fn get_most_recent_death_recap_returns_nil_when_empty() {
    let env = env();
    let is_nil: bool = env
        .eval("return C_DeathRecap.GetMostRecentDeathRecap() == nil")
        .unwrap();
    assert!(is_nil, "should return nil when death_recaps is empty");
}

#[test]
fn get_killing_blows_returns_array_after_seeding() {
    let env = env();
    {
        let mut state = env.state().borrow_mut();
        state.death_recaps.push(sample_death_recap());
    }
    let count: i32 = env.eval("return #C_DeathRecap.GetKillingBlows()").unwrap();
    assert_eq!(count, 2, "seeded recap has 2 killing blows");
}

#[test]
fn get_killing_blows_fields_match_seeded_data() {
    let env = env();
    {
        let mut state = env.state().borrow_mut();
        state.death_recaps.push(sample_death_recap());
    }
    let (spell_id, ability_name, caster_name, amount, is_overkill): (
        i32,
        String,
        String,
        i64,
        bool,
    ) = env
        .eval(
            r#"
            local blows = C_DeathRecap.GetKillingBlows()
            local b = blows[1]
            return b.spellID, b.abilityName, b.casterName, b.amount, b.isOverkill
            "#,
        )
        .unwrap();
    assert_eq!(spell_id, 49560);
    assert_eq!(ability_name, "Death Grip");
    assert_eq!(caster_name, "The Lich King");
    assert_eq!(amount, 98000);
    assert!(is_overkill);
}

#[test]
fn get_most_recent_death_recap_returns_table_with_correct_fields() {
    let env = env();
    {
        let mut state = env.state().borrow_mut();
        state.death_recaps.push(sample_death_recap());
    }
    let (recap_id, zone_name, blow_count): (i32, String, i32) = env
        .eval(
            r#"
            local r = C_DeathRecap.GetMostRecentDeathRecap()
            return r.recapID, r.zoneName, #r.killingBlows
            "#,
        )
        .unwrap();
    assert_eq!(recap_id, 1);
    assert_eq!(zone_name, "Icecrown Citadel");
    assert_eq!(blow_count, 2);
}

#[test]
fn get_killing_blows_reflects_most_recent_death() {
    let env = env();
    {
        let mut state = env.state().borrow_mut();
        // First death with 1 blow
        state.death_recaps.push(DeathRecapEntry {
            recap_id: 1,
            max_health: 10000,
            zone_name: "Stormwind".into(),
            killing_blows: vec![KillingBlowInfo {
                spell_id: 1,
                ability_name: "Stab".into(),
                caster_name: "Rogue".into(),
                amount: 9999,
                is_overkill: true,
            }],
        });
        // Second (most recent) death with 3 blows
        state.death_recaps.push(DeathRecapEntry {
            recap_id: 2,
            max_health: 20000,
            zone_name: "Orgrimmar".into(),
            killing_blows: vec![
                KillingBlowInfo {
                    spell_id: 10,
                    ability_name: "A".into(),
                    caster_name: "Warrior".into(),
                    amount: 1000,
                    is_overkill: false,
                },
                KillingBlowInfo {
                    spell_id: 11,
                    ability_name: "B".into(),
                    caster_name: "Mage".into(),
                    amount: 2000,
                    is_overkill: false,
                },
                KillingBlowInfo {
                    spell_id: 12,
                    ability_name: "C".into(),
                    caster_name: "Priest".into(),
                    amount: 3000,
                    is_overkill: true,
                },
            ],
        });
    }
    let (blow_count, recap_id): (i32, i32) = env
        .eval(
            r#"
            local blows = C_DeathRecap.GetKillingBlows()
            local r = C_DeathRecap.GetMostRecentDeathRecap()
            return #blows, r.recapID
            "#,
        )
        .unwrap();
    assert_eq!(blow_count, 3, "GetKillingBlows uses the most recent death");
    assert_eq!(
        recap_id, 2_i32,
        "GetMostRecentDeathRecap uses the most recent death"
    );
}
