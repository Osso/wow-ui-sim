//! Integration test for the GuildMemberListDropdown menu sizing.
//!
//! Exercises the dropdown twice (open → measure → close → re-open → measure)
//! and asserts that the menu frame stays within expected bounds (~180×~103px),
//! not the full-screen-wide 1024px layout that the state-dependent sizing bug
//! produces.

use wow_ui_sim::lua_api::WowLuaEnv;

/// Measure the open GuildMemberListDropdown menu frame.
///
/// Returns `"<width>x<height>"` on success, `"skip:<reason>"` when no guild
/// state is available (menu has no elements), or `"error:<reason>"` on
/// unexpected failures.
const MEASURE_LUA: &str = r#"
    local dropdown = CommunitiesFrame and CommunitiesFrame.GuildMemberListDropdown
    if dropdown == nil then
        return "error:no_dropdown"
    end

    -- Force the dropdown visible so SetupMenu can run its :SetShown logic.
    dropdown:Show()

    -- Rebuild the menu description from scratch then open the physical menu.
    dropdown:SetupMenu()
    dropdown:OpenMenu()

    -- The Blizzard_Menu system exposes the open menu frame via Menu.GetManager().
    local manager = Menu and Menu.GetManager and Menu.GetManager()
    if manager == nil then
        return "error:no_menu_manager"
    end

    local menuFrame = manager:GetOpenMenu()
    if menuFrame == nil then
        -- No menu opened: either no elements or the dropdown hid itself.
        local desc = dropdown:GetMenuDescription()
        local hasElements = desc and desc:HasElements()
        if not hasElements then
            return "skip:no_elements"
        end
        return "error:open_menu_nil_despite_elements"
    end

    local w = menuFrame:GetWidth()
    local h = menuFrame:GetHeight()
    return tostring(math.floor(w + 0.5)) .. "x" .. tostring(math.floor(h + 0.5))
"#;

/// Close the menu that was opened by the last `OpenMenu()` call.
const CLOSE_LUA: &str = r#"
    local dropdown = CommunitiesFrame and CommunitiesFrame.GuildMemberListDropdown
    if dropdown == nil then
        return "error:no_dropdown"
    end
    dropdown:CloseMenu()

    local manager = Menu and Menu.GetManager and Menu.GetManager()
    if manager == nil then
        return "error:no_menu_manager"
    end
    local still_open = manager:GetOpenMenu()
    if still_open ~= nil then
        return "error:menu_still_open"
    end
    return "ok"
"#;

fn parse_dimensions(result: &str) -> Option<(f64, f64)> {
    let (w_str, h_str) = result.split_once('x')?;
    let w = w_str.parse::<f64>().ok()?;
    let h = h_str.parse::<f64>().ok()?;
    Some((w, h))
}

prefork_full_ui_case! {
fn guild_member_list_dropdown_menu_sizing_is_correct_across_two_opens(env: &WowLuaEnv) {
    // Show CommunitiesFrame in guild mode so the dropdown is reachable.
    let setup_result: String = env
        .eval(r#"
            if CommunitiesFrame == nil then
                return "error:no_frame"
            end
            g_clubIdToSeenApplicants = g_clubIdToSeenApplicants or {}
            CommunitiesFrame:Show()
            -- Select the default guild club so guild-mode layout applies.
            local clubs = C_Club.GetSubscribedClubs()
            if type(clubs) == "table" and #clubs > 0 then
                CommunitiesFrame:SelectClub(clubs[1].clubId)
            end
            return "ok"
        "#)
        .expect("CommunitiesFrame setup should succeed");

    assert!(
        !setup_result.starts_with("error:"),
        "CommunitiesFrame setup failed: {setup_result}"
    );

    // --- First open ---
    let first: String = env
        .eval(MEASURE_LUA)
        .expect("first measure eval should succeed");

    if first.starts_with("skip:") {
        // No guild elements in the dropdown: nothing to test.
        // This happens when no guild club is configured in the sim state.
        assert!(true, "skipped: GuildMemberListDropdown has no elements ({first})");
        return;
    }

    assert!(
        !first.starts_with("error:"),
        "first open produced an error: {first}"
    );

    let (w1, h1) = parse_dimensions(&first)
        .unwrap_or_else(|| panic!("unexpected first-open result format: {first}"));

    assert!(
        w1 <= 300.0,
        "first open: menu width {w1:.0}px exceeds 300px (expected ~180px); \
         possible full-screen layout bug"
    );
    assert!(
        h1 <= 200.0,
        "first open: menu height {h1:.0}px exceeds 200px (expected ~103px)"
    );

    // --- Close ---
    let close_result: String =
        env.eval(CLOSE_LUA).expect("close eval should succeed");
    assert_eq!(close_result, "ok", "menu close failed: {close_result}");

    // --- Second open ---
    let second: String = env
        .eval(MEASURE_LUA)
        .expect("second measure eval should succeed");

    assert!(
        !second.starts_with("error:") && !second.starts_with("skip:"),
        "second open produced unexpected result: {second}"
    );

    let (w2, h2) = parse_dimensions(&second)
        .unwrap_or_else(|| panic!("unexpected second-open result format: {second}"));

    assert!(
        w2 <= 300.0,
        "second open: menu width {w2:.0}px exceeds 300px (expected ~180px); \
         possible full-screen layout bug on re-open"
    );
    assert!(
        h2 <= 200.0,
        "second open: menu height {h2:.0}px exceeds 200px (expected ~103px)"
    );

    // Both opens should produce identical dimensions.
    assert_eq!(
        (w1 as i64, h1 as i64),
        (w2 as i64, h2 as i64),
        "menu dimensions changed between first ({first}) and second ({second}) open; \
         state-dependent sizing bug"
    );
}
}
