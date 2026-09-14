#![cfg(feature = "retail-12-0-0")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn seeded_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut state = env.state().borrow_mut();
        let template = state.player.buffs[0].clone();
        state.player.buffs = [
            (31, "Zulu", true, true, 90.0, "player"),
            (12, "Curse", false, false, 30.0, "target"),
            (53, "Alpha", true, false, 60.0, "party1"),
            (74, "Echo", true, true, 0.0, "player"),
        ]
        .into_iter()
        .map(|(id, name, helpful, own, expiration, source)| {
            let mut aura = template.clone();
            aura.aura_instance_id = id;
            aura.name = name.into();
            aura.is_helpful = helpful;
            aura.is_from_player_or_player_pet = own;
            aura.can_apply_aura = own;
            aura.expiration_time = expiration;
            aura.duration = expiration;
            aura.source_unit = source.into();
            aura
        })
        .collect();
    }
    env
}

#[test]
fn aura_instance_ids_follow_live_filters_and_blocking() {
    let env = seeded_env();
    env.exec(
        r#"
        local function ids(unit, filter, ...)
            local values, extra = C_UnitAuras.GetUnitAuraInstanceIDs(unit, filter, ...)
            assert(type(values) == 'table' and extra == nil, 'native API returns one array')
            return table.concat(values, ',')
        end
        assert(ids('player', 'HELPFUL') == '31,53,74')
        assert(ids('player', 'HARMFUL') == '12')
        assert(ids('player', 'HELPFUL|PLAYER') == '31,74')
        assert(ids('target', 'HELPFUL') == '')
        assert(ids('target', 'HARMFUL') == '1,2')
        assert(ids('missing-unit', 'HELPFUL') == '')
        C_UnitAuras.AddBlockedAura('player', 53)
        assert(ids('player', 'HELPFUL') == '31,74')
        local data = C_UnitAuras.GetAuraDataByAuraInstanceID('player', 31)
        assert(data.auraInstanceID == 31 and data.name == 'Zulu')
    "#,
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .player
        .buffs
        .retain(|a| a.aura_instance_id != 31);
    assert_eq!(
        env.eval::<String>(
            "return table.concat(C_UnitAuras.GetUnitAuraInstanceIDs('player', 'HELPFUL'), ',')"
        )
        .unwrap(),
        "74"
    );
}

#[test]
fn aura_instance_ids_apply_documented_sorting_and_limits() {
    let env = seeded_env();
    env.exec(r#"
        local function ids(rule, direction, limit)
            return table.concat(C_UnitAuras.GetUnitAuraInstanceIDs('player', 'HELPFUL', limit, rule, direction), ',')
        end
        assert(ids(0, 0, 2) == '31,53')
        assert(ids(0, 1, 2) == '74,53')
        assert(ids(0, 0, 0) == '')
        assert(ids(1, 0) == '31,74,53')
        assert(ids(2, 0) == '53,31,74')
        assert(ids(3, 0) == '31,74,53')
        assert(ids(4, 0) == '74,53,31')
        assert(ids(5, 0) == '74,31,53')
        assert(ids(6, 0) == '53,74,31')
        assert(ids(6, 1, 2) == '31,74')
    "#).unwrap();
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn aura_instance_ids_private_queries_follow_existing_private_state() {
    let env = seeded_env();
    env.exec(r#"
        C_UnitAurasPrivate._state.privateAurasByUnit.player = {
            { auraInstanceID = 903, spellID = 123 },
            { auraInstanceID = 901, spellID = 456 },
        }
        C_UnitAurasPrivate._state.privateAurasByUnit.target = {
            { auraInstanceID = 905, spellID = 789 },
        }
        local ids = C_UnitAurasPrivate.GetAllPrivateAuraInstanceIDs('player')
        assert(table.concat(ids, ',') == '903,901')
        ids[1] = 0
        assert(C_UnitAurasPrivate.GetAllPrivateAuraInstanceIDs('player')[1] == 903)
        assert(table.concat(C_UnitAurasPrivate.GetAllPrivateAuraInstanceIDs('target'), ',') == '905')
        assert(#C_UnitAurasPrivate.GetAllPrivateAuraInstanceIDs('missing-unit') == 0)
    "#).unwrap();
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn aura_instance_ids_source_reports_filter_match_separately() {
    let env = seeded_env();
    let path = wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(
        env!("CARGO_MANIFEST_DIR"),
    ))
    .join("Blizzard_AuraContainer/Blizzard_AuraContainerSources.lua");
    env.exec(&std::fs::read_to_string(path).unwrap()).unwrap();
    env.exec(r#"
        local ids, matched = AuraContainerPublicAuraSource:GetAllAuraInstanceIDs('player', 'HARMFUL')
        assert(table.concat(ids, ',') == '12' and matched == true)
        ids, matched = AuraContainerPublicAuraSource:GetAllAuraInstanceIDs('missing-unit', 'HELPFUL')
        assert(type(ids) == 'table' and #ids == 0 and matched == true)
        C_UnitAurasPrivate._state.privateAurasByUnit.player = { { auraInstanceID = 903 } }
        ids, matched = AuraContainerPrivateAuraSource:GetAllAuraInstanceIDs('player', 'HARMFUL')
        assert(table.concat(ids, ',') == '903' and matched == false)
    "#).unwrap();
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn aura_instance_ids_sound_trigger_enum_exists_before_secure_copy() {
    let env = seeded_env();
    env.exec(r#"
        for _, enums in ipairs({ Enum, __secureenv.Enum }) do
            local trigger = enums.UnitAuraSoundTrigger
            assert(trigger.Added == 0 and trigger.ApplicationsIncreased == 1 and trigger.Removed == 2)
            local count = 0
            for _ in pairs(trigger) do count = count + 1 end
            assert(count == 3)
            local meta = enums.UnitAuraSoundTriggerMeta
            assert(meta.MinValue == 0 and meta.MaxValue == 2 and meta.NumValues == 3)
        end
    "#).unwrap();
}
