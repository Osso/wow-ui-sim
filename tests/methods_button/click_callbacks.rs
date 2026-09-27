use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn test_click_dispatches_scripts_and_hooks_in_order_with_supplied_arguments() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local btn = CreateFrame("Button", nil, UIParent)
        local calls = {}
        for _, script in ipairs({"PreClick", "OnClick", "PostClick"}) do
            btn:SetScript(script, function(self, button, down)
                assert(self == btn)
                calls[#calls + 1] = script .. ":script:" .. button .. ":" .. tostring(down)
            end)
            btn:HookScript(script, function(self, button, down)
                assert(self == btn)
                calls[#calls + 1] = script .. ":hook:" .. button .. ":" .. tostring(down)
            end)
        end
        btn:Click()
        btn:Click("RightButton", true)
        __button_click_calls = table.concat(calls, ",")
        "#,
    )
    .unwrap();
    let calls: String = env.eval("return __button_click_calls").unwrap();
    let expected = ["LeftButton:false", "RightButton:true"]
        .into_iter()
        .flat_map(|args| {
            ["PreClick", "OnClick", "PostClick"]
                .into_iter()
                .flat_map(move |script| {
                    ["script", "hook"].map(move |binding| format!("{script}:{binding}:{args}"))
                })
        })
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(calls, expected);
}

#[test]
fn test_click_runs_pre_and_post_without_onclick() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local btn = CreateFrame("Button", nil, UIParent)
        local calls = {}
        btn:SetScript("PreClick", function() calls[#calls + 1] = "pre" end)
        btn:SetScript("PostClick", function() calls[#calls + 1] = "post" end)
        btn:Click()
        __button_click_calls = table.concat(calls, ",")
        "#,
    )
    .unwrap();
    let calls: String = env.eval("return __button_click_calls").unwrap();
    assert_eq!(calls, "pre,post");
}

#[test]
fn test_disabled_button_click_is_silent_until_enabled() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local btn = CreateFrame("Button", nil, UIParent)
        local calls = {}
        for _, script in ipairs({"PreClick", "OnClick", "PostClick"}) do
            btn:SetScript(script, function() calls[#calls + 1] = script end)
        end
        btn:Disable()
        btn:Click()
        __disabled_button_calls = table.concat(calls, ",")
        btn:Enable()
        btn:Click()
        __enabled_button_calls = table.concat(calls, ",")
        "#,
    )
    .unwrap();
    let disabled: String = env.eval("return __disabled_button_calls").unwrap();
    let enabled: String = env.eval("return __enabled_button_calls").unwrap();
    assert_eq!(disabled, "");
    assert_eq!(enabled, "PreClick,OnClick,PostClick");
}

#[test]
fn test_disabled_checkbutton_click_toggles_without_callbacks() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local btn = CreateFrame("CheckButton", nil, UIParent)
        local calls = 0
        btn:SetScript("OnClick", function() calls = calls + 1 end)
        btn:Disable()
        btn:Click()
        __disabled_check_checked = btn:GetChecked()
        __disabled_check_calls = calls
        btn:Enable()
        btn:Click()
        __enabled_check_checked = btn:GetChecked()
        __enabled_check_calls = calls
        "#,
    )
    .unwrap();
    let (disabled_checked, disabled_calls, enabled_checked, enabled_calls): (bool, i64, bool, i64) = env
        .eval("return __disabled_check_checked, __disabled_check_calls, __enabled_check_checked, __enabled_check_calls")
        .unwrap();
    assert!(disabled_checked);
    assert_eq!(disabled_calls, 0);
    assert!(!enabled_checked);
    assert_eq!(enabled_calls, 1);
}

#[test]
fn test_recursive_checkbutton_click_toggles_without_reentering_callbacks() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local btn = CreateFrame("CheckButton", nil, UIParent)
        local calls = 0
        btn:SetScript("OnClick", function(self)
            calls = calls + 1
            self:Click()
            __recursive_check_checked_inside = self:GetChecked()
        end)
        btn:Click()
        __recursive_check_checked = btn:GetChecked()
        __recursive_check_calls = calls
        "#,
    )
    .unwrap();
    let (inside, checked, calls): (bool, bool, i64) = env
        .eval("return __recursive_check_checked_inside, __recursive_check_checked, __recursive_check_calls")
        .unwrap();
    assert!(!inside);
    assert!(!checked);
    assert_eq!(calls, 1);
}

#[test]
fn test_click_suppresses_same_button_recursion_but_allows_another_button() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local first = CreateFrame("Button", nil, UIParent)
        local second = CreateFrame("Button", nil, UIParent)
        local calls = {}
        first:SetScript("OnClick", function(self)
            calls[#calls + 1] = "first"
            self:Click()
            second:Click()
        end)
        second:SetScript("OnClick", function() calls[#calls + 1] = "second" end)
        first:Click()
        __button_recursion_calls = table.concat(calls, ",")
        "#,
    )
    .unwrap();
    let calls: String = env.eval("return __button_recursion_calls").unwrap();
    assert_eq!(calls, "first,second");
}

#[test]
fn test_click_onclick_error_is_reported_and_next_click_dispatches() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local btn = CreateFrame("Button", nil, UIParent)
        local errors = {}
        local calls = 0
        seterrorhandler(function(message) errors[#errors + 1] = tostring(message) end)
        btn:SetScript("OnClick", function()
            calls = calls + 1
            if calls == 1 then error("click handler sentinel") end
        end)
        btn:Click()
        btn:Click()
        __button_error_calls = calls
        __button_error_count = #errors
        __button_error_message = errors[1] or ""
        "#,
    )
    .unwrap();
    let (calls, count, message): (i64, i64, String) = env
        .eval("return __button_error_calls, __button_error_count, __button_error_message")
        .unwrap();
    assert_eq!(calls, 2);
    assert_eq!(count, 1);
    assert!(message.contains("click handler sentinel"), "{message}");
}

#[test]
fn test_click_preclick_error_reports_and_later_phases_continue() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local btn = CreateFrame("Button", nil, UIParent)
        local errors, calls = {}, {}
        seterrorhandler(function(message) errors[#errors + 1] = tostring(message) end)
        btn:SetScript("PreClick", function()
            calls[#calls + 1] = "pre"
            if #calls == 1 then error("preclick sentinel") end
        end)
        btn:SetScript("OnClick", function() calls[#calls + 1] = "on" end)
        btn:SetScript("PostClick", function() calls[#calls + 1] = "post" end)
        btn:Click()
        btn:Click()
        __preclick_error_calls = table.concat(calls, ",")
        __preclick_error_count = #errors
        __preclick_error_message = errors[1] or ""
        "#,
    )
    .unwrap();
    let (calls, count, message): (String, i64, String) = env
        .eval("return __preclick_error_calls, __preclick_error_count, __preclick_error_message")
        .unwrap();
    assert_eq!(calls, "pre,on,post,pre,on,post");
    assert_eq!(count, 1);
    assert!(message.contains("preclick sentinel"), "{message}");
}
