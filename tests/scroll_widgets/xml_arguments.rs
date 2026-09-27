use wow_ui_sim::lua_api::WowLuaEnv;

const XML: &str = r#"
<Ui>
    <ScrollFrame name="XmlScrollArgsOrdinary" parent="UIParent">
        <Scripts>
            <OnHorizontalScroll>ArgCalls[#ArgCalls + 1] = 'h:' .. tostring(offset) .. ':' .. select('#', ...)</OnHorizontalScroll>
            <OnVerticalScroll>ArgCalls[#ArgCalls + 1] = 'v:' .. tostring(offset) .. ':' .. select('#', ...)</OnVerticalScroll>
            <OnScrollRangeChanged>ArgCalls[#ArgCalls + 1] = 'range:' .. tostring(xrange) .. ':' .. tostring(yrange) .. ':' .. select('#', ...)</OnScrollRangeChanged>
            <OnMouseWheel>ArgCalls[#ArgCalls + 1] = 'wheel:' .. tostring(delta) .. ':' .. select('#', ...)</OnMouseWheel>
        </Scripts>
    </ScrollFrame>
    <ScrollFrame name="XmlScrollArgsTemplate" virtual="true">
        <Scripts>
            <OnHorizontalScroll>ArgCalls[#ArgCalls + 1] = 'h:' .. tostring(offset) .. ':' .. select('#', ...)</OnHorizontalScroll>
            <OnVerticalScroll>ArgCalls[#ArgCalls + 1] = 'v:' .. tostring(offset) .. ':' .. select('#', ...)</OnVerticalScroll>
            <OnScrollRangeChanged>ArgCalls[#ArgCalls + 1] = 'range:' .. tostring(xrange) .. ':' .. tostring(yrange) .. ':' .. select('#', ...)</OnScrollRangeChanged>
            <OnMouseWheel>ArgCalls[#ArgCalls + 1] = 'wheel:' .. tostring(delta) .. ':' .. select('#', ...)</OnMouseWheel>
        </Scripts>
    </ScrollFrame>
</Ui>
"#;

fn xml_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    let root = tempfile::tempdir().unwrap();
    let toc = root.path().join("ScrollArgs.toc");
    std::fs::write(&toc, "## Title: Scroll arguments\nArgs.xml\n").unwrap();
    std::fs::write(root.path().join("Args.xml"), XML).unwrap();
    let loaded = wow_ui_sim::loader::load_addon(&env.loader_env(), &toc).unwrap();
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    env
}

fn assert_no_lua_errors(env: &WowLuaEnv) {
    let state = env.state();
    let state = state.borrow();
    assert!(state.lua_errors.is_empty(), "{:?}", state.lua_errors);
}

fn assert_inline_arguments(env: &WowLuaEnv, frame: &str) {
    let result = env.exec(&format!(
        r#"
        ArgCalls = {{}}
        offset, xrange, yrange, delta = 901, 902, 903, 904
        local sf = {frame}
        sf:SetSize(100, 100)
        local child = CreateFrame('Frame', nil, sf)
        child:SetSize(100, 100)
        child:SetPoint('TOPLEFT', sf, 'TOPLEFT')
        sf:SetScrollChild(child)
        local content = CreateFrame('Frame', nil, child)
        content:SetSize(180, 220)
        content:SetPoint('TOPLEFT', child, 'TOPLEFT')
        sf:UpdateScrollChildRect()
        sf:SetHorizontalScroll(35)
        sf:SetVerticalScroll(37)
        -- Direct script invocation proves XML parameter binding, not physical wheel input.
        sf:GetScript('OnMouseWheel')(sf, -1)
        assert(table.concat(ArgCalls, ',') ==
            'range:80:120:2,h:35:1,v:37:1,wheel:-1:1', table.concat(ArgCalls, ','))
        assert(offset == 901 and xrange == 902 and yrange == 903 and delta == 904)
    "#
    ));
    assert_no_lua_errors(env);
    result.unwrap();
}

#[test]
fn ordinary_xml_inline_scroll_arguments_shadow_globals_and_preserve_varargs() {
    let env = xml_env();
    assert_inline_arguments(&env, "XmlScrollArgsOrdinary");
}

#[test]
fn runtime_template_inline_scroll_arguments_shadow_globals_and_preserve_varargs() {
    let env = xml_env();
    assert_inline_arguments(
        &env,
        "CreateFrame('ScrollFrame', 'XmlScrollArgsRuntime', UIParent, 'XmlScrollArgsTemplate')",
    );
}

#[test]
fn cached_faux_scrollframe_inline_offset_and_wheel_delta_sync_scrollbar() {
    let env = super::env_with_shared_xml();
    let result = env.exec(
        r#"
        offset, delta = 901, 904
        local sf = CreateFrame('ScrollFrame', 'XmlFauxScrollArgs', UIParent,
            'FauxScrollFrameTemplateLight')
        sf:SetSize(100, 100)
        sf:UpdateScrollChildRect()
        assert(sf:GetVerticalScrollRange() == 234, sf:GetVerticalScrollRange())
        local bar = _G[sf:GetName() .. 'ScrollBar']
        assert(bar and type(sf:GetScript('OnVerticalScroll')) == 'function')
        assert(type(sf:GetScript('OnMouseWheel')) == 'function')
        bar:SetMinMaxValues(0, 234)
        bar.scrollStep = 11
        sf:SetVerticalScroll(37)
        assert(sf:GetVerticalScroll() == 37 and bar:GetValue() == 37,
            'inline offset must synchronize frame and bar at 37')
        -- Direct invocation of the actual XML handler, not physical wheel input.
        sf:GetScript('OnMouseWheel')(sf, -1)
        assert(sf:GetVerticalScroll() == 48 and bar:GetValue() == 48,
            'inline delta must advance frame and bar by scrollStep')
        assert(offset == 901 and delta == 904)
    "#,
    );
    assert_no_lua_errors(&env);
    result.unwrap();
}
