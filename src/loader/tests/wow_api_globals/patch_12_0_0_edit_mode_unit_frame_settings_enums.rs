//! Focused retail 12.0.0 Edit Mode unit-frame setting enum values.

#![cfg(feature = "retail-12-0-0")]

use super::super::*;

#[test]
fn test_patch_12_0_0_edit_mode_unit_frame_setting_enum_values() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
                local setting = Enum.EditModeUnitFrameSetting
                if type(setting) ~= "table" then
                    return "EditModeUnitFrameSetting: expected table"
                end

                local current = CURRENT_RETAIL
                local expected = {
                    AuraOrganizationType = 18,
                    Opacity = 20,
                }
                if current then
                    expected.DebuffIconSize = 19
                    expected.BigDefensiveIconSize = 21
                    expected.BuffIconSize = 22
                    if setting.IconSize ~= nil then
                        return "IconSize: obsolete current-retail alias"
                    end
                else
                    expected.IconSize = 19
                end
                for name, expected_value in pairs(expected) do
                    local value = setting[name]
                    if type(value) ~= "number" or value ~= expected_value then
                        return name .. ": expected " .. tostring(expected_value)
                            .. ", got " .. tostring(value)
                    end
                end
                if not current and setting.BigDefensiveIconSize ~= nil then
                    return "BigDefensiveIconSize: expected nil, got "
                        .. tostring(setting.BigDefensiveIconSize)
                end

                local meta = Enum.EditModeUnitFrameSettingMeta
                if type(meta) ~= "table" then
                    return "EditModeUnitFrameSettingMeta: expected table"
                end
                local max = current and 22 or 20
                local count = current and 23 or 21
                if meta.MinValue ~= 0 or meta.MaxValue ~= max or meta.NumValues ~= count then
                    return "metadata mismatch"
                end
                return "ok"
            "#
            .replace(
                "CURRENT_RETAIL",
                if cfg!(feature = "client-retail") {
                    "true"
                } else {
                    "false"
                },
            )
            .as_str(),
        )
        .unwrap();
    assert_eq!(
        result, "ok",
        "retail 12.0.0 Edit Mode unit-frame setting enum mismatch: {result}"
    );
}
