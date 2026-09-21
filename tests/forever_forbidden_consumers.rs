//! Real Forever consumers of the shared forbidden-aspect capability.
#![cfg(feature = "client-wowforever")]

const FIXTURE: &str = include_str!("fixtures/forever_forbidden_consumers.lua");

fn setup() -> wow_ui_sim::lua_api::WowLuaEnv {
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let (env, _) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui,
        &[
            "Blizzard_RestrictedAddOnEnvironment",
            "Blizzard_AuraContainer",
        ],
        &[],
    );
    env.exec(FIXTURE).unwrap();
    env
}

#[test]
fn forever_forbidden_consumers_secure_handler_accepts_plain_and_rejects_marked() {
    let env = setup();
    env.exec("ForeverForbiddenConsumers.secure_handlers()")
        .unwrap();
}

#[test]
fn forever_forbidden_consumers_tainted_aura_creation_and_dirty_lifecycle() {
    let env = setup();
    env.exec("ForeverForbiddenConsumers.aura_shell()").unwrap();
    env.fire_on_update(0.016).unwrap();
    env.fire_on_update(0.016).unwrap();
    env.exec(
        "assert(ForbiddenConsumerDirtyCalls == 1); \
         assert(not GetForbiddenObjectTable(ForbiddenConsumerAura):IsDirty()); \
         assert(ForbiddenConsumerAura:GetOnUpdateMode() == Enum.OnUpdateMode.Disabled)",
    )
    .unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
fn forever_forbidden_consumers_admin_aura_events_refresh_intrinsic_only_container() {
    let env = setup();
    env.exec(r#"
        A_Admin.ClearBuffs()
        NativeAuraEventProbe = CreateFrame('AuraContainer', nil, UIParent, 'CustomAuraContainerTemplate')
        local container = NativeAuraEventProbe
        container:SetSize(100, 20)
        container:AddAuraGroup('probe', 'HELPFUL', {
            maxFrameCount = 2,
            initializeFrame = function(button) button:SetSize(20, 20) end,
        })
        container:SetUnit('player')
        local registered, unit = container:IsEventRegistered('UNIT_AURA')
        assert(registered and unit == 'player')
        assert(container:GetScript('OnEvent', 1) == nil, 'native container has no normal OnEvent')
        assert(type(container:GetScript('OnEvent', 0)) == 'function'
            or type(container:GetScript('OnEvent', 2)) == 'function', 'native intrinsic binding missing')
        function AssertNativeAuraEventState(expectedCount, stacks)
            local count = 0
            for index = 1, container:GetAuraGroupFrameCount('probe') do
                local button = container:GetAuraGroupFrame('probe', index)
                local unit, data = GetForbiddenObjectTable(button):GetAuraInstance()
                if unit == 'player' and data and data.spellId == 19750 then
                    count = count + 1
                    assert(data.applications == stacks, 'assignment retained an earlier aura snapshot')
                end
            end
            assert(count == expectedCount, 'native aura assignment did not follow the event')
            assert(not GetForbiddenObjectTable(container):IsDirty())
            assert(container:GetOnUpdateMode() == Enum.OnUpdateMode.Disabled)
        end
    "#).expect("construct the real intrinsic-only container before any test aura exists");
    settle_native_aura_event(&env);
    env.exec("AssertNativeAuraEventState(0)").unwrap();
    for stacks in [1, 2] {
        env.exec(&format!(
            "A_Admin.AddBuff(19750, 'Event lifecycle aura', '135907', 30, {stacks}); \
             assert(GetForbiddenObjectTable(NativeAuraEventProbe):IsDirty()); \
             assert(NativeAuraEventProbe:GetOnUpdateMode() == Enum.OnUpdateMode.RunWhenVisibleOnce)"
        ))
        .expect("adding an aura after construction must rearm native event processing");
        settle_native_aura_event(&env);
        env.exec(&format!("AssertNativeAuraEventState(1, {stacks})"))
            .unwrap();
        env.exec(
            "A_Admin.RemoveBuff(19750); \
             assert(C_UnitAuras.GetPlayerAuraBySpellID(19750) == nil); \
             assert(GetForbiddenObjectTable(NativeAuraEventProbe):IsDirty()); \
             assert(NativeAuraEventProbe:GetOnUpdateMode() == Enum.OnUpdateMode.RunWhenVisibleOnce)",
        )
        .expect("removing an aura must rearm the already-settled native container");
        settle_native_aura_event(&env);
        env.exec("AssertNativeAuraEventState(0)").unwrap();
    }
    assert!(env.state().borrow().lua_errors.is_empty());
}

fn settle_native_aura_event(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    env.fire_on_update(0.016).unwrap();
    env.fire_on_update(0.016).unwrap();
}

#[test]
fn forever_forbidden_consumers_native_masks_and_inheritance_rejections() {
    let env = setup();
    env.exec("ForeverForbiddenConsumers.enums_and_inheritance()")
        .unwrap();
}
