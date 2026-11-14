use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn game_tooltip_minimum_width_returns_two_values_and_round_trips() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r##"
        local width, forced = GameTooltip:GetMinimumWidth()
        assert(select("#", GameTooltip:GetMinimumWidth()) == 2)
        assert(width == 0 and forced == false)

        GameTooltip:SetMinimumWidth(173, true)
        width, forced = GameTooltip:GetMinimumWidth()
        assert(select("#", GameTooltip:GetMinimumWidth()) == 2)
        assert(width == 173 and forced == true)

        GameTooltip:SetMinimumWidth(91, false)
        width, forced = GameTooltip:GetMinimumWidth()
        assert(width == 91 and forced == false)
        "##,
    )
    .unwrap();
}

#[test]
fn game_tooltip_add_font_strings_stores_pair_without_adding_a_line() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r##"
        local left = UIParent:CreateFontString("TooltipPairLeft", "OVERLAY")
        local right = UIParent:CreateFontString("TooltipPairRight", "OVERLAY")
        assert(select("#", GameTooltip:AddFontStrings(left, right)) == 0)
        assert(GameTooltip:NumLines() == 0)
        assert(left:GetParent() == UIParent and right:GetParent() == UIParent)
        "##,
    )
    .unwrap();

    let state = env.state().borrow();
    let tooltip_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let left_id = state.widgets.get_id_by_name("TooltipPairLeft").unwrap();
    let right_id = state.widgets.get_id_by_name("TooltipPairRight").unwrap();
    assert_eq!(
        state.tooltips[&tooltip_id].font_string_pairs,
        vec![(left_id, right_id)]
    );
}

#[test]
fn game_tooltip_method_conversion_errors_use_native_reasons() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r##"
        local ok, err = pcall(GameTooltip.GetMinimumWidth, "not a tooltip")
        assert(not ok)
        assert(err:find(
            "calling 'GetMinimumWidth' on bad self (Attempt to find 'this' in non-table object (used '.' instead of ':' ?))",
            1,
            true
        ), err)
        ok, err = pcall(GameTooltip.GetMinimumWidth)
        assert(not ok)
        assert(err:find("Usage: local width, forced = self:GetMinimumWidth()", 1, true), err)

        local left = UIParent:CreateFontString("TooltipErrorLeft", "OVERLAY")
        local right = UIParent:CreateFontString("TooltipErrorRight", "OVERLAY")
        ok, err = pcall(GameTooltip.AddFontStrings, {}, left, right)
        assert(not ok)
        assert(err:find(
            "calling 'AddFontStrings' on bad self (Attempt to find 'this' in non-framescript object)",
            1,
            true
        ), err)
        ok, err = pcall(GameTooltip.AddFontStrings, nil, left, right)
        assert(not ok)
        assert(err:find(
            "Usage: self:AddFontStrings(leftFontString, rightFontString)",
            1,
            true
        ), err)

        ok, err = pcall(GameTooltip.AddFontStrings, GameTooltip, "not a FontString", right)
        assert(not ok)
        assert(err:find(
            "Usage: self:AddFontStrings(leftFontString, rightFontString) - Attempt to find 'this' in non-table object (used '.' instead of ':' ?)",
            1,
            true
        ), err)

        local wrongType = CreateFrame("Frame", "TooltipErrorWrongType")
        ok, err = pcall(GameTooltip.AddFontStrings, GameTooltip, wrongType, right)
        assert(not ok)
        assert(err:find(
            "Usage: self:AddFontStrings(leftFontString, rightFontString) - Wrong object type for function",
            1,
            true
        ), err)
        ok, err = pcall(GameTooltip.AddFontStrings, GameTooltip, left)
        assert(not ok)
        assert(err:find(
            "Usage: self:AddFontStrings(leftFontString, rightFontString)",
            1,
            true
        ), err)
        "##,
    )
    .unwrap();
}

#[cfg(all(feature = "optional-tests", feature = "client-retail"))]
#[test]
fn retail_shared_tooltip_insert_frame_preserves_minimum_width_consumer() {
    use wow_ui_sim::loader::find_toc_file;
    use wow_ui_sim::paths::default_blizzard_ui_addons_path;

    let addons = default_blizzard_ui_addons_path().unwrap();
    let toc = find_toc_file(&addons.join("Blizzard_SharedXML"))
        .expect("profile-selected Blizzard_SharedXML TOC");
    assert_eq!(
        toc.file_name().and_then(|name| name.to_str()),
        Some("Blizzard_SharedXML.toc")
    );
    let toc_body = std::fs::read_to_string(&toc).unwrap();
    assert!(
        toc_body
            .lines()
            .any(|line| line.trim() == "SharedTooltipTemplates.lua")
    );

    let env = crate::tooltip_full_env_helpers::setup_full_env();
    env.exec(
        r##"
        assert(type(GameTooltip_InsertFrame) == "function")
        local source = debug.getinfo(GameTooltip_InsertFrame, "S").source
        assert(source:find("SharedTooltipTemplates.lua", 1, true), source)

        local tooltip = GameTooltip
        tooltip:SetOwner(UIParent, "ANCHOR_NONE")
        tooltip:AddLine("Header")
        tooltip:AddLine("Body")
        tooltip:SetMinimumWidth(0, false)

        local inserted = CreateFrame("Frame", "TooltipInsertedFrame", UIParent)
        inserted:SetSize(245, 20)
        GameTooltip_InsertFrame(tooltip, inserted, 0)

        assert(tooltip:GetMinimumWidth() >= inserted:GetWidth())
        assert(inserted:GetParent() == tooltip)
        assert(tooltip:NumLines() > 2)
        "##,
    )
    .unwrap();
}
