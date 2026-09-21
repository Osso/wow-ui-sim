//! Bounded simulator policy for secret-origin aura widget readouts.

use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(feature = "client-wowforever")]
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

#[cfg(feature = "client-wowforever")]
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

#[cfg(feature = "client-wowforever")]
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

#[cfg(feature = "client-wowforever")]
#[test]
fn aura_secret_geometry_decodes_native_arguments_and_keeps_mixed_origins() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local f = CreateFrame('Frame', nil, UIParent)
        f:SetPoint(secretwrap('CENTER', UIParent, 'CENTER', 3, 4))
        f:SetSize(secretwrap(42, 24))
        local point, relative, relativePoint, x, y = f:GetPoint()
        assert(point == 'CENTER' and relative == UIParent and relativePoint == 'CENTER')
        assert(x == 3 and y == 4 and f:GetWidth() == 42 and f:GetHeight() == 24)
        local child = CreateFrame('Frame', nil, f)
        child:SetAllPoints(f)
        local relativeChild = CreateFrame('Frame', nil, UIParent)
        relativeChild:SetPoint('CENTER', f, 'CENTER')
        relativeChild:SetSize(3, 4)
        local wrappedWidth, wrappedHeight = secretwrap(99, 88)
        local wrappedPoint = secretwrap('LEFT')
        local function addon()
            for _, method in ipairs({'GetPoint','GetNumPoints','GetSize','GetWidth','GetHeight',
                'GetRect','GetScaledRect','GetLeft','GetRight','GetTop','GetBottom','GetCenter'}) do
                assert(not pcall(f[method], f), method)
            end
            assert(not pcall(child.GetRect, child), 'parent-derived geometry must remain secret')
            assert(not pcall(relativeChild.GetCenter, relativeChild), 'anchor-derived geometry must remain secret')
            assert(not pcall(f.SetPoint, f, wrappedPoint, UIParent, 'LEFT', 9, 8))
            assert(not pcall(f.SetSize, f, wrappedWidth, wrappedHeight))
            assert(not pcall(f.SetSize, f, 99, wrappedHeight))
        end
        debug.setobjecttaint(addon, 'SecretGeometryProbe')
        addon()
        assert(f:GetWidth() == 42 and f:GetHeight() == 24)
        assert(select(4, f:GetPoint()) == 3)
        assert(not pcall(f.SetPoint, f, secretwrap('BAD_POINT'), UIParent))
        assert(select(4, f:GetPoint()) == 3)
        f:ClearAllPoints()
        f:SetPoint('CENTER', UIParent, 'CENTER', 0, 0)
        f:SetWidth(42)
        local function mixed() assert(not pcall(f.GetHeight, f)) end
        debug.setobjecttaint(mixed, 'SecretGeometryProbe')
        mixed()
        f:SetHeight(24)
        local function public() assert(f:GetWidth() == 42 and f:GetHeight() == 24) end
        debug.setobjecttaint(public, 'SecretGeometryProbe')
        public()
        f:SetPoint(secretwrap('TOPLEFT', nil, nil, 7, 8))
        assert(select(4, f:GetPointByName('TOPLEFT')) == 7)
        f:SetPoint('TOPLEFT', nil, nil, 7, 8)
        public()
        f:SetPoint(secretwrap('CENTER', 11, 12))
        assert(select(4, f:GetPointByName('CENTER')) == 11)
    "#).unwrap();
}

#[cfg(feature = "client-wowforever")]
#[test]
fn aura_secret_texture_replaces_and_clears_without_plain_readout() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local texture = CreateFrame('Frame'):CreateTexture()
        texture:SetTexture(135907)
        local wrapped = secretwrap(136243)
        texture:SetTexture(wrapped)
        assert(texture:GetTexture() == 136243 and texture:GetTextureFileID() == 136243)
        local function addon()
            for _, method in ipairs({'GetTexture','GetTextureFileID','GetTextureFilePath','GetAtlas'}) do
                assert(not pcall(texture[method], texture), method)
            end
            assert(not pcall(texture.SetTexture, texture, wrapped))
        end
        debug.setobjecttaint(addon, 'SecretTextureProbe')
        addon()
        assert(texture:GetTexture() == 136243)
        texture:SetTexture({})
        addon()
        assert(texture:GetTexture() == 136243, 'ignored input must retain old source and secrecy')
        texture:SetTexture(secretwrap(nil))
        assert(texture:GetTexture() == nil and texture:GetTextureFileID() == nil)
        addon()
        texture:SetTexture(secretwrap('Interface\\Icons\\Spell_Holy_FlashHeal'))
        assert(texture:GetTexture() ~= nil)
        texture:SetTexture(135907)
        local function public() assert(texture:GetTexture() == 135907) end
        debug.setobjecttaint(public, 'SecretTextureProbe')
        public()
        texture:SetTexture(wrapped)
        texture:SetColorTexture(1, 0, 0, 1)
        local function cleared() assert(texture:GetTexture() == nil) end
        debug.setobjecttaint(cleared, 'SecretTextureProbe')
        cleared()
    "#).unwrap();
}

#[test]
fn aura_geometry_texture_plain_handoffs_preserve_values() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local f = CreateFrame('Frame', nil, UIParent)
        f:SetPoint('CENTER', UIParent, 'CENTER', 3, 4)
        f:SetSize(42, 24)
        assert(f:GetWidth() == 42 and f:GetHeight() == 24)
        assert(select(4, f:GetPoint()) == 3)
        f:SetWidth(43)
        f:SetHeight(25)
        assert(f:GetWidth() == 43 and f:GetHeight() == 25)
        f:SetPoint('CENTER', 7, 8)
        assert(select(4, f:GetPoint()) == 7)
        local texture = f:CreateTexture()
        texture:SetTexture(135907)
        assert(texture:GetTexture() == 135907)
        texture:SetTexture(nil)
        assert(texture:GetTexture() == nil)
    "#,
    )
    .unwrap();
}
