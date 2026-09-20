#![cfg(feature = "client-wowforever")]

use wow_ui_sim::c_api::weapon_enchants::{TEMPORARY_ENCHANT_TYPE, WeaponEnchant};
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn forever_weapon_enchants_empty_lists() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("assert(type(C_Item.GetWeaponEnchantInfo(0)) == 'table'); assert(next(C_Item.GetWeaponEnchantInfo(1)) == nil)").unwrap();
}

fn enchant(id: u32, time_left: f64) -> WeaponEnchant {
    WeaponEnchant {
        enchant_type: TEMPORARY_ENCHANT_TYPE,
        time_left,
        charges: 3,
        enchant_id: id,
        icon_id: 135913,
    }
}

#[test]
fn forever_weapon_enchants_share_updates_with_legacy_query() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().weapon_enchants[0].push(enchant(42, 60000.0));
    env.state().borrow_mut().weapon_enchants[1].push(enchant(43, 30000.0));
    env.exec(
        r#"
        local a = C_Item.GetWeaponEnchantInfo(0)[1]
        assert(a.hasEnchant and a.enchantType == 2 and a.timeLeft == 60000)
        assert(a.charges == 3 and a.enchantID == 42 and a.enchantIconID == 135913)
        local has, time, charges, id, off, ot, oc, oi = GetWeaponEnchantInfo()
        assert(has and time == 60000 and charges == 3 and id == 42)
        assert(off and ot == 30000 and oc == 3 and oi == 43)
        a.timeLeft = 1
        assert(C_Item.GetWeaponEnchantInfo(0)[1].timeLeft == 60000)
    "#,
    )
    .unwrap();
    env.state().borrow_mut().weapon_enchants[0][0].time_left = 15000.0;
    env.state().borrow_mut().weapon_enchants[1].clear();
    env.exec(
        r#"
        assert(C_Item.GetWeaponEnchantInfo(0)[1].timeLeft == 15000)
        assert(next(C_Item.GetWeaponEnchantInfo(1)) == nil)
        local _, t, _, _, off = GetWeaponEnchantInfo()
        assert(t == 15000 and off == false)
        assert(not pcall(C_Item.GetWeaponEnchantInfo, 3))
    "#,
    )
    .unwrap();
    assert!(
        WowLuaEnv::new()
            .unwrap()
            .eval::<bool>("return next(C_Item.GetWeaponEnchantInfo(0)) == nil")
            .unwrap()
    );
}

#[test]
fn forever_ranged_weapon_enchants_preserve_legacy_returns() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("assert(Enum.WeaponSlot.Ranged == 2); assert(Enum.WeaponSlotMeta.MaxValue == 2); assert(Enum.WeaponSlotMeta.NumValues == 3); assert(next(C_Item.GetWeaponEnchantInfo(2)) == nil)").unwrap();
    env.state()
        .borrow_mut()
        .weapon_enchants
        .get_mut(2)
        .unwrap()
        .push(enchant(44, 45000.0));
    env.exec(
        r#"
        local ranged = C_Item.GetWeaponEnchantInfo(Enum.WeaponSlot.Ranged)[1]
        assert(ranged.hasEnchant and ranged.enchantID == 44 and ranged.timeLeft == 45000)
        assert(select('#', GetWeaponEnchantInfo()) == 8)
        local main, mt, mc, mi, off, ot, oc, oi = GetWeaponEnchantInfo()
        assert(main == false and mt == 0 and mc == 0 and mi == 0)
        assert(off == false and ot == 0 and oc == 0 and oi == 0)
        assert(not pcall(C_Item.GetWeaponEnchantInfo, 3))
    "#,
    )
    .unwrap();
}

#[test]
fn forever_buff_consumer_reads_current_enchants() {
    let env = WowLuaEnv::new().unwrap();
    let source = std::fs::read_to_string(format!(
        "{}/.cache/wow-ui-sim/blizzard-ui/wowforever/AddOns/Blizzard_BuffFrame/BuffFrame.lua",
        std::env::var("HOME").unwrap()
    ))
    .unwrap();
    let mapping = source
        .split("local textureMapping = ")
        .nth(1)
        .unwrap()
        .split("};")
        .next()
        .unwrap();
    let consumer = source
        .split("function BuffFrameMixin:UpdateTemporaryEnchantmentBuffs()")
        .nth(1)
        .unwrap()
        .split("function BuffFrameMixin:UpdateAuras()")
        .next()
        .unwrap();
    env.exec(&format!("local textureMapping = {mapping}}};\nBuffFrameMixin = {{}}\nfunction BuffFrameMixin:UpdateTemporaryEnchantmentBuffs(){consumer}")).unwrap();
    env.exec("BUFF_DURATION_WARNING_TIME = 120; probe = {auraInfo={}, maxAuras=32, numHideableBuffs=0}; BuffFrameMixin.UpdateTemporaryEnchantmentBuffs(probe); assert(#probe.auraInfo == 0)").unwrap();
    env.state().borrow_mut().weapon_enchants[0].push(enchant(42, 60000.0));
    env.exec(
        r#"
        BuffFrameMixin.UpdateTemporaryEnchantmentBuffs(probe)
        assert(#probe.auraInfo == 1)
        local aura = probe.auraInfo[1]
        assert(aura.auraType == 'TempEnchant' and aura.ID == 16 and aura.count == 3)
        assert(math.abs(aura.expirationTime - GetTime() - 60) < 1)
        assert(type(aura.OnCancel) == 'function')
        assert(C_Item.GetWeaponEnchantInfo(0)[1].timeLeft == 60000)
    "#,
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .weapon_enchants
        .get_mut(2)
        .unwrap()
        .push(enchant(44, 45000.0));
    env.state().borrow_mut().weapon_enchants[1].push(enchant(43, 30000.0));
    env.exec("probe.auraInfo = {}; BuffFrameMixin.UpdateTemporaryEnchantmentBuffs(probe); assert(#probe.auraInfo == 3); local ids = {}; for _, aura in ipairs(probe.auraInfo) do ids[aura.ID] = true end; assert(ids[16] and ids[17] and ids[18])").unwrap();
    env.state().borrow_mut().weapon_enchants[1].clear();
    env.state()
        .borrow_mut()
        .weapon_enchants
        .get_mut(2)
        .unwrap()
        .clear();
    env.state().borrow_mut().weapon_enchants[0].clear();
    env.exec("probe.auraInfo = {}; BuffFrameMixin.UpdateTemporaryEnchantmentBuffs(probe); assert(#probe.auraInfo == 0)").unwrap();
}
