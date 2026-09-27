use wow_ui_sim::lua_api::WowLuaEnv;

const SCROLL_BINDINGS_XML: &str = r#"
<Ui>
    <ScrollFrame name="ScrollBindingPre" virtual="true" intrinsic="true">
        <Scripts>
            <OnHorizontalScroll intrinsicOrder="precall">ScrollBindingProbe(self, 'h', 'pre', ...)</OnHorizontalScroll>
            <OnVerticalScroll intrinsicOrder="precall">ScrollBindingProbe(self, 'v', 'pre', ...)</OnVerticalScroll>
            <OnScrollRangeChanged intrinsicOrder="precall">ScrollBindingProbe(self, 'range', 'pre', ...)</OnScrollRangeChanged>
        </Scripts>
    </ScrollFrame>
    <ScrollFrame name="ScrollBindingPost" virtual="true">
        <Scripts>
            <OnHorizontalScroll intrinsicOrder="postcall">ScrollBindingProbe(self, 'h', 'post', ...)</OnHorizontalScroll>
            <OnVerticalScroll intrinsicOrder="postcall">ScrollBindingProbe(self, 'v', 'post', ...)</OnVerticalScroll>
            <OnScrollRangeChanged intrinsicOrder="postcall">ScrollBindingProbe(self, 'range', 'post', ...)</OnScrollRangeChanged>
        </Scripts>
    </ScrollFrame>
</Ui>
"#;

fn synthetic_bindings_env(xml: &str) -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    let root = tempfile::tempdir().unwrap();
    let toc = root.path().join("ScrollBindings.toc");
    std::fs::write(&toc, "## Title: Scroll bindings probe\nBindings.xml\n").unwrap();
    std::fs::write(root.path().join("Bindings.xml"), xml).unwrap();
    let loaded = wow_ui_sim::loader::load_addon(&env.loader_env(), &toc).unwrap();
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    env
}

const SCROLL_SETUP: &str = r#"
    local sf = CreateFrame('ScrollFrame', 'ScrollBindingTarget', UIParent,
        'ScrollBindingPre,ScrollBindingPost')
    sf:SetSize(100, 100)
    local child = CreateFrame('Frame', nil, sf)
    child:SetSize(100, 100)
    child:SetPoint('TOPLEFT', sf, 'TOPLEFT')
    sf:SetScrollChild(child)
    local content = CreateFrame('Frame', nil, child)
    content:SetSize(180, 220)
    content:SetPoint('TOPLEFT', child, 'TOPLEFT')
"#;

