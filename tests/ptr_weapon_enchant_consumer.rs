#![cfg(feature = "client-ptr")]

use wow_ui_sim::c_api::weapon_enchants::{TEMPORARY_ENCHANT_TYPE, WeaponEnchant};
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn ptr_weapon_enchant_event_updates_buff_frame() {
    let env = WowLuaEnv::new().unwrap();
    let ui = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    for (_, toc) in wow_ui_sim::loader::discover_blizzard_addons(&ui) {
        wow_ui_sim::loader::load_addon(&env.loader_env(), &toc).unwrap();
    }
    env.apply_post_load_workarounds();
    for event in ["VARIABLES_LOADED", "PLAYER_LOGIN", "PLAYER_ENTERING_WORLD"] {
        let _ = env.fire_event(event);
    }
    env.exec("assert(BuffFrame:IsEventRegistered('WEAPON_ENCHANT_CHANGED'))")
        .unwrap();
    for (charges, time_left) in [(2, 60000.0), (5, 60000.0), (5, 30000.0)] {
        env.state().borrow_mut().weapon_enchants[0] = vec![WeaponEnchant {
            enchant_type: TEMPORARY_ENCHANT_TYPE,
            time_left,
            charges,
            enchant_id: 42,
            icon_id: 135913,
        }];
        env.state().borrow_mut().lua_errors.clear();
        env.fire_event("WEAPON_ENCHANT_CHANGED").unwrap();
        let errors = env.state().borrow().lua_errors.clone();
        assert!(errors.is_empty(), "weapon enchant event errors: {errors:?}");
        let actual = env.eval::<i32>("for _, aura in ipairs(BuffFrame.auraInfo) do if aura.auraType == 'TempEnchant' and aura.ID == 16 then return aura.count end end; return 0").unwrap();
        assert_eq!(actual, charges as i32);
        let remaining = env.eval::<f64>("for _, aura in ipairs(BuffFrame.auraInfo) do if aura.auraType == 'TempEnchant' and aura.ID == 16 then return aura.expirationTime - GetTime() end end; return 0").unwrap();
        assert!((remaining - time_left / 1000.0).abs() < 1.0);
        let legacy = env
            .eval::<i32>("local _, _, charges = GetWeaponEnchantInfo(); return charges")
            .unwrap();
        assert_eq!(legacy, charges as i32);
    }
}
