use crate::common;

use common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_AutoCompletePopupList";
const DISPLAY_CALLBACK_PROBE_LUA: &str = r#"
local failures = {}

local function expect(condition, message)
  if not condition then
    table.insert(failures, message)
  end
end

local texturePath = "Interface\\Icons\\Ability_ThunderClap"
local textureID = 132326
local popup = CreateFrame("Frame", "TestPopupListDisplayCallbackFrame",
                         UIParent, "AutoCompletePopupListTemplate")
expect(popup ~= nil, "AutoCompletePopupListTemplate must instantiate")

if popup ~= nil then
  popup:OnLoad()
  popup:SetSize(176, 100)
  popup:SetPoint("CENTER", UIParent, "CENTER")
  popup.ScrollBox:SetSize(176, 100)
  popup.resultsListCallback = function()
    return 2, {
      { token = "first" },
      { token = "second" },
    }, function()
      return "Custom", "sub", texturePath
    end
  end

  popup:UpdateResults()

  local rowCount = 0
  popup.ScrollBox:ForEachFrame(function(row)
    rowCount = rowCount + 1
    expect(row.Name:GetText() == "Custom",
           "row " .. tostring(rowCount) .. " Name must use displayText")
    expect(row.Subtext:GetText() == "sub",
           "row " .. tostring(rowCount) .. " Subtext must use subtext")
    expect(row.Subtext:IsShown(),
           "row " .. tostring(rowCount) .. " Subtext must be shown")
    expect(row.Icon:IsShown(),
           "row " .. tostring(rowCount) .. " Icon must be shown")
    expect(row.IconFrame:IsShown(),
           "row " .. tostring(rowCount) .. " IconFrame must be shown")
    expect(row.Icon:GetTexture() == textureID,
           "row " .. tostring(rowCount) .. " Icon texture path " ..
           texturePath .. " must resolve to FDID " .. tostring(textureID) ..
           ", got " .. tostring(row.Icon:GetTexture()))
    expect(row.Name:GetMaxLines() == 1,
           "row " .. tostring(rowCount) .. " Name max lines must be 1")
  end)
  expect(rowCount == 2,
         "display callback popup must initialize 2 rows, got " ..
         tostring(rowCount))
end

local noSubtextPopup = CreateFrame("Frame", "TestPopupListNoSubtextFrame",
                                  UIParent, "AutoCompletePopupListTemplate")
expect(noSubtextPopup ~= nil,
       "AutoCompletePopupListTemplate must instantiate no-subtext popup")

if noSubtextPopup ~= nil then
  noSubtextPopup:OnLoad()
  noSubtextPopup:SetSize(176, 100)
  noSubtextPopup:SetPoint("CENTER", UIParent, "CENTER")
  noSubtextPopup.ScrollBox:SetSize(176, 100)
  noSubtextPopup.resultsListCallback = function()
    return 1, { { token = "plain" } }, function()
      return "Plain", nil, nil
    end
  end

  noSubtextPopup:UpdateResults()

  local rowCount = 0
  noSubtextPopup.ScrollBox:ForEachFrame(function(row)
    rowCount = rowCount + 1
    expect(row.Name:GetText() == "Plain",
           "no-subtext row Name must use displayText")
    expect(not row.Subtext:IsShown(),
           "no-subtext row Subtext must be hidden")
    expect(not row.Icon:IsShown(), "no-subtext row Icon must be hidden")
    expect(row.Name:GetMaxLines() == 2,
           "no-subtext row Name max lines must be 2")
  end)
  expect(rowCount == 1,
         "no-subtext popup must initialize 1 row, got " .. tostring(rowCount))
end

return table.concat(failures, "\n")
"#;

post_wrath_ui_fixture! {
#[test]
fn blizzard_auto_complete_popup_list_display_callback_initializes_rows() {
    common::with_perf_lock(|| {
        common::with_timeout(240, || {
            with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, loaded| {
                assert!(
                    loaded.iter().any(|name| name == ROOT),
                    "`{ROOT}` must load before PopupList display callback can be checked. \
                     Loaded set: {loaded:?}"
                );

                let failures: String = env
                    .eval(DISPLAY_CALLBACK_PROBE_LUA)
                    .expect("AutoCompletePopupList display callback probe should run");
                assert!(
                    failures.is_empty(),
                    "`{ROOT}` display callback mismatches:\n{failures}"
                );
            });
        });
    });
}
}
