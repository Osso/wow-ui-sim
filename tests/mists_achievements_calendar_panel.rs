#![cfg(feature = "client-mists")]

use std::path::PathBuf;
use std::process::Command;

#[path = "common/bounded_process.rs"]
mod bounded_process;

fn wow_sim_binary() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_wow-sim")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("target")
                .join("debug")
                .join("wow-sim")
        })
}

#[test]
fn mists_achievements_tabs_and_calendar_navigation_round_trip() {
    let output = bounded_process::output(
        90,
        Command::new(wow_sim_binary())
        .args([
            "--no-addons",
            "--no-saved-vars",
            "--exec-lua",
            r#"
            ToggleAchievementFrame()
            if AchievementFrame == nil or AchievementFrame:IsShown() ~= true then
                error("AchievementFrame did not open")
            end
            if AchievementFrameAchievements == nil then
                error("AchievementFrameAchievements missing")
            end
            if GetCategoryList()[1] == nil then
                error("achievement category list is empty")
            end
            local _, completed, incomplete = GetCategoryNumAchievements(GetCategoryList()[1])
            if type(completed) ~= "number" or type(incomplete) ~= "number" then
                error("achievement category counts are not numeric")
            end
            AchievementFrameTab_OnClick(2)
            if AchievementFrameStats == nil or AchievementFrame.selectedTab ~= 2 then
                error("achievement stats tab did not select")
            end
            AchievementFrameTab_OnClick(1)
            if AchievementFrame.selectedTab ~= 1 then
                error("achievement summary tab did not restore")
            end

            ToggleCalendar()
            if CalendarFrame == nil or CalendarFrame:IsShown() ~= true then
                error("CalendarFrame did not open")
            end
            local monthInfo = C_Calendar.GetMonthInfo()
            if type(monthInfo) ~= "table" or type(monthInfo.month) ~= "number" then
                error("calendar month info missing")
            end
            if CalendarTodayFrame == nil or CalendarViewEventFrame == nil then
                error("calendar child frames missing")
            end
            CalendarFrame_Update()
            local viewedMonth = CalendarFrame.viewedMonth
            CalendarNextMonthButton_OnClick()
            CalendarPrevMonthButton_OnClick()
            if type(CalendarFrame.viewedMonth) ~= "number" or viewedMonth ~= CalendarFrame.viewedMonth then
                error("calendar month navigation did not round-trip")
            end
            "#,
            "lua-errors",
        ])
        )
        .expect("failed to run wow-sim");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "wow-sim failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_no_lua_errors(&stdout, &stderr);
}

fn assert_no_lua_errors(stdout: &str, stderr: &str) {
    assert!(
        stdout.trim().ends_with("[]")
            && !stdout.contains("Lua error")
            && !stderr.contains("Lua error")
            && !stderr.contains("[exec-lua] error"),
        "Achievements/Calendar panel flow emitted Lua errors\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
}
