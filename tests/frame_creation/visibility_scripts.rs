use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn test_cross_frame_show_recursion_does_not_overflow() {
    let env = WowLuaEnv::new().unwrap();
    env.eval::<()>(
        r#"
        local a = CreateFrame("Frame", "RecurseA", UIParent)
        local b = CreateFrame("Frame", "RecurseB", UIParent)
        a:Hide()
        b:Hide()
        a:SetScript("OnShow", function() b:Show() end)
        b:SetScript("OnShow", function() a:Show() end)
        a:Show()
        "#,
    )
    .unwrap();
}

#[test]
fn test_onshow_hide_preserves_handler_selected_hidden_state() {
    let env = WowLuaEnv::new().unwrap();
    let (log, is_shown, is_visible): (String, bool, bool) = env
        .eval(
            r#"
            local log = {}
            local f = CreateFrame("Frame", "OneWayVisibilityFrame", UIParent)
            f:Hide()
            f:SetScript("OnShow", function(self)
                table.insert(log, self:IsVisible() and "show-visible" or "show-hidden")
                self:Hide()
                table.insert(log, self:IsVisible() and "after-visible" or "after-hidden")
            end)
            f:SetScript("OnHide", function(self)
                table.insert(log, self:IsVisible() and "hide-visible" or "hide-hidden")
            end)

            f:Show()
            return table.concat(log, ","), f:IsShown(), f:IsVisible()
        "#,
        )
        .unwrap();

    assert_eq!(log, "show-visible,after-hidden,hide-hidden");
    assert!(!is_shown, "OnShow-selected hidden state must persist");
    assert!(!is_visible, "hidden frame must not remain visible");
}

#[test]
fn test_cross_frame_show_recursion_stops_at_dispatch_depth_limit() {
    let env = WowLuaEnv::new().unwrap();
    let (fired, last_shown): (i32, bool) = env
        .eval(
            r#"
            local frames = {}
            local fired = 0
            for i = 1, 41 do
                frames[i] = CreateFrame("Frame", "DepthFrame" .. i, UIParent)
                frames[i]:Hide()
                frames[i]:SetScript("OnShow", function()
                    fired = fired + 1
                    if frames[i + 1] then
                        frames[i + 1]:Show()
                    end
                end)
            end

            frames[1]:Show()
            return fired, frames[41]:IsShown()
        "#,
        )
        .unwrap();

    assert_eq!(fired, 40, "cross-frame dispatch must stop at depth 40");
    assert!(
        last_shown,
        "the depth-limited frame still receives its requested state"
    );
}

#[test]
fn test_onshow_onhide_mutual_recursion_terminates_with_reference_order() {
    let env = WowLuaEnv::new().unwrap();
    let log: String = env
        .eval(
            r#"
            local log = {}
            local f = CreateFrame("Frame", "MutualVisibilityFrame", UIParent)
            f:SetScript("OnShow", function(self)
                table.insert(log, self:IsVisible() and "A" or "a")
                self:Hide()
                table.insert(log, self:IsVisible() and "B" or "b")
            end)
            f:SetScript("OnHide", function(self)
                table.insert(log, self:IsVisible() and "C" or "c")
                self:Show()
                table.insert(log, self:IsVisible() and "D" or "d")
            end)

            f:Hide()
            return table.concat(log)
        "#,
        )
        .unwrap();

    assert_eq!(
        log,
        "cDAb".repeat(6),
        "OnShow/OnHide mutual recursion should unwind iteratively with wowless/master ordering"
    );
}

#[test]
fn test_child_onshow_fires_when_parent_becomes_visible() {
    let env = WowLuaEnv::new().unwrap();
    let fired: i32 = env
        .eval(
            r#"
            local parent = CreateFrame("Frame", "ChildOnShowParent", UIParent)
            local child = CreateFrame("Frame", "ChildOnShowChild", parent)
            parent:Hide()
            child:Hide()

            local fired = 0
            child:SetScript("OnShow", function()
                fired = fired + 1
            end)

            child:Show()
            parent:Show()
            return fired
        "#,
        )
        .unwrap();
    assert_eq!(
        fired, 1,
        "child OnShow should fire when a hidden parent becomes visible"
    );
}

#[test]
fn test_child_onhide_fires_when_parent_becomes_hidden() {
    let env = WowLuaEnv::new().unwrap();
    let fired: i32 = env
        .eval(
            r#"
            local parent = CreateFrame("Frame", "ChildOnHideParent", UIParent)
            local child = CreateFrame("Frame", "ChildOnHideChild", parent)

            local fired = 0
            child:SetScript("OnHide", function()
                fired = fired + 1
            end)

            parent:Hide()
            return fired
        "#,
        )
        .unwrap();
    assert_eq!(
        fired, 1,
        "child OnHide should fire when a visible parent becomes hidden"
    );
}

const VISIBILITY_BINDINGS_XML: &str = r#"
    <Ui>
        <Frame name="VisibilityPrecall" intrinsic="true">
            <Scripts>
                <OnShow intrinsicOrder="precall">VisibilityBindingProbe(self, 'show', 'pre')</OnShow>
                <OnHide intrinsicOrder="precall">VisibilityBindingProbe(self, 'hide', 'pre')</OnHide>
            </Scripts>
        </Frame>
        <Frame name="VisibilityPostcall" virtual="true">
            <Scripts>
                <OnShow intrinsicOrder="postcall">VisibilityBindingProbe(self, 'show', 'post')</OnShow>
                <OnHide intrinsicOrder="postcall">VisibilityBindingProbe(self, 'hide', 'post')</OnHide>
            </Scripts>
        </Frame>
    </Ui>
