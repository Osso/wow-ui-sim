//! Original consumer behind the erased securecallfunction defensive-query callee.
#![cfg(feature = "aura-spell-classifications")]

use wow_ui_sim::c_api::c_unit_aura_classification::AuraSpellClassification;
use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
use wow_ui_sim::lua_api::WowLuaEnv;

fn original_aura_util_env() -> WowLuaEnv {
    let ui = crate::common::blizzard_addon_harness::blizzard_ui_dir();
    let (env, _) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui,
        &["Blizzard_FrameXMLUtil"],
        &[],
    );
    env
}

#[test]
fn original_aura_util_classifies_an_unlisted_spell() {
    let env = original_aura_util_env();
    env.exec(
        r#"
        local result = AuraUtil.IsBigDefensive({spellId = 999999})
        assert(type(result) == 'boolean', 'original consumer must return a boolean')
        assert(result == false, 'unlisted spells have no declared defensive classification')
        assert(AuraUtil.IsBigDefensive({spellId = 999999}) == false,
            'the original consumer preserves its cached false result')
        "#,
    )
    .unwrap();
}

#[test]
fn original_aura_util_reads_declared_defensive_classification() {
    let env = original_aura_util_env();
    env.state()
        .borrow_mut()
        .aura_spell_classifications
        .spells
        .insert(
            101,
            AuraSpellClassification {
                is_big_defensive: true,
                is_private: false,
            },
        );
    env.exec(
        r#"
        assert(AuraUtil.IsBigDefensive({spellId = 101}) == true)
        assert(AuraUtil.IsBigDefensive({spellId = 202}) == false)
        assert(select('#', C_UnitAuras.AuraIsBigDefensive(101)) == 1)
        assert(C_UnitAuras.UnlistedFactoryMember == nil)
        "#,
    )
    .unwrap();
    // Forever publishes AuraIsPrivate (UnitAuraDocumentation.lua:70, byte-identical).
    if matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    ) {
        env.exec("assert(C_UnitAuras.AuraIsPrivate(101) == false)")
            .unwrap();
    } else {
        env.exec("assert(rawget(C_UnitAuras, 'AuraIsPrivate') == nil)")
            .unwrap();
    }
}
