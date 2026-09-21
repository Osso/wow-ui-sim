//! Numeric script-object update modes shared by Mainline and Forever.
#![cfg(any(feature = "retail-12-1-0", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn on_update_modes_publish_numeric_contract_and_default() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local expected = {Disabled=0, RunWhenVisible=1, RunWhenVisibleOnce=2, RunOnce=3, RunAlways=4}
        local count = 0
        for key, value in pairs(Enum.OnUpdateMode) do
            assert(expected[key] == value, key)
            count = count + 1
        end
        assert(count == 5)
        assert(Enum.OnUpdateModeMeta.MinValue == 0)
        assert(Enum.OnUpdateModeMeta.MaxValue == 4)
        assert(Enum.OnUpdateModeMeta.NumValues == 5)
        assert(Enum.ScriptObjectOnUpdateMode == nil)
        local f = CreateFrame("Frame", nil, UIParent)
        assert(f:GetOnUpdateMode() == 1)
        for mode = 0, 4 do
            f:SetOnUpdateMode(mode)
            assert(f:GetOnUpdateMode() == mode)
        end
        for _, bad in ipairs({"RunAlways", -1, 5, 1.5}) do
            assert(not pcall(f.SetOnUpdateMode, f, bad))
        end
        assert(f:GetOnUpdateMode() == 4)
    "#).unwrap();
}

#[test]
fn on_update_modes_obey_ancestor_visibility_and_one_shots() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        UpdateModeParent = CreateFrame("Frame", nil, UIParent)
        UpdateModeParent:Hide()
        ModeFrames, ModeCalls, ObservedModes = {}, {}, {}
        for mode = 0, 4 do
            local f = CreateFrame("Frame", nil, UpdateModeParent)
            ModeFrames[mode], ModeCalls[mode] = f, 0
            f:SetOnUpdateMode(mode)
            f:SetScript("OnUpdate", function(self)
                ModeCalls[mode] = ModeCalls[mode] + 1
                ObservedModes[mode] = self:GetOnUpdateMode()
            end)
        end
    "#,
    )
    .unwrap();
    for _ in 0..2 {
        env.fire_on_update(0.016).unwrap();
    }
    env.exec(
        r#"
        assert(ModeCalls[0] == 0 and ModeCalls[1] == 0 and ModeCalls[2] == 0)
        assert(ModeCalls[3] == 1 and ModeCalls[4] == 2)
        assert(ObservedModes[3] == 0 and ObservedModes[4] == 4)
        assert(ModeFrames[2]:GetOnUpdateMode() == 2)
        UpdateModeParent:Show()
    "#,
    )
    .unwrap();
    for _ in 0..2 {
        env.fire_on_update(0.016).unwrap();
    }
    env.exec(
        r#"
        assert(ModeCalls[0] == 0 and ModeCalls[1] == 2 and ModeCalls[2] == 1)
        assert(ModeCalls[3] == 1 and ModeCalls[4] == 4)
        assert(ObservedModes[2] == 0)
        assert(ModeFrames[2]:GetOnUpdateMode() == 0)
    "#,
    )
    .unwrap();
}

#[test]
fn on_update_modes_reset_before_handlers_and_preserve_rearming() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        RearmFrames, RearmCalls, HookCalls = {}, {}, {}
        for _, mode in ipairs({2, 3}) do
            local f = CreateFrame("Frame", nil, UIParent)
            if mode == 3 then f:Hide() end
            RearmFrames[mode], RearmCalls[mode], HookCalls[mode] = f, 0, 0
            f:SetOnUpdateMode(mode)
            f:SetScript("OnUpdate", function(self)
                assert(self:GetOnUpdateMode() == 0, "one-shot must reset before callback")
                RearmCalls[mode] = RearmCalls[mode] + 1
                if RearmCalls[mode] == 1 then self:SetOnUpdateMode(mode) end
            end)
            f:HookScript("OnUpdate", function(self)
                HookCalls[mode] = HookCalls[mode] + 1
                local expected = HookCalls[mode] == 1 and mode or 0
                assert(self:GetOnUpdateMode() == expected, "rearm was clobbered between handlers")
            end)
        end
    "#,
    )
    .unwrap();
    env.fire_on_update(0.016).unwrap();
    env.exec(
        "assert(RearmFrames[2]:GetOnUpdateMode() == 2 and RearmFrames[3]:GetOnUpdateMode() == 3)",
    )
    .unwrap();
    env.fire_on_update(0.016).unwrap();
    env.fire_on_update(0.016).unwrap();
    env.exec(
        r#"
        assert(RearmCalls[2] == 2 and RearmCalls[3] == 2)
        assert(HookCalls[2] == 2 and HookCalls[3] == 2)
        assert(RearmFrames[2]:GetOnUpdateMode() == 0 and RearmFrames[3]:GetOnUpdateMode() == 0)
    "#,
    )
    .unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
fn on_update_modes_xml_names_select_numeric_modes() {
    let env = WowLuaEnv::new().unwrap();
    let root = tempfile::tempdir().unwrap();
    let toc = root.path().join("UpdateModes.toc");
    std::fs::write(&toc, "Frames.xml\n").unwrap();
    std::fs::write(
        root.path().join("Frames.xml"),
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
          <Frame name="XmlUpdateOnce" parent="UIParent" hidden="true" onUpdateMode="RunOnce">
            <Scripts><OnUpdate>self.calls = (self.calls or 0) + 1</OnUpdate></Scripts>
          </Frame>
        </Ui>
    "#,
    )
    .unwrap();
    let loaded = wow_ui_sim::loader::load_addon(&env.loader_env(), &toc).unwrap();
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    env.exec("assert(XmlUpdateOnce:GetOnUpdateMode() == 3)")
        .unwrap();
    env.fire_on_update(0.016).unwrap();
    env.fire_on_update(0.016).unwrap();
    env.exec("assert(XmlUpdateOnce.calls == 1 and XmlUpdateOnce:GetOnUpdateMode() == 0)")
        .unwrap();
}

#[test]
fn on_update_modes_process_actual_managed_aura_dirty_phases() {
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let (env, _) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui,
        &["Blizzard_AuraContainer"],
        &[],
    );
    env.exec(r#"
        ModeAuraContainer = CreateFrame("ManagedAuraContainer", nil, UIParent, "CustomAuraContainerTemplate")
        local private = GetForbiddenObjectTable(ModeAuraContainer)
        assert(type(private.ProcessDirtyFlags) == "function")
        AuraPhaseCalls = 0
        private:MarkClean(private.dirtyFlags:GetFlags())
        private:SetDirtyPhases({{flag=1, handler=function()
            assert(ModeAuraContainer:GetOnUpdateMode() == 0)
            AuraPhaseCalls = AuraPhaseCalls + 1
        end}})
        ModeAuraContainer:Hide()
        private:MarkDirty(1)
        assert(ModeAuraContainer:GetOnUpdateMode() == 2)
        assert(private:IsDirty())
    "#).unwrap();
    env.fire_on_update(0.016).unwrap();
    env.exec("assert(AuraPhaseCalls == 0); ModeAuraContainer:Show()")
        .unwrap();
    env.fire_on_update(0.016).unwrap();
    env.fire_on_update(0.016).unwrap();
    env.exec(
        r#"
        assert(AuraPhaseCalls == 1)
        assert(not GetForbiddenObjectTable(ModeAuraContainer):IsDirty())
        assert(ModeAuraContainer:GetOnUpdateMode() == 0)
    "#,
    )
    .unwrap();
    assert!(
        env.state().borrow().lua_errors.is_empty(),
        "{:?}",
        env.state().borrow().lua_errors
    );
}
