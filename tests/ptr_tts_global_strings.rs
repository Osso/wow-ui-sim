#![cfg(feature = "client-ptr")]

use crate::common::blizzard_addon_harness::{blizzard_ui_dir, new_blizzard_addon_env};
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::screen::ScreenKind;

#[test]
fn ptr_original_combat_audio_registers_pulse_health_command() {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    wow_ui_sim::xml::register_intrinsic_templates();
    for (_, toc) in discover_blizzard_addons_for_screen(&ui, ScreenKind::Game) {
        load_addon(&env.loader_env(), &toc).expect("load Blizzard addon");
    }
    env.exec(r#"
        assert(SLASH_CAA_PULSE_YOUR_HEALTH == 'pulseyourhealth', 'PTR GlobalStrings row 60236')
        local command = CAACommands:GetCommand('pulseyourhealth')
        assert(command and command.option == nil, 'original TextToSpeechCommands.lua:842 registered pulse command')
    "#).expect("original combat audio command registration");
}