#[test]
fn xml_scroll_bindings_order_payload_commit_and_unchanged_suppression() {
    let env = synthetic_bindings_env(SCROLL_BINDINGS_XML);
    env.exec(&format!(
        r#"
        ScrollBindingCalls = {{}}
        function ScrollBindingProbe(self, event, phase, ...)
            assert(self == ScrollBindingTarget)
            local x, y = self:GetHorizontalScrollRange(), self:GetVerticalScrollRange()
            local offset_h, offset_v = self:GetHorizontalScroll(), self:GetVerticalScroll()
            local payload
            if event == 'range' then
                assert(select('#', ...) == 2)
                local new_x, new_y = ...
                assert(new_x == x and new_y == y)
                payload = new_x .. ':' .. new_y
            else
                assert(select('#', ...) == 1)
                local offset = ...
                assert(offset == (event == 'h' and offset_h or offset_v))
                payload = tostring(offset)
            end
            ScrollBindingCalls[#ScrollBindingCalls + 1] = event .. ':' .. phase .. ':' .. payload
        end
        {SCROLL_SETUP}
        local sf = ScrollBindingTarget
        for _, event in ipairs({{'OnHorizontalScroll', 'OnVerticalScroll', 'OnScrollRangeChanged'}}) do
            assert(type(sf:GetScript(event, 0)) == 'function', event .. ' precall missing')
            assert(type(sf:GetScript(event, 2)) == 'function', event .. ' postcall missing')
            sf:SetScript(event, function(self, ...) ScrollBindingProbe(self,
                event == 'OnScrollRangeChanged' and 'range' or
                (event == 'OnHorizontalScroll' and 'h' or 'v'), 'normal', ...) end)
            sf:HookScript(event, function(self, ...) ScrollBindingProbe(self,
                event == 'OnScrollRangeChanged' and 'range' or
                (event == 'OnHorizontalScroll' and 'h' or 'v'), 'hook', ...) end)
        end
        sf:UpdateScrollChildRect()
        assert(sf:GetHorizontalScrollRange() == 80 and sf:GetVerticalScrollRange() == 120)
        sf:UpdateScrollChildRect()
        sf:SetHorizontalScroll(35)
        sf:SetHorizontalScroll(35)
        sf:SetVerticalScroll(-7)
        sf:SetVerticalScroll(-7)
        local expected = {{}}
        for _, pair in ipairs({{'range:80:120', 'h:35', 'v:-7'}}) do
            for _, phase in ipairs({{'pre', 'normal', 'hook', 'post'}}) do
                expected[#expected + 1] = pair:gsub(':', ':' .. phase .. ':', 1)
            end
        end
        assert(table.concat(ScrollBindingCalls, ',') == table.concat(expected, ','),
            'bindings: ' .. table.concat(ScrollBindingCalls, ','))
    "#
    ))
    .unwrap();
}

#[test]
fn xml_scroll_precall_error_reports_and_continues_other_bindings() {
    let env = synthetic_bindings_env(SCROLL_BINDINGS_XML);
    env.exec(&format!(
        r#"
        local calls, errors = {{}}, {{}}
        seterrorhandler(function(message) errors[#errors + 1] = tostring(message) end)
        function ScrollBindingProbe(self, event, phase, ...)
            assert(self == ScrollBindingTarget and event == 'h' and select('#', ...) == 1)
            assert((...) == self:GetHorizontalScroll())
            calls[#calls + 1] = phase
            if phase == 'pre' then error('scroll precall sentinel') end
        end
        {SCROLL_SETUP}
        local sf = ScrollBindingTarget
        assert(type(sf:GetScript('OnHorizontalScroll', 0)) == 'function')
        assert(type(sf:GetScript('OnHorizontalScroll', 2)) == 'function')
        sf:SetScript('OnHorizontalScroll', function(self, ...) ScrollBindingProbe(self, 'h', 'normal', ...) end)
        sf:HookScript('OnHorizontalScroll', function(self, ...) ScrollBindingProbe(self, 'h', 'hook', ...) end)
        sf:SetHorizontalScroll(19)
        assert(table.concat(calls, ',') == 'pre,normal,hook,post', table.concat(calls, ','))
        assert(#errors == 1 and string.find(errors[1], 'scroll precall sentinel', 1, true),
            'precall failure must be reported once')
    "#
    ))
    .unwrap();
}

#[test]
fn ordinary_xml_scrollframe_registers_and_dispatches_all_scroll_scripts() {
    let env = synthetic_bindings_env(
        r#"
<Ui>
    <ScrollFrame name="OrdinaryXmlScrollBindingTarget" parent="UIParent">
        <Scripts>
            <OnHorizontalScroll>OrdinaryScrollProbe(self, 'h', ...)</OnHorizontalScroll>
            <OnVerticalScroll>OrdinaryScrollProbe(self, 'v', ...)</OnVerticalScroll>
            <OnScrollRangeChanged>OrdinaryScrollProbe(self, 'range', ...)</OnScrollRangeChanged>
        </Scripts>
    </ScrollFrame>
</Ui>
"#,
    );
    env.exec(
        r#"
        local sf = OrdinaryXmlScrollBindingTarget
        local calls = {}
        function OrdinaryScrollProbe(self, event, ...)
            assert(self == sf)
            local x, y = self:GetHorizontalScrollRange(), self:GetVerticalScrollRange()
            if event == 'range' then
                assert(select('#', ...) == 2)
                local new_x, new_y = ...
                assert(new_x == 80 and new_y == 120 and x == new_x and y == new_y)
                calls[#calls + 1] = 'range:' .. new_x .. ':' .. new_y
            elseif event == 'h' then
                assert(select('#', ...) == 1 and (...) == 35 and self:GetHorizontalScroll() == 35)
                assert(x == 80 and y == 120)
                calls[#calls + 1] = 'h:35'
            else
                assert(event == 'v' and select('#', ...) == 1)
                assert((...) == -7 and self:GetVerticalScroll() == -7)
                assert(x == 80 and y == 120)
                calls[#calls + 1] = 'v:-7'
            end
        end
        for _, event in ipairs({'OnHorizontalScroll', 'OnVerticalScroll', 'OnScrollRangeChanged'}) do
            assert(type(sf:GetScript(event)) == 'function', event .. ' XML handler missing')
        end
        sf:SetSize(100, 100)
        local child = CreateFrame('Frame', nil, sf)
        child:SetSize(100, 100)
        child:SetPoint('TOPLEFT', sf, 'TOPLEFT')
        sf:SetScrollChild(child)
        local content = CreateFrame('Frame', nil, child)
        content:SetSize(180, 220)
        content:SetPoint('TOPLEFT', child, 'TOPLEFT')
        sf:UpdateScrollChildRect()
        assert(sf:GetHorizontalScrollRange() == 80 and sf:GetVerticalScrollRange() == 120)
        sf:SetHorizontalScroll(35)
        sf:SetVerticalScroll(-7)
        assert(table.concat(calls, ',') == 'range:80:120,h:35,v:-7', table.concat(calls, ','))
    "#,
    )
    .unwrap();
    let state = env.state();
    let state = state.borrow();
    assert!(state.lua_errors.is_empty(), "{:?}", state.lua_errors);
}

#[test]
fn bare_scrollframe_property_named_intrinsic_is_not_a_registered_binding() {
    WowLuaEnv::new()
        .unwrap()
        .exec(
            r#"
        local sf = CreateFrame('ScrollFrame', nil, UIParent)
        local calls = {}
        sf.OnHorizontalScroll_Intrinsic = function() calls[#calls + 1] = 'h' end
        sf.OnVerticalScroll_Intrinsic = function() calls[#calls + 1] = 'v' end
        sf.OnScrollRangeChanged_Intrinsic = function() calls[#calls + 1] = 'range' end
        sf:SetScript('OnHorizontalScroll', function(self, offset)
            assert(self == sf and offset == self:GetHorizontalScroll())
            calls[#calls + 1] = 'normal'
        end)
        sf:HookScript('OnHorizontalScroll', function(self, offset)
            assert(self == sf and offset == self:GetHorizontalScroll())
            calls[#calls + 1] = 'hook'
        end)
        assert(sf:GetScript('OnHorizontalScroll', 0) == nil)
        assert(sf:GetScript('OnVerticalScroll', 0) == nil)
        assert(sf:GetScript('OnScrollRangeChanged', 0) == nil)
        sf:SetSize(100, 100)
        local child = CreateFrame('Frame', nil, sf)
        child:SetSize(180, 220)
        child:SetPoint('TOPLEFT', sf, 'TOPLEFT')
        sf:SetScrollChild(child)
        sf:UpdateScrollChildRect()
        sf:SetHorizontalScroll(8)
        sf:SetVerticalScroll(9)
        assert(table.concat(calls, ',') == 'normal,hook',
            'normal script and hook run, but unregistered intrinsic property does not: ' .. table.concat(calls, ','))
    "#,
        )
        .unwrap();
}

#[test]
fn cached_event_scrollframe_callbacks_precede_normal_scripts_once() {
    let env = super::env_with_shared_xml();
    env.exec(
        r#"
        local sf = CreateFrame('ScrollFrame', 'RealEventScrollBindingTarget', UIParent, 'EventScrollFrame')
        sf:SetSize(100, 100)
        local child = CreateFrame('Frame', nil, sf)
        child:SetSize(100, 100)
        child:SetPoint('TOPLEFT', sf, 'TOPLEFT')
        sf:SetScrollChild(child)
        local content = CreateFrame('Frame', nil, child)
        content:SetSize(180, 220)
        content:SetPoint('TOPLEFT', child, 'TOPLEFT')
        local calls = {}
        for _, event in ipairs({'OnHorizontalScroll', 'OnVerticalScroll', 'OnScrollRangeChanged'}) do
            assert(type(sf:GetScript(event, 0)) == 'function', event .. ' precall missing')
            sf:RegisterCallback(event, function(owner, ...)
                assert(owner == sf)
                local x, y = sf:GetHorizontalScrollRange(), sf:GetVerticalScrollRange()
                if event == 'OnScrollRangeChanged' then
                    assert(select('#', ...) == 2)
                    local new_x, new_y = ...
                    assert(new_x == 80 and new_y == 120 and x == new_x and y == new_y)
                elseif event == 'OnHorizontalScroll' then
                    assert(select('#', ...) == 1 and (...) == 35 and sf:GetHorizontalScroll() == 35)
                else
                    assert(select('#', ...) == 1 and (...) == -7 and sf:GetVerticalScroll() == -7)
                end
                calls[#calls + 1] = event .. ':callback'
            end, sf)
            sf:SetScript(event, function(self, ...)
                assert(self == sf)
                calls[#calls + 1] = event .. ':normal'
            end)
        end
        sf:UpdateScrollChildRect()
        assert(sf:GetHorizontalScrollRange() == 80 and sf:GetVerticalScrollRange() == 120)
        sf:UpdateScrollChildRect()
        sf:SetHorizontalScroll(35)
        sf:SetHorizontalScroll(35)
        sf:SetVerticalScroll(-7)
        sf:SetVerticalScroll(-7)
        assert(table.concat(calls, ',') == table.concat({
            'OnScrollRangeChanged:callback', 'OnScrollRangeChanged:normal',
            'OnHorizontalScroll:callback', 'OnHorizontalScroll:normal',
            'OnVerticalScroll:callback', 'OnVerticalScroll:normal',
        }, ','), table.concat(calls, ','))
    "#,
    )
    .unwrap();
    let state = env.state();
    let state = state.borrow();
    assert!(state.lua_errors.is_empty(), "{:?}", state.lua_errors);
}
