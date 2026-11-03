//! WM61 S8 rejection through the runtime-selected CooldownViewer consumer.
#![cfg(feature = "retail-12-1-0")]

use crate::common::aura_container_harness::aura;
use crate::common::blizzard_addon_harness::{blizzard_ui_dir, build_blizzard_addon_closure_env};

#[test]
fn cooldown_viewer_skips_restricted_spell_and_keeps_never_secret_aura() {
    let ui = blizzard_ui_dir();
    let (env, loaded) = build_blizzard_addon_closure_env(&ui, &["Blizzard_CooldownViewer"], &[]);
    assert!(loaded.iter().any(|name| name == "Blizzard_CooldownViewer"));
    {
        let mut state = env.state().borrow_mut();
        state.player.buffs = vec![aura(343960), aura(1126)];
        state.unit_auras_restricted = true;
    }
    env.exec(
        r#"
        local consumer = CooldownViewerItemDataMixin.FindLinkedSpellForCurrentAuras
        local info = debug.getinfo(consumer, 'S')
        assert(info.what == 'Lua' and info.source:find('CooldownViewerItemData.lua', 1, true))
        local item = Mixin({cooldownInfo = {linkedSpellIDs = {343960, 1126}}}, CooldownViewerItemDataMixin)
        local function addon()
            local spell, result = item:FindLinkedSpellForCurrentAuras('player')
            assert(spell == 1126 and result.auraInstanceID == 1126, 'consumer selected the never-secret aura')
            assert(select('#', C_UnitAuras.GetUnitAuraBySpellID('player', 343960)) == 0, 'unit rejection must return zero values')
            assert(select('#', C_UnitAuras.GetPlayerAuraBySpellID(343960)) == 0, 'player rejection must return zero values')
            assert(select('#', C_UnitAuras.GetUnitAuraBySpellID('player', 1126)) == 1, 'never-secret aura returns one value')
        end
        debug.setobjecttaint(addon, 'SecretAuraSpellProbe')
        addon()
        "#,
    )
    .unwrap();
}
