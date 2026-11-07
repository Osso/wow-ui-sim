#![cfg(any(feature = "client-retail", feature = "client-ptr"))]

use wow_ui_sim::lua_api::WowLuaEnv;

fn load_shared_tooltip_templates(env: &WowLuaEnv) {
    let addons = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let source =
        std::fs::read_to_string(addons.join("Blizzard_SharedXML/SharedTooltipTemplates.lua"))
            .unwrap();
    env.exec(&source).unwrap();
}

#[test]
fn tooltip_lifecycle_original_insert_frame_uses_default_spacing() {
    let env = WowLuaEnv::new().unwrap();
    load_shared_tooltip_templates(&env);
    env.exec(r#"
        Round = function(value) return math.floor(value + 0.5) end
        local tip = CreateFrame("GameTooltip", "LifecycleSpacingTip", UIParent)
        tip:AddLine("first")
        tip:AddLine("second")
        local height = tip:GetLeftLine(2):GetLineHeight()
        local embedded = CreateFrame("Frame", nil, UIParent)
        embedded:SetSize(10, height * 2 + 1)
        local before = tip:NumLines()
        local reserved = GameTooltip_InsertFrame(tip, embedded, 0)
        local h = Round(height)
        local expected = math.ceil(Round(height * 2 + 1) / (h + 2))
        assert(tip:NumLines() - before == expected, "default spacing must reserve correct lines")
        assert(reserved == expected * h + (expected - 1) * 2, "default spacing must contribute to reserved height")
        assert(select('#', tip:GetCustomLineSpacing()) == 0, "unset spacing returns no values")
        tip:SetCustomLineSpacing(0)
        assert(tip:GetCustomLineSpacing() == 0, "explicit zero spacing remains an override")
    "#).unwrap();
}

#[test]
fn tooltip_lifecycle_explicit_zero_spacing_reaches_insert_frame() {
    let env = WowLuaEnv::new().unwrap();
    load_shared_tooltip_templates(&env);
    env.exec(r#"
        Round = function(value) return math.floor(value + 0.5) end

        local function reserve(tip)
            tip:AddLine("first")
            tip:AddLine("second")
            local height = Round(tip:GetLeftLine(2):GetLineHeight())
            local embedded = CreateFrame("Frame", nil, UIParent)
            embedded:SetSize(10, height * 2 + 1)
            local before = tip:NumLines()
            local reserved = GameTooltip_InsertFrame(tip, embedded, 0)
            return tip:NumLines() - before, reserved, height
        end

        -- Default tooltip: no override, so the consumer's `or 2` selects 2.
        local defaultTip = CreateFrame("GameTooltip", "DefaultLifecycleSpacingTip", UIParent)
        assert(select('#', defaultTip:GetCustomLineSpacing()) == 0, "fresh tooltip has no override")
        local defaultRows, defaultReserved, defaultHeight = reserve(defaultTip)
        assert(defaultRows == 2, "unset spacing must reserve two rows")
        assert(defaultReserved == defaultHeight * 2 + 2, "unset spacing must reserve a two-unit gap")

        -- Explicit zero tooltip: the consumer keeps 0 (truthy in Lua) and reserves rows with no gap.
        local zeroTip = CreateFrame("GameTooltip", "ZeroLifecycleSpacingTip", UIParent)
        zeroTip:SetCustomLineSpacing(0)
        assert(zeroTip:GetCustomLineSpacing() == 0, "explicit zero override must read back as 0")
        local zeroRows, zeroReserved, zeroHeight = reserve(zeroTip)
        assert(zeroRows == 3, "explicit zero spacing must reserve one row more than the default")
        assert(zeroReserved == zeroHeight * 3, "explicit zero spacing must contribute no gap")

        -- Explicit nonzero tooltip reads its own stored value.
        local wideTip = CreateFrame("GameTooltip", "WideLifecycleSpacingTip", UIParent)
        wideTip:SetCustomLineSpacing(5)
        assert(wideTip:GetCustomLineSpacing() == 5, "explicit five override must read back as 5")

        -- The override is state of the receiving tooltip only.
        assert(select('#', defaultTip:GetCustomLineSpacing()) == 0, "another tooltip keeps its unset state")
        assert(defaultTip:GetCustomLineSpacing() == nil, "unset tooltip publishes nothing")
    "#).unwrap();
}
