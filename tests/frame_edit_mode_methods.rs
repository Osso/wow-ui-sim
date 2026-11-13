use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("create Lua env")
}

#[test]
#[cfg(all(feature = "optional-tests", feature = "client-retail"))]
fn plain_managed_frame_is_added_by_original_managed_frame_system() {
    test_timeout! {
        crate::common::blizzard_addon_harness::with_blizzard_addon_closure(
            &["Blizzard_ManagedFrameSystem"],
            &[],
            |env, loaded| {
                assert!(loaded.iter().any(|addon| addon == "Blizzard_ManagedFrameSystem"));

                let actual: (bool, bool, bool, bool, bool, bool, bool) = env
                    .eval(
                        r#"
                        local plain = CreateFrame("Frame")
                        local managed = CreateFrame(
                            "Frame", nil, UIParent, "BottomManagedFrameTemplate"
                        )
                        managed:Hide()
                        managed:SetSize(200, 20)
                        managed:Show()

                        local skipped = managed.IsInDefaultPosition ~= nil
                            and not managed:IsInDefaultPosition()
                        return skipped,
                            BottomManagedFrameContainer.showingFrames[managed] == managed,
                            managed:GetParent() == BottomManagedFrameContainer,
                            plain.IsEditModeDragging == nil,
                            plain.IsInitialized == nil,
                            plain.IsInDefaultPosition == nil,
                            plain.IsSystemSettingDefault == nil
                        "#,
                    )
                    .expect("original managed-frame consumer should lay out a plain frame");

                assert_eq!(actual, (false, true, true, true, true, true, true));
            },
        );
    }
}

#[test]
fn edit_mode_position_methods_are_native_frame_methods() {
    let env = env();
    let result: (String, bool, bool, bool, bool) = env
        .eval(
            r#"
            local frame = CreateFrame("Frame")
            local methodType = type(frame.IsInDefaultPosition)
            local initiallyDefault = frame:IsInDefaultPosition()
            frame.systemInfo = { isInDefaultPosition = true }
            local defaultAfterSystemInfo = frame:IsInDefaultPosition()
            local initialized = frame:IsInitialized()
            local dragging = frame:IsEditModeDragging()
            return methodType, initiallyDefault, defaultAfterSystemInfo, initialized, dragging
            "#,
        )
        .expect("probe edit-mode frame methods");

    assert_eq!(result, ("function".to_string(), false, true, true, false));
}

#[test]
fn unit_level_non_attackable_color_is_available() {
    let env = env();
    let result: (f64, f64, f64) = env
        .eval("return UNIT_LEVEL_NON_ATTACKABLE.r, UNIT_LEVEL_NON_ATTACKABLE.g, UNIT_LEVEL_NON_ATTACKABLE.b")
        .expect("read unit-level color");

    assert_eq!(result, (1.0, 1.0, 0.0));
}

#[test]
fn edit_mode_setting_default_reads_system_settings() {
    let env = env();
    let result: (bool, bool) = env
        .eval(
            r#"
            Enum = Enum or {}
            Enum.EditModeActionBarSetting = { Orientation = 1, NumRows = 2 }
            local frame = CreateFrame("Frame")
            frame.systemInfo = {
                settings = {
                    { setting = Enum.EditModeActionBarSetting.Orientation, value = 0 },
                    { setting = Enum.EditModeActionBarSetting.NumRows, value = 3 },
                }
            }
            return frame:IsSystemSettingDefault(Enum.EditModeActionBarSetting.Orientation),
                   frame:IsSystemSettingDefault(Enum.EditModeActionBarSetting.NumRows)
            "#,
        )
        .expect("probe edit-mode setting defaults");

    assert_eq!(result, (true, false));
}
