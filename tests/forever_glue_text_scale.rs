#![cfg(feature = "client-wowforever")]

use crate::common::blizzard_addon_harness::with_blizzard_addon_glue_smoke_shape_in_dir;

#[test]
fn glue_text_size_manager_scales_registered_frames_from_cvar_storage() {
    with_blizzard_addon_glue_smoke_shape_in_dir(
        &wow_ui_sim::client_profile::blizzard_ui_addons_dir(),
        &["Blizzard_AccessibilityTemplates"],
        &[],
        |env, loaded| {
            assert!(
                loaded
                    .iter()
                    .any(|name| name == "Blizzard_AccessibilityTemplates")
            );
            let errors_before = env.state().borrow().lua_errors.len();
            env.exec(
                r#"
                assert(TextSizeManager:GetReadCVarName() == "userFontScaleGlue")
                assert(C_CVar.GetCVarDefault("userFontScaleGlue") == "1.0")
                assert(TextSizeManager:GetScale() == 1)

                local frame = CreateFrame("Frame", nil, nil, "UserScaledFrameTemplate")
                assert(TextSizeManager.registeredObjects[frame] == frame)
                frame.baseWidth = 120
                frame.baseHeight = 24
                TextSizeManager:UpdateObject(frame)
                assert(frame:GetWidth() == 120 and frame:GetHeight() == 24)

                -- Temporary inputs exercise the existing CVar store without persisting.
                C_CVar.SetTempCVar("userFontScaleGlue", "1.5")
                assert(TextSizeManager:GetScale() == 1.5)
                TextSizeManager:UpdateObject(frame)
                assert(frame:GetWidth() == 180 and frame:GetHeight() == 36)
                C_CVar.RemoveTempCVar("userFontScaleGlue")
                "#,
            )
            .expect("original glue TextSizeManager should scale registered frames");
            assert_eq!(
                env.state().borrow().lua_errors.len(),
                errors_before,
                "text-scaled frame creation and updates should not record Lua errors"
            );
        },
    );
}
