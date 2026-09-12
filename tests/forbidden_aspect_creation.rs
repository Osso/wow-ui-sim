#![cfg(feature = "retail-12-1-0")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn load_xml_fixture(env: &WowLuaEnv, xml: &str) -> wow_ui_sim::loader::LoadResult {
    let directory = tempfile::tempdir().expect("create fixture directory");
    let toc = directory.path().join("AspectFixture.toc");
    std::fs::write(&toc, "## Title: AspectFixture\nfixture.xml\n").unwrap();
    std::fs::write(directory.path().join("fixture.xml"), xml).unwrap();
    wow_ui_sim::loader::load_addon(&env.loader_env(), &toc).expect("load aspect fixture")
}

#[test]
fn native_children_inherit_only_hierarchy_aspects_at_creation() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local e, path = Enum.ForbiddenAspect, Enum.ScriptObjectPropagationPath
        assert(path.Hierarchy == 0 and path.Layout == 1, 'native propagation path values')
        local meta = Enum.ScriptObjectPropagationPathMeta
        assert(meta.MinValue == 0 and meta.MaxValue == 1 and meta.NumValues == 2)
        local parent = CreateFrame('Frame')
        parent:AddForbiddenAspects(bit.bor(e.UntrustedScriptExecution, e.UntrustedLayoutScriptExecution, e.AlwaysPropagateInput))
        assert(parent:GetInheritableForbiddenAspects(path.Hierarchy) == 44, 'hierarchy path is enum value zero')
        assert(parent:GetInheritableForbiddenAspects(path.Layout) == 8, 'only layout restrictions follow anchors by default')
        local frame = CreateFrame('Frame', nil, parent)
        local children = {frame, parent:CreateTexture(), parent:CreateFontString(), parent:CreateMaskTexture(), parent:CreateLine()}
        for _, child in ipairs(children) do
            assert(child:GetForbiddenAspects() == 45, 'child owns inherited aspects plus SetToDefaults')
            assert(child:GetInheritableForbiddenAspects(path.Hierarchy) == 44)
            assert(child:GetInheritableForbiddenAspects(path.Layout) == 8)
        end
        local grandchild = frame:CreateTexture()
        assert(grandchild:GetForbiddenAspects() == 45, 'hierarchy inheritance remains transitive')
        local slider = CreateFrame('Slider', nil, parent)
        local editBox = CreateFrame('EditBox', nil, parent)
        assert(slider.Low:GetForbiddenAspects() == 45 and slider.ThumbTexture:GetForbiddenAspects() == 45)
        assert(editBox.Text:GetForbiddenAspects() == 45, 'native default regions inherit before lookup')
        parent:AddForbiddenAspects(e.RemoveSecretAspects)
        assert(bit.band(parent:GetInheritableForbiddenAspects(path.Layout), e.RemoveSecretAspects) == 0)
        assert(bit.band(frame:GetForbiddenAspects(), e.RemoveSecretAspects) == 0, 'non-inheritable additions stay local')

        local foreign = CreateFrame('Frame')
        foreign:AddForbiddenAspects(e.UntrustedLayoutScriptExecution)
        local plain = CreateFrame('Frame')
        local ok, err = pcall(plain.SetPoint, plain, 'TOPLEFT', foreign, 'TOPLEFT')
        assert(not ok and string.find(err, 'Cannot implicitly gain forbidden aspects', 1, true))
        ok, err = pcall(plain.SetParent, plain, foreign)
        assert(not ok and string.find(err, 'Cannot implicitly gain forbidden aspects', 1, true))
        assert(plain:GetNumPoints() == 0 and plain:GetParent() ~= foreign)
    "#).expect("creation inherits, later foreign relationships remain denied");
}

