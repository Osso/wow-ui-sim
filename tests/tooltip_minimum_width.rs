#![cfg(all(
    feature = "client-retail",
    feature = "retail-12-1-5",
    feature = "forbidden-aspects"
))]

use wow_ui_sim::lua_api::WowLuaEnv;

const CONVERSION_ERRORS_LUA: &str = r#"
local ok, err = pcall(GameTooltip.GetMinimumWidth, "not a tooltip")
assert(not ok and err:find(
    "Attempt to find 'this' in non-table object (used '.' instead of ':' ?)", 1, true
), err)
ok, err = pcall(GameTooltip.GetMinimumWidth)
assert(not ok and err:find(
    "Usage: local width, forced = self:GetMinimumWidth()", 1, true
), err)

local left = UIParent:CreateFontString("TooltipErrorLeft", "OVERLAY")
local right = UIParent:CreateFontString("TooltipErrorRight", "OVERLAY")
ok, err = pcall(GameTooltip.AddFontStrings, {}, left, right)
assert(not ok and err:find("Attempt to find 'this' in non-framescript object", 1, true), err)
ok, err = pcall(GameTooltip.AddFontStrings, nil, left, right)
assert(not ok and err:find(
    "Usage: self:AddFontStrings(leftFontString, rightFontString)", 1, true
), err)
ok, err = pcall(GameTooltip.AddFontStrings, GameTooltip, "not a FontString", right)
assert(not ok and err:find(
    "Attempt to find 'this' in non-table object (used '.' instead of ':' ?)", 1, true
), err)

local wrongType = CreateFrame("Frame", "TooltipErrorWrongType")
ok, err = pcall(GameTooltip.AddFontStrings, GameTooltip, wrongType, right)
assert(not ok and err:find("Wrong object type for function", 1, true), err)
ok, err = pcall(GameTooltip.AddFontStrings, GameTooltip, left)
assert(not ok and err:find(
    "Usage: self:AddFontStrings(leftFontString, rightFontString)", 1, true
), err)
"#;

fn create_font_string_pair(env: &WowLuaEnv) -> (u64, u64, u64) {
    env.exec(
        r#"
        TooltipPairLeft = UIParent:CreateFontString("TooltipPairLeft", "OVERLAY")
        TooltipPairRight = UIParent:CreateFontString("TooltipPairRight", "OVERLAY")
        "#,
    )
    .unwrap();
    let state = env.state().borrow();
    (
        state.widgets.get_id_by_name("GameTooltip").unwrap(),
        state.widgets.get_id_by_name("TooltipPairLeft").unwrap(),
        state.widgets.get_id_by_name("TooltipPairRight").unwrap(),
    )
}

fn set_forbidden_aspect(env: &WowLuaEnv, frame_id: u64, aspect_name: &str) {
    let expression = format!("return Enum.ForbiddenAspect.{aspect_name}");
    let aspect = env.eval::<f64>(&expression).unwrap() as u64;
    env.state()
        .borrow_mut()
        .widgets
        .get_mut(frame_id)
        .unwrap()
        .forbidden_aspects |= aspect;
}

fn attempt_tainted_add_font_strings(env: &WowLuaEnv) -> String {
    env.eval::<String>(
        r#"
        forceinsecure()
        local ok, err = pcall(
            GameTooltip.AddFontStrings,
            GameTooltip,
            TooltipPairLeft,
            TooltipPairRight
        )
        assert(not ok, "forbidden AddFontStrings call succeeded")
        return tostring(err)
        "#,
    )
    .unwrap()
}

fn assert_no_font_string_pairs(env: &WowLuaEnv, tooltip_id: u64) {
    assert!(
        env.state()
            .borrow()
            .tooltips
            .get(&tooltip_id)
            .is_none_or(|tooltip| tooltip.font_string_pairs.is_empty())
    );
}

const APPEND_SECOND_FONT_STRING_PAIR_LUA: &str = r##"
TooltipPairLeft2 = UIParent:CreateFontString("TooltipPairLeft2", "OVERLAY")
TooltipPairRight2 = UIParent:CreateFontString("TooltipPairRight2", "OVERLAY")
assert(select("#", GameTooltip:AddFontStrings(TooltipPairLeft2, TooltipPairRight2)) == 0)
assert(GameTooltip:NumLines() == 0)
assert(TooltipPairLeft:GetParent() == UIParent)
assert(TooltipPairRight:GetParent() == UIParent)
"##;

const RETAIL_CONSUMER_LUA: &str = r##"
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
"##;

