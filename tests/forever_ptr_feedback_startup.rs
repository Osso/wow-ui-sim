#![cfg(feature = "client-wowforever")]

use wow_ui_sim::loader::discover_blizzard_startup_addons_for_screen;
use wow_ui_sim::screen::ScreenKind;

#[test]
fn login_startup_excludes_ptr_feedback_glue_on_forever() {
    let ui = wow_ui_sim::client_profile::blizzard_ui_addons_dir();
    let startup = discover_blizzard_startup_addons_for_screen(&ui, ScreenKind::Login);

    assert!(
        startup
            .iter()
            .all(|addon| addon.name != "Blizzard_PTRFeedbackGlue"),
        "R26: normal Forever startup excludes addons marked OnlyBetaAndPTR"
    );
}
