use crate::common;

use common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_AutoComplete";
const REALM_APPEND_PROBE_LUA: &str = r#"
local failures = {}

local function expect(condition, message)
  if not condition then
    table.insert(failures, message)
  end
end

A_Admin.SetAutoCompleteRealms({ "Stormrage", "Stormscale" })

local function makeEditBox(sourceFn)
  local editBox = CreateFrame("EditBox", nil, UIParent)
  editBox:SetSize(200, 20)
  editBox:SetPoint("CENTER", UIParent, "CENTER", 0, 0)
  AutoCompleteEditBox_SetAutoCompleteSource(editBox, sourceFn)
  return editBox
end

local function assertDisplayedRealm(index, name)
  local button = _G["AutoCompleteButton" .. index]
  expect(button:IsShown(), name .. " must be displayed")
  expect(button.nameInfo and button.nameInfo.name == name,
         "expected displayed result " .. name)
  expect(button:GetText() ~= nil, name .. " must have visible text")
end

local editBox = makeEditBox(function()
  return {}
end)

AutoComplete_Update(editBox, "playername-Storm", strlen("playername-Storm"))
assertDisplayedRealm(1, "playername-Stormrage")
assertDisplayedRealm(2, "playername-Stormscale")

return table.concat(failures, "\n")
"#;

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
#[test]
fn blizzard_auto_complete_appends_matching_realm_names() {
    common::with_perf_lock(|| {
        common::with_timeout(240, || {
            with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, loaded| {
                assert!(
                    loaded.iter().any(|name| name == ROOT),
                    "`{ROOT}` must load before AutoComplete realm appending can be checked. \
                     Loaded set: {loaded:?}"
                );

                let failures: String = env
                    .eval(REALM_APPEND_PROBE_LUA)
                    .expect("AutoComplete realm append probe should run");
                assert!(
                    failures.is_empty(),
                    "`{ROOT}` realm appending mismatches:\n{failures}"
                );
            });
        });
    });
}
