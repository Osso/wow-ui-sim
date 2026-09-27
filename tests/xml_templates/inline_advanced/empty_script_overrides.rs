use super::*;

fn assert_no_lua_errors(env: &WowLuaEnv) {
    let state = env.state();
    let state = state.borrow();
    assert!(state.lua_errors.is_empty(), "{:?}", state.lua_errors);
}

#[test]
fn ordinary_xml_empty_function_clears_inherited_normal_script() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec("XmlEmptyFunctionCalls = 0; function XmlInheritedRange() XmlEmptyFunctionCalls = XmlEmptyFunctionCalls + 1 end")
        .unwrap();
    let dir = create_test_addon(
        r#"<Ui>
            <ScrollFrame name="XmlRangeParent" virtual="true">
                <Scripts><OnScrollRangeChanged function="XmlInheritedRange"/></Scripts>
            </ScrollFrame>
            <ScrollFrame name="XmlRangeCleared" parent="UIParent" inherits="XmlRangeParent">
                <Scripts><OnScrollRangeChanged function=""/></Scripts>
            </ScrollFrame>
        </Ui>"#,
        "XmlEmptyOrdinaryRange",
    );
    load_addon(
        &env.loader_env(),
        &dir.path().join("XmlEmptyOrdinaryRange.toc"),
    )
    .unwrap();
    env.exec(
        r#"local frame = XmlRangeCleared
           assert(frame:GetScript('OnScrollRangeChanged') == nil, 'ordinary empty function did not clear inherited handler')
           frame:SetSize(100, 100)
           local child = CreateFrame('Frame', nil, frame)
           child:SetSize(200, 200)
           frame:SetScrollChild(child)
           frame:UpdateScrollChildRect()
           assert(XmlEmptyFunctionCalls == 0)"#,
    )
    .unwrap();
    assert_no_lua_errors(&env);
}

