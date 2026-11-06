//! Native `UnitAuraUpdateInfo` shape on simulator `UNIT_AURA` dispatches.
//!
//! Retail, PTR and Forever declare the structure at
//! `Blizzard_APIDocumentationGenerated/UnitConstantsDocumentation.lua:56-64`;
//! Mists, Era and Anniversary declare it at `:32-40` of the same pinned file.
//! `isFullUpdate` is non-nilable; the three change lists are nilable. Wrath
//! publishes no generated documentation for this event and keeps the existing
//! `isFullUpdate`-only payload, so this test does not build for Wrath.

#![cfg(any(
    feature = "retail-12-0-5",
    feature = "client-mists",
    feature = "client-era",
    feature = "client-anniversary",
    feature = "client-wowforever"
))]

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::AuraInfo;

fn aura(id: i32) -> AuraInfo {
    AuraInfo {
        name: format!("Payload Aura {id}"),
        spell_id: 98000 + id,
        icon: 134973,
        duration: 30.0,
        expiration_time: 30.0,
        applications: 1,
        source_unit: "player".into(),
        is_helpful: true,
        is_raid: false,
        is_nameplate_only: false,
        is_stealable: false,
        can_apply_aura: true,
        is_from_player_or_player_pet: true,
        dispel_type: None,
        aura_instance_id: id,
    }
}

fn capture_unit_aura(env: &WowLuaEnv) {
    env.exec(
        r#"
        PayloadUnit, PayloadInfo = nil, nil
        local frame = CreateFrame('Frame')
        frame:RegisterEvent('UNIT_AURA')
        frame:SetScript('OnEvent', function(_, _, unit, info)
            PayloadUnit, PayloadInfo = unit, info
        end)
        "#,
    )
    .unwrap();
}

#[test]
fn full_update_publishes_added_auras_from_state() {
    let env = WowLuaEnv::new().unwrap();
    capture_unit_aura(&env);
    env.state().borrow_mut().player.buffs = vec![aura(4101), aura(4102)];

    env.fire_unit_aura_full_update("player").unwrap();

    let (unit, is_full_update, added_count): (String, bool, f64) = env
        .eval(
            r#"
            return PayloadUnit, PayloadInfo.isFullUpdate, #PayloadInfo.addedAuras
            "#,
        )
        .unwrap();
    assert_eq!(unit, "player");
    assert!(is_full_update);
    assert_eq!(added_count, 2.0);

    let added_ids: String = env
        .eval(
            r#"
            local ids = {}
            for _, entry in ipairs(PayloadInfo.addedAuras) do
                ids[#ids + 1] = entry.auraInstanceID
            end
            table.sort(ids)
            return table.concat(ids, ',')
            "#,
        )
        .unwrap();
    assert_eq!(added_ids, "4101,4102");

    let (updated_absent, removed_absent): (bool, bool) = env
        .eval(
            r#"
            return PayloadInfo.updatedAuraInstanceIDs == nil,
                PayloadInfo.removedAuraInstanceIDs == nil
            "#,
        )
        .unwrap();
    assert!(updated_absent);
    assert!(removed_absent);
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn incremental_removal_publishes_removed_instance_ids() {
    let env = WowLuaEnv::new().unwrap();
    capture_unit_aura(&env);
    env.state().borrow_mut().player.buffs = vec![aura(4201), aura(4202)];

    env.exec("C_UnitAuras.CancelAuraByInstanceID('player', 4201)")
        .unwrap();

    let (unit, is_full_update): (String, bool) = env
        .eval("return PayloadUnit, PayloadInfo.isFullUpdate")
        .unwrap();
    assert_eq!(unit, "player");
    assert!(!is_full_update);

    let (removed_id, added_absent): (f64, bool) = env
        .eval(
            r#"
            return PayloadInfo.removedAuraInstanceIDs[1],
                PayloadInfo.addedAuras == nil
            "#,
        )
        .unwrap();
    assert_eq!(removed_id, 4201.0);
    assert!(added_absent);
}
