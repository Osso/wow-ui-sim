//! 12.1.0 forbidden-aspect enforcement on script dispatch, events, and auto-focus.
#![cfg(feature = "retail-12-1-0")]

use wow_ui_sim::lua_api::WowLuaEnv;

/// Shared Lua helpers: `AddonFn` stamps addon taint on a closure, `Record` appends to a log.
const PROBE_HELPERS: &str = r#"
    AspectLog = {}
    function Record(tag)
        return function() AspectLog[#AspectLog + 1] = tag end
    end
    function AddonFn(fn)
        debug.setobjecttaint(fn, 'AspectEnforcementProbe')
        return fn
    end
    function TakeLog()
        local log = table.concat(AspectLog, ',')
        AspectLog = {}
        return log
    end
"#;

fn env_with_helpers() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.exec(PROBE_HELPERS).unwrap();
    env
}

fn take_log(env: &WowLuaEnv) -> String {
    env.eval::<String>("return TakeLog()").unwrap()
}

fn widget_id(env: &WowLuaEnv, name: &str) -> u64 {
    env.state()
        .borrow()
        .widgets
        .get_id_by_name(name)
        .unwrap_or_else(|| panic!("{name} must exist"))
}

/// Each case: (aspect, handler, how it is dispatched) -> which bindings run.
#[test]
fn untrusted_script_execution_runs_only_secure_handlers_on_restricted_hierarchy() {
    let env = env_with_helpers();
    env.exec(
        r#"
        local e = Enum.ForbiddenAspect
        local restricted = CreateFrame('Frame', 'AspectRestricted', UIParent)
        restricted:AddForbiddenAspects(e.UntrustedScriptExecution)
        local child = CreateFrame('Frame', 'AspectRestrictedChild', restricted)
        local plain = CreateFrame('Frame', 'AspectPlain', UIParent)
        for _, frame in ipairs({restricted, child, plain}) do
            local name = frame:GetName()
            frame:SetScript('OnShow', AddonFn(Record(name .. ':addon')))
            frame:HookScript('OnShow', Record(name .. ':secure'))
            frame:SetScript('OnEvent', AddonFn(Record(name .. ':addonEvent')))
            frame:RegisterEvent('PLAYER_LOGIN')
        end
        local secureEvent = CreateFrame('Frame', 'AspectRestrictedSecureEvent', restricted)
        secureEvent:SetScript('OnEvent', Record('AspectRestrictedSecureEvent:secureEvent'))
        secureEvent:RegisterEvent('PLAYER_LOGIN')
        "#,
    )
    .unwrap();
    env.exec("AspectRestricted:Hide(); AspectRestricted:Show(); AspectPlain:Hide(); AspectPlain:Show()")
        .unwrap();
    assert_eq!(
        take_log(&env),
        "AspectRestrictedChild:secure,AspectRestricted:secure,AspectPlain:addon,AspectPlain:secure"
    );

    // INFERRED: a tainted caller triggering dispatch does not block secure handlers.
    env.exec("AddonFn(function() AspectRestricted:Hide(); AspectRestricted:Show() end)()")
        .unwrap();
    assert_eq!(
        take_log(&env),
        "AspectRestrictedChild:secure,AspectRestricted:secure"
    );

    env.fire_event("PLAYER_LOGIN").unwrap();
    let mut events: Vec<String> = take_log(&env).split(',').map(str::to_string).collect();
    events.sort();
    assert_eq!(
        events,
        ["AspectPlain:addonEvent", "AspectRestrictedSecureEvent:secureEvent"]
    );

    let restricted = widget_id(&env, "AspectRestricted");
    env.exec("AspectRestricted:SetScript('OnEnter', AddonFn(Record('enter:addon')))")
        .unwrap();
    env.fire_script_handler(restricted, "OnEnter", Vec::new())
        .unwrap();
    assert_eq!(take_log(&env), "", "engine mouse dispatch skips addon handlers");
    env.exec(
        r#"
        assert(type(AspectRestricted:GetScript('OnShow')) == 'function',
            'the binding stays installed; only its execution is suppressed')
        local ok = pcall(AddonFn(function() AspectRestricted:SetHyperlinkPropagateToParent(true) end))
        assert(not ok, 'SetHyperlinkPropagateToParent checks UntrustedScriptExecution')
        "#,
    )
    .unwrap();
}

