use crate::common;

use common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_AutoCompletePopupList";
const ROW_ENTER_PROBE_LUA: &str = r#"
local failures = {}

local function expect(condition, message)
  if not condition then
    table.insert(failures, message)
  end
end

local originalGetAppropriateTooltip = GetAppropriateTooltip
local originalAddColoredLine = GameTooltip_AddColoredLine

local tooltip = {
  owner = nil,
  anchor = nil,
  showCount = 0,
  lines = {},
  SetOwner = function(self, owner, anchor)
    self.owner = owner
    self.anchor = anchor
  end,
  Show = function(self)
    self.showCount = self.showCount + 1
  end,
}

GetAppropriateTooltip = function()
  return tooltip
end

GameTooltip_AddColoredLine = function(targetTooltip, text, color)
  table.insert(targetTooltip.lines, { text = text, color = color })
end

local function newRow(name, index)
  local popup = CreateFrame("Frame", name .. "Owner", UIParent,
                           "AutoCompletePopupListTemplate")
  popup.HighlightResult = function(self, highlightIndex)
    self.highlightedIndex = highlightIndex
  end

  local row = CreateFrame("Button", name, popup,
                          "AutoCompletePopupListResultTemplate")
  row:Init({
    resultInfo = { text = name },
    index = index,
    owner = popup,
    displayText = name .. " display",
    subtext = name .. " subtext",
    displayTexture = nil,
  })

  return row, popup
end

local truncatedRow, truncatedPopup = newRow("TestPopupListRowEnterTruncated", 2)
truncatedRow.Name.IsTruncated = function() return true end
truncatedRow.Subtext.IsTruncated = function() return false end
truncatedRow:OnEnter()

expect(truncatedPopup.highlightedIndex == 2,
       "OnEnter must highlight row index 2, got " ..
       tostring(truncatedPopup.highlightedIndex))
expect(tooltip.owner == truncatedRow,
       "OnEnter tooltip owner must be the entered row")
expect(tooltip.anchor == "ANCHOR_RIGHT",
       "OnEnter tooltip anchor must be ANCHOR_RIGHT")
expect(tooltip.showCount == 1,
       "OnEnter must show tooltip when row text is truncated")
expect(#tooltip.lines == 2,
       "truncated row tooltip must have 2 lines, got " .. tostring(#tooltip.lines))
expect(tooltip.lines[1] and tooltip.lines[1].text == truncatedRow.Name:GetText(),
       "first tooltip line must use Name text")
expect(tooltip.lines[1] and tooltip.lines[1].color == HIGHLIGHT_FONT_COLOR,
       "first tooltip line must use HIGHLIGHT_FONT_COLOR")
expect(tooltip.lines[2] and tooltip.lines[2].text == truncatedRow.Subtext:GetText(),
       "second tooltip line must use Subtext text")
expect(tooltip.lines[2] and tooltip.lines[2].color == GRAY_FONT_COLOR,
       "second tooltip line must use GRAY_FONT_COLOR")

tooltip.owner = nil
tooltip.anchor = nil
tooltip.showCount = 0
tooltip.lines = {}

local untruncatedRow, untruncatedPopup = newRow("TestPopupListRowEnterPlain", 2)
untruncatedRow.Name.IsTruncated = function() return false end
untruncatedRow.Subtext.IsTruncated = function() return false end
untruncatedRow:OnEnter()

expect(untruncatedPopup.highlightedIndex == 2,
       "untruncated OnEnter must still highlight row index 2, got " ..
       tostring(untruncatedPopup.highlightedIndex))
expect(tooltip.showCount == 0,
       "OnEnter must not show tooltip when neither text field is truncated")
expect(#tooltip.lines == 0,
       "OnEnter must not add tooltip lines when neither text field is truncated")
expect(tooltip.owner == nil,
       "OnEnter must not set tooltip owner when neither text field is truncated")

GetAppropriateTooltip = originalGetAppropriateTooltip
GameTooltip_AddColoredLine = originalAddColoredLine

return table.concat(failures, "\n")
"#;

timerunning_util_panel_fixture! {
#[test]
fn blizzard_auto_complete_popup_list_row_enter_highlights_and_tooltips() {
    common::with_perf_lock(|| {
        common::with_timeout(240, || {
            with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, loaded| {
                assert!(
                    loaded.iter().any(|name| name == ROOT),
                    "`{ROOT}` must load before PopupList row enter can be checked. \
                     Loaded set: {loaded:?}"
                );

                let failures: String = env
                    .eval(ROW_ENTER_PROBE_LUA)
                    .expect("AutoCompletePopupList row enter probe should run");
                assert!(
                    failures.is_empty(),
                    "`{ROOT}` row enter mismatches:\n{failures}"
                );
            });
        });
    });
}
}
