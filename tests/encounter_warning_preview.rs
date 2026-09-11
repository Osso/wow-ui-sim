//! Synthetic Edit Mode previews only; no encounter warning producer or store.
use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(feature = "client-ptr")]
#[test]
fn warning_preview_ptr_has_complete_independent_records() {
    let env = WowLuaEnv::new().unwrap();
    for _ in 0..2 {
        env.exec(r#"
            local fields = {
                text="string", casterGUID="string", casterName="string",
                targetGUID="string", targetName="string", iconFileID="number",
                tooltipSpellID="number", isDeadly="boolean", color="table",
                duration="number", severity="number", shouldPlaySound="boolean",
                shouldShowChatMessage="boolean", shouldShowWarning="boolean",
            }
            local cases = {
                {Enum.EncounterEventSeverity.Low, "Simulated Low Warning", 1, 1, 1},
                {Enum.EncounterEventSeverity.Medium, "Simulated Medium Warning", 1, 0.75, 0.1},
                {Enum.EncounterEventSeverity.High, "Simulated High Warning", 1, 0.15, 0.05},
            }
            for _, case in ipairs(cases) do
                local severity, text, r, g, b = unpack(case)
                local info = C_EncounterWarnings.GetEditModeWarningInfo(severity)
                assert(select('#', C_EncounterWarnings.GetEditModeWarningInfo(severity)) == 1)
                for field, kind in pairs(fields) do
                    assert(type(info[field]) == kind, field .. ": expected " .. kind)
                end
                local count = 0
                for field in pairs(info) do assert(fields[field]); count = count + 1 end
                assert(count == 14 and info.severity == severity and info.text == text)
                assert(info.duration == 5 and info.duration < math.huge)
                assert(info.iconFileID == 136122 and info.tooltipSpellID == 0)
                assert(info.isDeadly == (severity == Enum.EncounterEventSeverity.High))
                assert(info.shouldShowWarning and not info.shouldPlaySound and not info.shouldShowChatMessage)
                assert(info.casterGUID == "Sim-Warning-Caster" and info.targetGUID == "Sim-Warning-Target")
                assert(info.casterName == "Simulator Caster" and info.targetName == "Simulator Target")
                local cr, cg, cb, ca = info.color:GetRGBA()
                assert(cr == r and cg == g and cb == b and ca == 1)
                assert(type(info.color.WrapTextInColorCode) == "function")
                info.text = "modified"; info.duration = -1; info.color:SetRGBA(0, 0, 0, 0)
                local fresh = C_EncounterWarnings.GetEditModeWarningInfo(severity)
                assert(fresh ~= info and fresh.color ~= info.color)
                assert(fresh.text == text and fresh.duration == 5)
                cr, cg, cb, ca = fresh.color:GetRGBA()
                assert(cr == r and cg == g and cb == b and ca == 1)
            end
        "#).unwrap();
        wow_ui_sim::ptr::compat_bootstrap::apply_post_load(&env);
    }
}

#[cfg(feature = "client-ptr")]
#[test]
fn warning_preview_ptr_rejects_invalid_severity_without_state() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        for _, severity in ipairs({-1, 3, 0.5, math.huge, -math.huge, 0/0, "1", false, {}}) do
            local ok, err = pcall(C_EncounterWarnings.GetEditModeWarningInfo, severity)
            assert(not ok and tostring(err):find("severity"), tostring(err))
        end
        assert(not pcall(C_EncounterWarnings.GetEditModeWarningInfo))
        assert(not pcall(C_EncounterWarnings.GetEditModeWarningInfo, nil))
        assert(C_EncounterWarnings.GetEditModeWarningInfo(2).severity == 2)
        assert(EncounterWarningInfo == nil)
    "#).unwrap();
}