#[test]
fn untrusted_layout_script_execution_suppresses_only_addon_size_handlers() {
    let env = env_with_helpers();
    env.exec(
        r#"
        local e = Enum.ForbiddenAspect
        local layout = CreateFrame('Frame', 'AspectLayout', UIParent)
        layout:AddForbiddenAspects(e.UntrustedLayoutScriptExecution)
        local anchored = CreateFrame('Frame', 'AspectAnchored', UIParent)
        anchored:AddForbiddenAspects(layout:GetInheritableForbiddenAspects(Enum.ScriptObjectPropagationPath.Layout))
        anchored:SetPoint('TOPLEFT', layout, 'BOTTOMLEFT')
        for _, frame in ipairs({layout, anchored}) do
            local name = frame:GetName()
            frame:SetScript('OnSizeChanged', AddonFn(Record(name .. ':addonSize')))
            frame:HookScript('OnSizeChanged', Record(name .. ':secureSize'))
            frame:SetScript('OnShow', AddonFn(Record(name .. ':addonShow')))
        end
        layout:Hide(); layout:Show()
        "#,
    )
    .unwrap();
    assert_eq!(
        take_log(&env),
        "AspectLayout:addonShow",
        "layout restriction leaves non-layout addon handlers running"
    );
    for name in ["AspectLayout", "AspectAnchored"] {
        let id = widget_id(&env, name);
        env.fire_script_handler(id, "OnSizeChanged", Vec::new())
            .unwrap();
    }
    assert_eq!(
        take_log(&env),
        "AspectLayout:secureSize,AspectAnchored:secureSize"
    );
}

#[test]
fn scripted_input_and_query_focus_allow_secure_callers_only() {
    let env = env_with_helpers();
    env.exec(
        r#"
        local e = Enum.ForbiddenAspect
        local button = CreateFrame('Button', 'AspectSecureClick', UIParent)
        button:SetScript('OnClick', Record('click'))
        button:AddForbiddenAspects(bit.bor(e.ScriptedInput, e.QueryFocus))
        local box = CreateFrame('EditBox', 'AspectSecureBox', UIParent)
        box:AddForbiddenAspects(bit.bor(e.ScriptedInput, e.QueryFocus))
        assert(not pcall(AddonFn(function() button:Click() end)))
        assert(not pcall(AddonFn(function() box:SetFocus() end)))
        assert(not pcall(AddonFn(function() return box:HasFocus() end)))
        assert(not pcall(AddonFn(function() return button:IsMouseMotionFocus() end)))
        button:Click()
        box:SetFocus()
        assert(box:HasFocus() and not button:IsMouseMotionFocus())
        box:ClearFocus()
        assert(not box:HasFocus())
        "#,
    )
    .expect("tainted input calls reject; secure callers keep every operation");
    assert_eq!(take_log(&env), "click");
}

#[test]
fn autofocus_editbox_skips_focus_while_shown_secret_aspect_applies() {
    let env = env_with_helpers();
    env.exec(
        r#"
        local function makeBox(name)
            local box = CreateFrame('EditBox', name, UIParent)
            box:SetAutoFocus(true)
            box:Hide()
            box:SetScript('OnEditFocusGained', Record(name .. ':gained'))
            return box
        end
        local plain = makeBox('AutoFocusPlain')
        local secret = makeBox('AutoFocusSecret')
        local secretParent = CreateFrame('Frame', nil, UIParent)
        local inherited = CreateFrame('EditBox', 'AutoFocusInherited', secretParent)
        inherited:SetAutoFocus(true)
        inherited:SetScript('OnEditFocusGained', Record('AutoFocusInherited:gained'))
        secretParent:Hide()

        plain:Show()
        assert(plain:HasFocus(), 'autoFocus EditBox takes focus when shown')
        plain:ClearFocus()
        secret:SetShown(secretwrap(true))
        assert(not secret:HasFocus(), 'Shown secret aspect blocks auto-focus')
        secretParent:SetShown(secretwrap(true))
        assert(not inherited:HasFocus(), 'ancestor Shown secret also blocks auto-focus')
        secret:Hide()
        secret:Show()
        assert(secret:HasFocus(), 'a plain Show clears the secret and restores auto-focus')
        "#,
    )
    .expect("auto-focus follows visibility unless the Shown secret aspect applies");
    assert_eq!(take_log(&env), "AutoFocusPlain:gained,AutoFocusSecret:gained");
}