"#;

fn visibility_binding_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.exec("VisibilityBindingProbe = function() end").unwrap();
    let root = tempfile::tempdir().unwrap();
    let toc = root.path().join("VisibilityBindings.toc");
    std::fs::write(&toc, "## Title: Visibility bindings\nBindings.xml\n").unwrap();
    std::fs::write(root.path().join("Bindings.xml"), VISIBILITY_BINDINGS_XML).unwrap();
    let loaded = wow_ui_sim::loader::load_addon(&env.loader_env(), &toc).unwrap();
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    env.exec(r#"
        VisibilityParent = CreateFrame('Frame', 'VisibilityParent', UIParent, 'VisibilityPrecall,VisibilityPostcall')
        VisibilityParent:Hide()
        VisibilityChild = CreateFrame('Frame', 'VisibilityChild', VisibilityParent, 'VisibilityPrecall,VisibilityPostcall')
        VisibilityHiddenChild = CreateFrame('Frame', 'VisibilityHiddenChild', VisibilityParent, 'VisibilityPrecall,VisibilityPostcall')
        VisibilityHiddenChild:Hide()
        for _, frame in ipairs({VisibilityParent, VisibilityChild, VisibilityHiddenChild}) do
            for _, event in ipairs({'OnShow', 'OnHide'}) do
                assert(type(frame:GetScript(event, 0)) == 'function', 'precall binding missing')
                assert(frame:GetScript(event, 1) == nil, 'unexpected normal binding')
                assert(type(frame:GetScript(event, 2)) == 'function', 'postcall binding missing')
            end
            frame:SetScript('OnShow', function(self) VisibilityBindingProbe(self, 'show', 'normal') end)
            frame:SetScript('OnHide', function(self) VisibilityBindingProbe(self, 'hide', 'normal') end)
        end
        VisibilityCalls = {}
        function VisibilityBindingProbe(self, event, binding)
            VisibilityCalls[#VisibilityCalls + 1] = self:GetName() .. ':' .. event .. ':' .. binding
        end
        function AssertVisibilityCalls(event)
            local expected = {}
            for _, name in ipairs({'VisibilityChild', 'VisibilityParent'}) do
                for _, binding in ipairs({'pre', 'normal', 'post'}) do
                    expected[#expected + 1] = name .. ':' .. event .. ':' .. binding
                end
            end
            assert(table.concat(VisibilityCalls, ',') == table.concat(expected, ','),
                table.concat(VisibilityCalls, ','))
            VisibilityCalls = {}
        end
    "#).expect("real XML supplies intrinsic bindings independently of normal scripts");
    env
}

#[test]
fn test_visibility_intrinsic_bindings_keep_children_first_order() {
    let env = visibility_binding_env();
    env.exec(
        r#"
        VisibilityParent:Show()
        AssertVisibilityCalls('show')
        assert(VisibilityChild:IsShown() and VisibilityChild:IsVisible())
        assert(not VisibilityHiddenChild:IsShown())
        VisibilityParent:Hide()
        AssertVisibilityCalls('hide')
        assert(VisibilityChild:IsShown() and not VisibilityChild:IsVisible())
        VisibilityParent:Hide()
        assert(#VisibilityCalls == 0, 'unchanged visibility must not dispatch')
        VisibilityParent:Show()
        AssertVisibilityCalls('show')
    "#,
    )
    .expect(
        "ancestor transitions deliver every binding, children first, excluding hidden children",
    );
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
fn test_visibility_intrinsic_bindings_preserve_reentrant_hide() {
    let env = visibility_binding_env();
    env.exec(
        r#"
        VisibilityParent:SetScript('OnShow', function(self)
            VisibilityBindingProbe(self, 'show', 'normal')
            self:Hide()
        end)
        VisibilityParent:Show()
        local expected = {}
        for _, event in ipairs({'show', 'hide'}) do
            for _, name in ipairs({'VisibilityChild', 'VisibilityParent'}) do
                for _, binding in ipairs({'pre', 'normal', 'post'}) do
                    expected[#expected + 1] = name .. ':' .. event .. ':' .. binding
                end
            end
        end
        assert(table.concat(VisibilityCalls, ',') == table.concat(expected, ','),
            table.concat(VisibilityCalls, ','))
        assert(not VisibilityParent:IsShown() and not VisibilityParent:IsVisible())
        assert(VisibilityChild:IsShown() and not VisibilityChild:IsVisible())
    "#,
    )
    .expect("postcall completes before the deferred opposite visibility transition");
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
fn test_visibility_intrinsic_error_does_not_skip_later_bindings() {
    let env = visibility_binding_env();
    env.exec(
        r#"
        local record = VisibilityBindingProbe
        VisibilityBindingProbe = function(self, event, binding)
            record(self, event, binding)
            if self == VisibilityChild and event == 'show' and binding == 'pre' then
                error('visibility precall failure')
            end
        end
        VisibilityParent:Show()
        AssertVisibilityCalls('show')
        assert(VisibilityParent:IsVisible() and VisibilityChild:IsVisible())
    "#,
    )
    .expect("a failing intrinsic handler must not abort sibling bindings or parent delivery");
    assert!(
        env.state()
            .borrow()
            .lua_errors
            .iter()
            .any(|error| error.contains("visibility precall failure"))
    );
}