#[test]
fn runtime_xml_empty_function_clears_only_requested_intrinsic_slots() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    let dir = create_test_addon(
        r#"<Ui>
            <ScrollFrame name="XmlSlotBase" virtual="true" intrinsic="true">
                <Scripts><OnHorizontalScroll intrinsicOrder="precall">XmlSlotCalls[#XmlSlotCalls + 1] = 'pre'</OnHorizontalScroll></Scripts>
            </ScrollFrame>
            <ScrollFrame name="XmlSlotPost" virtual="true">
                <Scripts><OnHorizontalScroll intrinsicOrder="postcall">XmlSlotCalls[#XmlSlotCalls + 1] = 'post'</OnHorizontalScroll></Scripts>
            </ScrollFrame>
            <ScrollFrame name="XmlSlotClearPre" virtual="true">
                <Scripts><OnHorizontalScroll function="" intrinsicOrder="precall"/></Scripts>
            </ScrollFrame>
            <ScrollFrame name="XmlSlotClearPost" virtual="true">
                <Scripts><OnHorizontalScroll function="" intrinsicOrder="postcall"/></Scripts>
            </ScrollFrame>
        </Ui>"#,
        "XmlEmptySlotOverrides",
    );
    load_addon(
        &env.loader_env(),
        &dir.path().join("XmlEmptySlotOverrides.toc"),
    )
    .unwrap();
    env.exec(
        r#"XmlSlotCalls = {}
           local pre = CreateFrame('ScrollFrame', nil, UIParent, 'XmlSlotBase,XmlSlotPost,XmlSlotClearPre')
           local post = CreateFrame('ScrollFrame', nil, UIParent, 'XmlSlotBase,XmlSlotPost,XmlSlotClearPost')
           for _, frame in ipairs({pre, post}) do
               frame:SetScript('OnHorizontalScroll', function() XmlSlotCalls[#XmlSlotCalls + 1] = 'normal' end)
           end
           assert(pre:GetScript('OnHorizontalScroll', 0) == nil, 'precall not cleared')
           assert(type(pre:GetScript('OnHorizontalScroll')) == 'function', 'normal dropped with precall')
           assert(type(pre:GetScript('OnHorizontalScroll', 2)) == 'function', 'postcall dropped with precall')
           assert(type(post:GetScript('OnHorizontalScroll', 0)) == 'function', 'precall dropped with postcall')
           assert(type(post:GetScript('OnHorizontalScroll')) == 'function', 'normal dropped with postcall')
           assert(post:GetScript('OnHorizontalScroll', 2) == nil, 'postcall not cleared')
           pre:SetHorizontalScroll(10)
           post:SetHorizontalScroll(20)
           assert(table.concat(XmlSlotCalls, ',') == 'normal,post,pre,normal', table.concat(XmlSlotCalls, ','))"#,
    )
    .unwrap();
    assert_no_lua_errors(&env);
}

#[test]
fn runtime_xml_intrinsic_default_empty_function_clears_pre_only() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    let dir = create_test_addon(
        r#"<Ui>
            <ScrollFrame name="XmlDefaultPre" virtual="true" intrinsic="true">
                <Scripts><OnHorizontalScroll>XmlDefaultCalls[#XmlDefaultCalls + 1] = 'pre'</OnHorizontalScroll></Scripts>
            </ScrollFrame>
            <ScrollFrame name="XmlDefaultNormal" virtual="true">
                <Scripts><OnHorizontalScroll>XmlDefaultCalls[#XmlDefaultCalls + 1] = 'normal'</OnHorizontalScroll></Scripts>
            </ScrollFrame>
            <ScrollFrame name="XmlDefaultPost" virtual="true">
                <Scripts><OnHorizontalScroll intrinsicOrder="postcall">XmlDefaultCalls[#XmlDefaultCalls + 1] = 'post'</OnHorizontalScroll></Scripts>
            </ScrollFrame>
            <ScrollFrame name="XmlDefaultClear" virtual="true" intrinsic="true">
                <Scripts><OnHorizontalScroll function=""/></Scripts>
            </ScrollFrame>
        </Ui>"#,
        "XmlDefaultIntrinsicClear",
    );
    load_addon(
        &env.loader_env(),
        &dir.path().join("XmlDefaultIntrinsicClear.toc"),
    )
    .unwrap();
    env.exec(
        r#"XmlDefaultCalls = {}
           local frame = CreateFrame('ScrollFrame', nil, UIParent,
               'XmlDefaultPre,XmlDefaultNormal,XmlDefaultPost,XmlDefaultClear')
           assert(frame:GetScript('OnHorizontalScroll', 0) == nil, 'default precall retained')
           assert(type(frame:GetScript('OnHorizontalScroll')) == 'function', 'normal lost')
           assert(type(frame:GetScript('OnHorizontalScroll', 2)) == 'function', 'postcall lost')
           frame:SetHorizontalScroll(12)
           assert(table.concat(XmlDefaultCalls, ',') == 'normal,post', table.concat(XmlDefaultCalls, ','))"#,
    )
    .unwrap();
    assert_no_lua_errors(&env);
}

#[test]
fn ordinary_xml_whitespace_function_and_empty_bodies_clear_inherited_scripts() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec("XmlWhitespaceCalls = 0; function XmlWhitespaceBase() XmlWhitespaceCalls = XmlWhitespaceCalls + 1 end")
        .unwrap();
    let dir = create_test_addon(
        r#"<Ui>
            <ScrollFrame name="XmlWhitespaceBaseTemplate" virtual="true">
                <Scripts><OnHorizontalScroll function="XmlWhitespaceBase"/></Scripts>
            </ScrollFrame>
            <ScrollFrame name="XmlWhitespaceFunctionFrame" parent="UIParent" inherits="XmlWhitespaceBaseTemplate">
                <Scripts><OnHorizontalScroll function="  "/></Scripts>
            </ScrollFrame>
            <ScrollFrame name="XmlEmptyBodyFrame" parent="UIParent" inherits="XmlWhitespaceBaseTemplate">
                <Scripts><OnHorizontalScroll/></Scripts>
            </ScrollFrame>
            <ScrollFrame name="XmlWhitespaceBodyFrame" parent="UIParent" inherits="XmlWhitespaceBaseTemplate">
                <Scripts><OnHorizontalScroll>
                </OnHorizontalScroll></Scripts>
            </ScrollFrame>
        </Ui>"#,
        "XmlOrdinaryWhitespaceClear",
    );
    load_addon(
        &env.loader_env(),
        &dir.path().join("XmlOrdinaryWhitespaceClear.toc"),
    )
    .unwrap();
    env.exec(
        r#"for _, frame in ipairs({XmlWhitespaceFunctionFrame, XmlEmptyBodyFrame, XmlWhitespaceBodyFrame}) do
               assert(frame:GetScript('OnHorizontalScroll') == nil, frame:GetName() .. ' retained handler')
               frame:SetHorizontalScroll(12)
           end
           assert(XmlWhitespaceCalls == 0, XmlWhitespaceCalls)"#,
    )
    .unwrap();
    assert_no_lua_errors(&env);
}

#[test]
fn runtime_xml_whitespace_function_and_empty_bodies_clear_inherited_scripts() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec("XmlRuntimeWhitespaceCalls = 0; function XmlRuntimeWhitespaceBase() XmlRuntimeWhitespaceCalls = XmlRuntimeWhitespaceCalls + 1 end")
        .unwrap();
    let dir = create_test_addon(
        r#"<Ui>
            <ScrollFrame name="XmlRuntimeWhitespaceBase" virtual="true">
                <Scripts><OnHorizontalScroll function="XmlRuntimeWhitespaceBase"/></Scripts>
            </ScrollFrame>
            <ScrollFrame name="XmlRuntimeWhitespaceFunction" virtual="true">
                <Scripts><OnHorizontalScroll function="  "/></Scripts>
            </ScrollFrame>
            <ScrollFrame name="XmlRuntimeEmptyBody" virtual="true">
                <Scripts><OnHorizontalScroll/></Scripts>
            </ScrollFrame>
            <ScrollFrame name="XmlRuntimeWhitespaceBody" virtual="true">
                <Scripts><OnHorizontalScroll>
                </OnHorizontalScroll></Scripts>
            </ScrollFrame>
        </Ui>"#,
        "XmlRuntimeWhitespaceClear",
    );
    load_addon(
        &env.loader_env(),
        &dir.path().join("XmlRuntimeWhitespaceClear.toc"),
    )
    .unwrap();
    env.exec(
        r#"for _, template in ipairs({'XmlRuntimeWhitespaceFunction', 'XmlRuntimeEmptyBody', 'XmlRuntimeWhitespaceBody'}) do
               local frame = CreateFrame('ScrollFrame', nil, UIParent, 'XmlRuntimeWhitespaceBase,' .. template)
               assert(frame:GetScript('OnHorizontalScroll') == nil, template .. ' retained handler')
               frame:SetHorizontalScroll(12)
           end
           assert(XmlRuntimeWhitespaceCalls == 0, XmlRuntimeWhitespaceCalls)"#,
    )
    .unwrap();
    assert_no_lua_errors(&env);
}

#[test]
fn ordinary_xml_method_with_empty_function_stays_callable() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        "XmlEmptyFunctionMethodMixin = {}; function XmlEmptyFunctionMethodMixin:OnEnter() self.methodCalls = (self.methodCalls or 0) + 1 end",
    )
    .unwrap();
    let dir = create_test_addon(
        r#"<Ui><Frame name="XmlMethodEmptyFunctionFrame" parent="UIParent" mixin="XmlEmptyFunctionMethodMixin">
            <Scripts><OnEnter method="OnEnter" function=""/></Scripts>
        </Frame></Ui>"#,
        "XmlOrdinaryMethodEmptyFunction",
    );
    load_addon(
        &env.loader_env(),
        &dir.path().join("XmlOrdinaryMethodEmptyFunction.toc"),
    )
    .unwrap();
    env.exec(
        r#"local frame = XmlMethodEmptyFunctionFrame
           local handler = frame:GetScript('OnEnter')
           assert(type(handler) == 'function', 'method was cleared')
           handler(frame)
           assert(frame.methodCalls == 1, frame.methodCalls)"#,
    )
    .unwrap();
    assert_no_lua_errors(&env);
}

#[test]
fn runtime_xml_method_with_empty_function_stays_callable() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        "XmlRuntimeEmptyMethodMixin = {}; function XmlRuntimeEmptyMethodMixin:OnEnter() self.methodCalls = (self.methodCalls or 0) + 1 end",
    )
    .unwrap();
    let dir = create_test_addon(
        r#"<Ui><Frame name="XmlRuntimeMethodEmptyFunction" virtual="true" mixin="XmlRuntimeEmptyMethodMixin">
            <Scripts><OnEnter method="OnEnter" function=""/></Scripts>
        </Frame></Ui>"#,
        "XmlRuntimeMethodEmptyFunction",
    );
    load_addon(
        &env.loader_env(),
        &dir.path().join("XmlRuntimeMethodEmptyFunction.toc"),
    )
    .unwrap();
    env.exec(
        r#"local frame = CreateFrame('Frame', nil, UIParent, 'XmlRuntimeMethodEmptyFunction')
           local handler = frame:GetScript('OnEnter')
           assert(type(handler) == 'function', 'method was cleared')
           handler(frame)
           assert(frame.methodCalls == 1, frame.methodCalls)"#,
    )
    .unwrap();
    assert_no_lua_errors(&env);
}

