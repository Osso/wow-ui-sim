use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("create Lua env")
}

#[test]
#[cfg(all(feature = "optional-tests", feature = "client-retail"))]
fn plain_managed_frame_is_added_by_original_managed_frame_system() {
    test_timeout! {
        crate::common::blizzard_addon_harness::with_blizzard_addon_closure(
            &["Blizzard_EditMode"],
            &[],
            |env, loaded| {
                assert!(loaded.iter().any(|addon| addon == "Blizzard_EditMode"));
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
#[cfg(all(feature = "optional-tests", feature = "client-retail"))]
fn edit_mode_methods_are_owned_by_edit_mode_system_mixin() {
    test_timeout! {
        crate::common::blizzard_addon_harness::with_blizzard_addon_closure(
            &["Blizzard_EditMode", "Blizzard_ActionBar"],
            &[],
            |env, loaded| {
                assert!(loaded.iter().any(|addon| addon == "Blizzard_EditMode"));
                assert!(loaded.iter().any(|addon| addon == "Blizzard_ActionBar"));

                let actual: (bool, bool, bool, bool, bool, bool, bool, bool, bool) = env
                    .eval(
                        r#"
                        local owner = assert(EditModeSystemMixin)
                        local actionBarOwner = assert(MainActionBarMixin)
                        local plain = CreateFrame("Frame")
                        return type(owner.IsEditModeDragging) == "function",
                            type(owner.IsInitialized) == "function",
                            type(owner.IsInDefaultPosition) == "function",
                            type(owner.IsSystemSettingDefault) == "function",
                            type(actionBarOwner.IsInDefaultPosition) == "function",
                            plain.IsEditModeDragging == nil,
                            plain.IsInitialized == nil,
                            plain.IsInDefaultPosition == nil,
                            plain.IsSystemSettingDefault == nil
                        "#,
                    )
                    .expect("original EditModeSystemMixin methods should stay mixin-owned");

                assert_eq!(actual, (true, true, true, true, true, true, true, true, true));
            },
        );
    }
}

#[test]
fn unit_level_non_attackable_color_is_available() {
    let env = env();
    let result: (f64, f64, f64) = env
        .eval("return UNIT_LEVEL_NON_ATTACKABLE.r, UNIT_LEVEL_NON_ATTACKABLE.g, UNIT_LEVEL_NON_ATTACKABLE.b")
        .expect("read unit-level color");

    assert_eq!(result, (1.0, 1.0, 0.0));
}
