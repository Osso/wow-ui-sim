//! Bounded simulator policy for secret-origin aura widget readouts.
#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn aura_secret_shown_preserves_false_and_restricts_ancestor_readout() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local parent = CreateFrame('Frame')
        local child = CreateFrame('Frame', nil, parent)
        local hidden = secretwrap(false)
        parent:SetShown(hidden)
        assert(not parent:IsShown() and not child:IsVisible())
        local function addon()
            assert(not pcall(parent.IsShown, parent))
            assert(not pcall(child.IsVisible, child))
            assert(not pcall(parent.SetShown, parent, hidden))
        end
        debug.setobjecttaint(addon, 'SecretDisplayProbe')
        addon()
        assert(not parent:IsShown(), 'rejected mutation must be atomic')
        parent:SetShown(newproxy())
        assert(parent:IsShown(), 'ordinary userdata keeps Lua truthiness')
        parent:SetShown(true)
        local function plain()
            assert(parent:IsShown() and child:IsVisible())
        end
        debug.setobjecttaint(plain, 'SecretDisplayProbe')
        plain()
    "#,
    )
    .unwrap();
}

#[test]
fn aura_secret_text_rejects_tainted_reads_and_failed_writes_leave_text() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local label = CreateFrame('Frame'):CreateFontString()
        local wrapped = secretwrap('secret duration')
        label:SetText(wrapped)
        assert(label:GetText() == 'secret duration')
        local button = CreateFrame('Button')
        button:SetText('plain button')
        local buttonLabel = button:GetFontString()
        buttonLabel:SetText(wrapped)
        local function addon()
            assert(not pcall(label.GetText, label))
            assert(not pcall(button.GetText, button), 'button must not expose secret child text')
            assert(not pcall(label.SetText, label, wrapped))
        end
        debug.setobjecttaint(addon, 'SecretDisplayProbe')
        addon()
        assert(label:GetText() == 'secret duration')
        assert(not pcall(label.SetText, label, newproxy()))
        assert(label:GetText() == 'secret duration')
        label:SetText('plain')
        local function plain() assert(label:GetText() == 'plain') end
        debug.setobjecttaint(plain, 'SecretDisplayProbe')
        plain()
    "#,
    )
    .unwrap();
}

#[test]
fn aura_secret_duration_widget_handoffs_keep_readout_restricted() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local duration = C_DurationUtil.CreateDuration()
        duration:SetTimeFromStart(secretwrap(20, 30, 1))
        local cooldown = CreateFrame('Cooldown')
        local bar = CreateFrame('StatusBar')
        cooldown:SetCooldownFromDurationObject(duration)
        bar:SetTimerDuration(duration)
        local start, span = cooldown:GetCooldownTimes()
        assert(start == 20000 and span == 30000)
        assert(bar:GetTimerDuration() == duration)
        local function addon()
            for _, method in ipairs({'GetCooldownTimes', 'GetCooldownDuration', 'GetCooldownDisplayDuration'}) do
                assert(not pcall(cooldown[method], cooldown), method)
            end
            for _, method in ipairs({'GetTimerDuration', 'GetValue', 'GetMinMaxValues', 'GetInterpolatedValue'}) do
                assert(not pcall(bar[method], bar), method)
            end
            assert(not pcall(cooldown.SetCooldownFromDurationObject, cooldown, duration))
            assert(not pcall(bar.SetTimerDuration, bar, duration))
        end
        debug.setobjecttaint(addon, 'SecretDisplayProbe')
        addon()
        assert(cooldown:GetCooldownDisplayDuration() == 30000)
        local plain = C_DurationUtil.CreateDuration()
        plain:SetTimeFromStart(10, 5)
        cooldown:SetCooldownFromDurationObject(plain)
        bar:SetTimerDuration(plain)
        local function read_plain()
            assert(cooldown:GetCooldownDisplayDuration() == 5000)
            assert(bar:GetTimerDuration() == plain)
            assert(type(bar:GetValue()) == 'number')
        end
        debug.setobjecttaint(read_plain, 'SecretDisplayProbe')
        read_plain()
    "#).unwrap();
}
