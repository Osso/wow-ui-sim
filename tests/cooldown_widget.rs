use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::widget::Color;

#[test]
fn cooldown_consumes_real_duration_proxy_and_updates() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local d = C_DurationUtil.CreateDuration()
        local cd = CreateFrame('Cooldown', 'DurationProxyCooldown')
        d:SetTimeFromStart(10, 20, 2)
        cd:SetCooldownFromDurationObject(d)
        local start, total = cd:GetCooldownTimes()
        assert(start == 10 and total == 10)
        CooldownDurationProbe = d
    "#).unwrap();
    {
        let state = env.state().borrow();
        let id = state.widgets.get_id_by_name("DurationProxyCooldown").unwrap();
        assert_eq!(state.widgets.get(id).unwrap().cooldown_mod_rate, 2.0);
    }
    env.exec(r#"
        local d = CooldownDurationProbe
        local cd = DurationProxyCooldown
        d:SetTimeFromStart(30, 12, 3)
        cd:SetCooldownFromDurationObject(d)
        start, total = cd:GetCooldownTimes()
        assert(start == 30 and total == 4)
    "#).unwrap();
    let state = env.state().borrow();
    let id = state.widgets.get_id_by_name("DurationProxyCooldown").unwrap();
    assert_eq!(state.widgets.get(id).unwrap().cooldown_mod_rate, 3.0);
    drop(state);
    env.exec(r#"
        local d = C_DurationUtil.CreateDuration()
        DurationProxyCooldown:SetCooldownFromDurationObject(d)
        local start, total = DurationProxyCooldown:GetCooldownTimes()
        assert(start == 0 and total == 0)
        d:SetTimeFromStart(10, 20, 2)
        DurationProxyCooldown:SetCooldownFromDurationObject(d)
        d:Reset()
        DurationProxyCooldown:SetCooldownFromDurationObject(d)
        start, total = DurationProxyCooldown:GetCooldownTimes()
        assert(start == 0 and total == 0)
    "#).unwrap();
}

#[test]
fn cooldown_duration_method_errors_propagate() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local cd = CreateFrame('Cooldown')
        for _, name in ipairs({'IsZero', 'GetStartTime', 'GetTotalDuration', 'GetModRate'}) do
            for _, lookupError in ipairs({false, true}) do
                cd:SetCooldown(7, 9, 2)
                local methods = {
                    IsZero = function() return false end,
                    GetStartTime = function() return 10 end,
                    GetTotalDuration = function() return 20 end,
                    GetModRate = function() return 2 end,
                }
                local proxy = setmetatable({}, {__index = function(_, key)
                    if key == name then
                        if lookupError then error('lookup:' .. name) end
                        return function() error('call:' .. name) end
                    end
                    return methods[key]
                end})
                local ok, message = pcall(cd.SetCooldownFromDurationObject, cd, proxy)
                assert(not ok and string.find(message, name, 1, true))
                local start, total = cd:GetCooldownTimes()
                assert(start == 7 and total == 9)
            end
        end
    "#).unwrap();
}

#[test]
fn cooldown_set_tex_coord_range_persists_vector_bounds() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local cd = CreateFrame("Cooldown", "CooldownTexCoordProbe", UIParent)
        local low = { x = 0.125, y = 0.25 }
        local high = { GetXY = function() return 0.75, 0.875 end }
        cd:SetSwipeTexture("Interface\\Cooldown\\swipe")
        cd:SetTexCoordRange(low, high)
        "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let cooldown_id = state.widgets.get_id_by_name("CooldownTexCoordProbe").unwrap();
    let cooldown = state.widgets.get(cooldown_id).unwrap();

    assert_eq!(
        cooldown.cooldown_tex_coord_range,
        Some((0.125, 0.25, 0.75, 0.875)),
        "SetTexCoordRange should parse WoW-style Vector2D tables and persist swipe UV bounds"
    );
}