#[test]
fn xml_aspects_use_active_enum_values_and_merge_inherited_state() {
    let env = WowLuaEnv::new().unwrap();
    let result = load_xml_fixture(
        &env,
        r#"<Ui>
      <Frame name="AllAspectTemplate" virtual="true"><ForbiddenAspects>
        <ForbiddenAspect aspect="SetToDefaults"/><ForbiddenAspect aspect="ScriptBindings"/>
        <ForbiddenAspect aspect="UntrustedScriptExecution"/><ForbiddenAspect aspect="UntrustedLayoutScriptExecution"/>
        <ForbiddenAspect aspect="EventRegistrations"/><ForbiddenAspect aspect="AlwaysPropagateInput"/>
        <ForbiddenAspect aspect="ScriptedInput"/><ForbiddenAspect aspect="QueryFocus"/>
        <ForbiddenAspect aspect="ChangeAnimationTarget"/><ForbiddenAspect aspect="RemoveSecretAspects"/>
        <ForbiddenAspect aspect="ChangeParent"/>
      </ForbiddenAspects></Frame>
      <Frame name="AspectParent"><ForbiddenAspects>
        <ForbiddenAspect aspect="UntrustedScriptExecution"/>
        <ForbiddenAspect aspect="UntrustedLayoutScriptExecution"/>
        <ForbiddenAspect aspect="AlwaysPropagateInput"/>
      </ForbiddenAspects><Frames><Frame parentKey="Child"><ForbiddenAspects>
        <ForbiddenAspect aspect="ScriptedInput"/>
      </ForbiddenAspects></Frame></Frames></Frame>
      <Frame name="HierarchyOnly"><ForbiddenAspects>
        <ForbiddenAspect aspect="UntrustedScriptExecution"/>
      </ForbiddenAspects></Frame>
      <Frame name="HierarchyAndLayout"><ForbiddenAspects>
        <ForbiddenAspect aspect="UntrustedLayoutScriptExecution"/>
      </ForbiddenAspects></Frame>
      <Frame name="AllLiteral" inherits="AllAspectTemplate"/>
    </Ui>"#,
    );
    assert!(result.warnings.is_empty(), "{:?}", result.warnings);
    env.exec(r#"
        local path = Enum.ScriptObjectPropagationPath
        assert(AllLiteral:GetForbiddenAspects() == 2047, 'literal XML resolves all eleven active enum names')
        local runtime = CreateFrame('Frame', nil, UIParent, 'AllAspectTemplate')
        assert(runtime:GetForbiddenAspects() == 2047, 'runtime templates apply the same aspects')
        assert(AspectParent.Child:GetForbiddenAspects() == 109, 'own XML aspects retain inherited 44 and implied bit 1')
        assert(HierarchyOnly:GetInheritableForbiddenAspects(path.Hierarchy) == 4)
        assert(HierarchyOnly:GetInheritableForbiddenAspects(path.Layout) == 0)
        assert(HierarchyAndLayout:GetInheritableForbiddenAspects(path.Hierarchy) == 8)
        assert(HierarchyAndLayout:GetInheritableForbiddenAspects(path.Layout) == 8)
    "#).expect("XML and runtime template mapping match native enum values");
}

#[test]
fn xml_unknown_forbidden_aspect_reports_a_load_error() {
    let env = WowLuaEnv::new().unwrap();
    let result = load_xml_fixture(
        &env,
        r#"<Ui><Frame name="InvalidAspect"><ForbiddenAspects>
        <ForbiddenAspect aspect="NotARealForbiddenAspect"/>
    </ForbiddenAspects></Frame></Ui>"#,
    );
    assert!(
        result
            .warnings
            .iter()
            .any(|warning| warning.contains("NotARealForbiddenAspect")),
        "unknown aspects must not silently become zero: {:?}",
        result.warnings
    );
}

#[test]
fn aura_button_icon_and_overlay_border_follow_real_initializer_order() {
    crate::common::with_timeout(90, || {
        crate::common::blizzard_addon_harness::with_blizzard_addon_closure(
            &["Blizzard_AuraContainer"],
            &[],
            |env, _| {
                env.exec(r#"
                    local button = CreateFrame('AuraButton', nil, UIParent, 'CustomAuraButtonTemplate')
                    local icon = button:CreateTexture(nil, 'BACKGROUND')
                    icon:SetAllPoints(button)
                    button:SetIcon(icon)
                    local cooldown = CreateFrame('Cooldown', nil, button, 'CooldownFrameTemplate')
                    cooldown:SetAllPoints(icon)
                    button:SetDurationCooldown(cooldown)
                    local overlay = CreateFrame('Frame', nil, button)
                    overlay:SetAllPoints(button)
                    local count = overlay:CreateFontString(nil, 'OVERLAY', 'NumberFontNormalSmall')
                    button:SetApplicationCount(count)
                    local border = overlay:CreateTexture(nil, 'OVERLAY', nil, 6)
                    border:ClearAllPoints()
                    border:SetPoint('TOPLEFT', icon, 'TOPLEFT', -1, 1)
                    border:SetPoint('BOTTOMRIGHT', icon, 'BOTTOMRIGHT', 1, -1)
                    assert(border:GetNumPoints() == 2)
                    local _, target = border:GetPoint(1)
                    assert(target == icon)
                    local e = Enum.ForbiddenAspect
                    assert(bit.band(button:GetForbiddenAspects(), e.ChangeParent) ~= 0, 'intrinsic XML aspect is applied')
                    assert(bit.band(border:GetForbiddenAspects(), e.UntrustedLayoutScriptExecution) ~= 0)
                    local foreign = CreateFrame('Frame')
                    local ok, err = pcall(foreign.SetPoint, foreign, 'CENTER', icon, 'CENTER')
                    assert(not ok and string.find(err, 'Cannot implicitly gain forbidden aspects', 1, true))
                "#).expect("BetterBlizzFrames icon/cooldown/overlay/border initialization keeps valid ownership");
            },
        );
    });
}

#[test]
fn event_registration_aspect_rejects_all_mutations_without_caller_exception() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local operations = {
            {'RegisterEvent', 'PLAYER_LOGIN'},
            {'RegisterUnitEvent', 'UNIT_HEALTH', 'player'},
            {'RegisterAllEvents'},
            {'RegisterEventCallback', 'MINIMAP_PING', function() end},
            {'RegisterUnitEventCallback', 'UNIT_HEALTH', function() end, 'player'},
            {'UnregisterEvent', 'PLAYER_LOGIN'},
            {'UnregisterAllEvents'},
        }
        local accepted = {}
        for _, taint in ipairs({false, 'EventRegistrationProbe'}) do
            for _, operation in ipairs(operations) do
                local frame = CreateFrame('Frame')
                frame:RegisterEvent('PLAYER_LOGIN')
                frame:AddForbiddenAspects(Enum.ForbiddenAspect.EventRegistrations)
                local function invoke()
                    return frame[operation[1]](frame, unpack(operation, 2))
                end
                if taint then debug.setobjecttaint(invoke, taint) end
                local ok, err = pcall(invoke)
                if ok then
                    accepted[#accepted + 1] = operation[1] .. ':' .. tostring(taint)
                else
                    assert(type(err) == 'string', 'rejection must report an error')
                end
            end
        end
        assert(#accepted == 0, 'aspect accepted mutations: ' .. table.concat(accepted, ', '))
    "#).expect("the modeled aspect rejects all seven mutations for both caller contexts");
}

#[test]
fn event_registration_aspect_preserves_existing_listener_delivery() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        EventAspectListeners = {individual=0, unit=0, all=0}
        local state = EventAspectListeners
        state.individualFrame = CreateFrame('Frame')
        state.unitFrame = CreateFrame('Frame')
        state.allFrame = CreateFrame('Frame')
        state.individualFrame:SetScript('OnEvent', function(_, event)
            assert(event == 'PLAYER_LOGIN')
            state.individual = state.individual + 1
        end)
        state.unitFrame:SetScript('OnEvent', function(_, event, unit)
            assert(event == 'UNIT_HEALTH' and unit == 'player')
            state.unit = state.unit + 1
        end)
        state.allFrame:SetScript('OnEvent', function(_, event)
            if event == 'PLAYER_LOGIN' or event == 'UNIT_HEALTH' then
                state.all = state.all + 1
            end
        end)
        state.individualFrame:RegisterEvent('PLAYER_LOGIN')
        state.unitFrame:RegisterUnitEvent('UNIT_HEALTH', 'player')
        state.allFrame:RegisterAllEvents()
        for _, frame in ipairs({state.individualFrame, state.unitFrame, state.allFrame}) do
            frame:AddForbiddenAspects(Enum.ForbiddenAspect.EventRegistrations)
        end
        state.unregister = pcall(state.individualFrame.UnregisterEvent, state.individualFrame, 'PLAYER_LOGIN')
        state.unitClear = pcall(state.unitFrame.UnregisterAllEvents, state.unitFrame)
        state.allClear = pcall(state.allFrame.UnregisterAllEvents, state.allFrame)
        state.unitReplace = pcall(state.unitFrame.RegisterUnitEvent, state.unitFrame, 'UNIT_HEALTH', 'target')
    "#).expect("prepare individual, unit-filtered, and all-event registrations before restriction");
    env.fire_event("PLAYER_LOGIN").unwrap();
    env.fire_event_with_args("UNIT_HEALTH", &[env.lua_string("target")])
        .unwrap();
    env.fire_event_with_args("UNIT_HEALTH", &[env.lua_string("player")])
        .unwrap();
    env.exec(r#"
        local state = EventAspectListeners
        assert(state.individual == 1, 'individual registration was removed')
        assert(state.unit == 1, 'unit registration/filter was changed')
        assert(state.all == 3, 'all-event registration was removed')
        assert(not state.unregister and not state.unitClear and not state.allClear and not state.unitReplace)
        assert(state.individualFrame:IsEventRegistered('PLAYER_LOGIN'))
        local registered, unit = state.unitFrame:IsEventRegistered('UNIT_HEALTH')
        assert(registered and unit == 'player', 'query must retain the original unit filter')
    "#).expect("rejected mutations leave listener queries and actual delivery intact");
}

#[test]
fn event_registration_aspect_preserves_callbacks_when_replacement_is_rejected() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        EventAspectCallbacks = {ordinary=0, unit=0, replacement=0}
        local state = EventAspectCallbacks
        state.ordinaryFrame = CreateFrame('Frame')
        state.unitFrame = CreateFrame('Frame')
        state.ordinaryFrame:RegisterEventCallback('MINIMAP_PING', function(owner)
            assert(owner == state.ordinaryFrame)
            state.ordinary = state.ordinary + 1
        end)
        state.unitFrame:RegisterUnitEventCallback('UNIT_HEALTH', function(owner, unit)
            assert(owner == state.unitFrame and unit == 'player')
            state.unit = state.unit + 1
        end, 'player')
        state.ordinaryFrame:AddForbiddenAspects(Enum.ForbiddenAspect.EventRegistrations)
        state.unitFrame:AddForbiddenAspects(Enum.ForbiddenAspect.EventRegistrations)
        local function replacement() state.replacement = state.replacement + 1 end
        state.clearOrdinary = pcall(state.ordinaryFrame.UnregisterAllEvents, state.ordinaryFrame)
        state.clearUnit = pcall(state.unitFrame.UnregisterEvent, state.unitFrame, 'UNIT_HEALTH')
        state.replaceOrdinary = pcall(state.ordinaryFrame.RegisterEventCallback,
            state.ordinaryFrame, 'MINIMAP_PING', replacement)
        state.replaceUnit = pcall(state.unitFrame.RegisterUnitEventCallback,
            state.unitFrame, 'UNIT_HEALTH', replacement, 'target')
    "#).expect("prepare callback listeners and attempt replacement after adding the aspect");
    env.exec(r#"
        FireEvent('MINIMAP_PING')
        FireEvent('UNIT_HEALTH', 'target')
        FireEvent('UNIT_HEALTH', 'player')
        local state = EventAspectCallbacks
        assert(state.ordinary == 1, 'original event callback did not run')
        assert(state.unit == 1, 'original unit callback/filter did not survive')
        assert(state.replacement == 0, 'rejected replacement callback became active')
        assert(not state.clearOrdinary and not state.clearUnit)
        assert(not state.replaceOrdinary and not state.replaceUnit)
        assert(state.ordinaryFrame:IsEventRegistered('MINIMAP_PING'))
        assert(state.unitFrame:IsEventRegistered('UNIT_HEALTH'))
    "#).expect("callback replacement and unregistration are atomic on rejected calls");
}

#[test]
fn event_registration_aspect_leaves_zero_mask_mutations_and_delivery_unchanged() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        EventAspectPlain = {individual=0, unit=0, all=0}
        local state = EventAspectPlain
        state.frames = {}
        for _, name in ipairs({'individual', 'unit', 'all'}) do
            local frame = CreateFrame('Frame')
            assert(frame:GetForbiddenAspects() == 0)
            state.frames[name] = frame
        end
        state.frames.individual:SetScript('OnEvent', function() state.individual = state.individual + 1 end)
        state.frames.unit:SetScript('OnEvent', function(_, _, unit)
            assert(unit == 'player')
            state.unit = state.unit + 1
        end)
        state.frames.all:SetScript('OnEvent', function() state.all = state.all + 1 end)
        assert(state.frames.individual:RegisterEvent('PLAYER_LOGIN'))
        assert(state.frames.unit:RegisterUnitEvent('UNIT_HEALTH', 'player'))
        state.frames.all:RegisterAllEvents()
    "#).expect("ordinary registration forms remain usable without the aspect");
    env.fire_event("PLAYER_LOGIN").unwrap();
    env.fire_event_with_args("UNIT_HEALTH", &[env.lua_string("target")])
        .unwrap();
    env.fire_event_with_args("UNIT_HEALTH", &[env.lua_string("player")])
        .unwrap();
    env.exec(r#"
        local state = EventAspectPlain
        assert(state.individual == 1 and state.unit == 1 and state.all == 3)
        assert(state.frames.individual:UnregisterEvent('PLAYER_LOGIN'))
        for name, frame in pairs(state.frames) do
            if name ~= 'individual' then frame:UnregisterAllEvents() end
        end
    "#).expect("zero-mask listeners receive events and both unregistration forms remain usable");
    env.fire_event("PLAYER_LOGIN").unwrap();
    env.fire_event_with_args("UNIT_HEALTH", &[env.lua_string("target")])
        .unwrap();
    env.fire_event_with_args("UNIT_HEALTH", &[env.lua_string("player")])
        .unwrap();
    env.exec(r#"
        local state = EventAspectPlain
        assert(state.individual == 1 and state.unit == 1 and state.all == 3)
    "#).expect("unregistered zero-mask frames receive no further delivery");
}

#[test]
fn event_registration_aspect_leaves_zero_mask_callback_delivery_unchanged() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local ordinary, unit = CreateFrame('Frame'), CreateFrame('Frame')
        assert(ordinary:GetForbiddenAspects() == 0 and unit:GetForbiddenAspects() == 0)
        local ordinaryCalls, unitCalls = 0, 0
        ordinary:RegisterEventCallback('MINIMAP_PING', function(owner)
            assert(owner == ordinary)
            ordinaryCalls = ordinaryCalls + 1
        end)
        unit:RegisterUnitEventCallback('UNIT_HEALTH', function(owner, token)
            assert(owner == unit and token == 'player')
            unitCalls = unitCalls + 1
        end, 'player')
        FireEvent('MINIMAP_PING')
        FireEvent('UNIT_HEALTH', 'target')
        FireEvent('UNIT_HEALTH', 'player')
        assert(ordinaryCalls == 1 and unitCalls == 1)
        ordinary:UnregisterAllEvents()
        unit:UnregisterAllEvents()
        assert(not ordinary:IsEventRegistered('MINIMAP_PING'))
        assert(not unit:IsEventRegistered('UNIT_HEALTH'))
        FireEvent('MINIMAP_PING')
        FireEvent('UNIT_HEALTH', 'player')
        assert(ordinaryCalls == 1 and unitCalls == 1)
    "#).expect("zero-mask callback listeners retain their public FireEvent delivery boundary");
}

#[test]
fn scripted_focus_aspect_rejects_click_before_checkbutton_or_handler_changes() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local restricted = CreateFrame('CheckButton')
        restricted:SetChecked(false)
        local restrictedCalls = 0
        restricted:SetScript('OnClick', function() restrictedCalls = restrictedCalls + 1 end)
        restricted:AddForbiddenAspects(Enum.ForbiddenAspect.ScriptedInput)
        local ok, err = pcall(restricted.Click, restricted)
        assert(not ok and type(err) == 'string', 'ScriptedInput must reject Click')
        assert(not restricted:GetChecked(), 'rejected click must not toggle checked state')
        assert(restrictedCalls == 0, 'rejected click must not invoke OnClick')

        local ordinary = CreateFrame('CheckButton')
        ordinary:SetChecked(false)
        local ordinaryCalls = 0
        ordinary:SetScript('OnClick', function() ordinaryCalls = ordinaryCalls + 1 end)
        ordinary:Click()
        assert(ordinary:GetChecked() and ordinaryCalls == 1)
    "#).expect("scripted click rejection is atomic and an unrestricted click still works");
}

#[test]
fn scripted_focus_aspect_preserves_focus_and_cursor_while_engine_typing_continues() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        ScriptedFocusBox = CreateFrame('EditBox')
        ScriptedOtherBox = CreateFrame('EditBox')
        ScriptedFocusBox:SetText('ab')
        ScriptedFocusBox:SetCursorPosition(2)
        ScriptedFocusBox:SetFocus()
        local aspect = Enum.ForbiddenAspect.ScriptedInput
        ScriptedFocusBox:AddForbiddenAspects(aspect)
        ScriptedOtherBox:AddForbiddenAspects(aspect)
        local operations = {
            {'SetFocus', ScriptedOtherBox},
            {'ClearFocus', ScriptedFocusBox},
            {'SetCursorPosition', ScriptedFocusBox, 0},
        }
        local accepted = {}
        for _, operation in ipairs(operations) do
            local receiver = operation[2]
            local ok, err = pcall(receiver[operation[1]], receiver, unpack(operation, 3))
            if ok then
                accepted[#accepted + 1] = operation[1]
            else
                assert(type(err) == 'string')
            end
        end
        assert(#accepted == 0, 'ScriptedInput accepted: ' .. table.concat(accepted, ', '))
        assert(ScriptedFocusBox:HasFocus() and not ScriptedOtherBox:HasFocus())
        assert(ScriptedFocusBox:GetCursorPosition() == 2)
        assert(ScriptedFocusBox:GetText() == 'ab')
    "#).expect("focus and cursor mutations are rejected without changing ownership or text");
    env.send_key_press("X", Some("x")).unwrap();
    env.exec(r#"
        assert(ScriptedFocusBox:GetText() == 'abx')
        assert(ScriptedFocusBox:GetCursorPosition() == 3)
        assert(ScriptedFocusBox:HasFocus() and not ScriptedOtherBox:HasFocus())
        assert(ScriptedOtherBox:GetText() == '')
    "#).expect("engine keyboard input bypasses the Lua scripted-input mutations");
}

#[test]
fn scripted_focus_aspect_query_rejection_preserves_real_keyboard_focus() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        QueryFocusBox = CreateFrame('EditBox')
        QueryIdleBox = CreateFrame('EditBox')
        QueryFocusBox:SetText('ab')
        QueryFocusBox:SetCursorPosition(1)
        QueryFocusBox:SetFocus()
        assert(QueryFocusBox:HasFocus() and not QueryIdleBox:HasFocus())
        for _, box in ipairs({QueryFocusBox, QueryIdleBox}) do
            box:AddForbiddenAspects(Enum.ForbiddenAspect.QueryFocus)
            local ok, err = pcall(box.HasFocus, box)
            assert(not ok and type(err) == 'string', 'QueryFocus must reject HasFocus')
        end
        assert(QueryFocusBox:GetCursorPosition() == 1)
        assert(QueryFocusBox:GetText() == 'ab')
    "#).expect("focus queries reject both focused and unfocused restricted receivers");
    env.send_key_press("X", Some("x")).unwrap();
    env.exec(r#"
        assert(QueryFocusBox:GetText() == 'axb')
        assert(QueryFocusBox:GetCursorPosition() == 2)
        assert(QueryIdleBox:GetText() == '')
        assert(not pcall(QueryFocusBox.HasFocus, QueryFocusBox))
    "#).expect("query rejection does not clear focus or block engine typing");
}

#[test]
fn scripted_focus_aspect_unrestricted_focus_mutations_and_queries_still_work() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        PlainFocusFirst = CreateFrame('EditBox')
        PlainFocusSecond = CreateFrame('EditBox')
        PlainFocusFirst:SetText('ab')
        PlainFocusFirst:SetCursorPosition(1)
        PlainFocusFirst:SetFocus()
        assert(PlainFocusFirst:HasFocus() and not PlainFocusSecond:HasFocus())
        assert(PlainFocusFirst:GetCursorPosition() == 1)
        PlainFocusSecond:SetFocus()
        assert(not PlainFocusFirst:HasFocus() and PlainFocusSecond:HasFocus())
        PlainFocusSecond:ClearFocus()
        assert(not PlainFocusFirst:HasFocus() and not PlainFocusSecond:HasFocus())
        PlainFocusFirst:SetFocus()
    "#).expect("unrestricted focus ownership, clearing, cursor mutation, and queries work");
    env.send_key_press("X", Some("x")).unwrap();
    env.exec(r#"
        assert(PlainFocusFirst:GetText() == 'axb')
        assert(PlainFocusFirst:GetCursorPosition() == 2)
        assert(PlainFocusFirst:HasFocus() and not PlainFocusSecond:HasFocus())
    "#).expect("unrestricted engine typing retains the current focus owner");
}
