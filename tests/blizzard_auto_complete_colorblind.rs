use crate::common;

use common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_AutoComplete";
const COLORBLIND_PROBE_LUA: &str = r#"
local failures = {}

local function expect(condition, message)
  if not condition then
    table.insert(failures, message)
  end
end

local function friendResult()
  return {
    { name = "Alice", priority = Enum.AutoCompletePriority.Friend },
  }
end

AutoComplete_OnLoad(AutoCompleteBox)

SetCVar("colorblindMode", "1")
AutoComplete_UpdateResults(AutoCompleteBox, friendResult())

local colorblindText = AutoCompleteButton1:GetText()
expect(colorblindText == "Alice " .. AUTOCOMPLETE_LABEL_FRIEND,
       "colorblind text must append friend label, got " .. tostring(colorblindText))

SetCVar("colorblindMode", "0")
AutoComplete_UpdateResults(AutoCompleteBox, friendResult())

local normalText = AutoCompleteButton1:GetText()
expect(not normalText:find(AUTOCOMPLETE_LABEL_FRIEND, 1, true),
       "normal text must not append friend label, got " .. tostring(normalText))
expect(normalText:match("^|cff%x%x%x%x%x%xAlice|r$") ~= nil,
       "normal text must use a color escape sequence, got " .. tostring(normalText))

return table.concat(failures, "\n")
"#;

post_wrath_ui_fixture! {
#[test]
fn blizzard_auto_complete_colorblind_mode_appends_priority_label() {
    common::with_perf_lock(|| {
        common::with_timeout(240, || {
            with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, loaded| {
                assert!(
                    loaded.iter().any(|name| name == ROOT),
                    "`{ROOT}` must load before AutoComplete colorblind text can be checked. \
                     Loaded set: {loaded:?}"
                );

                let failures: String = env
                    .eval(COLORBLIND_PROBE_LUA)
                    .expect("AutoComplete colorblind probe should run");
                assert!(
                    failures.is_empty(),
                    "`{ROOT}` colorblind text mismatches:\n{failures}"
                );
            });
        });
    });
}
}

const NONBINARY_COLORBLIND_PROBE_LUA: &str = r#"
local results = {{ name = "Alice", priority = Enum.AutoCompletePriority.Friend }}
AutoComplete_OnLoad(AutoCompleteBox)
local function renderedText(cvar)
  SetCVar("colorblindMode", cvar)
  AutoComplete_UpdateResults(AutoCompleteBox, results)
  return AutoCompleteButton1:GetText()
end
local normal = renderedText("0")
local nonbinary = renderedText("2")
local restored = renderedText("0")
return normal, nonbinary, restored, AUTOCOMPLETE_LABEL_FRIEND
"#;

post_wrath_ui_fixture! {
#[test]
fn blizzard_auto_complete_nonbinary_colorblind_mode_updates_label() {
    common::with_perf_lock(|| {
        common::with_timeout(90, || {
            with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, loaded| {
                assert!(loaded.iter().any(|name| name == ROOT));
                let (normal, nonbinary, restored, label): (String, String, String, String) = env
                    .eval(NONBINARY_COLORBLIND_PROBE_LUA)
                    .expect("original AutoComplete result text");
                assert!(normal.starts_with("|cff") && normal.ends_with("Alice|r"));
                assert_eq!(restored, normal);
                assert_eq!(nonbinary, format!("Alice {label}"));
            });
        });
    });
}
}
