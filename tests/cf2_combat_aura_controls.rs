//! Original arena cleanup; populated selected-unit block-removal acceptance.
#![cfg(any(feature = "client-retail", feature = "client-ptr",))]

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::AuraInfo;

const ARENA_SOURCE: &str = "Blizzard_UnitFrame/Mainline/CompactArenaFrame.lua";

fn load_original_consumer(root: &str) -> WowLuaEnv {
    let ui = crate::common::blizzard_addon_harness::blizzard_ui_dir();
    let env = crate::common::blizzard_addon_harness::new_blizzard_addon_env(&ui);
    let loaded = crate::common::blizzard_addon_harness::load_blizzard_addon_closure_into_env(
        &env,
        &ui,
        &[root],
        &[],
    );
    assert!(loaded.iter().any(|name| name == root));
    eprintln!("CF-2 original consumer cache: {}", ui.display());
    env
}

fn assert_original_source(env: &WowLuaEnv, function: &str, suffix: &str) {
    env.exec(&format!(
        r#"
        local info = debug.getinfo({function}, 'S')
        local source = info.source:gsub('\\', '/')
        local suffix = '{suffix}'
        assert(info.what == 'Lua' and source:sub(-#suffix) == suffix,
            'must execute the original cached Blizzard consumer')
        print('CF-2 consumer provenance', source, info.linedefined, info.lastlinedefined)
        "#,
    ))
    .expect("original cached consumer provenance");
}

fn select_original_debuff(env: &WowLuaEnv) {
    env.exec(
        r#"
        assert(CompactArenaFrame and #CompactArenaFrame.memberUnitFrames > 0, 'CF2 assertion original line 40')
        local member = CompactArenaFrame.memberUnitFrames[1]
        Cf2AuraDebuff = member.DebuffFrame
        assert(Cf2AuraDebuff and Cf2AuraDebuff:GetParent() == member,
            'must use the existing normal arena member child')
        assert(Cf2AuraDebuff.Update == ArenaUnitFrameDebuffMixin.Update, 'original Update identity')
        assert(Cf2AuraDebuff.SetUnit == ArenaUnitFrameDebuffMixin.SetUnit, 'original SetUnit identity')
        assert(type(Cf2AuraDebuff:GetScript('OnEvent')) == 'function',
            'original XML must install an event handler')
        "#,
    )
    .expect("normal arena member, child and installed handler identity");
    for function in [
        "Cf2AuraDebuff.Update",
        "Cf2AuraDebuff.SetUnit",
        "ArenaUnitFrameDebuffMixin.OnEvent",
    ] {
        assert_original_source(env, function, ARENA_SOURCE);
    }
    assert_original_source(
        env,
        "Cf2AuraDebuff:GetParent().UpdatePrivateAuras",
        "Blizzard_UnitFrame/Shared/CompactUnitFrame.lua",
    );
}

fn assert_installed_event_clears_selected_auras(env: &WowLuaEnv) {
    env.exec(
        r#"
        C_UnitAuras.AddBlockedAura('player', 101)
        C_UnitAuras.AddBlockedAura('party1', 201)
        assert(C_UnitAuras.GetAuraDataByIndex('player', 1, 'HELPFUL') == nil, 'CF2 assertion original line 71')
        local debuff = Cf2AuraDebuff
        local handler = debuff:GetScript('OnEvent')
        handler(debuff, 'UNRELATED_EVENT')
        assert(C_UnitAuras.GetAuraDataByIndex('player', 1, 'HELPFUL') == nil, 'CF2 assertion original line 75')
        assert(debug.gethook() == nil, 'must not replace an existing hook')
        local original = ArenaUnitFrameDebuffMixin.OnEvent
        local calls = 0
        debug.sethook(function(event)
            if event == 'call' and debug.getinfo(2, 'f').func == original then
                calls = calls + 1
            end
        end, 'c')
        local ok, err = pcall(handler, debuff, 'LOSS_OF_CONTROL_UPDATE')
        debug.sethook()
        assert(ok, err)
        assert(calls == 1, 'installed XML handler must execute the original OnEvent body once')
        assert(C_UnitAuras.GetAuraDataByIndex('player', 1, 'HELPFUL').auraInstanceID == 101, 'CF2 assertion original line 88')
        assert(C_UnitAuras.GetAuraDataByIndex('party1', 1, 'HELPFUL') == nil,
            'original matching-event branch clears only the selected unit')
        "#,
    )
    .expect("installed XML event dispatch executes original event-specific aura cleanup");
}

fn fixture_aura(id: i32) -> AuraInfo {
    AuraInfo {
        name: format!("CF-2 blocked aura {id}"),
        spell_id: 99000 + id,
        icon: wow_ui_sim::lua_api::state::AuraIcon::FileId(134973),
        duration: 30.0,
        expiration_time: 45.0,
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

fn seed_blockable_auras(env: &WowLuaEnv) {
    env.exec("A_Admin.ClearLossOfControl('player'); A_Admin.ClearLossOfControl('party1')")
        .expect("supplied no-loss-of-control fixture for both selected units");
    let mut state = env.state().borrow_mut();
    state.party_group_active = true;
    state.player.buffs = vec![fixture_aura(101)];
    let party = state
        .party_members
        .first_mut()
        .expect("seeded party member");
    party.buffs = vec![fixture_aura(201), fixture_aura(202)];
    party.debuffs.clear();
}

#[test]
fn original_arena_cleanup_restores_selected_unit_aura_enumeration() {
    let env = load_original_consumer("Blizzard_CompactRaidFrames");
    seed_blockable_auras(&env);
    select_original_debuff(&env);
    env.exec(
        r#"
        assert(C_LossOfControl.GetActiveLossOfControlDataCountByUnit('party1') == 0,
            'fixture must use no-loss-of-control branch')
        C_UnitAuras.AddBlockedAura('player', 101)
        C_UnitAuras.AddBlockedAura('party1', 201)
        C_UnitAuras.AddBlockedAura('party1', 202)
        assert(C_UnitAuras.GetAuraDataByIndex('party1', 1, 'HELPFUL') == nil, 'CF2 assertion original line 140')
        assert(C_UnitAuras.GetAuraDataByIndex('player', 1, 'HELPFUL') == nil, 'CF2 assertion original line 141')
        Cf2AuraDebuff:Show()
        "#,
    )
    .expect("existing public blocked-aura seeding on the normal arena member");
    let errors_before = env.state().borrow().lua_errors.len();
    env.exec(
        r#"
        for pass = 1, 3 do
            if pass == 3 then
                C_UnitAuras.AddBlockedAura('party1', 201)
                C_UnitAuras.AddBlockedAura('party1', 202)
                assert(C_UnitAuras.GetAuraDataByIndex('party1', 1, 'HELPFUL') == nil, 'CF2 assertion original line 153')
            end
            Cf2AuraDebuff:SetUnit('party1')
            Cf2AuraDebuff:Update()
            assert(Cf2AuraDebuff.auraData == nil, 'no active loss-of-control aura')
            assert(not Cf2AuraDebuff:IsShown(), 'original cleanup hides stale debuff widget')
            assert(C_UnitAuras.GetAuraDataByIndex('party1', 1, 'HELPFUL').auraInstanceID == 201, 'CF2 assertion original line 159')
            assert(C_UnitAuras.GetAuraDataByIndex('party1', 2, 'HELPFUL').auraInstanceID == 202, 'CF2 assertion original line 160')
            assert(C_UnitAuras.GetAuraDataByIndex('party1', 3, 'HELPFUL') == nil, 'CF2 assertion original line 161')
            assert(C_UnitAuras.GetAuraDataByIndex('player', 1, 'HELPFUL') == nil,
                'clearing party1 must preserve another unit block')
        end
        -- Opposite populated selection rejects an always-party1 producer independently.
        for pass = 1, 2 do
            C_UnitAuras.AddBlockedAura('party1', 201)
            C_UnitAuras.AddBlockedAura('party1', 202)
            C_UnitAuras.AddBlockedAura('player', 101)
            assert(C_UnitAuras.GetAuraDataByIndex('party1', 1, 'HELPFUL') == nil, 'CF2 assertion original line 170')
            assert(C_UnitAuras.GetAuraDataByIndex('player', 1, 'HELPFUL') == nil, 'CF2 assertion original line 171')
            assert(C_LossOfControl.GetActiveLossOfControlDataCountByUnit('player') == 0, 'CF2 assertion original line 172')
            Cf2AuraDebuff:SetUnit('player')
            Cf2AuraDebuff:Update()
            Cf2AuraDebuff:Update()
            assert(Cf2AuraDebuff.auraData == nil, 'CF2 assertion original line 176')
            assert(not Cf2AuraDebuff:IsShown(), 'CF2 assertion original line 177')
            assert(C_UnitAuras.GetAuraDataByIndex('player', 1, 'HELPFUL').auraInstanceID == 101, 'CF2 assertion original line 178')
            assert(C_UnitAuras.GetAuraDataByIndex('player', 2, 'HELPFUL') == nil, 'CF2 assertion original line 179')
            assert(C_UnitAuras.GetAuraDataByIndex('party1', 1, 'HELPFUL') == nil,
                'clearing player must preserve populated party blocks')
        end
        "#,
    )
    .expect("original arena cleanup must clear selected unit blocks without error");
    assert_installed_event_clears_selected_auras(&env);
    assert_eq!(env.state().borrow().lua_errors.len(), errors_before);
    let state = env.state().borrow();
    assert_eq!(state.player.buffs[0].aura_instance_id, 101);
    assert_eq!(state.party_members[0].buffs.len(), 2);
    assert_eq!(state.party_members[0].buffs[0].aura_instance_id, 201);
    assert_eq!(state.party_members[0].buffs[1].aura_instance_id, 202);
}

/// BEST GUESS, non-native: required unit rejects nil, like strict C_UnitAuras
/// siblings. Pending user-supplied Retail native evidence; keep separate so that
/// changing this policy cannot weaken the assigned-unit acceptance selector.
#[test]
fn original_arena_unassigned_unit_rejects_nil_best_guess() {
    let env = load_original_consumer("Blizzard_CompactRaidFrames");
    seed_blockable_auras(&env);
    select_original_debuff(&env);
    env.exec(
        r#"
        local debuff = Cf2AuraDebuff
        assert(type(C_UnitAuras.ClearBlockedAuras) == 'function',
            'nil rejection must come from the real endpoint, not a missing member')
        C_UnitAuras.AddBlockedAura('player', 101)
        C_UnitAuras.AddBlockedAura('party1', 201)
        C_UnitAuras.AddBlockedAura('party1', 202)
        assert(C_UnitAuras.GetAuraDataByIndex('party1', 1, 'HELPFUL') == nil, 'CF2 assertion original line 211')
        debuff:SetUnit('party1')
        assert(C_UnitAuras.GetAuraDataByIndex('party1', 1, 'HELPFUL').auraInstanceID == 201, 'CF2 assertion original line 213')
        assert(C_UnitAuras.GetAuraDataByIndex('party1', 2, 'HELPFUL').auraInstanceID == 202, 'CF2 assertion original line 214')
        assert(C_UnitAuras.GetAuraDataByIndex('player', 1, 'HELPFUL') == nil, 'CF2 assertion original line 215')

        C_UnitAuras.AddBlockedAura('party1', 201)
        C_UnitAuras.AddBlockedAura('party1', 202)
        local ok, err = pcall(function() debuff:SetUnit(nil) end)
        assert(not ok and tostring(err):find('string') and tostring(err):find('nil'),
            'BEST GUESS: original unassigned consumer must reject required nil unit')
        assert(debuff.unitToken == nil, 'original SetUnit stores nil before calling Update')
        local omittedOk, omittedErr = pcall(C_UnitAuras.ClearBlockedAuras)
        assert(not omittedOk and tostring(omittedErr):find('string') and
            tostring(omittedErr):find('nil'), 'omitted required unit must also reject')
        assert(C_UnitAuras.GetAuraDataByIndex('party1', 1, 'HELPFUL') == nil,
            'rejected unassigned cleanup must not clear the previous unit')
        assert(C_UnitAuras.GetAuraDataByIndex('player', 1, 'HELPFUL') == nil,
            'rejected unassigned cleanup must not clear unrelated units')
        debuff:SetUnit('party1')
        debuff:Update()
        assert(C_UnitAuras.GetAuraDataByIndex('party1', 1, 'HELPFUL').auraInstanceID == 201, 'CF2 assertion original line 232')
        assert(C_UnitAuras.GetAuraDataByIndex('party1', 2, 'HELPFUL').auraInstanceID == 202, 'CF2 assertion original line 233')
        assert(C_UnitAuras.GetAuraDataByIndex('player', 1, 'HELPFUL') == nil, 'CF2 assertion original line 234')

        -- Re-populate both buckets and reverse selection after nil recovery.
        for pass = 1, 2 do
            C_UnitAuras.AddBlockedAura('party1', 201)
            C_UnitAuras.AddBlockedAura('party1', 202)
            C_UnitAuras.AddBlockedAura('player', 101)
            assert(C_UnitAuras.GetAuraDataByIndex('party1', 1, 'HELPFUL') == nil, 'CF2 assertion original line 241')
            assert(C_UnitAuras.GetAuraDataByIndex('player', 1, 'HELPFUL') == nil, 'CF2 assertion original line 242')
            assert(C_LossOfControl.GetActiveLossOfControlDataCountByUnit('player') == 0, 'CF2 assertion original line 243')
            debuff:SetUnit('player')
            debuff:Update()
            debuff:Update()
            assert(debuff.auraData == nil, 'CF2 assertion original line 247')
            assert(not debuff:IsShown(), 'CF2 assertion original line 248')
            assert(C_UnitAuras.GetAuraDataByIndex('player', 1, 'HELPFUL').auraInstanceID == 101, 'CF2 assertion original line 249')
            assert(C_UnitAuras.GetAuraDataByIndex('player', 2, 'HELPFUL') == nil, 'CF2 assertion original line 250')
            assert(C_UnitAuras.GetAuraDataByIndex('party1', 1, 'HELPFUL') == nil,
                'clearing player must preserve populated party blocks')
        end
        "#,
    )
    .expect("best-guess nil rejection isolated from assigned-unit cleanup acceptance");
}
