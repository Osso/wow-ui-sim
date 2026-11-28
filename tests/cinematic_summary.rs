#![cfg(feature = "cinematic-summary")]
//! Native contract: pr46-49-decomp-answers.md, R88ap part A §1.
use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::lua_api::WowLuaEnv;

fn setup() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    let loaded = load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_FrameXML"], &[]);
    assert!(loaded.iter().any(|name| name == "Blizzard_FrameXML"));
    env.exec(
        r#"
        assert(MovieFrame and MovieFrame.CloseDialog.Summary, "original MovieFrame loaded")
        A_Admin.SetCinematicSummary(0, "ID zero must still be nil")
        A_Admin.SetCinematicSummary(101, "The first cinematic summary")
        A_Admin.SetCinematicSummary(102, "A different summary")
        A_Admin.SetCinematicSummary(103, "")
        A_Admin.SetCinematicSummary(104, nil)
        assert(not pcall(A_Admin.SetCinematicSummary, 105, 7), "non-string summary rejected")
    "#,
    )
    .unwrap();
    if let Ok(path) = std::env::var("WOW_SIM_CINEMATIC_SUMMARY_OVERLAY") {
        env.exec(&std::fs::read_to_string(path).unwrap()).unwrap();
    }
    env
}

#[test]
fn original_movie_dialog_follows_current_cinematic_summary() {
    let env = setup();
    env.exec(
        r#"
        local function check(id, expected)
            CinematicStarted(Enum.CinematicType.GameMovie, id)
            MovieFrame:ShowCloseDialog()
            local label = MovieFrame.CloseDialog.Summary
            assert(label:IsShown() == (expected ~= nil), "original movie summary visibility")
            if expected then assert(label:GetText() == expected, "original movie summary text") end
        end
        check(101, "The first cinematic summary")
        check(102, "A different summary")
        check(0, nil)
        check(999, nil)
        check(103, nil)
        check(104, nil)
        check(101, "The first cinematic summary")
        CinematicFinished(Enum.CinematicType.GameMovie)
        MovieFrame:ShowCloseDialog()
        assert(not MovieFrame.CloseDialog.Summary:IsShown(), "finished summary is hidden")
    "#,
    )
    .expect("original MovieFrame close dialog reflects current cinematic summary");
}

#[test]
fn cinematic_id_lifecycle_and_summary_arity() {
    let env = setup();
    env.exec(
        r#"
        assert(select('#', GetCurrentCinematicSummary()) == 1)
        assert(GetCurrentCinematicSummary() == nil)
        for kind = 0, 3 do
            CinematicStarted(kind, 101)
            assert(GetCurrentCinematicSummary() == "The first cinematic summary")
            assert(select('#', GetCurrentCinematicSummary()) == 1)
            CinematicFinished(kind)
            assert(GetCurrentCinematicSummary() == nil)
        end
        CinematicStarted(0, 102)
        CinematicStarted(4, 101)
        assert(GetCurrentCinematicSummary() == "A different summary")
    "#,
    )
    .unwrap();
}
