//! 12.1.0 managed AuraContainer item enchantments (B12e): temporary weapon
//! enchant buttons, permanent-enchant hiding, click-to-cancel, and clearing
//! when the container is disabled.
#![cfg(feature = "retail-12-1-0")]

use crate::common::aura_container_harness::{
    assert_no_lua_errors, aura, frame_id, load_env, set_player_auras, settle,
};
use rilua::Val;
use wow_ui_sim::c_api::weapon_enchants::{TEMPORARY_ENCHANT_TYPE, WeaponEnchant};
use wow_ui_sim::lua_api::WowLuaEnv;

fn seed_temporary_enchant(env: &WowLuaEnv, weapon_slot: usize, time_left_ms: f64) {
    env.state().borrow_mut().weapon_enchants[weapon_slot] = vec![WeaponEnchant {
        enchant_type: TEMPORARY_ENCHANT_TYPE,
        time_left: time_left_ms,
        charges: 0,
        enchant_id: 5400 + weapon_slot as u32,
        icon_id: 0,
    }];
}

/// `AuditEnchantState(frame)` is `shown@x` relative to its container.
const ENCHANT_HELPERS: &str = r#"
    function AuditEnchantState(container, frame)
        if not AuditIsShown(frame) then return 'hidden' end
        return 'shown@' .. (AuditPlain(frame:GetLeft()) - AuditPlain(container:GetLeft()))
    end
    function AuditAddEnchant(container, slot, options)
        options = options or {}
        options.initializeFrame = function(frame)
            AuditInitIcon(frame)
            frame:SetCancelAuraButtons('RightButtonUp')
        end
        debug.setobjecttaint(options.initializeFrame, AUDIT_ADDON)
        return AuditAddon(function() return container:AddItemEnchantment(slot, options) end)
    end
"#;

fn enchant_env() -> WowLuaEnv {
    let env = load_env();
    env.exec(ENCHANT_HELPERS).unwrap();
    set_player_auras(&env, vec![aura(501)]);
    env
}

fn state_of(env: &WowLuaEnv, container: &str, frame: &str) -> String {
    env.eval::<String>(&format!("return AuditEnchantState({container}, {frame})"))
        .unwrap()
}

#[test]
fn temporary_weapon_enchants_show_before_aura_groups_and_clear_when_disabled() {
    let env = enchant_env();
    seed_temporary_enchant(&env, 0, 600_000.0);
    env.exec(
        r#"
        AuditEnchantHost = AuditContainer({{'buffs', 'HELPFUL'}})
        AuditMainHand = AuditAddEnchant(AuditEnchantHost, AuraContainerItemEnchantmentSlot.MainHand)
        AuditOffHand = AuditAddEnchant(AuditEnchantHost, AuraContainerItemEnchantmentSlot.OffHand)
        assert(AuditMainHand:GetObjectType() == 'AuraButton' and AuditMainHand:GetParent() == AuditEnchantHost)
    "#,
    )
    .unwrap();
    settle(&env);
    let main = || state_of(&env, "AuditEnchantHost", "AuditMainHand");
    let off = || state_of(&env, "AuditEnchantHost", "AuditOffHand");
    let buff = || {
        env.eval::<String>(
            "return AuditEnchantState(AuditEnchantHost, AuditShown(AuditEnchantHost, 'buffs')[1].frame)",
        )
        .unwrap()
    };
    assert_eq!((main(), off(), buff()), ("shown@0".into(), "hidden".into(), "shown@20".into()));

    // A newly applied off-hand enchant appears after WEAPON_ENCHANT_CHANGED.
    seed_temporary_enchant(&env, 1, 300_000.0);
    env.fire_event("WEAPON_ENCHANT_CHANGED").unwrap();
    settle(&env);
    assert_eq!((main(), off(), buff()), ("shown@0".into(), "shown@20".into(), "shown@40".into()));

    env.exec("AuditAddon(function() AuditEnchantHost:SetEnabled(false) end)")
        .unwrap();
    settle(&env);
    assert_eq!((main(), off()), ("hidden".into(), "hidden".into()));
    env.exec("AuditAddon(function() AuditEnchantHost:SetEnabled(true) end)")
        .unwrap();
    settle(&env);
    assert_eq!((main(), off()), ("shown@0".into(), "shown@20".into()));
    assert_no_lua_errors(&env);
}

#[test]
fn hide_permanent_skips_enchants_without_expiration() {
    let env = enchant_env();
    seed_temporary_enchant(&env, 0, 0.0);
    env.exec(
        r#"
        AuditPermanentHost = AuditContainer({})
        AuditShownPermanent = AuditAddEnchant(AuditPermanentHost, AuraContainerItemEnchantmentSlot.MainHand)
        AuditHiddenHost = AuditContainer({})
        AuditHiddenPermanent = AuditAddEnchant(AuditHiddenHost, AuraContainerItemEnchantmentSlot.MainHand, {hidePermanent = true})
    "#,
    )
    .unwrap();
    settle(&env);
    assert_eq!(
        state_of(&env, "AuditPermanentHost", "AuditShownPermanent"),
        "shown@0"
    );
    assert_eq!(
        state_of(&env, "AuditHiddenHost", "AuditHiddenPermanent"),
        "hidden"
    );
    assert_no_lua_errors(&env);
}

#[test]
fn right_click_cancels_a_temporary_weapon_enchant() {
    let env = enchant_env();
    seed_temporary_enchant(&env, 0, 600_000.0);
    env.exec(
        r#"
        AuditCancelHost = AuditContainer({{'buffs', 'HELPFUL'}})
        AuditCancelEnchant = AuditAddEnchant(AuditCancelHost, AuraContainerItemEnchantmentSlot.MainHand)
    "#,
    )
    .unwrap();
    settle(&env);
    let button = frame_id(&env, "AuditCancelEnchant");
    let click = |which: &str| {
        let token = env.lua_string(which);
        env.fire_script_handler(button, "OnClick", vec![token, Val::Bool(false)])
            .unwrap();
        settle(&env);
    };
    click("LeftButton");
    assert_eq!(env.state().borrow().weapon_enchants[0].len(), 1);
    assert_eq!(
        state_of(&env, "AuditCancelHost", "AuditCancelEnchant"),
        "shown@0"
    );
    click("RightButton");
    assert!(env.state().borrow().weapon_enchants[0].is_empty());
    assert_eq!(
        state_of(&env, "AuditCancelHost", "AuditCancelEnchant"),
        "hidden"
    );
    assert_eq!(
        env.eval::<String>(
            "return AuditEnchantState(AuditCancelHost, AuditShown(AuditCancelHost, 'buffs')[1].frame)"
        )
        .unwrap(),
        "shown@0",
        "the remaining aura group reflows into the cancelled enchant's place"
    );
    assert_no_lua_errors(&env);
}