#[test]
fn aura_container_intrinsic_event_registrations_block_addons_not_blizzard() {
    crate::common::with_timeout(90, || {
        crate::common::blizzard_addon_harness::with_blizzard_addon_closure(
            &["Blizzard_AuraContainer"],
            &[],
            |env, _| {
                env.exec(
                    r#"
                    local e = Enum.ForbiddenAspect
                    local container
                    local function addonSetup()
                        container = CreateFrame('AuraContainer', nil, UIParent, 'CustomAuraContainerTemplate')
                        container:SetUnit('player')
                    end
                    debug.setobjecttaint(addonSetup, 'AuraContainerProbe')
                    local ok, err = pcall(addonSetup)
                    assert(ok, tostring(err))
                    assert(bit.band(container:GetForbiddenAspects(), e.EventRegistrations) ~= 0,
                        'AuraContainer intrinsic carries EventRegistrations')
                    assert(container:IsEventRegistered('AURA_DATA_PROVIDER_SWITCH'),
                        'Blizzard OnLoad_Intrinsic registers its static event under the aspect')
                    local function addonRegister() return container:RegisterEvent('PLAYER_LOGIN') end
                    local function addonQuery() return container:IsEventRegistered('AURA_DATA_PROVIDER_SWITCH') end
                    local function addonClear() return container:UnregisterAllEvents() end
                    for _, fn in ipairs({addonRegister, addonQuery, addonClear}) do
                        debug.setobjecttaint(fn, 'AuraContainerProbe')
                        assert(not pcall(fn), 'addon event registration access is rejected')
                    end
                    assert(not container:IsEventRegistered('PLAYER_LOGIN'))
                    assert(container:IsEventRegistered('AURA_DATA_PROVIDER_SWITCH'),
                        'rejected clear left registrations')
                    assert(container:RegisterEvent('PLAYER_LOGIN') and container:IsEventRegistered('PLAYER_LOGIN'),
                        'secure callers keep registration access')
                    local plain = CreateFrame('Frame')
                    assert(bit.band(plain:GetForbiddenAspects(), e.EventRegistrations) == 0)
                    "#,
                )
                .expect("intrinsic AuraContainer aspect restricts addons only");
            },
        );
    });
}

#[test]
fn aura_button_aspects_block_addon_handlers_and_input_on_button_and_children() {
    crate::common::with_timeout(90, || {
        crate::common::blizzard_addon_harness::with_blizzard_addon_closure(
            &["Blizzard_AuraContainer"],
            &[],
            |env, _| {
                env.exec(PROBE_HELPERS).unwrap();
                env.exec(
                    r#"
                    local e = Enum.ForbiddenAspect
                    local button = CreateFrame('AuraButton', 'AspectAuraButton', UIParent, 'CustomAuraButtonTemplate')
                    local expected = bit.bor(e.UntrustedScriptExecution, e.UntrustedLayoutScriptExecution,
                        e.AlwaysPropagateInput, e.ScriptedInput, e.QueryFocus)
                    assert(bit.band(button:GetForbiddenAspects(), expected) == expected)
                    local overlay = CreateFrame('Frame', 'AspectAuraOverlay', button)
                    for _, frame in ipairs({button, overlay}) do
                        frame:HookScript('OnShow', AddonFn(Record(frame:GetName() .. ':addon')))
                    end
                    button:Hide(); button:Show()
                    assert(not pcall(AddonFn(function() button:Click() end)))
                    assert(not pcall(AddonFn(function() return button:IsMouseMotionFocus() end)))
                    assert(overlay:GetPropagateKeyboardInput() and overlay:CanPropagateMouseClicks())
                    "#,
                )
                .expect("aura button aspects apply to the button and addon-created children");
                assert_eq!(take_log(env), "");
            },
        );
    });
}