fn seed_first_pair_layout_state(env: &WowLuaEnv, tooltip_id: u64) {
    let mut state = env.state().borrow_mut();
    let tooltip = state.tooltips.get_mut(&tooltip_id).unwrap();
    tooltip.font_string_line_states[0] = true;
    tooltip.font_string_numeric_slots[0] = 47.0;
    tooltip.font_string_layout_progress = 3;
}

fn assert_two_pair_layout_state(env: &WowLuaEnv, tooltip_id: u64, left_id: u64, right_id: u64) {
    let state = env.state().borrow();
    let tooltip = &state.tooltips[&tooltip_id];
    let left2_id = state.widgets.get_id_by_name("TooltipPairLeft2").unwrap();
    let right2_id = state.widgets.get_id_by_name("TooltipPairRight2").unwrap();
    assert_eq!(
        tooltip.font_string_pairs,
        vec![(left_id, right_id), (left2_id, right2_id)]
    );
    assert_eq!(tooltip.font_string_line_states, vec![true, false]);
    assert_eq!(tooltip.font_string_numeric_slots, vec![47.0, 0.0]);
    assert_eq!(tooltip.font_string_layout_progress, 0);
}

fn assert_retail_shared_tooltip_source_selected() {
    use wow_ui_sim::loader::find_toc_file;
    use wow_ui_sim::paths::default_blizzard_ui_addons_path;

    let addons = default_blizzard_ui_addons_path().unwrap();
    let toc = find_toc_file(&addons.join("Blizzard_SharedXML"))
        .expect("profile-selected Blizzard_SharedXML TOC");
    assert_eq!(
        toc.file_name().and_then(|name| name.to_str()),
        Some("Blizzard_SharedXML.toc")
    );
    let body = std::fs::read_to_string(&toc).unwrap();
    assert!(
        body.lines()
            .any(|line| line.trim() == "SharedTooltipTemplates.lua")
    );
}

fn run_retail_shared_tooltip_consumer() {
    let env = crate::tooltip_full_env_helpers::setup_full_env();
    env.exec(RETAIL_CONSUMER_LUA).unwrap();
}

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
fn game_tooltip_add_font_strings_resets_pair_storage_without_adding_a_line() {
    let env = WowLuaEnv::new().unwrap();
    let (tooltip_id, left_id, right_id) = create_font_string_pair(&env);
    env.exec("GameTooltip:AddFontStrings(TooltipPairLeft, TooltipPairRight)")
        .unwrap();
    seed_first_pair_layout_state(&env, tooltip_id);
    env.exec(APPEND_SECOND_FONT_STRING_PAIR_LUA).unwrap();
    assert_two_pair_layout_state(&env, tooltip_id, left_id, right_id);
}

#[test]
fn game_tooltip_method_conversion_errors_include_verified_reasons() {
    WowLuaEnv::new()
        .unwrap()
        .exec(CONVERSION_ERRORS_LUA)
        .unwrap();
}

#[test]
fn game_tooltip_add_font_strings_checks_change_parent_before_storage() {
    let env = WowLuaEnv::new().unwrap();
    let (tooltip_id, left_id, right_id) = create_font_string_pair(&env);
    set_forbidden_aspect(&env, right_id, "ChangeParent");

    let error = attempt_tainted_add_font_strings(&env);

    assert!(error.contains("ChangeParent"), "{error}");
    assert_no_font_string_pairs(&env, tooltip_id);
}

#[test]
fn game_tooltip_add_font_strings_checks_inherited_aspects_before_storage() {
    let env = WowLuaEnv::new().unwrap();
    let (tooltip_id, _, _) = create_font_string_pair(&env);
    set_forbidden_aspect(&env, tooltip_id, "QueryFocus");

    let error = attempt_tainted_add_font_strings(&env);

    assert!(
        error.contains(
            "Reparenting disallowed as child object would inherit forbidden aspects: QueryFocus"
        ),
        "{error}"
    );
    assert_no_font_string_pairs(&env, tooltip_id);
}

#[test]
fn game_tooltip_add_font_strings_checks_remove_secret_aspects_before_storage() {
    let env = WowLuaEnv::new().unwrap();
    let (tooltip_id, _, right_id) = create_font_string_pair(&env);
    set_forbidden_aspect(&env, right_id, "RemoveSecretAspects");

    let error = attempt_tainted_add_font_strings(&env);

    assert!(error.contains("RemoveSecretAspects"), "{error}");
    assert_no_font_string_pairs(&env, tooltip_id);
}

#[cfg(all(
    feature = "optional-tests",
    feature = "client-retail",
    feature = "retail-12-1-5"
))]
#[test]
fn retail_shared_tooltip_insert_frame_preserves_minimum_width_consumer() {
    assert_retail_shared_tooltip_source_selected();
    run_retail_shared_tooltip_consumer();
}
