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

local originalGetRealms = C_AutoComplete.GetAutoCompleteRealms
local originalUpdateResults = AutoComplete_UpdateResults
local capturedResults

C_AutoComplete.GetAutoCompleteRealms = function()
  return { "Stormrage", "Stormscale" }
end

AutoComplete_UpdateResults = function(self, results)
  capturedResults = results
end

local function makeEditBox(sourceFn)
  local editBox = CreateFrame("EditBox", nil, UIParent)
  editBox:SetSize(200, 20)
  editBox:SetPoint("CENTER", UIParent, "CENTER", 0, 0)
  AutoCompleteEditBox_SetAutoCompleteSource(editBox, sourceFn)
  return editBox
end

local function countNamedResult(results, expectedName)
  local count = 0
  local priority
  for index = 1, #results do
    local result = results[index]
    if type(result) == "table" and result.name == expectedName then
      count = count + 1
      priority = result.priority
    elseif result == expectedName then
      count = count + 1
    end
  end
  return count, priority
end

local function assertAppendedRealm(results, name)
  local count, priority = countNamedResult(results, name)
  expect(count == 1, name .. " must appear exactly once, got " .. tostring(count))
  expect(priority == Enum.AutoCompletePriority.Other,
         name .. " must use Enum.AutoCompletePriority.Other")
end

local editBox = makeEditBox(function()
  return {}
end)

AutoComplete_Update(editBox, "playername-Storm", strlen("playername-Storm"))
expect(#capturedResults == 2, "two matching realms must be appended")
assertAppendedRealm(capturedResults, "playername-Stormrage")
assertAppendedRealm(capturedResults, "playername-Stormscale")

local duplicateEditBox = makeEditBox(function()
  return { "playername-Stormscale" }
end)

capturedResults = nil
AutoComplete_Update(duplicateEditBox, "playername-Storm", strlen("playername-Storm"))

expect(#capturedResults == 2, "duplicate source string must suppress one appended realm")
assertAppendedRealm(capturedResults, "playername-Stormrage")
local stormscaleCount = countNamedResult(capturedResults, "playername-Stormscale")
expect(stormscaleCount == 1, "duplicate Stormscale completion must not be appended")

C_AutoComplete.GetAutoCompleteRealms = originalGetRealms
AutoComplete_UpdateResults = originalUpdateResults

return table.concat(failures, "\n")
"#;

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
