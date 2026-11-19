#![cfg(all(feature = "optional-tests", feature = "client-retail"))]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

use wow_ui_sim::c_api::{
    ItemInteractionTooltipInput, ItemInteractionTooltipLine, SlottedKeystoneKeyFloor,
    SlottedKeystoneTooltipInput, c_challenge_mode::KeystoneLocation,
    c_item_interaction::ItemInteractionInfo,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{CursorInfo, CursorItemOrigin};

fn add_bag_item(env: &WowLuaEnv, bag: u32, slot: u32, item_id: u32) -> String {
    env.exec(&format!("A_Admin.AddBagItem({bag}, {slot}, {item_id}, 1)"))
        .unwrap();
    env.eval(&format!(
        "return C_Item.GetItemGUID({{bagID={bag}, slotIndex={slot}}})"
    ))
    .unwrap()
}

fn set_interaction_input(
    env: &WowLuaEnv,
    interaction_type: i32,
    object_kind: i32,
    selected_item_guid: &str,
    conversion_output_item_id: Option<u32>,
    cleansing_rows: Vec<ItemInteractionTooltipLine>,
) {
    let mut sim = env.state().borrow_mut();
    sim.item_interaction = Some(ItemInteractionInfo {
        interaction_type,
        ..Default::default()
    });
    sim.item_interaction_tooltip_input = Some(ItemInteractionTooltipInput {
        object_kind: Some(object_kind),
        selected_item_guid: Some(selected_item_guid.to_string()),
        conversion_output_item_id,
        cleansing_rows,
        ..Default::default()
    });
}

fn cleansing_rows(title: &str, detail: &str) -> Vec<ItemInteractionTooltipLine> {
    [title, detail]
        .into_iter()
        .enumerate()
        .map(|(index, text)| ItemInteractionTooltipLine {
            left_text: text.to_string(),
            left_color: (index == 1).then_some((0.12, 1.0, 0.0)),
            ..Default::default()
        })
        .collect()
}

fn load_item_interaction_env() -> WowLuaEnv {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    let (loaded, reason): (bool, Option<String>) = env
        .eval(r#"return C_AddOns.LoadAddOn("Blizzard_ItemInteractionUI")"#)
        .unwrap();
    assert!(loaded, "Blizzard_ItemInteractionUI load failed: {reason:?}");
    env.exec(
        r#"
        assert(C_AddOns.IsAddOnLoaded("Blizzard_ItemInteractionUI"))
        assert(debug.getinfo(ItemInteractionMixin.SetInputItemSlotTooltip, "S").source
            :find("Blizzard_ItemInteractionUI.lua", 1, true))
        assert(debug.getinfo(ItemInteractionItemConversionOutputSlotMixin.OnEnter, "S").source
            :find("Blizzard_ItemInteractionUI.lua", 1, true))
    "#,
    )
    .unwrap();
    env
}

fn load_challenges_env() -> WowLuaEnv {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    load_challenges_ui(&env);
    env
}

fn load_challenges_ui(env: &WowLuaEnv) {
    for addon in ["Blizzard_GroupFinder", "Blizzard_ChallengesUI"] {
        let (loaded, reason): (bool, Option<String>) = env
            .eval(&format!("return C_AddOns.LoadAddOn('{addon}')"))
            .unwrap();
        assert!(loaded, "{addon} load failed: {reason:?}");
    }
    env.exec(
        r##"
        assert(PVEFrame, "selected GroupFinder path creates PVEFrame")
        assert(C_AddOns.IsAddOnLoaded("Blizzard_ChallengesUI"))
        assert(debug.getinfo(ChallengesKeystoneSlotMixin.OnEnter, "S").source
            :find("Blizzard_ChallengesUI.lua", 1, true))
        assert(debug.getinfo(GameTooltip.SetSlottedKeystone, "S").source
            :find("TooltipDataHandler.lua", 1, true))
        R7ChallengesKeystoneSlot = CreateFrame("Button", nil, UIParent)
    "##,
    )
    .unwrap();
}

fn register_keystone_event_probe(env: &WowLuaEnv) {
    env.exec(
        r##"
        R7KeystoneSlottedState = {}
        R7KeystoneEventProbe = CreateFrame("Frame")
        R7KeystoneEventProbe:RegisterEvent("CHALLENGE_MODE_KEYSTONE_SLOTTED")
        R7KeystoneEventProbe:SetScript("OnEvent", function(_, _, itemID)
            local data = C_TooltipInfo.GetSlottedKeystone()
            assert(C_ChallengeMode.HasSlottedKeystone())
            assert(data and data.id == itemID)
            table.insert(R7KeystoneSlottedState, {
                eventItemID = itemID,
                tooltipItemID = data.id,
                guid = data.guid,
            })
        end)
    "##,
    )
    .unwrap();
}

// INFERRED fixture: this bag cursor stands in for native item/source GUID fields.
fn slot_keystone_from_bag(env: &WowLuaEnv, slot: u32, item_id: u32) -> String {
    let guid = add_bag_item(env, 0, slot, item_id);
    {
        let mut sim = env.state().borrow_mut();
        sim.keystone.item_modifiers.insert(
            KeystoneLocation::Bag(0, slot as i32),
            (item_id, vec![(17, 2)]),
        );
        sim.keystone.current_map_selector = Some(960);
    }
    env.exec(&format!(
        r##"
        C_Container.PickupContainerItem(0, {slot})
        assert(CursorHasItem())
        local count = select("#", C_ChallengeMode.SlotKeystone())
        assert(count == 0, "SlotKeystone returns no Lua values")
        assert(not CursorHasItem(), "SlotKeystone clears the cursor")
        assert(C_Item.GetItemGUID({{bagID = 0, slotIndex = {slot}}}) == nil,
            "cursor clear does not restore the source item")
        "##
    ))
    .unwrap();
    guid
}

fn assert_challenges_hover_hidden(env: &WowLuaEnv) {
    env.exec(
        r#"
        GameTooltip:Hide()
        ChallengesKeystoneSlotMixin.OnEnter(R7ChallengesKeystoneSlot)
        assert(not GameTooltip:IsShown())
    "#,
    )
    .unwrap();
}

fn assert_challenges_hover_item(env: &WowLuaEnv, item_id: u32, guid: &str) {
    env.exec(&format!(
        r#"
        GameTooltip:Hide()
        ChallengesKeystoneSlotMixin.OnEnter(R7ChallengesKeystoneSlot)
        assert(GameTooltip:IsShown(), "original ChallengesUI hover shows the selected keystone")
        local data = assert(GameTooltip:GetPrimaryTooltipData(), "ChallengesUI tooltip missing")
        assert(data.type == 0 and data.id == {item_id}, "ChallengesUI item identity")
        assert(data.guid == "{guid}", "ChallengesUI instance GUID")
        for _, line in ipairs(data.lines or {{}}) do
            local text = line.leftText or ""
            assert(not text:match("^Map: ") and text ~= "Weekly Keystone")
            assert(not text:match("^Keystone Power ") and not text:match("^Resilient "))
            assert(text ~= "Modifier heading" and text:sub(1, 2) ~= "  ")
        end
    "#
    ))
    .unwrap();
}

const ASSERT_KEYSTONE_HOVER_ROWS: &str = r##"
local expected = R7ExpectedKeystoneRows
GameTooltip:Hide()
ChallengesKeystoneSlotMixin.OnEnter(R7ChallengesKeystoneSlot)
assert(GameTooltip:IsShown(), "original ChallengesUI hover shows the selected keystone")
local data = assert(GameTooltip:GetPrimaryTooltipData(), "ChallengesUI tooltip missing")
assert(data.type == 0 and data.id == R7ExpectedKeystoneItemID, "selected keystone identity")
assert(data.guid == R7ExpectedKeystoneGUID, "selected keystone instance GUID")
local function find(text)
    for index, line in ipairs(data.lines or {}) do
        if line.leftText == text then
            return index, line
        end
    end
end
local function same_color(left, right)
    if not left or not right then return false end
    local lr, lg, lb = left:GetRGB()
    local rr, rg, rb = right:GetRGB()
    return math.abs(lr - rr) < 0.001 and math.abs(lg - rg) < 0.001 and math.abs(lb - rb) < 0.001
end
if expected.no_keystone_title_or_affixes then
    for _, line in ipairs(data.lines or {}) do
        local text = line.leftText or ""
        assert(not text:match("^Map: ") and text ~= "Weekly Keystone", "unexpected title")
        assert(text ~= "Modifier heading" and text:sub(1, 2) ~= "  ", "unexpected affix stage")
    end
    return
end
local title_index, title = find(expected.title)
assert(title, "verified keystone title row missing")
assert(title.type == 0 and title.quality ~= nil, "title type and quality field")
assert(title.rightText == nil and title.rightColor == nil, "title has no right-side fields")
assert(title.leftColor ~= nil, "title has a quality-based color")
local power_index, power = find(expected.power)
assert(power and power.type == 0, "verified keystone power row missing")
assert(same_color(power.leftColor, NORMAL_FONT_COLOR), "power row uses normal color")
local last_index = power_index
assert(title_index < last_index, "title precedes power")
if expected.resilient then
    local index, line = find(expected.resilient)
    assert(line and line.type == 0, "verified resilient-level row missing")
    assert(same_color(line.leftColor, NORMAL_FONT_COLOR), "resilient row uses normal color")
    assert(last_index < index, "resilient row follows power")
    last_index = index
else
    for _, line in ipairs(data.lines or {}) do
        assert(not (line.leftText or ""):match("^Resilient "), "unexpected resilient row")
    end
end
if expected.heading then
    local index, line = find(expected.heading)
    assert(line and line.type == 0, "verified modifier heading missing")
    assert(same_color(line.leftColor, HIGHLIGHT_FONT_COLOR), "heading uses highlight color")
    assert(last_index < index, "heading follows level rows")
    last_index = index
else
    assert(find("Modifier heading") == nil, "all-zero affixes omit heading")
end
local affixes = {}
for index, line in ipairs(data.lines or {}) do
    if line.type == 0 and (line.leftText or ""):sub(1, 2) == "  " then
        table.insert(affixes, {index = index, text = line.leftText, color = line.leftColor})
    end
end
assert(#affixes == #expected.affixes, "affix rows preserve only backed names")
for index, text in ipairs(expected.affixes) do
    local line = affixes[index]
    assert(line.text == text, "affix slot order and leading spaces")
    assert(line.index > last_index, "affix name follows its heading")
    assert(same_color(line.color, FRIENDLY_STATUS_COLOR), "affix uses friendly color")
    last_index = line.index
end
"##;

const CAPTURE_KEYSTONE_HOVER_ROWS: &str = r##"
GameTooltip:Hide()
ChallengesKeystoneSlotMixin.OnEnter(R7ChallengesKeystoneSlot)
assert(GameTooltip:IsShown(), "original ChallengesUI hover shows selected keystone")
local data = assert(GameTooltip:GetPrimaryTooltipData(), "ChallengesUI tooltip missing")
assert(data.type == 0 and data.id == R7ExpectedKeystoneItemID, "selected keystone identity")
assert(data.guid == R7ExpectedKeystoneGUID, "selected keystone instance GUID")
local rows = {}
for _, line in ipairs(data.lines or {}) do
    local color = "-"
    if line.leftColor then
        local red, green, blue = line.leftColor:GetRGB()
        color = string.format("%.3f,%.3f,%.3f", red, green, blue)
    end
    local quality = line.quality and tostring(line.quality) or "-"
    rows[#rows + 1] = table.concat({line.leftText or "", tostring(line.type), quality, color}, "\t")
end
return table.concat(rows, "\n")
"##;

fn set_keystone_tooltip_strings(env: &WowLuaEnv) {
    env.exec(
        r#"
        CHALLENGE_MODE_KEYSTONE_NAME = "Map: %s"
        WEEKLY_REWARDS_MYTHIC_KEYSTONE = "Weekly Keystone"
        CHALLENGE_MODE_ITEM_POWER_LEVEL = "Keystone Power %d"
        RESILIENT_KEY_LEVEL = "Resilient %d"
        CHALLENGE_MODE_DUNGEON_MODIFIERS = "Modifier heading"
        "#,
    )
    .unwrap();
}

fn keystone_hover_fixture(slot: u32, item_id: u32) -> (WowLuaEnv, String) {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    env.exec("A_Admin.ClearBags(); C_ChallengeMode.ClearKeystone()")
        .unwrap();
    let guid = slot_keystone_from_bag(&env, slot, item_id);
    load_challenges_ui(&env);
    set_keystone_tooltip_strings(&env);
    (env, guid)
}

fn slotted_tooltip_input(_item_id: u32, _guid: &str) -> SlottedKeystoneTooltipInput {
    SlottedKeystoneTooltipInput::default()
}

fn set_slotted_tooltip_input(
    env: &WowLuaEnv,
    slot: u32,
    input: SlottedKeystoneTooltipInput,
    modifiers: Vec<(u32, u32)>,
) {
    let item_id = env
        .state()
        .borrow()
        .keystone
        .selected_item_id
        .expect("selected keystone item id");
    let mut sim = env.state().borrow_mut();
    sim.keystone
        .item_modifiers
        .insert(KeystoneLocation::Bag(0, slot as i32), (item_id, modifiers));
    sim.keystone.slotted_tooltip_input = Some(input);
}

fn set_keystone_context_without_item_identity(
    env: &WowLuaEnv,
    slot: u32,
    item_id: u32,
    input: SlottedKeystoneTooltipInput,
    modifiers: Vec<(u32, u32)>,
) {
    let mut sim = env.state().borrow_mut();
    let location = KeystoneLocation::Bag(0, slot as i32);
    sim.keystone.item_modifiers.insert(location, (item_id, modifiers));
    sim.keystone.slotted_tooltip_input = Some(input);
}

fn assert_exact_keystone_row_sequence(env: &WowLuaEnv, expected: &str) {
    env.exec(&format!("R7ExpectedKeystoneRowSequence = {expected:?}"))
        .unwrap();
    env.exec(
        r##"
        local rows = {}
        for _, line in ipairs(GameTooltip:GetPrimaryTooltipData().lines) do
            local text = line.leftText or ""
            if text:match("^Map: ") or text:match("^Weekly Keystone")
                or text:match("^Keystone Power ") or text:match("^Resilient ")
                or text == "Modifier heading" or text:sub(1, 2) == "  " then
                rows[#rows + 1] = text
            end
        end
        assert(table.concat(rows, "\n") == R7ExpectedKeystoneRowSequence,
            "exact slotted-keystone row sequence")
    "##,
    )
    .unwrap();
}

fn keystone_expected(
    title: &str,
    power: &str,
    resilient: Option<&str>,
    heading: bool,
    affixes: &[&str],
) -> String {
    let resilient = resilient.map_or_else(|| "false".to_string(), |text| format!("{text:?}"));
    let heading = heading.then_some("\"Modifier heading\"").unwrap_or("false");
    let affixes = affixes
        .iter()
        .map(|text| format!("{text:?}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "{{ title = {title:?}, power = {power:?}, resilient = {resilient}, heading = {heading}, affixes = {{{affixes}}} }}"
    )
}

fn assert_keystone_hover_rows(env: &WowLuaEnv, item_id: u32, guid: &str, expected: &str) {
    env.exec(&format!(
        "R7ExpectedKeystoneItemID = {item_id}; R7ExpectedKeystoneGUID = {guid:?}; R7ExpectedKeystoneRows = {expected}"
    ))
    .unwrap();
    env.exec(ASSERT_KEYSTONE_HOVER_ROWS)
        .expect("keystone row assertions run after the original ChallengesUI hover");
}

fn capture_keystone_hover_rows(env: &WowLuaEnv, item_id: u32, guid: &str) -> String {
    env.exec(&format!(
        "R7ExpectedKeystoneItemID = {item_id}; R7ExpectedKeystoneGUID = {guid:?}"
    ))
    .unwrap();
    env.eval(CAPTURE_KEYSTONE_HOVER_ROWS)
        .expect("capture rows after original ChallengesUI hover")
}

fn set_and_capture_keystone_rows(
    env: &WowLuaEnv,
    slot: u32,
    item_id: u32,
    guid: &str,
    input: SlottedKeystoneTooltipInput,
    modifiers: Vec<(u32, u32)>,
) -> String {
    set_slotted_tooltip_input(env, slot, input, modifiers);
    capture_keystone_hover_rows(env, item_id, guid)
}

fn keystone_map_input(
    item_id: u32,
    guid: &str,
    map_id: u32,
    map_name: Option<&str>,
) -> SlottedKeystoneTooltipInput {
    let mut input = slotted_tooltip_input(item_id, guid);
    input.map_names.insert(map_id, map_name.map(str::to_string));
    input
}

fn keystone_row<'a>(rows: &'a str, text: &str) -> Option<&'a str> {
    rows.lines()
        .find(|row| row.split('\t').next() == Some(text))
}

fn keystone_affix_texts(rows: &str) -> Vec<String> {
    rows.lines()
        .filter_map(|row| row.split('\t').next())
        .filter(|text| text.starts_with("  "))
        .map(str::to_string)
        .collect()
}

fn clear_keystone_tooltip_input(env: &WowLuaEnv) {
    env.state().borrow_mut().keystone.slotted_tooltip_input = None;
}

fn assert_no_keystone_title_or_affixes(env: &WowLuaEnv, item_id: u32, guid: &str) {
    assert_keystone_hover_rows(
        env,
        item_id,
        guid,
        "{ no_keystone_title_or_affixes = true }",
    );
}

fn resilient_tooltip_input(
    item_id: u32,
    guid: &str,
    season: u32,
    floors: &[(u32, u32, u32)],
    passing_conditions: &[u32],
) -> SlottedKeystoneTooltipInput {
    let mut input = slotted_tooltip_input(item_id, guid);
    input.map_names.insert(971, Some("Floor Map".to_string()));
    input.active_player_guid = Some("Player-1".to_string());
    input.associated_player_guid = Some("Player-1".to_string());
    input.display_season_id = Some(season);
    input.key_floors = floors
        .iter()
        .map(
            |(display_season, key_floor, condition)| SlottedKeystoneKeyFloor {
                display_season_id: *display_season,
                key_floor: *key_floor,
                player_condition_id: *condition,
            },
        )
        .collect();
    input.player_condition_results = passing_conditions.iter().map(|id| (*id, true)).collect();
    input
}

fn clear_item_interaction_input(env: &WowLuaEnv, interaction_type: i32) {
    let mut sim = env.state().borrow_mut();
    sim.item_interaction = Some(ItemInteractionInfo {
        interaction_type,
        ..Default::default()
    });
    sim.item_interaction_tooltip_input = None;
}

fn assert_cleansing_consumer_absent(env: &WowLuaEnv) {
    clear_item_interaction_input(env, 2);
    env.exec(
        r#"
        ItemInteractionFrame.interactionType = Enum.UIItemInteractionType.CleanseCorruption
        ItemInteractionFrame:SetInputItemSlotTooltip(ItemInteractionFrame.ItemSlot, {})
        assert(GameTooltip:GetPrimaryTooltipData() == nil, "absent cleansing input publishes no data")
        "#,
    )
    .unwrap();
}

fn assert_cleansing_consumer_rows(env: &WowLuaEnv, title: &str, detail: &str) {
    env.exec(&format!(
        r#"
        ItemInteractionFrame.interactionType = Enum.UIItemInteractionType.CleanseCorruption
        ItemInteractionFrame:SetInputItemSlotTooltip(ItemInteractionFrame.ItemSlot, {{}})
        local data = assert(GameTooltip:GetPrimaryTooltipData(), "cleansing tooltip missing")
        assert(data.type == 20 and data.id == nil, "verified cleansing identity")
        assert(data.lines[1].leftText == "{title}")
        assert(data.lines[1].leftColor == nil, "omitted host color stays absent")
        assert(data.lines[2].leftText == "{detail}")
        assert(math.abs(data.lines[2].leftColor.g - 1) < 0.001)
        "#,
    ))
    .unwrap();
}

#[test]
fn item_interaction_cleansing_original_consumer_uses_distinct_supplied_rows() {
    let env = load_item_interaction_env();
    let selected_guid = add_bag_item(&env, 0, 3, 128_910);
    assert_cleansing_consumer_absent(&env);

    for (title, detail) in [
        ("Fixture title A", "Fixture cleansing row A"),
        ("Fixture title B", "Fixture cleansing row B"),
    ] {
        set_interaction_input(
            &env,
            2,
            2,
            &selected_guid,
            None,
            cleansing_rows(title, detail),
        );
        assert_cleansing_consumer_rows(&env, title, detail);
    }
}

fn assert_conversion_consumer_absent(env: &WowLuaEnv) {
    clear_item_interaction_input(env, 4);
    env.exec(
        r#"
        ItemInteractionFrame.interactionType = Enum.UIItemInteractionType.ItemConversion
        ItemInteractionFrame.itemLocation = {}
        ItemInteractionItemConversionOutputSlotMixin.OnEnter(
            ItemInteractionFrame.ItemConversionFrame.ItemConversionOutputSlot)
        assert(GameTooltip:GetPrimaryTooltipData() == nil, "absent conversion input publishes no data")
        "#,
    )
    .unwrap();
}

fn assert_conversion_consumer_item(env: &WowLuaEnv, output_item_id: u32) {
    env.exec(&format!(
        r#"
        ItemInteractionFrame.interactionType = Enum.UIItemInteractionType.ItemConversion
        ItemInteractionFrame.itemLocation = {{}}
        ItemInteractionItemConversionOutputSlotMixin.OnEnter(
            ItemInteractionFrame.ItemConversionFrame.ItemConversionOutputSlot)
        local data = assert(GameTooltip:GetPrimaryTooltipData(), "conversion tooltip missing")
        assert(data.type == 0 and data.id == {output_item_id}, "conversion output identity")
        "#
    ))
    .unwrap();
}

#[test]
fn item_interaction_conversion_original_consumer_uses_distinct_output_ids() {
    let env = load_item_interaction_env();
    let selected_guid = add_bag_item(&env, 0, 3, 128_910);
    assert_conversion_consumer_absent(&env);

    for output_item_id in [1728, 231_268] {
        set_interaction_input(&env, 4, 4, &selected_guid, Some(output_item_id), Vec::new());
        assert_conversion_consumer_item(&env, output_item_id);
    }
}

fn assert_zero_conversion_output_shape(env: &WowLuaEnv) {
    assert_zero_conversion_data_fields(env);
    assert_zero_conversion_retrieving_line(env);
    assert_zero_conversion_retrieving_color(env);
    assert_zero_conversion_without_retrieving_text(env);
}

fn assert_zero_conversion_data_fields(env: &WowLuaEnv) {
    env.exec(
        r##"
        local function capture(...) return {n = select("#", ...), ...} end
        local result = capture(C_TooltipInfo.GetItemInteractionItem())
        assert(
            result.n == 1 and type(result[1]) == "table",
            "zero conversion output must publish one TooltipData table"
        )
        local data = result[1]
        assert(type(data.dataInstanceID) == "number", "result must carry dataInstanceID")
        assert(data.id == nil and data.type == nil, "result omits top-level id and type")
        local fields = 0
        for key in pairs(data) do
            fields = fields + 1
            assert(key == "lines" or key == "dataInstanceID")
        end
        assert(fields == 2 and type(data.lines) == "table", "result has only lines and dataInstanceID")
        "##,
    )
    .expect("zero conversion output publishes only the q1 unresolved-item fields");
}

fn assert_zero_conversion_retrieving_line(env: &WowLuaEnv) {
    env.exec(
        r##"
        local expectedText = RETRIEVING_ITEM_INFO
        if expectedText == nil then return end
        local function capture(...) return {n = select("#", ...), ...} end
        local result = capture(C_TooltipInfo.GetItemInteractionItem())
        local line = assert(result[1].lines[1])
        assert(line.type == 41 and line.leftText == expectedText)
        assert(line.rightText == nil and line.rightColor == nil)
        local fields = 0
        for key in pairs(line) do
            fields = fields + 1
            assert(key == "type" or key == "leftText" or key == "leftColor")
        end
        assert(fields == 3, "retrieving line has only type, leftText, and leftColor")
        "##,
    )
    .expect("retrieving conversion line uses the supplied localization");
}

fn assert_zero_conversion_retrieving_color(env: &WowLuaEnv) {
    env.exec(
        r##"
        if RETRIEVING_ITEM_INFO == nil then return end
        local function capture(...) return {n = select("#", ...), ...} end
        local result = capture(C_TooltipInfo.GetItemInteractionItem())
        local color = assert(result[1].lines[1].leftColor)
        assert(type(rawget(color, "r")) == "number", "error color publishes red")
        assert(type(rawget(color, "g")) == "number", "error color publishes green")
        assert(type(rawget(color, "b")) == "number", "error color publishes blue")
        assert(rawget(color, "a") == nil, "error color omits alpha")
        local r, g, b = color:GetRGB()
        local er, eg, eb = 1, 0, 1
        if ERROR_COLOR ~= nil then er, eg, eb = ERROR_COLOR:GetRGB() end
        assert(math.abs(r - er) < 0.001)
        assert(math.abs(g - eg) < 0.001)
        assert(math.abs(b - eb) < 0.001)
        "##,
    )
    .expect("retrieving conversion line uses the error color");
}

fn assert_zero_conversion_without_retrieving_text(env: &WowLuaEnv) {
    env.exec(
        r##"
        local previousText = RETRIEVING_ITEM_INFO
        RETRIEVING_ITEM_INFO = nil
        local function capture(...) return {n = select("#", ...), ...} end
        local result = capture(C_TooltipInfo.GetItemInteractionItem())
        assert(result.n == 1 and type(result[1]) == "table")
        local data = result[1]
        assert(type(data.dataInstanceID) == "number")
        assert(data.id == nil and data.type == nil)
        assert(type(data.lines) == "table" and next(data.lines) == nil)
        local fields = 0
        for key in pairs(data) do
            fields = fields + 1
            assert(key == "lines" or key == "dataInstanceID")
        end
        assert(fields == 2, "result has only lines and dataInstanceID")
        RETRIEVING_ITEM_INFO = previousText
        "##,
    )
    .expect("missing retrieval localization appends no conversion line");
}

// SUPPLEMENTARY contract coverage: this calls C_TooltipInfo directly, not its original consumer.
#[test]
fn zero_conversion_output_publishes_verified_unresolved_item_shape() {
    let env = load_item_interaction_env();
    let selected_guid = add_bag_item(&env, 0, 3, 128_910);
    set_interaction_input(&env, 4, 4, &selected_guid, Some(0), Vec::new());
    assert_zero_conversion_output_shape(&env);
}

fn assert_possession_consumer_absent(env: &WowLuaEnv) {
    env.exec(
        r#"
        GameTooltip:Hide()
        local button = CreateFrame("Button", nil, UIParent)
        button:SetID(1)
        PossessButtonMixin.OnEnter(button)
        assert(GameTooltip:GetPrimaryTooltipData() == nil, "absent possession input publishes no data")
        "#,
    )
    .unwrap();
}

fn assert_possession_consumer_spell(env: &WowLuaEnv, spell_id: u32) {
    env.exec(&format!(
        r#"
        GameTooltip:Hide()
        local button = CreateFrame("Button", nil, UIParent)
        button:SetID(1)
        PossessButtonMixin.OnEnter(button)
        local data = assert(GameTooltip:GetPrimaryTooltipData(), "possession tooltip missing")
        assert(data.type == 1 and data.id == {spell_id}, "possession spell identity")
        "#
    ))
    .unwrap();
}

fn load_possession_env() -> WowLuaEnv {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    env.exec(
        r#"
        assert(C_AddOns.IsAddOnLoaded("Blizzard_ActionBar"))
        assert(debug.getinfo(PossessButtonMixin.OnEnter, "S").source
            :find("PossessActionBar.lua", 1, true))
    "#,
    )
    .unwrap();
    env
}

fn possession_candidate(
    spell_id: u32,
    tested: bool,
    effects: &[u32],
    candidate_identity: Option<(bool, u32, u32)>,
    classification_field: Option<u32>,
) -> String {
    let effects = effects
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let (identity_type_present, identity_first, identity_second) =
        candidate_identity.unwrap_or((false, 0, 0));
    let classification =
        classification_field.map_or_else(|| "nil".to_string(), |value| value.to_string());
    format!(
        "{{spellID={spell_id}, tested={tested}, effects={{{effects}}}, identityTypePresent={identity_type_present}, comparedIdentityFirst={identity_first}, comparedIdentitySecond={identity_second}, classificationField={classification}}}"
    )
}

fn set_possession_object(env: &WowLuaEnv, identity: (u32, u32), candidates: &[String]) {
    let candidates = candidates.join(",");
    let object = format!(
        "{{identityFirst={}, identitySecond={}, candidates={{{candidates}}}}}",
        identity.0, identity.1
    );
    env.exec(&format!("A_Admin.SetPossessionObject({object})"))
        .unwrap();
}

fn update_possession_player(env: &WowLuaEnv, identity: (u32, u32)) {
    env.exec(&format!(
        "A_Admin.UpdatePossessionPlayer({}, {})",
        identity.0, identity.1
    ))
    .unwrap();
}

fn update_possession_player_with_candidates(
    env: &WowLuaEnv,
    identity: (u32, u32),
    candidates: &[String],
) {
    set_possession_object(env, identity, candidates);
    update_possession_player(env, identity);
}

fn update_possession_player_without_candidates(env: &WowLuaEnv, identity: (u32, u32)) {
    env.exec(&format!(
        "A_Admin.SetPossessionObject(nil); A_Admin.UpdatePossessionPlayer({}, {})",
        identity.0, identity.1
    ))
    .unwrap();
}

fn set_possession_spell_id(env: &WowLuaEnv, spell_id: u32) {
    env.exec(&format!("A_Admin.SetPossessionSpellID({spell_id})"))
        .unwrap();
}

fn assert_possession_effect_selects_distinct_spells(
    env: &WowLuaEnv,
    effect: u32,
    candidate_identity: Option<(bool, u32, u32)>,
    classification_field: Option<u32>,
) {
    update_possession_player_without_candidates(env, (11, 22));
    assert_possession_consumer_absent(env);
    for spell_id in [116, 133] {
        let candidate = possession_candidate(
            spell_id,
            false,
            &[effect],
            candidate_identity,
            classification_field,
        );
        update_possession_player_with_candidates(env, (11, 22), &[candidate]);
        assert_possession_consumer_spell(env, spell_id);
    }
}

fn register_possession_event_probe(env: &WowLuaEnv) {
    env.exec(
        r#"
        R7PossessionEventSpellIDs = {}
        R7PossessionEventProbe = CreateFrame("Frame")
        R7PossessionEventProbe:RegisterEvent("UPDATE_POSSESS_BAR")
        R7PossessionEventProbe:SetScript("OnEvent", function()
            GameTooltip:Hide()
            local button = CreateFrame("Button", nil, UIParent)
            button:SetID(1)
            PossessButtonMixin.OnEnter(button)
            local data = GameTooltip:GetPrimaryTooltipData()
            table.insert(R7PossessionEventSpellIDs, data and data.id or 0)
        end)
    "#,
    )
    .unwrap();
}

#[test]
fn possession_original_action_bar_hover_uses_the_supplied_spell() {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    env.exec(
        r#"
        assert(C_AddOns.IsAddOnLoaded("Blizzard_ActionBar"))
        assert(debug.getinfo(PossessButtonMixin.OnEnter, "S").source
            :find("PossessActionBar.lua", 1, true))
    "#,
    )
    .unwrap();
    assert_possession_consumer_absent(&env);

    for spell_id in [116, 133] {
        env.state().borrow_mut().possession_spell_id = Some(spell_id);
        assert_possession_consumer_spell(&env, spell_id);
    }
}

#[test]
fn possession_rescan_accepts_direct_effect_two_for_distinct_spells() {
    let env = load_possession_env();
    assert_possession_effect_selects_distinct_spells(&env, 2, None, None);
}

#[test]
fn possession_rescan_accepts_direct_effect_six_for_distinct_spells() {
    let env = load_possession_env();
    assert_possession_effect_selects_distinct_spells(&env, 6, None, None);
}

#[test]
fn possession_rescan_requires_identity_match_for_effect_236() {
    let env = load_possession_env();
    assert_possession_effect_selects_distinct_spells(&env, 236, Some((true, 11, 22)), None);
    set_possession_spell_id(&env, 116);
    for identity in [
        Some((true, 12, 22)),
        Some((true, 11, 23)),
        Some((false, 11, 22)),
    ] {
        let rejected = possession_candidate(133, false, &[236], identity, None);
        update_possession_player_with_candidates(&env, (11, 22), &[rejected]);
        assert_possession_consumer_spell(&env, 116);
    }
}

#[test]
fn possession_rescan_accepts_alternate_effect_28_with_classification_three() {
    let env = load_possession_env();
    assert_possession_effect_selects_distinct_spells(&env, 28, None, Some(3));
}

#[test]
fn possession_rescan_accepts_alternate_effect_51_with_classification_three() {
    let env = load_possession_env();
    assert_possession_effect_selects_distinct_spells(&env, 51, None, Some(3));
}

#[test]
fn possession_rescan_runs_when_either_identity_part_is_nonzero() {
    let env = load_possession_env();
    for (identity, spell_id) in [((0, 22), 116), ((11, 0), 133)] {
        let candidate = possession_candidate(spell_id, false, &[2], None, None);
        update_possession_player_with_candidates(&env, identity, &[candidate]);
        assert_possession_consumer_spell(&env, spell_id);
    }
}

#[test]
fn possession_rescan_keeps_old_id_for_absent_unresolved_or_rejected_inputs() {
    let env = load_possession_env();
    set_possession_spell_id(&env, 116);
    update_possession_player_without_candidates(&env, (11, 22));
    assert_possession_consumer_spell(&env, 116);
    update_possession_player_with_candidates(&env, (11, 22), &[]);
    assert_possession_consumer_spell(&env, 116);
    let unresolved_object_candidate = possession_candidate(133, false, &[2], None, None);
    set_possession_object(&env, (12, 22), &[unresolved_object_candidate]);
    update_possession_player(&env, (11, 22));
    assert_possession_consumer_spell(&env, 116);

    let rejected = [
        possession_candidate(133, true, &[2], None, None),
        possession_candidate(u32::MAX, false, &[2], None, None),
        possession_candidate(133, false, &[], None, None),
        possession_candidate(133, false, &[28], None, Some(2)),
        possession_candidate(133, false, &[236], None, None),
    ];
    update_possession_player_with_candidates(&env, (11, 22), &rejected);
    assert_possession_consumer_spell(&env, 116);
}

#[test]
fn possession_rescan_uses_the_first_accepted_candidate() {
    let env = load_possession_env();
    let candidates = [
        possession_candidate(133, false, &[2], None, None),
        possession_candidate(116, false, &[6], None, None),
    ];
    update_possession_player_with_candidates(&env, (11, 22), &candidates);
    assert_possession_consumer_spell(&env, 133);
}

fn assert_possession_event_ids(env: &WowLuaEnv, expected: &[u32]) {
    let expected = expected
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",");
    env.exec(&format!(
        "local expected = {{{expected}}}; assert(#R7PossessionEventSpellIDs == #expected); for i, id in ipairs(expected) do assert(R7PossessionEventSpellIDs[i] == id) end"
    ))
    .unwrap();
}

#[test]
fn possession_setter_rescan_and_clear_notify_only_after_id_changes() {
    let env = load_possession_env();
    register_possession_event_probe(&env);
    set_possession_spell_id(&env, 116);
    assert_possession_event_ids(&env, &[116]);
    set_possession_spell_id(&env, 116);
    assert_possession_event_ids(&env, &[116]);
    set_possession_spell_id(&env, 133);
    assert_possession_event_ids(&env, &[116, 133]);

    let candidate = possession_candidate(116, false, &[2], None, None);
    update_possession_player_with_candidates(&env, (11, 22), &[candidate.clone()]);
    assert_possession_event_ids(&env, &[116, 133, 116]);
    update_possession_player_with_candidates(&env, (11, 22), &[candidate]);
    assert_possession_event_ids(&env, &[116, 133, 116]);
    update_possession_player_without_candidates(&env, (0, 0));
    assert_possession_event_ids(&env, &[116, 133, 116, 0]);
    update_possession_player_without_candidates(&env, (0, 0));
    assert_possession_event_ids(&env, &[116, 133, 116, 0]);
}

// INFERRED model coverage: selection uses the simulator's bag-cursor projection.
#[test]
fn challenges_ui_hover_uses_public_keystone_selection() {
    for (slot, item_id) in [(3, 180_653), (4, 158_923)] {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        env.exec("A_Admin.ClearBags(); C_ChallengeMode.ClearKeystone()")
            .expect("empty challenge selection fixture");
        env.exec(
            r##"
            local function capture(...) return {n = select("#", ...), ...} end
            local has = capture(C_ChallengeMode.HasSlottedKeystone("ignored"))
            local removed = capture(C_ChallengeMode.RemoveKeystone())
            assert(has.n == 1 and has[1] == false)
            assert(removed.n == 1 and removed[1] == false)
        "##,
        )
        .unwrap();

        let guid = slot_keystone_from_bag(&env, slot, item_id);
        load_challenges_ui(&env);
        assert_challenges_hover_item(&env, item_id, &guid);
        env.exec(
            r##"
            local function capture(...) return {n = select("#", ...), ...} end
            local removed = capture(C_ChallengeMode.RemoveKeystone())
            assert(removed.n == 1 and removed[1] == true)
            assert(not C_ChallengeMode.HasSlottedKeystone())
            assert(select("#", C_TooltipInfo.GetSlottedKeystone()) == 0)
        "##,
        )
        .unwrap();
        assert_challenges_hover_hidden(&env);
    }
}

#[test]
fn challenges_ui_hover_projects_two_keystone_map_and_power_values() {
    for (slot, item_id, map_id, map_name, level) in
        [(3, 180_653, 960, "Alpha", 3), (4, 158_923, 961, "Beta", 8)]
    {
        let (env, guid) = keystone_hover_fixture(slot, item_id);
        let mut input = slotted_tooltip_input(item_id, &guid);
        input.map_names.insert(map_id, Some(map_name.to_string()));
        set_slotted_tooltip_input(
            &env,
            slot,
            input,
            vec![
                (17, map_id),
                (17, map_id + 100),
                (18, level),
                (18, level + 1),
            ],
        );
        let title = format!("Map: {map_name}");
        let power = format!("Keystone Power {level}");
        let expected = keystone_expected(&title, &power, None, false, &[]);
        assert_keystone_hover_rows(&env, item_id, &guid, &expected);
    }
}

fn assert_resilient_keystone_consumer_rows() {
    let (env, guid) = keystone_hover_fixture(3, 180_653);
    let mut input = resilient_tooltip_input(
        180_653,
        &guid,
        7,
        &[(7, 1, 70), (7, 5, 71), (7, 8, 72)],
        &[70, 71, 72],
    );
    input.affix_names.insert(101, Some("Fortified".to_string()));
    input.affix_names.insert(102, Some("Raging".to_string()));
    set_keystone_context_without_item_identity(
        &env,
        3,
        180_653,
        input,
        vec![(17, 971), (18, 7), (19, 101), (21, 102)],
    );
    capture_keystone_hover_rows(&env, 180_653, &guid);
    assert_exact_keystone_row_sequence(
        &env,
        "Map: Floor Map\\nKeystone Power 7\\nResilient 5\\nModifier heading\\n  Fortified\\n  Raging",
    );
}

fn assert_plain_keystone_consumer_rows() {
    let (env, guid) = keystone_hover_fixture(4, 158_923);
    let mut input = slotted_tooltip_input(158_923, &guid);
    input.map_names.insert(961, Some("Beta".to_string()));
    set_keystone_context_without_item_identity(
        &env,
        4,
        158_923,
        input,
        vec![(17, 961), (18, 8)],
    );
    capture_keystone_hover_rows(&env, 158_923, &guid);
    assert_exact_keystone_row_sequence(&env, "Map: Beta\\nKeystone Power 8");
}

#[test]
fn challenges_ui_hover_derives_slotted_keystone_rows_from_item_state() {
    assert_resilient_keystone_consumer_rows();
    assert_plain_keystone_consumer_rows();
}

#[test]
fn challenges_ui_hover_uses_generic_and_empty_map_titles() {
    let (env, guid) = keystone_hover_fixture(3, 180_653);
    let item_id = 180_653;
    let mut missing_map = slotted_tooltip_input(item_id, &guid);
    missing_map.map_names.insert(962, None);
    set_slotted_tooltip_input(&env, 3, missing_map, vec![(17, 962)]);
    let expected = keystone_expected("Weekly Keystone", "Keystone Power 0", None, false, &[]);
    assert_keystone_hover_rows(&env, item_id, &guid, &expected);

    let mut empty_name = slotted_tooltip_input(item_id, &guid);
    empty_name.map_names.insert(963, Some(String::new()));
    set_slotted_tooltip_input(&env, 3, empty_name, vec![(17, 963), (18, 0)]);
    let expected = keystone_expected("Map: ", "Keystone Power 0", None, false, &[]);
    assert_keystone_hover_rows(&env, item_id, &guid, &expected);
}

#[test]
fn challenges_ui_hover_selects_resilient_level_from_season_floors() {
    let (env, guid) = keystone_hover_fixture(3, 180_653);
    let item_id = 180_653;
    let mut first = resilient_tooltip_input(
        item_id,
        &guid,
        7,
        &[(7, 1, 70), (7, 5, 71), (7, 8, 72)],
        &[70, 71, 72],
    );
    set_slotted_tooltip_input(&env, 3, first.clone(), vec![(17, 971), (18, 7)]);
    let expected = keystone_expected(
        "Map: Floor Map",
        "Keystone Power 7",
        Some("Resilient 5"),
        false,
        &[],
    );
    assert_keystone_hover_rows(&env, item_id, &guid, &expected);

    first = resilient_tooltip_input(
        item_id,
        &guid,
        8,
        &[(8, 2, 80), (8, 6, 81), (8, 9, 82)],
        &[80, 81, 82],
    );
    set_slotted_tooltip_input(&env, 3, first.clone(), vec![(17, 971), (18, 8)]);
    let expected = keystone_expected(
        "Map: Floor Map",
        "Keystone Power 8",
        Some("Resilient 6"),
        false,
        &[],
    );
    assert_keystone_hover_rows(&env, item_id, &guid, &expected);

    first.associated_player_guid = Some("Player-2".to_string());
    set_slotted_tooltip_input(&env, 3, first.clone(), vec![(17, 971), (18, 8)]);
    let expected = keystone_expected("Map: Floor Map", "Keystone Power 8", None, false, &[]);
    assert_keystone_hover_rows(&env, item_id, &guid, &expected);
    first.associated_player_guid = Some("Player-1".to_string());
    first.player_condition_results.clear();
    set_slotted_tooltip_input(&env, 3, first, vec![(17, 971), (18, 8)]);
    assert_keystone_hover_rows(&env, item_id, &guid, &expected);
}

#[test]
fn challenges_ui_hover_preserves_affix_slot_order_and_duplicates() {
    let (env, guid) = keystone_hover_fixture(3, 180_653);
    let item_id = 180_653;
    let mut input = slotted_tooltip_input(item_id, &guid);
    input.map_names.insert(980, Some("Affix Map".to_string()));
    input.affix_names.insert(101, Some("Fortified".to_string()));
    input.affix_names.insert(102, Some(String::new()));
    input.affix_names.insert(103, Some("Raging".to_string()));
    let modifiers = vec![
        (17, 980),
        (18, 2),
        (19, 101),
        (19, 103),
        (20, 102),
        (20, 101),
        (21, 103),
        (21, 101),
        (22, 101),
        (22, 102),
    ];
    set_slotted_tooltip_input(&env, 3, input, modifiers);
    let expected = keystone_expected(
        "Map: Affix Map",
        "Keystone Power 2",
        None,
        true,
        &["  Fortified", "  ", "  Raging", "  Fortified"],
    );
    assert_keystone_hover_rows(&env, item_id, &guid, &expected);
}

#[test]
fn challenges_ui_hover_omits_missing_affix_records() {
    let (env, guid) = keystone_hover_fixture(3, 180_653);
    let item_id = 180_653;
    let mut input = slotted_tooltip_input(item_id, &guid);
    input.map_names.insert(981, Some("Second Map".to_string()));
    input
        .affix_names
        .insert(201, Some("Tyrannical".to_string()));
    input.affix_names.insert(202, None);
    input.affix_names.insert(203, Some("Quaking".to_string()));
    set_slotted_tooltip_input(
        &env,
        3,
        input,
        vec![(17, 981), (18, 4), (19, 201), (20, 202), (21, 0), (22, 203)],
    );
    let expected = keystone_expected(
        "Map: Second Map",
        "Keystone Power 4",
        None,
        true,
        &["  Tyrannical", "  Quaking"],
    );
    assert_keystone_hover_rows(&env, item_id, &guid, &expected);
}

#[test]
fn challenges_ui_hover_omits_heading_for_all_zero_affix_ids() {
    let (env, guid) = keystone_hover_fixture(3, 180_653);
    let item_id = 180_653;
    let mut input = slotted_tooltip_input(item_id, &guid);
    input.map_names.insert(982, Some("No Affixes".to_string()));
    set_slotted_tooltip_input(
        &env,
        3,
        input,
        vec![(17, 982), (18, 1), (19, 0), (20, 0), (21, 0), (22, 0)],
    );
    let expected = keystone_expected("Map: No Affixes", "Keystone Power 1", None, false, &[]);
    assert_keystone_hover_rows(&env, item_id, &guid, &expected);
}

fn set_keystone_item_class(env: &WowLuaEnv, item_id: u32, class_id: u8, subclass_id: u8) {
    let mut sim = env.state().borrow_mut();
    let mut item = sim.item_info(item_id).expect("known keystone item").clone();
    item.item_class = Some(class_id);
    item.item_subclass = Some(subclass_id);
    sim.item_overrides.insert(item_id, item);
}

#[test]
fn challenges_ui_hover_skips_rows_for_non_keystone_item_class_or_subclass() {
    let (env, guid) = keystone_hover_fixture(3, 180_653);
    let item_id = 180_653;
    for (class_id, subclass_id) in [(4, 1), (5, 0)] {
        set_keystone_item_class(&env, item_id, class_id, subclass_id);
        let mut input = slotted_tooltip_input(item_id, &guid);
        input
            .map_names
            .insert(990, Some("Rejected Map".to_string()));
        input
            .affix_names
            .insert(301, Some("Rejected Affix".to_string()));
        set_slotted_tooltip_input(&env, 3, input, vec![(17, 990), (18, 7), (19, 301)]);
        assert_no_keystone_title_or_affixes(&env, item_id, &guid);
    }
}

#[test]
fn challenges_ui_hover_selects_map_titles_from_modifier_17() {
    let (env, guid) = keystone_hover_fixture(3, 180_653);
    let mut first = keystone_map_input(180_653, &guid, 960, Some("First Map"));
    first.map_names.insert(1960, Some("Wrong Map".to_string()));
    let first_rows = set_and_capture_keystone_rows(
        &env,
        3,
        180_653,
        &guid,
        first,
        vec![(17, 960), (17, 1960), (18, 3)],
    );
    let second = keystone_map_input(180_653, &guid, 961, Some("Second Map"));
    let second_rows =
        set_and_capture_keystone_rows(&env, 3, 180_653, &guid, second, vec![(17, 961), (18, 8)]);
    let missing = keystone_map_input(180_653, &guid, 962, None);
    let missing_rows =
        set_and_capture_keystone_rows(&env, 3, 180_653, &guid, missing, vec![(17, 962), (18, 1)]);
    assert!(
        keystone_row(&first_rows, "Map: First Map").is_some(),
        "map-title selector input 1"
    );
    assert!(
        keystone_row(&second_rows, "Map: Second Map").is_some(),
        "map-title selector input 2"
    );
    assert!(
        keystone_row(&missing_rows, "Weekly Keystone").is_some(),
        "map-record miss fallback"
    );
}

#[test]
fn challenges_ui_hover_selects_generic_and_empty_map_titles() {
    let (env, guid) = keystone_hover_fixture(3, 180_653);
    let missing = keystone_map_input(180_653, &guid, 963, None);
    let generic_rows =
        set_and_capture_keystone_rows(&env, 3, 180_653, &guid, missing, vec![(17, 963), (18, 3)]);
    let empty = keystone_map_input(180_653, &guid, 964, Some(""));
    let empty_rows =
        set_and_capture_keystone_rows(&env, 3, 180_653, &guid, empty, vec![(17, 964), (18, 4)]);
    clear_keystone_tooltip_input(&env);
    let absent_rows = capture_keystone_hover_rows(&env, 180_653, &guid);
    assert!(
        keystone_row(&generic_rows, "Weekly Keystone").is_some(),
        "generic-title selector"
    );
    assert!(
        keystone_row(&empty_rows, "Map: ").is_some(),
        "found empty map name is not a miss"
    );
    assert!(
        keystone_row(&absent_rows, "Weekly Keystone").is_none(),
        "empty input stays neutral"
    );
}

#[test]
fn challenges_ui_hover_selects_power_from_modifier_18() {
    let (env, guid) = keystone_hover_fixture(3, 180_653);
    let first = keystone_map_input(180_653, &guid, 965, Some("Power Map"));
    let first_rows =
        set_and_capture_keystone_rows(&env, 3, 180_653, &guid, first, vec![(17, 965), (18, 3)]);
    let second = keystone_map_input(180_653, &guid, 965, Some("Power Map"));
    let second_rows =
        set_and_capture_keystone_rows(&env, 3, 180_653, &guid, second, vec![(17, 965), (18, 8)]);
    let missing_level = keystone_map_input(180_653, &guid, 965, Some("Power Map"));
    let zero_rows =
        set_and_capture_keystone_rows(&env, 3, 180_653, &guid, missing_level, vec![(17, 965)]);
    clear_keystone_tooltip_input(&env);
    let absent_rows = capture_keystone_hover_rows(&env, 180_653, &guid);
    assert!(
        keystone_row(&first_rows, "Keystone Power 3").is_some(),
        "power selector input 1"
    );
    assert!(
        keystone_row(&second_rows, "Keystone Power 8").is_some(),
        "power selector input 2"
    );
    assert!(
        keystone_row(&zero_rows, "Keystone Power 0").is_some(),
        "missing modifier 18 uses zero"
    );
    assert!(
        keystone_row(&absent_rows, "Keystone Power 0").is_none(),
        "empty input has no power row"
    );
}

#[test]
fn challenges_ui_hover_selects_resilient_floor_by_display_season() {
    let (env, guid) = keystone_hover_fixture(3, 180_653);
    let floors = [(7, 3, 70), (7, 5, 71), (8, 6, 80)];
    let first = resilient_tooltip_input(180_653, &guid, 7, &floors, &[70, 71, 80]);
    let season_seven =
        set_and_capture_keystone_rows(&env, 3, 180_653, &guid, first, vec![(17, 971), (18, 8)]);
    let second = resilient_tooltip_input(180_653, &guid, 8, &floors, &[70, 71, 80]);
    let season_eight =
        set_and_capture_keystone_rows(&env, 3, 180_653, &guid, second, vec![(17, 971), (18, 8)]);
    let absent = resilient_tooltip_input(180_653, &guid, 9, &floors, &[70, 71, 80]);
    let season_missing =
        set_and_capture_keystone_rows(&env, 3, 180_653, &guid, absent, vec![(17, 971), (18, 8)]);
    assert!(
        keystone_row(&season_seven, "Resilient 5").is_some(),
        "season selector input 1"
    );
    assert!(
        keystone_row(&season_eight, "Resilient 6").is_some(),
        "season selector input 2"
    );
    assert!(
        keystone_row(&season_missing, "Resilient 6").is_none(),
        "unmatched display season is absent"
    );
}

#[test]
fn challenges_ui_hover_selects_modifier_heading_from_nonzero_slots() {
    let (env, guid) = keystone_hover_fixture(3, 180_653);
    let first = keystone_map_input(180_653, &guid, 980, Some("Heading Map"));
    let first_rows = set_and_capture_keystone_rows(
        &env,
        3,
        180_653,
        &guid,
        first,
        vec![(17, 980), (18, 1), (19, 101)],
    );
    let second = keystone_map_input(180_653, &guid, 980, Some("Heading Map"));
    let second_rows = set_and_capture_keystone_rows(
        &env,
        3,
        180_653,
        &guid,
        second,
        vec![(17, 980), (18, 1), (22, 202)],
    );
    let zero = keystone_map_input(180_653, &guid, 980, Some("Heading Map"));
    let zero_rows = set_and_capture_keystone_rows(
        &env,
        3,
        180_653,
        &guid,
        zero,
        vec![(17, 980), (18, 1), (19, 0), (20, 0), (21, 0), (22, 0)],
    );
    assert!(
        keystone_row(&first_rows, "Modifier heading").is_some(),
        "heading selector input 1"
    );
    assert!(
        keystone_row(&second_rows, "Modifier heading").is_some(),
        "heading selector input 2"
    );
    assert!(
        keystone_row(&zero_rows, "Modifier heading").is_none(),
        "all-zero selector has no heading"
    );
}

#[test]
fn challenges_ui_hover_selects_affix_names_in_modifier_order() {
    let (env, guid) = keystone_hover_fixture(3, 180_653);
    let mut first = keystone_map_input(180_653, &guid, 981, Some("Affix Map A"));
    first.affix_names.insert(101, Some("Fortified".to_string()));
    first.affix_names.insert(102, Some("Raging".to_string()));
    let first_rows = set_and_capture_keystone_rows(
        &env,
        3,
        180_653,
        &guid,
        first,
        vec![(17, 981), (18, 2), (19, 101), (20, 102)],
    );
    let mut second = keystone_map_input(180_653, &guid, 982, Some("Affix Map B"));
    second
        .affix_names
        .insert(201, Some("Tyrannical".to_string()));
    second.affix_names.insert(202, Some("Quaking".to_string()));
    let second_rows = set_and_capture_keystone_rows(
        &env,
        3,
        180_653,
        &guid,
        second,
        vec![(17, 982), (18, 3), (19, 202), (20, 201), (21, 202)],
    );
    let mut missing = keystone_map_input(180_653, &guid, 983, Some("Affix Map C"));
    missing.affix_names.insert(301, None);
    let missing_rows = set_and_capture_keystone_rows(
        &env,
        3,
        180_653,
        &guid,
        missing,
        vec![(17, 983), (18, 4), (19, 301), (20, 302)],
    );
    assert_eq!(
        keystone_affix_texts(&first_rows),
        vec!["  Fortified", "  Raging"],
        "affix names input 1"
    );
    assert_eq!(
        keystone_affix_texts(&second_rows),
        vec!["  Quaking", "  Tyrannical", "  Quaking"],
        "affix names input 2"
    );
    assert!(
        keystone_affix_texts(&missing_rows).is_empty(),
        "missing affix records have no names"
    );
}

fn capture_selected_keystone_quality_titles() -> (String, String) {
    // Existing class 5/subclass 1 items 180653 and 168195 supply qualities 4 and 7.
    let (env_four, guid_four) = keystone_hover_fixture(3, 180_653);
    let input_four = keystone_map_input(180_653, &guid_four, 984, Some("Quality Map"));
    let rows_four = set_and_capture_keystone_rows(
        &env_four,
        3,
        180_653,
        &guid_four,
        input_four,
        vec![(17, 984), (18, 3)],
    );
    let (env_seven, guid_seven) = keystone_hover_fixture(4, 168_195);
    let input_seven = keystone_map_input(168_195, &guid_seven, 984, Some("Quality Map"));
    let rows_seven = set_and_capture_keystone_rows(
        &env_seven,
        4,
        168_195,
        &guid_seven,
        input_seven,
        vec![(17, 984), (18, 3)],
    );
    let title_four = keystone_row(&rows_four, "Map: Quality Map")
        .expect("quality 4 title")
        .to_owned();
    let title_seven = keystone_row(&rows_seven, "Map: Quality Map")
        .expect("quality 7 title")
        .to_owned();
    (title_four, title_seven)
}

#[test]
fn challenges_ui_hover_preserves_selected_item_quality() {
    let (title_four, title_seven) = capture_selected_keystone_quality_titles();
    let (env_absent, guid_absent) = keystone_hover_fixture(5, 180_653);
    let rows_absent = capture_keystone_hover_rows(&env_absent, 180_653, &guid_absent);
    assert_eq!(
        title_four.split('\t').nth(2),
        Some("4"),
        "title quality selector input 1"
    );
    assert_eq!(
        title_seven.split('\t').nth(2),
        Some("7"),
        "title quality selector input 2"
    );
    assert!(
        keystone_row(&rows_absent, "Map: Quality Map").is_none(),
        "empty input has no quality row"
    );
}

// INFERRED: the simulator's current item-quality mapping distinguishes these values.
#[test]
fn challenges_ui_hover_infers_quality_color_discrimination() {
    let (title_four, title_seven) = capture_selected_keystone_quality_titles();
    assert_ne!(
        title_four.split('\t').nth(3),
        title_seven.split('\t').nth(3),
        "simulator quality colors discriminate inputs",
    );
}

#[test]
fn challenges_ui_hover_uses_zero_map_key_when_modifier_17_is_absent() {
    let (env, guid) = keystone_hover_fixture(3, 180_653);
    let mut supplied = slotted_tooltip_input(180_653, &guid);
    supplied
        .map_names
        .insert(0, Some("Key Zero Map".to_string()));
    let supplied_rows =
        set_and_capture_keystone_rows(&env, 3, 180_653, &guid, supplied, vec![(18, 3)]);
    let unsupplied = slotted_tooltip_input(180_653, &guid);
    let unsupplied_rows =
        set_and_capture_keystone_rows(&env, 3, 180_653, &guid, unsupplied, vec![(18, 4)]);
    assert!(
        keystone_row(&supplied_rows, "Map: Key Zero Map").is_some(),
        "missing modifier 17 reads key zero"
    );
    assert!(
        keystone_row(&supplied_rows, "Keystone Power 3").is_some(),
        "missing modifier 17 keeps power row"
    );
    assert!(
        keystone_row(&unsupplied_rows, "Weekly Keystone").is_none(),
        "unsupplied key zero remains neutral"
    );
    assert!(
        keystone_row(&unsupplied_rows, "Keystone Power 4").is_none(),
        "unsupplied key zero blocks detail rows"
    );
}

#[test]
fn challenges_ui_hover_uses_generic_title_for_confirmed_missing_zero_map_without_modifier_17() {
    let (env, guid) = keystone_hover_fixture(3, 180_653);
    let mut input = slotted_tooltip_input(180_653, &guid);
    input.map_names.insert(0, None);
    let rows = set_and_capture_keystone_rows(&env, 3, 180_653, &guid, input, vec![(18, 4)]);
    assert!(
        keystone_row(&rows, "Weekly Keystone").is_some(),
        "missing modifier 17 and confirmed key-zero miss use generic title"
    );
    assert!(
        keystone_row(&rows, "Keystone Power 4").is_some(),
        "confirmed key-zero miss keeps power row"
    );
}

// SUPPLEMENTARY, INFERRED: a test-defined event callback observes the public API, not an original consumer.
#[test]
fn slot_keystone_notifies_after_selection_state() {
    for (slot, item_id) in [(3, 180_653), (4, 158_923)] {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        register_keystone_event_probe(&env);
        let guid = slot_keystone_from_bag(&env, slot, item_id);
        env.exec(&format!(
            r##"
            assert(#R7KeystoneSlottedState == 1)
            local selected = R7KeystoneSlottedState[1]
            assert(selected.eventItemID == {item_id})
            assert(selected.tooltipItemID == {item_id})
            assert(selected.guid == "{guid}")
            local removed = C_ChallengeMode.RemoveKeystone()
            assert(removed == true and not C_ChallengeMode.HasSlottedKeystone())
            assert(select("#", C_TooltipInfo.GetSlottedKeystone()) == 0)
            assert(select("#", C_ChallengeMode.SlotKeystone()) == 0)
            assert(#R7KeystoneSlottedState == 1)
            "##
        ))
        .unwrap();
    }
}

// SUPPLEMENTARY, INFERRED model coverage: this constructs the simulator's bag cursor directly.
#[test]
fn slot_keystone_leaves_unresolved_cursor_untouched() {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    register_keystone_event_probe(&env);
    assert!(env.state().borrow().item_info(999_999).is_none());
    env.state().borrow_mut().cursor_item = Some(CursorInfo::Item {
        owned_identity: None,
        item_id: 999_999,
        stack_count: 1,
        origin: CursorItemOrigin::Bag { bag: 0, slot: 3 },
        applied_transmog: None,
    });
    env.exec(
        r##"
        assert(select("#", C_ChallengeMode.SlotKeystone()) == 0)
        assert(CursorHasItem())
        local kind, itemID = GetCursorInfo()
        assert(kind == "item" and itemID == 999999)
        assert(not C_ChallengeMode.HasSlottedKeystone())
        assert(select("#", C_TooltipInfo.GetSlottedKeystone()) == 0)
        assert(#R7KeystoneSlottedState == 0)
        "##,
    )
    .unwrap();
}

// SUPPLEMENTARY, INFERRED model coverage: rejection cases use the context proxy.
fn assert_slot_keystone_rejection_case(
    item_id: u32,
    modifier_id: Option<u32>,
    map_id: Option<u32>,
    context_present: bool,
    seed_lock: bool,
) {
    let (env, slot) = load_slot_keystone_rejection_fixture(
        item_id,
        modifier_id,
        map_id,
        context_present,
        seed_lock,
    );
    assert_slot_keystone_rejection_not_published(&env);
    if seed_lock {
        assert_rejection_clears_seeded_item_lock(&env, slot);
    }
}

fn load_slot_keystone_rejection_fixture(
    item_id: u32,
    modifier_id: Option<u32>,
    map_id: Option<u32>,
    context_present: bool,
    seed_lock: bool,
) -> (WowLuaEnv, u32) {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    register_keystone_event_probe(&env);
    let slot = 3;
    add_bag_item(&env, 0, slot, item_id);
    env.exec(&format!("C_Container.PickupContainerItem(0, {slot})"))
        .unwrap();
    {
        let mut sim = env.state().borrow_mut();
        let location = KeystoneLocation::Bag(0, slot as i32);
        if let Some(modifier_id) = modifier_id {
            sim.keystone
                .item_modifiers
                .insert(location, (item_id, vec![(17, modifier_id)]));
        }
        sim.keystone.current_map_selector = map_id;
        sim.player.present = context_present;
        if seed_lock {
            sim.keystone.reason_one_locks.insert(location);
        }
    }
    (env, slot)
}

fn assert_slot_keystone_rejection_not_published(env: &WowLuaEnv) {
    env.exec(
        r##"
        assert(select("#", C_ChallengeMode.SlotKeystone()) == 0)
        assert(not CursorHasItem())
        assert(not C_ChallengeMode.HasSlottedKeystone())
        assert(select("#", C_TooltipInfo.GetSlottedKeystone()) == 0)
        assert(#R7KeystoneSlottedState == 0)
        "##,
    )
    .unwrap();
}

fn assert_rejection_clears_seeded_item_lock(env: &WowLuaEnv, slot: u32) {
    env.exec(&format!(
        r#"
        A_Admin.AddBagItem(0, {slot}, 6948, 1)
        assert(not C_Item.IsLocked(ItemLocation:CreateFromBagAndSlot(0, {slot})))
        "#
    ))
    .unwrap();
}

#[test]
fn slot_keystone_rejects_unmatched_or_missing_inputs_before_publication() {
    let cases = [
        (158_923, Some(2), Some(961), true, true),
        (158_923, Some(999_999), Some(960), true, false),
        (158_923, None, Some(960), true, false),
        (158_923, Some(2), None, true, false),
        (158_923, Some(2), Some(960), false, false),
        (6_948, Some(2), Some(960), true, false),
    ];
    for (item_id, modifier_id, map_id, context_present, seed_lock) in cases {
        assert_slot_keystone_rejection_case(
            item_id,
            modifier_id,
            map_id,
            context_present,
            seed_lock,
        );
    }
}

// SUPPLEMENTARY, INFERRED model coverage: one positive case cannot reject fixed staged output.
#[test]
fn remove_keystone_stages_item_for_nonzero_guid_category() {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    slot_keystone_from_bag(&env, 3, 158_923);
    env.state().borrow_mut().keystone.cursor_stage_permission = Some(true);
    env.exec(
        r##"
        assert(C_ChallengeMode.RemoveKeystone())
        assert(not C_ChallengeMode.HasSlottedKeystone())
        local kind, cursorItemID = GetCursorInfo()
        assert(kind == "item" and cursorItemID == 158923)
        "##,
    )
    .unwrap();
}

fn setup_keystone_removal_case(env: &WowLuaEnv, selected_guid: &str) {
    env.exec(
        r##"
        A_Admin.AddBagItem(0, 3, 158923, 1)
        A_Admin.AddBagItem(0, 4, 6948, 1)
        C_Container.PickupContainerItem(0, 4)
        "##,
    )
    .unwrap();
    let mut sim = env.state().borrow_mut();
    sim.keystone.selected_guid = Some(selected_guid.to_string());
    sim.keystone.selected_item_id = Some(158_923);
    sim.keystone.selected_location = Some(KeystoneLocation::Bag(0, 3));
    sim.keystone.cursor_stage_permission = Some(true);
    sim.player.present = true;
}

fn assert_keystone_removal_result(env: &WowLuaEnv) {
    env.exec(
        r##"
        local function capture(...) return {n = select("#", ...), ...} end
        local removed = capture(C_ChallengeMode.RemoveKeystone())
        assert(removed.n == 1 and removed[1] == true)
        assert(not C_ChallengeMode.HasSlottedKeystone())
        "##,
    )
    .unwrap();
}

// SUPPLEMENTARY direct API test: asserts only the verified Boolean and selection clear.
#[test]
fn remove_keystone_returns_true_and_clears_nonempty_selection() {
    for selected_guid in ["0-0-0-158923", "Item-0-3-158923"] {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        setup_keystone_removal_case(&env, selected_guid);
        assert_keystone_removal_result(&env);
    }
}

// SUPPLEMENTARY, INFERRED model coverage: all combinations deny through the context/permission proxies.
#[test]
fn remove_keystone_stage_gate_denial_preserves_cursor_and_clears_selection() {
    for (permission, context_present) in [(None, true), (Some(false), true), (Some(true), false)] {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        env.exec(
            r##"
            A_Admin.AddBagItem(0, 4, 6948, 1)
            C_Container.PickupContainerItem(0, 4)
            "##,
        )
        .unwrap();
        {
            let mut sim = env.state().borrow_mut();
            sim.keystone.selected_guid = Some("Item-0-3-158923".to_string());
            sim.keystone.cursor_stage_permission = permission;
            sim.player.present = context_present;
        }
        env.exec(
            r##"
            assert(C_ChallengeMode.RemoveKeystone())
            assert(not C_ChallengeMode.HasSlottedKeystone())
            local kind, cursorItemID = GetCursorInfo()
            assert(kind == "item" and cursorItemID == 6948)
            "##,
        )
        .unwrap();
    }
}

// SUPPLEMENTARY: the empty-selection path does not exercise nonempty staging.
#[test]
fn remove_keystone_without_selection_preserves_cursor() {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    env.exec(
        r##"
        A_Admin.AddBagItem(0, 4, 6948, 1)
        C_Container.PickupContainerItem(0, 4)
        "##,
    )
    .unwrap();
    env.state().borrow_mut().keystone.cursor_stage_permission = Some(true);
    env.exec(
        r##"
        assert(not C_ChallengeMode.RemoveKeystone())
        assert(not C_ChallengeMode.HasSlottedKeystone())
        local kind, cursorItemID = GetCursorInfo()
        assert(kind == "item" and cursorItemID == 6948)
        "##,
    )
    .unwrap();
}

// SUPPLEMENTARY contract test: this calls the public API directly.
#[test]
fn has_slotted_keystone_tracks_guid_without_resolving_the_item() {
    let env = load_challenges_env();
    env.exec("A_Admin.ClearBags(); C_ChallengeMode.ClearKeystone()")
        .expect("unresolved item fixture starts empty");
    env.exec(
        r##"
        local function capture(...) return {n = select("#", ...), ...} end
        local result = capture(C_ChallengeMode.HasSlottedKeystone("ignored", 17))
        assert(result.n == 1 and type(result[1]) == "boolean" and not result[1])
    "##,
    )
    .unwrap();

    env.state().borrow_mut().keystone.selected_guid = Some("Item-0-3-999999".to_string());
    env.exec(
        r##"
        local function capture(...) return {n = select("#", ...), ...} end
        local result = capture(C_ChallengeMode.HasSlottedKeystone("ignored", 17))
        assert(result.n == 1 and type(result[1]) == "boolean" and result[1])
        assert(select("#", C_TooltipInfo.GetSlottedKeystone()) == 0)
        local removed = capture(C_ChallengeMode.RemoveKeystone())
        assert(removed.n == 1 and removed[1] == true)
        assert(not C_ChallengeMode.HasSlottedKeystone())
    "##,
    )
    .unwrap();
}

fn assert_round7_tooltip_misses(env: &WowLuaEnv) {
    env.exec(
        r##"
        assert(select("#", C_TooltipInfo.GetItemInteractionItem()) == 0)
        assert(select("#", C_TooltipInfo.GetSlottedKeystone()) == 0)
        "##,
    )
    .unwrap();
}

fn assert_possession_tooltip_misses(env: &WowLuaEnv) {
    env.exec(
        r##"
        assert(select("#", C_TooltipInfo.GetPossession(1)) == 0)
        assert(select("#", C_TooltipInfo.GetPossession(2)) == 0)
        "##,
    )
    .unwrap();
    env.state().borrow_mut().possession_spell_id = Some(0);
    env.exec(r##"assert(select("#", C_TooltipInfo.GetPossession(1)) == 0)"##)
        .unwrap();
    env.state().borrow_mut().possession_spell_id = None;
}

fn assert_item_interaction_tooltip_misses(env: &WowLuaEnv) {
    env.state().borrow_mut().item_interaction = Some(ItemInteractionInfo {
        interaction_type: 2,
        ..Default::default()
    });
    env.state().borrow_mut().item_interaction_tooltip_input = Some(ItemInteractionTooltipInput {
        selected_item_guid: Some("Item-0-1-1".to_string()),
        ..Default::default()
    });
    env.exec(r##"assert(select("#", C_TooltipInfo.GetItemInteractionItem()) == 0)"##)
        .unwrap();
    env.state().borrow_mut().item_interaction_tooltip_input = None;
    env.exec(r##"assert(select("#", C_TooltipInfo.GetItemInteractionItem()) == 0)"##)
        .unwrap();
}

// PRESERVATION: these direct getter misses keep the verified zero-value boundary.
#[test]
fn round7_selected_getter_misses_keep_the_verified_result_arities() {
    let env = WowLuaEnv::new().unwrap();
    assert_round7_tooltip_misses(&env);
    assert_possession_tooltip_misses(&env);
    assert_item_interaction_tooltip_misses(&env);
}

fn set_unsupported_item_interaction(
    env: &WowLuaEnv,
    object_kind: Option<i32>,
    selected_guid: Option<String>,
) {
    let mut sim = env.state().borrow_mut();
    sim.item_interaction = Some(ItemInteractionInfo {
        interaction_type: 1,
        ..Default::default()
    });
    sim.item_interaction_tooltip_input = Some(ItemInteractionTooltipInput {
        object_kind,
        selected_item_guid: selected_guid,
        ..Default::default()
    });
}

fn assert_unsupported_item_interaction_miss(env: &WowLuaEnv, reason: &str) {
    env.exec(r##"assert(select("#", C_TooltipInfo.GetItemInteractionItem()) == 0)"##)
        .expect(reason);
}

fn assert_resolved_unsupported_item_shape(env: &WowLuaEnv) {
    env.exec(
        r##"
        local count, data = select("#", C_TooltipInfo.GetItemInteractionItem()),
            C_TooltipInfo.GetItemInteractionItem()
        assert(
            count == 1 and type(data) == "table",
            "resolved unsupported kind returns one TooltipData table"
        )
        assert(
            type(data.lines) == "table" and next(data.lines) == nil,
            "unsupported-kind lines are empty"
        )
        assert(data.dataInstanceID == nil, "unsupported-kind result has no dataInstanceID")
        local fields = 0
        for key in pairs(data) do
            fields = fields + 1
            assert(key == "lines", "unsupported-kind result has only lines")
        end
        assert(fields == 1, "unsupported-kind result has exactly one field")
        "##,
    )
    .expect("existing unsupported-kind object preserves its exact native result shape");
}

// SUPPLEMENTARY contract test: this calls C_TooltipInfo directly, not its original consumer.
#[test]
fn unsupported_item_interaction_kind_requires_resolved_item() {
    let env = load_item_interaction_env();
    env.exec("A_Admin.ClearBags()")
        .expect("absent and unresolved fixtures start without bag items");
    set_unsupported_item_interaction(&env, None, None);
    assert_unsupported_item_interaction_miss(&env, "unsupported kind with no object");
    set_unsupported_item_interaction(&env, None, Some("Item-0-3-1728".to_string()));
    assert_unsupported_item_interaction_miss(&env, "unsupported kind with unresolved GUID");

    let selected_guid = add_bag_item(&env, 0, 3, 1728);
    set_unsupported_item_interaction(&env, Some(1), Some(selected_guid));
    assert_resolved_unsupported_item_shape(&env);
}
