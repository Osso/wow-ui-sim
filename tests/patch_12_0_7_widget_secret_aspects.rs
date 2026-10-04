//! B36 authentication prerequisites and bounded B37/B38 secret-aspect contracts.
//! INFERRED: public overwrite clears only its own input origin; extras add origin.

use wow_ui_sim::lua_api::WowLuaEnv;

fn aspect_env() -> WowLuaEnv {
    WowLuaEnv::new().expect("create widget aspect environment")
}

#[cfg(feature = "retail-12-0-7")]
fn aspect_lua(env: &WowLuaEnv, script: &str) {
    env.eval::<()>(script).expect("widget secret-aspect behavior");
}

#[cfg(feature = "retail-12-0-7")]
#[test]
fn widget_aspects_defaults_and_public_round_trips() {
    let env = aspect_env();
    aspect_lua(&env, r#"
        local b = CreateFrame('Button')
        local s = CreateFrame('ScrollFrame')
        assert(b:GetButtonState() == 'NORMAL' and b:IsEnabled())
        assert(s:GetHorizontalScroll() == 0 and s:GetVerticalScroll() == 0)
        assert(not issecretvalue(b:GetButtonState()) and not issecretvalue(b:IsEnabled()))
        assert(not issecretvalue(s:GetHorizontalScroll()) and not issecretvalue(s:GetVerticalScroll()))
        assert(select('#', b:GetButtonState()) == 1 and select('#', b:IsEnabled()) == 1)
        assert(select('#', s:GetHorizontalScroll()) == 1 and select('#', s:GetVerticalScroll()) == 1)
        assert(select('#', b:SetButtonState('PUSHED', false)) == 0)
        assert(select('#', b:SetEnabled(false)) == 0)
        assert(select('#', s:SetHorizontalScroll(-17.25)) == 0)
        assert(select('#', s:SetVerticalScroll(83.5)) == 0)
        assert(b:GetButtonState() == 'PUSHED' and b:IsEnabled() == false)
        assert(s:GetHorizontalScroll() == -17.25 and s:GetVerticalScroll() == 83.5)
        b:SetButtonState('NORMAL'); b:SetEnabled(true)
        s:SetHorizontalScroll(4.75); s:SetVerticalScroll(-2.5)
        assert(b:GetButtonState() == 'NORMAL' and b:IsEnabled())
        assert(s:GetHorizontalScroll() == 4.75 and s:GetVerticalScroll() == -2.5)
    "#);
}

#[cfg(feature = "retail-12-0-7")]
#[test]
fn button_state_secret_token_lock_and_equal_public_overwrite() {
    let env = aspect_env();
    aspect_lua(&env, r#"
        local b = CreateFrame('Button')
        local other = CreateFrame('Button')
        local token, lock = secretwrap('PUSHED'), secretwrap(false)
        b:SetButtonState(token, lock)
        assert(issecretvalue(token) and issecretvalue(lock))
        assert(secretunwrap(token) == 'PUSHED' and secretunwrap(lock) == false)
        assert(issecretvalue(b:GetButtonState()) and issecretvalue(b:IsEnabled()))
        assert(secretunwrap(b:GetButtonState()) == 'PUSHED' and secretunwrap(b:IsEnabled()))
        assert(other:GetButtonState() == 'NORMAL' and not issecretvalue(other:IsEnabled()))
        -- INFERRED: clearing the same value's origin needs no numeric/token change.
        b:SetButtonState('PUSHED', false)
        assert(not issecretvalue(b:GetButtonState()) and not issecretvalue(b:IsEnabled()))
        b:SetButtonState('NORMAL', secretwrap(false))
        assert(issecretvalue(b:GetButtonState()) and secretunwrap(b:GetButtonState()) == 'NORMAL')
        b:SetButtonState('NORMAL', false, secretwrap(29))
        assert(issecretvalue(b:GetButtonState()))
        b:SetButtonState('NORMAL', false)
        assert(not issecretvalue(b:GetButtonState()) and debug.getstacktaint() == nil)
    "#);
}

#[cfg(feature = "retail-12-0-7")]
#[test]
fn button_enabled_secret_origin_callbacks_and_shared_lifecycle() {
    let env = aspect_env();
    aspect_lua(&env, r#"
        local b = CreateFrame('Button')
        local log = {}
        b:SetScript('OnDisable', function(self)
            assert(secretunwrap(self:IsEnabled()) == false)
            assert(issecretvalue(self:GetButtonState()))
            log[#log + 1] = 'disable'
        end)
        b:SetScript('OnEnable', function(self)
            assert(secretunwrap(self:IsEnabled()) == true)
            log[#log + 1] = 'enable'
        end)
        b:SetEnabled(secretwrap(false))
        b:SetEnabled(secretwrap(false))
        assert(table.concat(log, ',') == 'disable')
        b:SetButtonState(secretwrap('PUSHED'))
        b:SetEnabled(true)
        assert(table.concat(log, ',') == 'disable,enable')
        -- INFERRED: clearing enabled origin does not clear state-token origin.
        assert(issecretvalue(b:IsEnabled()) and secretunwrap(b:IsEnabled()) == true)
        b:SetButtonState('PUSHED')
        assert(not issecretvalue(b:IsEnabled()) and b:GetButtonState() == 'PUSHED')
        b:SetEnabled(secretwrap(true))
        assert(table.concat(log, ',') == 'disable,enable' and issecretvalue(b:IsEnabled()))
        b:SetEnabled(true)
        assert(not issecretvalue(b:IsEnabled()))
    "#);
}

#[cfg(feature = "retail-12-0-7")]
#[test]
fn widget_and_font_authentication_denies_tainted_inputs_before_mutation() {
    let env = aspect_env();
    aspect_lua(&env, r#"
        local b, s = CreateFrame('Button'), CreateFrame('ScrollFrame')
        b:SetButtonState('PUSHED'); b:SetEnabled(false)
        s:SetHorizontalScroll(12.5); s:SetVerticalScroll(-7.25)
        local token, lock, enabled = secretwrap('NORMAL'), secretwrap(false), secretwrap(true)
        local offset, extra = secretwrap(91.75), secretwrap(43)
        local secretButton, secretScroll = secretwrap(b), secretwrap(s)
        local receivers = {
            CreateFrame('EditBox'), CreateFont('AspectAuthenticationFont'),
            UIParent:CreateFontString(), CreateFrame('MessageFrame'), CreateFrame('SimpleHTML'),
        }
        local path, height, flags = 'Fonts\\FRIZQT__.TTF', 18.5, 'OUTLINE'
        local sp, sh, sf, st = secretwrap(path), secretwrap(height), secretwrap(flags), secretwrap('p')
        local function setFont(f, p, h, fl, tail)
            if f:GetObjectType() == 'SimpleHTML' then
                return f:SetFont('p', p, h, fl, tail)
            end
            return f:SetFont(p, h, fl, tail)
        end
        local function getFont(f)
            if f:GetObjectType() == 'SimpleHTML' then return f:GetFont('p') end
            return f:GetFont()
        end
        for _, f in ipairs(receivers) do setFont(f, path, height, flags) end
        local notifications = 0
        b:SetScript('OnEnable', function() notifications = notifications + 1 end)
        s:SetScript('OnHorizontalScroll', function() notifications = notifications + 1 end)
        s:SetScript('OnVerticalScroll', function() notifications = notifications + 1 end)
        local function addon()
            local denied = {
                function() b:SetButtonState(token) end,
                function() b:SetButtonState('NORMAL', lock) end,
                function() b:SetButtonState('NORMAL', false, extra) end,
                function() b:SetEnabled(enabled) end,
                function() b:SetEnabled(true, extra) end,
                function() s:SetHorizontalScroll(offset) end,
                function() s:SetHorizontalScroll(8, extra) end,
                function() s:SetVerticalScroll(offset) end,
                function() s:SetVerticalScroll(9, extra) end,
                function() b.SetButtonState(secretButton, 'NORMAL') end,
                function() b.SetEnabled(secretButton, true) end,
                function() s.SetHorizontalScroll(secretScroll, 8) end,
                function() s.SetVerticalScroll(secretScroll, 9) end,
                function() b:SetButtonState({}, false, extra) end,
            }
            for _, call in ipairs(denied) do
                local ok, err = pcall(call)
                assert(not ok and tostring(err):find('requires an untainted caller', 1, true))
            end
            -- B36 prerequisites only; no claim that path/height validation exists.
            for _, f in ipairs(receivers) do
                assert(not pcall(setFont, f, sp, height, flags))
                assert(not pcall(setFont, f, path, sh, flags))
                assert(not pcall(setFont, f, path, height, sf))
                assert(not pcall(setFont, f, path, height, flags, extra))
                -- Secret extra must win over an earlier nil-path early return.
                assert(not pcall(setFont, f, nil, height, flags, extra))
                if f:GetObjectType() == 'SimpleHTML' then
                    assert(not pcall(f.SetFont, f, st, path, height, flags))
                end
                local p, h, fl = getFont(f)
                assert(p == path and h == height and fl == flags)
            end
            assert(b:GetButtonState() == 'PUSHED' and b:IsEnabled() == false)
            assert(s:GetHorizontalScroll() == 12.5 and s:GetVerticalScroll() == -7.25)
            assert(not issecretvalue(b:GetButtonState()) and not issecretvalue(s:GetVerticalScroll()))
            assert(notifications == 0 and debug.getstacktaint() == 'AspectDeniedAddon')
            -- Ordinary tainted calls remain allowed on ordinary, unprotected widgets.
            b:SetButtonState('NORMAL'); b:SetEnabled(true)
            s:SetHorizontalScroll(8); s:SetVerticalScroll(9)
            assert(notifications == 3 and debug.getstacktaint() == 'AspectDeniedAddon')
        end
        debug.setobjecttaint(addon, 'AspectDeniedAddon'); addon()
        assert(debug.getstacktaint() == nil)
        for _, f in ipairs(receivers) do
            setFont(f, sp, sh, sf, extra)
            local p, h, fl = getFont(f)
            assert(p == path and h == height and fl == flags)
        end
        assert(issecretvalue(sp) and secretunwrap(sp) == path)
        assert(issecretvalue(sh) and secretunwrap(sh) == height)
        assert(issecretvalue(sf) and secretunwrap(sf) == flags)
    "#);
}

#[cfg(feature = "retail-12-0-7")]
#[test]
fn widget_secret_outputs_are_returned_but_opaque_to_tainted_callers() {
    let env = aspect_env();
    aspect_lua(&env, r#"
        local b, s = CreateFrame('Button'), CreateFrame('ScrollFrame')
        b:SetButtonState(secretwrap('PUSHED')); b:SetEnabled(secretwrap(false))
        s:SetHorizontalScroll(secretwrap(-13.75)); s:SetVerticalScroll(secretwrap(82.5))
        local function addon()
            local token, enabled = b:GetButtonState(), b:IsEnabled()
            local h, v = s:GetHorizontalScroll(), s:GetVerticalScroll()
            for _, value in ipairs({token, enabled, h, v}) do
                assert(issecretvalue(value) and not canaccessvalue(value))
                assert(not pcall(secretunwrap, value))
            end
            assert(select('#', b:GetButtonState()) == 1 and select('#', b:IsEnabled()) == 1)
            assert(select('#', s:GetHorizontalScroll()) == 1 and select('#', s:GetVerticalScroll()) == 1)
            assert(not pcall(function() return h + 1 end))
            assert(not pcall(function() return v < 100 end))
            assert(not pcall(function() if enabled then return 1 end end))
            assert(debug.getstacktaint() == 'AspectOpaqueAddon')
        end
        debug.setobjecttaint(addon, 'AspectOpaqueAddon'); addon()
        assert(debug.getstacktaint() == nil)
        assert(secretunwrap(b:GetButtonState()) == 'PUSHED' and secretunwrap(b:IsEnabled()) == false)
        assert(secretunwrap(s:GetHorizontalScroll()) == -13.75 and secretunwrap(s:GetVerticalScroll()) == 82.5)
    "#);
}

#[cfg(feature = "retail-12-0-7")]
#[test]
fn scroll_offset_shared_origin_live_updates_and_equal_value_clearing() {
    let env = aspect_env();
    aspect_lua(&env, r#"
        local s, other = CreateFrame('ScrollFrame'), CreateFrame('ScrollFrame')
        local h, v = secretwrap(14.25), secretwrap(-6.75)
        s:SetHorizontalScroll(h)
        assert(issecretvalue(s:GetHorizontalScroll()) and issecretvalue(s:GetVerticalScroll()))
        assert(secretunwrap(s:GetHorizontalScroll()) == 14.25 and secretunwrap(s:GetVerticalScroll()) == 0)
        s:SetVerticalScroll(v)
        s:SetHorizontalScroll(14.25)
        assert(issecretvalue(s:GetHorizontalScroll()) and issecretvalue(s:GetVerticalScroll()))
        assert(secretunwrap(s:GetHorizontalScroll()) == 14.25 and secretunwrap(s:GetVerticalScroll()) == -6.75)
        s:SetVerticalScroll(-6.75)
        assert(not issecretvalue(s:GetHorizontalScroll()) and not issecretvalue(s:GetVerticalScroll()))
        s:SetVerticalScroll(3, secretwrap(21))
        assert(issecretvalue(s:GetHorizontalScroll()) and secretunwrap(s:GetVerticalScroll()) == 3)
        s:SetVerticalScroll(3)
        assert(not issecretvalue(s:GetHorizontalScroll()))
        assert(other:GetHorizontalScroll() == 0 and other:GetVerticalScroll() == 0)
        assert(not issecretvalue(other:GetVerticalScroll()))
        assert(issecretvalue(h) and secretunwrap(h) == 14.25)
        assert(issecretvalue(v) and secretunwrap(v) == -6.75)
    "#);
}

#[cfg(feature = "retail-12-0-7")]
#[test]
fn scroll_callbacks_observe_committed_offset_and_origin_once() {
    let env = aspect_env();
    aspect_lua(&env, r#"
        local s = CreateFrame('ScrollFrame')
        local log = {}
        s:SetScript('OnHorizontalScroll', function(self)
            local offset = self:GetHorizontalScroll()
            assert(issecretvalue(offset) and secretunwrap(offset) == 12.25)
            assert(issecretvalue(self:GetVerticalScroll()))
            log[#log + 1] = 'h'
        end)
        s:SetScript('OnVerticalScroll', function(self)
            assert(secretunwrap(self:GetVerticalScroll()) == -4.5)
            assert(issecretvalue(self:GetHorizontalScroll()))
            log[#log + 1] = 'v'
        end)
        s:SetHorizontalScroll(secretwrap(12.25))
        s:SetHorizontalScroll(secretwrap(12.25))
        s:SetVerticalScroll(secretwrap(-4.5))
        assert(table.concat(log, ',') == 'h,v')
        s:SetHorizontalScroll(12.25); s:SetVerticalScroll(-4.5)
        assert(table.concat(log, ',') == 'h,v')
        assert(not issecretvalue(s:GetHorizontalScroll()) and not issecretvalue(s:GetVerticalScroll()))
    "#);
}

#[cfg(feature = "retail-12-0-7")]
#[test]
fn host_inputs_are_read_live_and_are_isolated_between_environments() {
    let first = aspect_env();
    let second = aspect_env();
    for env in [&first, &second] {
        aspect_lua(env, "AspectHostButton = CreateFrame('Button', 'AspectHostButton'); AspectHostScroll = CreateFrame('ScrollFrame', 'AspectHostScroll')");
    }
    {
        let mut state = first.state().borrow_mut();
        let button = state.widgets.get_id_by_name("AspectHostButton").unwrap();
        let scroll = state.widgets.get_id_by_name("AspectHostScroll").unwrap();
        let b = state.widgets.get_mut(button).unwrap();
        b.button_state = 1;
        b.attributes.insert(
            "__enabled".to_string(),
            wow_ui_sim::widget::AttributeValue::Boolean(false),
        );
        b.secret_button_state = true;
        let s = state.widgets.get_mut(scroll).unwrap();
        s.scroll_horizontal = 23.75;
        s.scroll_vertical = -18.25;
        s.secret_scroll_vertical = true;
    }
    aspect_lua(&first, r#"
        assert(secretunwrap(AspectHostButton:GetButtonState()) == 'PUSHED')
        assert(issecretvalue(AspectHostButton:IsEnabled()) and secretunwrap(AspectHostButton:IsEnabled()) == false)
        assert(secretunwrap(AspectHostScroll:GetHorizontalScroll()) == 23.75)
        assert(secretunwrap(AspectHostScroll:GetVerticalScroll()) == -18.25)
        assert(issecretvalue(AspectHostScroll:GetHorizontalScroll()))
    "#);
    {
        let mut state = first.state().borrow_mut();
        let button = state.widgets.get_id_by_name("AspectHostButton").unwrap();
        let scroll = state.widgets.get_id_by_name("AspectHostScroll").unwrap();
        let b = state.widgets.get_mut(button).unwrap();
        b.button_state = 0;
        b.attributes.insert(
            "__enabled".to_string(),
            wow_ui_sim::widget::AttributeValue::Boolean(true),
        );
        b.secret_button_state = false;
        let s = state.widgets.get_mut(scroll).unwrap();
        s.scroll_horizontal = -6.5;
        s.secret_scroll_vertical = false;
    }
    aspect_lua(&first, r#"
        assert(AspectHostButton:GetButtonState() == 'NORMAL' and not issecretvalue(AspectHostButton:IsEnabled()))
        assert(AspectHostScroll:GetHorizontalScroll() == -6.5 and AspectHostScroll:GetVerticalScroll() == -18.25)
        assert(not issecretvalue(AspectHostScroll:GetVerticalScroll()))
    "#);
    aspect_lua(&second, r#"
        assert(AspectHostButton:GetButtonState() == 'NORMAL' and AspectHostButton:IsEnabled())
        assert(AspectHostScroll:GetHorizontalScroll() == 0 and AspectHostScroll:GetVerticalScroll() == 0)
        assert(not issecretvalue(AspectHostButton:GetButtonState()))
        assert(not issecretvalue(AspectHostScroll:GetHorizontalScroll()))
    "#);
}

#[cfg(not(feature = "retail-12-0-7"))]
#[test]
fn earlier_epochs_keep_plain_widget_outputs() {
    let env = aspect_env();
    env.eval::<()>("EarlierAspectButton = CreateFrame('Button', 'EarlierAspectButton'); EarlierAspectScroll = CreateFrame('ScrollFrame', 'EarlierAspectScroll')")
        .expect("create earlier-profile fixtures");
    {
        let mut state = env.state().borrow_mut();
        let button = state.widgets.get_id_by_name("EarlierAspectButton").unwrap();
        let scroll = state.widgets.get_id_by_name("EarlierAspectScroll").unwrap();
        let b = state.widgets.get_mut(button).unwrap();
        b.secret_button_state = true;
        b.secret_button_enabled = true;
        let s = state.widgets.get_mut(scroll).unwrap();
        s.secret_scroll_horizontal = true;
        s.secret_scroll_vertical = true;
    }
    env.eval::<()>(r#"
        local b, s = EarlierAspectButton, EarlierAspectScroll
        b:SetButtonState('PUSHED'); b:SetEnabled(false)
        s:SetHorizontalScroll(16.25); s:SetVerticalScroll(-3.75)
        assert(b:GetButtonState() == 'PUSHED' and b:IsEnabled() == false)
        assert(s:GetHorizontalScroll() == 16.25 and s:GetVerticalScroll() == -3.75)
        assert(not issecretvalue(b:GetButtonState()) and not issecretvalue(b:IsEnabled()))
        assert(not issecretvalue(s:GetHorizontalScroll()) and not issecretvalue(s:GetVerticalScroll()))
    "#).expect("earlier profile unchanged");
}
