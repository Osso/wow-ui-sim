//! Public custom group creation with real native aura data and managed frames.
#![cfg(feature = "retail-12-1-0")]
use wow_ui_sim::lua_api::{WowLuaEnv, state::AuraInfo};

#[test]
fn patch_12_1_0_aura_groups_filter_limit_and_refresh_actual_frames() {
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let (env, _) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui, &["Blizzard_AuraContainer"], &[],
    );
    env.state().borrow_mut().player.buffs = [(11, "player"), (22, "party1"), (33, "player")]
        .into_iter().map(|(id, source)| AuraInfo {
            name: format!("Audit Aura {id}"), spell_id: 1000 + id, icon: 1,
            duration: 30.0, expiration_time: 30.0, applications: 1,
            source_unit: source.into(), is_helpful: true, is_raid: true,
            is_nameplate_only: false, is_stealable: false, can_apply_aura: true,
            is_from_player_or_player_pet: source == "player", dispel_type: None,
            aura_instance_id: id,
        }).collect();
    env.exec(r#"
        AuditGroups = CreateFrame('ManagedAuraContainer', nil, UIParent, 'CustomAuraContainerTemplate')
        AuditGroups:SetUnit('player')
        local function addon()
            assert(select('#', AuditGroups:AddAuraGroup('all:custom-key', 'HELPFUL', {maxFrameCount=1})) == 0)
            AuditGroups:AddAuraGroup('mine:custom-key', 'HELPFUL|PLAYER', {})
            assert(AuditGroups:HasAuraGroup('all:custom-key') and AuditGroups:HasAuraGroup('mine:custom-key'))
            assert(not AuditGroups:HasAuraGroup('unknown'))
            assert(AuditGroups:GetAuraGroupFrame('unknown', 1) == nil)
            assert(not pcall(AuditGroups.AddAuraGroup, AuditGroups, 'all:custom-key', 'HELPFUL', {}))
            assert(not pcall(AuditGroups.AddAuraGroup, AuditGroups, '', 'HELPFUL', {}))
            assert(debug.getstacktaint() == 'AuditGroupAddon')
        end
        debug.setobjecttaint(addon, 'AuditGroupAddon')
        addon()
        function AuditVisibleGroupFrames(key)
            local visible = 0
            for index=1, AuditGroups:GetAuraGroupFrameCount(key) do
                local frame = AuditGroups:GetAuraGroupFrame(key, index)
                assert(frame:GetObjectType() == 'AuraButton' and frame:GetParent() == AuditGroups)
                local shown = frame:IsShown()
                if issecretvalue(shown) then shown = secretunwrap(shown) end
                if shown then visible = visible + 1 end
            end
            return visible
        end
    "#).unwrap();
    env.fire_on_update(0.016).unwrap();
    env.exec(r#"
        assert(AuditVisibleGroupFrames('all:custom-key') == 1, 'maxFrameCount caps actual shown frames')
        assert(AuditVisibleGroupFrames('mine:custom-key') == 2, 'PLAYER group uses independent filter')
        AuditGroups:SetAuraGroupMaxFrameCount('all:custom-key', 3)
        AuditGroups:SetAuraGroupFilterString('mine:custom-key', 'HARMFUL')
    "#).unwrap();
    env.fire_on_update(0.016).unwrap();
    env.exec(r#"
        assert(AuditVisibleGroupFrames('all:custom-key') == 3, 'maxFrameCount increase refreshes presentation')
        assert(AuditVisibleGroupFrames('mine:custom-key') == 0, 'filter change hides old assignments')
    "#).unwrap();
    assert!(env.state().borrow().lua_errors.is_empty(), "{:?}", env.state().borrow().lua_errors);
}
