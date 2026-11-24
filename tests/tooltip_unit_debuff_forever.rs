//! Populated player debuffs reach the original TooltipDataHandler setters on both profiles.

#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::AuraInfo;

fn aura(id: i32, name: &str) -> AuraInfo {
    AuraInfo {
        name: name.into(),
        spell_id: 19750,
        icon: wow_ui_sim::lua_api::state::AuraIcon::FileId(135987),
        duration: 30.0,
        expiration_time: 30.0,
        applications: 1,
        source_unit: "target".into(),
        is_helpful: false,
        is_raid: false,
        is_nameplate_only: false,
        is_stealable: false,
        can_apply_aura: true,
        is_from_player_or_player_pet: false,
        dispel_type: Some("Magic".into()),
        aura_instance_id: id,
    }
}

fn assert_tooltip_name(env: &WowLuaEnv, call: &str, expected: &str) {
    env.exec(&format!(
        "GameTooltip:Hide(); GameTooltip:ClearHandlerInfo(); GameTooltip:ClearLines(); {call}; \
         local data = GameTooltip:GetPrimaryTooltipData(); \
         assert(data and data.lines[1] and data.lines[1].leftText == '{expected}', \
             'populated debuff tooltip must publish {expected}')"
    ))
    .unwrap();
}

#[test]
fn unit_debuff_setters_publish_two_distinct_supplied_player_auras() {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    env.state().borrow_mut().player.buffs = vec![
        aura(91001, "Supplied First Debuff"),
        aura(91002, "Supplied Second Debuff"),
    ];
    for (index, id, name) in [
        (1, 91001, "Supplied First Debuff"),
        (2, 91002, "Supplied Second Debuff"),
    ] {
        assert_tooltip_name(
            &env,
            &format!("GameTooltip:SetUnitDebuff('player', {index}, 'HARMFUL')"),
            name,
        );
        assert_tooltip_name(
            &env,
            &format!("GameTooltip:SetUnitDebuffByAuraInstanceID('player', {id}, 'HARMFUL')"),
            name,
        );
    }
    assert_target_debuff_tooltips(&env);
}

fn assert_target_debuff_tooltips(env: &WowLuaEnv) {
    env.exec("A_Admin.SetTarget('Supplied Target', 80, 1, true)")
        .unwrap();
    env.state().borrow_mut().target_auras = vec![aura(92001, "Supplied Target Debuff")];
    assert_tooltip_name(
        env,
        "GameTooltip:SetUnitDebuff('target', 1, 'HARMFUL')",
        "Supplied Target Debuff",
    );
    assert_tooltip_name(
        env,
        "GameTooltip:SetUnitDebuffByAuraInstanceID('target', 92001, 'HARMFUL')",
        "Supplied Target Debuff",
    );
}
