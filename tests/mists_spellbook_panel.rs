#![cfg(feature = "client-mists")]

#[path = "common/lua_error_output.rs"]
mod lua_error_output;

use std::process::Command;

#[path = "common/bounded_process.rs"]
mod bounded_process;

#[path = "common/wow_sim_binary.rs"]
mod wow_sim_binary;

const SPELLBOOK_PANEL_PROBE_LUA: &str = r#"
ToggleSpellBook(BOOKTYPE_SPELL)
if SpellBookFrame and SpellBookFrame.Update then
    SpellBookFrame:Update()
end

local populated = 0
for i = 1, 12 do
    local button = _G["SpellButton" .. i]
    local text = button and button.SpellName and button.SpellName:GetText()
    local texture = button and button.IconTexture and button.IconTexture:GetTexture()
    if text and text ~= "" and texture then
        if button:GetWidth() < 37 or button:GetHeight() < 37 then
            error(("SpellButton%d collapsed to %sx%s"):format(
                i,
                tostring(button:GetWidth()),
                tostring(button:GetHeight())
            ))
        end
        populated = populated + 1
    end
end

if populated == 0 then
    error("spellbook has no populated spell buttons")
end

local expectedTabs = {
    { frame = SpellBookFrameTabButton1, text = "Spellbook" },
    { frame = SpellBookFrameTabButton2, text = "Professions" },
    { frame = SpellBookFrameTabButton3, text = "Core Abilities" },
    { frame = SpellBookFrameTabButton4, text = "What's Changed" },
}
local function assertVisibleTabChrome(index, tab)
    local name = tab:GetName()
    local normalVisible = _G[name .. "Left"]:IsVisible()
        and _G[name .. "Middle"]:IsVisible()
        and _G[name .. "Right"]:IsVisible()
    local activeVisible = _G[name .. "LeftDisabled"]:IsVisible()
        and _G[name .. "MiddleDisabled"]:IsVisible()
        and _G[name .. "RightDisabled"]:IsVisible()
    if not normalVisible and not activeVisible then
        error(("spellbook bottom tab %d has no visible texture set"):format(index))
    end
    for _, suffix in ipairs({
        "Left",
        "Middle",
        "Right",
        "LeftDisabled",
        "MiddleDisabled",
        "RightDisabled",
    }) do
        local texture = _G[name .. suffix]
        local asset = texture and texture:GetTexture()
        -- GetTexture can return a file ID for Blizzard assets.
        if not ((type(asset) == "number" and asset > 0)
            or (type(asset) == "string" and asset ~= "")) then
            error(("spellbook bottom tab %d %s texture has no asset"):format(
                index,
                suffix
            ))
        end
    end
end

local previousRight = nil
for index, expected in ipairs(expectedTabs) do
    local tab = expected.frame
    if not tab or not tab:IsShown() then
        error(("spellbook bottom tab %d is missing or hidden"):format(index))
    end
    local text = tab:GetText()
    if type(text) ~= "string" or text == "" then
        error(("spellbook bottom tab %d has empty text"):format(index))
    end
    if text ~= expected.text then
        error(("spellbook bottom tab %d text is %q instead of %q"):format(
            index,
            tostring(text),
            expected.text
        ))
    end
    local label = tab:GetFontString()
    local labelWidth = label and label:GetWidth() or 0
    if tab:GetWidth() + 0.5 < labelWidth + 40 then
        error(("spellbook bottom tab %d is too narrow: tab=%s text=%s"):format(
            index,
            tostring(tab:GetWidth()),
            tostring(labelWidth)
        ))
    end
    if previousRight and tab:GetLeft() < previousRight - 15.5 then
        error(("spellbook bottom tab %d overlaps the previous tab"):format(index))
    end
    assertVisibleTabChrome(index, tab)
    previousRight = tab:GetRight()
end
if SpellBookFrameTabButton5 and SpellBookFrameTabButton5:IsShown() then
    error("unexpected fifth spellbook bottom tab is visible")
