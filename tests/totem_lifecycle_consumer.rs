#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]
//! Original consumer: Blizzard_UnitFrame/Mainline/TotemFrame.lua (Update, OnUpdate, OnClick).

use crate::common::blizzard_addon_harness as harness;

const TOTEM_FRAME_LIFECYCLE: &str = r#"
    assert(TotemFrame and not TotemFrame:IsShown(), "original TotemFrame starts empty")
    A_Admin.SetManualClock(100)
    A_Admin.SetTotem(1, { name = "Searing Totem", startTime = 100, duration = 60, icon = 135825,
        modRate = 1, spellID = 3599, cannotDismiss = false })
    A_Admin.SetTotem(2, { name = "Earthbind Totem", startTime = 100, duration = 20, icon = 136102,
        modRate = 1, spellID = 2484, cannotDismiss = true })
    assert(TotemFrame:IsShown() and TotemFrame.activeTotems == 2, "two supplied totems shown")
    assert(GetTotemTimeLeft(1) == 60 and GetTotemTimeLeft(2) == 20 and GetTotemTimeLeft(3) == 0)
    assert(GetTotemCannotDismiss(1) == false and GetTotemCannotDismiss(2) == true)
    assert(GetTotemCannotDismiss(3) == nil, "empty slot has no dismissal flag")
    A_Admin.AdvanceTime(5)
    local buttons = {}
    for button in TotemFrame.totemPool:EnumerateActive() do buttons[button.slot] = button end
    assert(buttons[1].Duration:GetText() == format(SecondsToTimeAbbrev(55)), buttons[1].Duration:GetText())
    assert(buttons[2].Duration:GetText() == format(SecondsToTimeAbbrev(15)), buttons[2].Duration:GetText())
    assert(GetTotemDuration(2):GetRemainingDuration() == 15)
    buttons[2]:Click("RightButton")
    assert(GetTotemInfo(2) == true and TotemFrame.activeTotems == 2, "cannotDismiss blocks DestroyTotem")
    buttons[1]:Click("RightButton")
    assert(GetTotemInfo(1) == false and GetTotemTimeLeft(1) == 0, "right-click destroyed slot 1")
    assert(TotemFrame:IsShown() and TotemFrame.activeTotems == 1, "PLAYER_TOTEM_UPDATE refreshed")
    A_Admin.AdvanceTime(16)
    assert(GetTotemTimeLeft(2) == 0 and GetTotemCannotDismiss(2) == nil, "expired totem reads empty")
    A_Admin.SetTotem(2, nil)
    assert(not TotemFrame:IsShown(), "cleared slots hide the frame")
    assert(not pcall(A_Admin.SetTotem, 9, nil), "slot outside GetNumTotemSlots rejected")
    assert(not pcall(A_Admin.SetTotem, 1, { name = "x", startTime = 0, duration = 0, icon = 1,
        modRate = 1, spellID = 1, cannotDismiss = false }), "non-positive duration rejected")
    return true
"#;

#[test]
fn totem_frame_follows_time_left_dismissal_and_destroy() {
    harness::with_blizzard_addon_startup_shape(&["Blizzard_UnitFrame"], &[], |env, loaded| {
        assert!(loaded.iter().any(|name| name == "Blizzard_UnitFrame"));
        let result: bool = env
            .eval(TOTEM_FRAME_LIFECYCLE)
            .expect("original TotemFrame consumer runs over supplied totems");
        assert!(result);
    });
}
