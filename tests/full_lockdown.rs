//! R18 static native decisions; completed Lua outcomes are INFERRED.

use wow_ui_sim::lua_api::WowLuaEnv;

fn lockdown_environment() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    let directory = tempfile::tempdir().unwrap();
    let toc = directory.path().join("LockdownFixture.toc");
    std::fs::write(&toc, "## Title: LockdownFixture\nfixture.xml\n").unwrap();
    std::fs::write(
        directory.path().join("fixture.xml"),
        r#"<Ui>
            <ScopedModifier fullLockdown="true">
                <Frame name="FullLockParent" parent="UIParent">
                    <Layers><Layer><FontString name="FullLockText" parentKey="Text"/></Layer></Layers>
                </Frame>
                <ScopedModifier fullLockdown="false">
                    <Frame name="UnlockedScope" parent="UIParent"/>
                    <Frame name="InheritedLock" parent="FullLockParent">
                        <Layers><Layer><FontString name="InheritedLockText" parentKey="Text"/></Layer></Layers>
                    </Frame>
                </ScopedModifier>
                <Frame name="RestoredLock" parent="UIParent"/>
            </ScopedModifier>
            <Frame name="AfterLockScope" parent="UIParent"/>
        </Ui>"#,
    )
    .unwrap();
    let loaded = wow_ui_sim::loader::load_addon(&env.loader_env(), &toc).unwrap();
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    env.exec(
        r#"
        function AssertFullLockRefused(callback, ...)
            local ok, message = pcall(callback, ...)
            assert(not ok, 'fully locked-down API accepted the object')
            assert(string.find(message, 'Attempt to access fully locked-down object.', 1, true), message)
        end
        "#,
    )
    .unwrap();
    env
}

#[test]
fn full_lockdown_scope_false_restores_and_parent_inheritance_wins() {
    let env = lockdown_environment();
    env.exec(
        r#"
        assert(issecure())
        assert(FullLockParent:IsForbidden() and FullLockText:IsForbidden())
        assert(not UnlockedScope:IsForbidden(), 'explicit false clears the creation default')
        assert(InheritedLock:IsForbidden() and InheritedLockText:IsForbidden(), 'parent inheritance survives false default')
        assert(RestoredLock:IsForbidden(), 'outer true default must be restored')
        assert(not AfterLockScope:IsForbidden(), 'outer scope must restore the previous default')
        "#,
    )
    .unwrap();
}

#[test]
fn full_lockdown_rejects_measured_font_string_apis_for_secure_and_tainted_callers() {
    let env = lockdown_environment();
    env.exec(
        r#"
        local function probe(label)
            for _, method in ipairs({'CanWordWrap', 'CanNonSpaceWrap', 'ClearAlphaGradient'}) do
                AssertFullLockRefused(label[method], label)
            end
            local binding = C_DurationUtil.CreateDurationTextBinding()
            AssertFullLockRefused(binding.SetFontString, binding, label)
        end
        assert(issecure())
        probe(FullLockText)
        probe(InheritedLockText)
        probe(GetForbiddenObjectTable(FullLockText))
        local function addon()
            assert(not issecure())
            probe(FullLockText)
            probe(GetForbiddenObjectTable(FullLockText))
        end
        debug.setobjecttaint(addon, 'FullLockdownFixture')
        addon()
        "#,
    )
    .unwrap();
}

#[test]
fn full_lockdown_create_font_string_rejects_before_creating_a_child() {
    let env = lockdown_environment();
    env.exec(
        r#"
        assert(issecure())
        AssertFullLockRefused(FullLockParent.CreateFontString, FullLockParent, 'MustNotExist')
        assert(MustNotExist == nil)
        local function addon()
            AssertFullLockRefused(FullLockParent.CreateFontString, FullLockParent, 'MustNotExist')
        end
        debug.setobjecttaint(addon, 'FullLockdownFixture')
        addon()
        assert(MustNotExist == nil)
        "#,
    )
    .unwrap();
}

#[test]
fn full_lockdown_set_forbidden_alone_keeps_secure_font_string_access() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        FullLockParent = CreateFrame('Frame')
        FullLockLabel = FullLockParent:CreateFontString()
        FullLockLabel:SetForbidden()
        assert(FullLockLabel:IsForbidden())
        assert(issecure())
        assert(type(FullLockLabel:CanWordWrap()) == 'boolean')
        assert(type(FullLockLabel:CanNonSpaceWrap()) == 'boolean')
        FullLockLabel:ClearAlphaGradient()
        FullLockBinding = C_DurationUtil.CreateDurationTextBinding()
        FullLockBinding:SetFontString(FullLockLabel)
        "#,
    )
    .expect("SetForbidden alone must leave the secure FontString usable");
}

#[test]
fn missing_access_allowed_flag_locks_parent_and_xml_created_children() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("AccessLostParent = CreateFrame('Frame', 'AccessLostParent', UIParent)")
        .unwrap();
    let parent_id = env
        .state()
        .borrow()
        .widgets
        .get_id_by_name("AccessLostParent")
        .unwrap();
    env.state()
        .borrow_mut()
        .widgets
        .get_mut(parent_id)
        .unwrap()
        .access_allowed = false;

    let directory = tempfile::tempdir().unwrap();
    let toc = directory.path().join("AccessLostFixture.toc");
    std::fs::write(&toc, "## Title: AccessLostFixture\nfixture.xml\n").unwrap();
    std::fs::write(
        directory.path().join("fixture.xml"),
        r#"<Ui><Frame name="AccessLostChild" parent="AccessLostParent">
            <Layers><Layer><FontString name="$parentText" parentKey="Text"/></Layer></Layers>
        </Frame></Ui>"#,
    )
    .unwrap();
    let loaded = wow_ui_sim::loader::load_addon(&env.loader_env(), &toc).unwrap();
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    env.exec(
        r#"
        assert(AccessLostParent:IsForbidden())
        assert(AccessLostChild:IsForbidden())
        assert(AccessLostChildText:IsForbidden())
        local ok, message = pcall(AccessLostChildText.CanWordWrap, AccessLostChildText)
        assert(not ok and message == 'Attempt to access fully locked-down object.', tostring(message))
        "#,
    )
    .expect("a child created from a parent without access-allowed state is fully locked down");
}
