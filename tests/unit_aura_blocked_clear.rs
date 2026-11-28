//! C_UnitAuras.ClearBlockedAuras over the AddBlockedAura enumeration model.
//! Retail, PTR and Forever declare it byte-identically
//! (UnitAuraDocumentation.lua:98).
#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

const BLOCK_TWO_PLAYER_BUFFS: &str = r#"
    A_Admin.ClearBuffs()
    A_Admin.AddBuff(21562, "Power Word: Fortitude", 135987, 0, 0)
    A_Admin.AddBuff(1126, "Mark of the Wild", 136078, 0, 0)
    for _, name in ipairs({"Power Word: Fortitude", "Mark of the Wild"}) do
        local aura = C_UnitAuras.GetAuraDataBySpellName("player", name)
        C_UnitAuras.AddBlockedAura("player", aura.auraInstanceID)
    end
"#;

const VISIBLE_HELPFUL_NAMES: &str = r#"
    local names = {}
    for index = 1, 4 do
        local aura = C_UnitAuras.GetAuraDataByIndex("player", index, "HELPFUL")
        if aura then names[#names + 1] = aura.name end
    end
    table.sort(names)
    return table.concat(names, ",")
"#;

#[test]
fn clear_blocked_auras_restores_only_the_selected_unit() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(BLOCK_TWO_PLAYER_BUFFS).unwrap();
    let visible = |env: &WowLuaEnv| env.eval::<String>(VISIBLE_HELPFUL_NAMES).unwrap();
    assert_eq!(visible(&env), "", "both blocked buffs are hidden");
    let returns: i64 = env
        .eval("return select('#', C_UnitAuras.ClearBlockedAuras('party1'))")
        .unwrap();
    assert_eq!(returns, 0);
    assert_eq!(visible(&env), "", "another unit's cleanup keeps player blocks");
    env.exec("C_UnitAuras.ClearBlockedAuras('player')").unwrap();
    assert_eq!(visible(&env), "Mark of the Wild,Power Word: Fortitude");
}