#[test]
fn cooldown_widget_methods_persist_runtime_state() {
    let env = WowLuaEnv::new().unwrap();

    let result: String = env
        .eval(
            r#"
            local cd = CreateFrame("Cooldown", "TestCooldown", UIParent)
            TestCooldownFont = {
                __font = "Fonts\\FRIZQT__.TTF",
                __height = 14,
                __outline = "OUTLINE",
            }

            if cd:GetCooldownDisplayDuration() ~= 0 then
                return "display_duration_should_default_zero"
            end
            if cd:GetDrawBling() ~= true then
                return "draw_bling_should_default_true"
            end
            if cd:GetDrawEdge() ~= false then
                return "draw_edge_should_default_false"
            end
            if cd:GetDrawSwipe() ~= true then
                return "draw_swipe_should_default_true"
            end
            if cd:GetEdgeScale() ~= 1 then
                return "edge_scale_should_default_one"
            end
            if cd:GetMinimumCountdownDuration() ~= 0 then
                return "minimum_countdown_should_default_zero"
            end
            if cd:GetUseAuraDisplayTime() ~= false then
                return "use_aura_display_time_should_default_false"
            end

            cd:SetDrawBling(false)
            cd:SetDrawEdge(true)
            cd:SetDrawSwipe(false)
            cd:SetEdgeScale(1.5)
            cd:SetEdgeColor(0.1, 0.2, 0.3, 0.4)
            cd:SetEdgeTexture("Interface\\Cooldown\\edge", 0.7, 0.6, 0.5, 0.4)
            cd:SetSwipeTexture("Interface\\Cooldown\\swipe")
            cd:SetBlingTexture("Interface\\Cooldown\\bling")
            cd:SetUseCircularEdge(true)
            cd:SetCountdownAbbrevThreshold(5)
            cd:SetUseAuraDisplayTime(true)
            cd:SetMinimumCountdownDuration(2500)
            cd:SetCountdownFont("TestCooldownFont")
            cd:SetCooldownFromExpirationTime(20, 8, 1.25)

            local startTime, duration = cd:GetCooldownTimes()
            if startTime ~= 12 or duration ~= 8 then
                return "expiration_time_should_convert_to_start_and_duration"
            end
            if cd:GetCooldownDisplayDuration() ~= 8000 then
                return "display_duration_should_be_milliseconds"
            end
            if cd:GetDrawBling() ~= false then
                return "draw_bling_should_round_trip"
            end
            if cd:GetDrawEdge() ~= true then
                return "draw_edge_should_round_trip"
            end
            if cd:GetDrawSwipe() ~= false then
                return "draw_swipe_should_round_trip"
            end
            if cd:GetEdgeScale() ~= 1.5 then
                return "edge_scale_should_round_trip"
            end
            if cd:GetMinimumCountdownDuration() ~= 2500 then
                return "minimum_countdown_should_round_trip"
            end
            if cd:GetUseAuraDisplayTime() ~= true then
                return "use_aura_display_time_should_round_trip"
            end

            local countdown = cd:GetCountdownFontString()
            if countdown == nil then
                return "countdown_font_string_should_exist"
            end
            if countdown:GetObjectType() ~= "FontString" then
                return "countdown_font_string_should_be_fontstring"
            end

            local durationObject = {
                GetStartTime = function() return 30 end,
                GetTotalDuration = function() return 4 end,
                GetModRate = function() return 2 end,
                IsZero = function() return false end,
            }
            cd:SetCooldownFromDurationObject(durationObject, true)
            startTime, duration = cd:GetCooldownTimes()
            if startTime ~= 30 or duration ~= 4 then
                return "duration_object_should_update_cooldown"
            end
            if cd:GetCooldownDisplayDuration() ~= 4000 then
                return "duration_object_display_duration_should_use_milliseconds"
            end

            local zeroDurationObject = {
                GetStartTime = function() return 99 end,
                GetTotalDuration = function() return 0 end,
                GetModRate = function() return 1 end,
                IsZero = function() return true end,
            }
            cd:SetCooldownFromDurationObject(zeroDurationObject, true)
            startTime, duration = cd:GetCooldownTimes()
            if startTime ~= 0 or duration ~= 0 then
                return "zero_duration_object_should_clear_cooldown"
            end

            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");

    let state = env.state().borrow();
    let cooldown_id = state.widgets.get_id_by_name("TestCooldown").unwrap();
    let cooldown = state.widgets.get(cooldown_id).unwrap();

    assert_eq!(cooldown.cooldown_edge_scale, 1.5);
    assert_eq!(cooldown.cooldown_min_countdown_duration_ms, 2500.0);
    assert_eq!(
        cooldown.cooldown_edge_texture.as_deref(),
        Some("Interface\\Cooldown\\edge")
    );
    assert_eq!(
        cooldown.cooldown_swipe_texture.as_deref(),
        Some("Interface\\Cooldown\\swipe")
    );
    assert_eq!(
        cooldown.cooldown_bling_texture.as_deref(),
        Some("Interface\\Cooldown\\bling")
    );
    assert!(cooldown.cooldown_use_circular_edge);
    assert_eq!(cooldown.cooldown_countdown_abbrev_threshold_seconds, 5.0);
    assert!(cooldown.cooldown_use_aura_display_time);
    assert_eq!(
        cooldown.cooldown_edge_color,
        Color::new(0.7, 0.6, 0.5, 0.4),
        "SetEdgeTexture color arguments should persist the cooldown edge tint"
    );
    let countdown_id = cooldown.cooldown_countdown_font_string_id.expect(
        "SetCountdownFont/GetCountdownFontString should create and retain a countdown fontstring",
    );
    let countdown = state.widgets.get(countdown_id).unwrap();
    assert_eq!(countdown.font.as_deref(), Some("Fonts\\FRIZQT__.TTF"));
    assert_eq!(countdown.font_size, 14.0);
}

#[test]
fn cooldown_widgets_accept_on_cooldown_done_scripts() {
    let env = WowLuaEnv::new().unwrap();

    let result: bool = env
        .eval(
            r#"
            local cd = CreateFrame("Cooldown", "CooldownScriptProbe", UIParent)
            return pcall(function()
                cd:SetScript("OnCooldownDone", function() end)
            end)
            "#,
        )
        .unwrap();

    assert!(
        result,
        "Cooldown widgets should accept the Blizzard OnCooldownDone script handler"
    );
}
