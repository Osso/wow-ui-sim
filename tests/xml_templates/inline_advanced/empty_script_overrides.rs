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
    register_first_template(
        r#"<Ui><ScrollFrame name="XmlRangeParent" virtual="true">
            <Scripts><OnScrollRangeChanged function="XmlInheritedRange"/></Scripts>
        </ScrollFrame></Ui>"#,
        "XmlRangeParent",
        "ScrollFrame",
    );
    create_first_frame(
        &env,
        r#"<Ui><ScrollFrame name="XmlRangeCleared" parent="UIParent" inherits="XmlRangeParent">
            <Scripts><OnScrollRangeChanged function=""/></Scripts>
        </ScrollFrame></Ui>"#,
        "ScrollFrame",
    );
    env.exec(
        r#"local frame = XmlRangeCleared
           assert(frame:GetScript('OnScrollRangeChanged') == nil)
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
    load_addon(&env.loader_env(), &dir.path().join("XmlEmptySlotOverrides.toc")).unwrap();
    env.exec(
        r#"XmlSlotCalls = {}
           local pre = CreateFrame('ScrollFrame', nil, UIParent, 'XmlSlotBase,XmlSlotPost,XmlSlotClearPre')
           local post = CreateFrame('ScrollFrame', nil, UIParent, 'XmlSlotBase,XmlSlotPost,XmlSlotClearPost')
           for _, frame in ipairs({pre, post}) do
               frame:SetScript('OnHorizontalScroll', function() XmlSlotCalls[#XmlSlotCalls + 1] = 'normal' end)
           end
           assert(pre:GetScript('OnHorizontalScroll', 0) == nil)
           assert(type(pre:GetScript('OnHorizontalScroll')) == 'function')
           assert(type(pre:GetScript('OnHorizontalScroll', 2)) == 'function')
           assert(type(post:GetScript('OnHorizontalScroll', 0)) == 'function')
           assert(type(post:GetScript('OnHorizontalScroll')) == 'function')
           assert(post:GetScript('OnHorizontalScroll', 2) == nil)
           pre:SetHorizontalScroll(10)
           post:SetHorizontalScroll(20)
           assert(table.concat(XmlSlotCalls, ',') == 'normal,post,pre,normal', table.concat(XmlSlotCalls, ','))"#,
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
    load_addon(&env.loader_env(), &dir.path().join("XmlNonemptyFunction.toc")).unwrap();
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
           assert(frame:GetScript('OnScrollRangeChanged') == nil)
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