end
for _, suffix in ipairs({
    "Left",
    "Middle",
    "Right",
    "LeftDisabled",
    "MiddleDisabled",
    "RightDisabled",
    "Text",
}) do
    local region = _G["SpellBookFrameTabButton5" .. suffix]
    if region and region:IsVisible() then
        error("hidden fifth spellbook tab leaked visible " .. suffix)
    end
end

SpellBookFrameTabButton_OnClick(SpellBookFrameTabButton4)
local changedItems = SpellBookWhatHasChanged and SpellBookWhatHasChanged.ChangedItems
if not SpellBookWhatHasChanged or not SpellBookWhatHasChanged:IsShown() then
    error("What's Changed frame is not visible")
end
if not changedItems or #changedItems < 3 then
    error("What's Changed tab has no populated class rows")
end

local visibleChangedRows = 0
for index, item in ipairs(changedItems) do
    local numberText = item.Number and item.Number:GetText()
    local titleText = item.Title and item.Title:GetText()
    local bodyText = item.GetText and item:GetText()
    if item:IsShown() then
        visibleChangedRows = visibleChangedRows + 1
        if numberText ~= tostring(index) then
            error(("What's Changed row %d has number %q"):format(index, tostring(numberText)))
        end
        if type(titleText) ~= "string" or titleText == "" then
            error(("What's Changed row %d has no title"):format(index))
        end
        if type(bodyText) ~= "string" or bodyText == "" then
            error(("What's Changed row %d has no body text"):format(index))
        end
    end
end
if visibleChangedRows < 3 then
    error("What's Changed tab has too few visible rows: " .. tostring(visibleChangedRows))
end

SpellBookFrameTabButton_OnClick(SpellBookFrameTabButton2)
SpellBook_UpdateProfTab()

local professionButtons = {}
for _, profession in ipairs({
    PrimaryProfession1,
    PrimaryProfession2,
    SecondaryProfession1,
    SecondaryProfession2,
    SecondaryProfession3,
}) do
    if profession then
        table.insert(professionButtons, profession.SpellButton1)
        table.insert(professionButtons, profession.SpellButton2)
    end
end
local sizedProfessionButtons = 0
for _, button in ipairs(professionButtons) do
    if button and button:IsShown() then
        if button:GetWidth() < 40 or button:GetHeight() < 40 then
            error(("profession button collapsed to %sx%s"):format(
                tostring(button:GetWidth()),
                tostring(button:GetHeight())
            ))
        end
        sizedProfessionButtons = sizedProfessionButtons + 1
    end
end

if sizedProfessionButtons == 0 then
    error("professions tab has no visible profession buttons")
end

local miningButton = PrimaryProfession2 and PrimaryProfession2.SpellButton1
if miningButton == nil or miningButton:IsShown() ~= true then
    error("mining profession button is not visible")
end
local onClick = miningButton:GetScript("OnClick")
if type(onClick) ~= "function" then
    error("mining profession button has no OnClick handler")
end

local beforeLineName = GetTradeSkillLine()
if beforeLineName == "Mining" then
    error("profession button test started with Mining already selected")
end

onClick(miningButton, "LeftButton")
local lineName = GetTradeSkillLine()
if lineName ~= "Mining" then
    error("profession button click selected " .. tostring(lineName) .. " instead of Mining")
end
"#;

#[test]
fn mists_spellbook_populates_visible_spell_buttons() {
    let output = bounded_process::output(
        90,
        Command::new(wow_sim_binary::path()).args([
            "--no-addons",
            "--no-saved-vars",
            "--exec-lua",
            SPELLBOOK_PANEL_PROBE_LUA,
            "dump-tree",
            "--filter-key",
            "SpellBookFrame",
        ]),
    )
    .expect("failed to run wow-sim");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "wow-sim failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    lua_error_output::assert_no_lua_sink_errors("spellbook opened", &stdout, &stderr);
}

