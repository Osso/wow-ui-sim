//! WM61 S8 rejection through the runtime-selected CooldownViewer consumer.
#![cfg(feature = "retail-12-1-0")]

use crate::common::aura_container_harness::{aura, load_env};
use crate::common::blizzard_addon_harness::{blizzard_ui_dir, build_blizzard_addon_closure_env};

#[test]
fn aura_util_resolves_names_before_restricted_lookup() {
    let env = load_env();
    {
        let mut state = env.state().borrow_mut();
        let mut secret = aura(343960);
        secret.name = "Sinstone Bond".into();
        let mut public = aura(1126);
        public.name = "Mark of the Wild".into();
        state.player.buffs = vec![secret, public];
        state
            .spell_id_aliases
            .insert("sinstone bond".into(), 343960);
        state
            .spell_id_aliases
            .insert("mark of the wild".into(), 1126);
        state.restricted_actions[0] = 2;
    }
    env.exec(
        r#"
        local info = debug.getinfo(AuraUtil.FindAuraByName, 'S')
        assert(info.what == 'Lua' and info.source:find('AuraUtil.lua', 1, true))
        local function addon()
            assert(AuraUtil.FindAuraByName('Sinstone Bond', 'player') == nil)
            assert(AuraUtil.FindAuraByName('Mark of the Wild', 'player') == 'Mark of the Wild')
            assert(select('#', C_UnitAuras.GetAuraDataBySpellName('player', 'Sinstone Bond')) == 0,
                'name rejection must return zero values')
        end
        debug.setobjecttaint(addon, 'SecretAuraSpellProbe')
        addon()
    "#,
    )
    .unwrap();
}

#[test]
fn cooldown_viewer_skips_restricted_spell_and_keeps_never_secret_aura() {
    let ui = blizzard_ui_dir();
    let (env, loaded) = build_blizzard_addon_closure_env(&ui, &["Blizzard_CooldownViewer"], &[]);
    assert!(loaded.iter().any(|name| name == "Blizzard_CooldownViewer"));
    {
        let mut state = env.state().borrow_mut();
        state.player.buffs = vec![aura(343960), aura(1126)];
        state.restricted_actions[0] = 2;
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
