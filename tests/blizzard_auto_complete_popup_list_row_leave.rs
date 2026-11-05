use crate::common;

use common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_AutoCompletePopupList";
const ROW_LEAVE_PROBE_LUA: &str = r#"
local failures = {}

local function expect(condition, message)
  if not condition then
    table.insert(failures, message)
  end
end

local originalGetAppropriateTooltip = GetAppropriateTooltip

local tooltip = {
  shown = true,
  Hide = function(self)
    self.shown = false
  end,
  IsShown = function(self)
    return self.shown
  end,
}

GetAppropriateTooltip = function()
  return tooltip
end

local popup = CreateFrame("Frame", "TestPopupListRowLeaveOwner", UIParent,
                         "AutoCompletePopupListTemplate")
local row = CreateFrame("Button", "TestPopupListRowLeaveButton", popup,
                       "AutoCompletePopupListResultTemplate")

row:Init({
  resultInfo = { text = "Leave" },
  index = 1,
  owner = popup,
  displayText = "Leave",
  subtext = nil,
  displayTexture = nil,
})

row:OnLeave()

expect(not tooltip:IsShown(),
       "OnLeave must hide GetAppropriateTooltip()")

GetAppropriateTooltip = originalGetAppropriateTooltip

return table.concat(failures, "\n")
"#;

timerunning_util_panel_fixture! {
#[test]
fn blizzard_auto_complete_popup_list_row_leave_hides_tooltip() {
    common::with_perf_lock(|| {
        common::with_timeout(240, || {
            with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, loaded| {
                assert!(
                    loaded.iter().any(|name| name == ROOT),
                    "`{ROOT}` must load before PopupList row leave can be checked. \
                     Loaded set: {loaded:?}"
                );

                let failures: String = env
                    .eval(ROW_LEAVE_PROBE_LUA)
                    .expect("AutoCompletePopupList row leave probe should run");
                assert!(
                    failures.is_empty(),
                    "`{ROOT}` row leave mismatches:\n{failures}"
                );
            });
        });
    });
}
}
