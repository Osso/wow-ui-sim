//! Selected Blizzard handlers own state snippets once the restricted runtime loads.
#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn restricted_env() -> WowLuaEnv {
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let (env, loaded) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui,
        &["Blizzard_RestrictedAddOnEnvironment"],
        &[],
    );
    assert!(
        loaded
            .iter()
            .any(|name| name == "Blizzard_RestrictedAddOnEnvironment")
    );
    assert!(
        wow_ui_sim::xml::get_template("SecureHandlerStateTemplate")
            .is_some_and(|template| template.declaration_taint.is_none())
    );
    env
}

#[test]
fn forever_secure_state_dispatch_uses_restricted_handle_once() {
    let env = restricted_env();
    env.exec(
        r#"
        local header = CreateFrame('Frame', nil, UIParent, 'SecureHandlerStateTemplate')
        local child = CreateFrame('Frame', nil, UIParent, 'SecureHandlerBaseTemplate')
        SecureHandlerSetFrameRef(header, 'child', child)
        header:SetAttribute('apply', [[
            local id, value = ...
            self:SetAttribute('received-stateid', id)
            self:GetFrameRef('child'):SetAttribute('received-value', value)
            return value
        ]])
        header:SetAttribute('_onstate-probe', [[
            self:SetAttribute('runs', (self:GetAttribute('runs') or 0) + 1)
            self:SetAttribute('returned-value', self:RunAttribute('apply', stateid, newstate))
        ]])
        RegisterStateDriver(header, 'probe', 'first')
        assert(header:GetAttribute('runs') == 1, 'state snippet must execute exactly once')
        assert(header:GetAttribute('received-stateid') == 'probe')
        assert(header:GetAttribute('returned-value') == 'first')
        assert(child:GetAttribute('received-value') == 'first')
        header:SetAttribute('state-probe', 'second')
        assert(header:GetAttribute('runs') == 2)
        assert(header:GetAttribute('returned-value') == 'second')
        assert(child:GetAttribute('received-value') == 'second')
        SecureStateDispatchHeader = header
        SecureStateDispatchChild = child
        "#,
    )
    .unwrap();
    env.apply_post_load_workarounds();
    env.exec(
        r#"
        SecureStateDispatchHeader:SetAttribute('state-probe', 'after-cleanup')
        assert(SecureStateDispatchHeader:GetAttribute('runs') == 3)
        assert(SecureStateDispatchHeader:GetAttribute('received-stateid') == 'probe')
        assert(SecureStateDispatchChild:GetAttribute('received-value') == 'after-cleanup')
        "#,
    )
    .unwrap();

    let fixture = tempfile::tempdir().expect("create temporary addon root");
    let addon = fixture.path().join("SecureStateTemplateShadow");
    std::fs::create_dir(&addon).expect("create third-party addon directory");
    let toc = addon.join("SecureStateTemplateShadow.toc");
    std::fs::write(&toc, "## Interface: 16001\nShadow.xml\n").expect("write addon TOC");
    std::fs::write(
        addon.join("Shadow.xml"),
        r#"<Ui><Frame name="SecureHandlerStateTemplate" virtual="true"/></Ui>"#,
    )
    .expect("write template shadow");
    let loaded = wow_ui_sim::loader::load_addon(&env.loader_env(), &toc)
        .expect("load third-party template through its TOC");
    assert_eq!(loaded.xml_files, 1, "{loaded:?}");
    let shadow = wow_ui_sim::xml::get_template("SecureHandlerStateTemplate")
        .expect("third-party declaration must replace the registered template");
    assert_eq!(
        shadow.declaration_taint.as_deref(),
        Some("SecureStateTemplateShadow")
    );
    env.exec(
        r#"
        local header = SecureStateDispatchHeader
        header:SetAttribute('state-probe', 'after-shadow')
        assert(header:GetAttribute('runs') == 4, 'late template shadow must not replay the snippet')
        assert(header:GetAttribute('received-stateid') == 'probe')
        assert(header:GetAttribute('returned-value') == 'after-shadow')
        assert(SecureStateDispatchChild:GetAttribute('received-value') == 'after-shadow')
        "#,
    )
    .expect("existing vendor header must dispatch through its restricted handle after a template shadow");
    let sim = env.state().borrow();
    assert!(sim.lua_errors.is_empty(), "{:?}", sim.lua_errors);
}

#[test]
fn forever_secure_state_dispatch_leaves_plain_frames_to_their_callbacks() {
    let env = restricted_env();
    env.exec(
        r#"
        local frame = CreateFrame('Frame')
        local deliveries = 0
        frame:SetAttribute('_onstate-probe', "self:SetAttribute('unexpected-snippet', true)")
        frame:SetScript('OnAttributeChanged', function(_, name, value)
            if name == 'state-probe' then
                deliveries = deliveries + 1
                assert(value == 'changed')
            end
        end)
        frame:SetAttribute('state-probe', 'changed')
        assert(deliveries == 1)
        assert(frame:GetAttribute('unexpected-snippet') == nil,
            'plain frames do not acquire secure state-template behavior')
        "#,
    )
    .unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
fn forever_secure_state_dispatch_preserves_pre_vendor_fallback() {
    let env = WowLuaEnv::new().unwrap();
    assert!(wow_ui_sim::xml::get_template("SecureHandlerStateTemplate").is_none());
    env.exec(
        r#"
        assert(rawget(__secureenv, 'SecureHandler_StateOnAttributeChanged') == nil)
        SecureHandler_StateOnAttributeChanged = function() error('addon shadow') end
        local frame = CreateFrame('Frame')
        local deliveries = 0
        frame:SetScript('OnAttributeChanged', function(_, name)
            if name == 'state-probe' then deliveries = deliveries + 1 end
        end)
        frame:SetAttribute('_onstate-probe', [[
            self:SetAttribute('runs', (self:GetAttribute('runs') or 0) + 1)
            self:SetAttribute('received-value', newstate)
        ]])
        frame:SetAttribute('state-probe', 'first')
        assert(deliveries == 1 and frame:GetAttribute('runs') == 1)
        assert(frame:GetAttribute('received-value') == 'first')
        RegisterStateDriver(frame, 'probe', 'second')
        assert(frame:GetAttribute('runs') == 2)
        assert(frame:GetAttribute('received-value') == 'second')
        "#,
    )
    .unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}