#[test]
fn runtime_xml_nonempty_function_still_runs() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec("XmlNonemptyFunctionCalls = 0; function XmlNonemptyRange() XmlNonemptyFunctionCalls = XmlNonemptyFunctionCalls + 1 end")
        .unwrap();
    let dir = create_test_addon(
        r#"<Ui><ScrollFrame name="XmlNonemptyRangeTemplate" virtual="true">
            <Scripts><OnScrollRangeChanged function="XmlNonemptyRange"/></Scripts>
        </ScrollFrame></Ui>"#,
        "XmlNonemptyFunction",
    );
    load_addon(
        &env.loader_env(),
        &dir.path().join("XmlNonemptyFunction.toc"),
    )
    .unwrap();
    env.exec(
        r#"local frame = CreateFrame('ScrollFrame', nil, UIParent, 'XmlNonemptyRangeTemplate')
           local handler = frame:GetScript('OnScrollRangeChanged')
           assert(type(handler) == 'function')
           handler(frame, 1, 2)
           assert(XmlNonemptyFunctionCalls == 1)"#,
    )
    .unwrap();
    assert_no_lua_errors(&env);
}

#[test]
fn cached_faux_scrollframe_empty_function_disables_inherited_auto_range() {
    let env = crate::common::env_with_shared_xml();
    env.exec(
        r#"local frame = CreateFrame('ScrollFrame', 'XmlEmptyFauxRange', UIParent, 'FauxScrollFrameTemplate')
           assert(frame:GetScript('OnScrollRangeChanged') == nil, 'cached Faux inherited range handler retained')
           frame:SetSize(100, 100)
           local bar = _G[frame:GetName() .. 'ScrollBar']
           assert(bar)
           bar:SetMinMaxValues(0, 17)
           frame:UpdateScrollChildRect()
           local _, max = bar:GetMinMaxValues()
           assert(max == 17, max)"#,
    )
    .unwrap();
    assert_no_lua_errors(&env);
}