#[cfg(feature = "client-retail")]
#[test]
fn warning_preview_preserves_earlier_retail_record() {
    let env = WowLuaEnv::new().unwrap();
    for _ in 0..2 {
        env.exec(r#"
            local cases = {
                {0, "Encounter Warning", false, 1, 1},
                {1, "Encounter Warning", false, 1, 1},
                {2, "Important Encounter Warning", false, 0.75, 0.1},
                {3, "Critical Encounter Warning", true, 0.15, 0.05},
            }
            for _, case in ipairs(cases) do
                local severity, text, deadly, g, b = unpack(case)
                local info = C_EncounterWarnings.GetEditModeWarningInfo(severity)
                assert(info.severity == severity and info.text == text and info.isDeadly == deadly)
                assert(info.duration == 30 and info.iconFileID == 136122 and info.casterName == "")
                assert(info.casterGUID == nil and info.targetGUID == nil and info.targetName == nil)
                assert(info.tooltipSpellID == nil and info.shouldShowWarning)
                assert(not info.shouldPlaySound and not info.shouldShowChatMessage)
                local r, actual_g, actual_b, a = info.color:GetRGBA()
                assert(r == 1 and actual_g == g and actual_b == b and a == 1)
            end
        "#).unwrap();
        wow_ui_sim::ptr::compat_bootstrap::apply_post_load(&env);
    }
}

#[cfg(feature = "client-ptr")]
fn load_warning_view() -> WowLuaEnv {
    let ui = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let (env, loaded) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui, &["Blizzard_EncounterWarnings"], &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_EncounterWarnings"));
    env.state().borrow_mut().mouse_position = Some((-1000.0, -1000.0));
    env
}

#[cfg(feature = "client-ptr")]
fn expire_warning_timer(env: &WowLuaEnv, id: u64) {
    env.state().borrow_mut().rilua_timers.iter_mut().find(|timer| timer.id == id)
        .expect("queued warning timer").fire_at = std::time::Instant::now();
    env.process_timers().unwrap();
    env.fire_on_update(0.5).unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn warning_preview_real_blizzard_frame_expires_cancels_and_reuses() {
    let env = load_warning_view();
    env.exec(r#"
        system = CriticalEncounterWarnings
        system:SetIsEditing(true)
        view = system:GetView()
        first = view:GetCurrentWarning()
        assert(first and first.severity == Enum.EncounterEventSeverity.High, "high severity preview")
        assert(first.isDeadly and first.duration == 5, "high preview flags and seconds")
        assert(view:IsShown() and view.expirationTimer ~= nil, "visible preview with timer")
        assert(view.Text:GetText() == first.text, "preview text")
        assert(view.LeftIcon.Icon:GetTexture() == first.iconFileID, "preview icon")
        assert(view.LeftIcon.DeadlyOverlay:IsShown(), "deadly overlay")
        local r, g, b = view.Text:GetTextColor()
        assert(r == 1 and g == 0.15 and b == 0.05, "text color: " .. r .. ", " .. g .. ", " .. b)
    "#).unwrap();
    let first_timer = env.state().borrow().rilua_timers.back().unwrap().id;
    env.fire_on_update(0.5).unwrap();
    env.exec(r#"
        second = C_EncounterWarnings.GetEditModeWarningInfo(Enum.EncounterEventSeverity.Low)
        view:ShowWarning(second)
        assert(view:GetCurrentWarning() == second and view.Text:GetText() == second.text)
        assert(view.LeftIcon.NormalOverlay:IsShown() and not view.LeftIcon.DeadlyOverlay:IsShown())
    "#).unwrap();
    let second_timer = env.state().borrow().rilua_timers.back().unwrap().id;
    assert_ne!(first_timer, second_timer);
    expire_warning_timer(&env, first_timer);
    env.exec("assert(view:IsShown() and view:GetCurrentWarning() == second and view.expirationTimer ~= nil)").unwrap();
    expire_warning_timer(&env, second_timer);
    env.exec(r#"
        assert(not view:IsShown() and not view:HasCurrentWarning() and view.expirationTimer == nil)
        assert(view.Text:GetText() == "" and view.LeftIcon.Icon:GetTexture() == nil)
        third = C_EncounterWarnings.GetEditModeWarningInfo(Enum.EncounterEventSeverity.Medium)
        view:ShowWarning(third)
        assert(view:IsShown() and view:GetCurrentWarning() == third and view.expirationTimer ~= nil)
        view:ClearWarning()
        assert(not view:IsShown() and not view:HasCurrentWarning() and view.expirationTimer == nil)
    "#).unwrap();
    let third_timer = env.state().borrow().rilua_timers.back().unwrap().id;
    expire_warning_timer(&env, third_timer);
    env.exec(r#"
        assert(not view:HasCurrentWarning() and not view:IsShown())
        view:ShowWarning(C_EncounterWarnings.GetEditModeWarningInfo(Enum.EncounterEventSeverity.High))
        assert(view:IsShown() and view:HasCurrentWarning() and view.expirationTimer ~= nil)
        view:ClearWarning()
    "#).unwrap();
    let errors: Vec<_> = env.state().borrow().lua_errors.iter()
        .filter(|message| message.contains("EncounterWarnings"))
        .cloned().collect();
    assert!(errors.is_empty(), "warning consumer errors: {errors:#?}");
}
