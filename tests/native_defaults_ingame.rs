//! User-approved defaults exercised through selected Blizzard consumers.
#![cfg(feature = "client-ptr")]

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::loader::find_toc_file;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn load_bank() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let toc =
        TocFile::from_file(&find_toc_file(&ui.join("Blizzard_UIPanels_Game")).unwrap()).unwrap();
    assert!(toc.allows_screen(ScreenKind::Game));
    assert!(
        toc.files
            .iter()
            .any(|path| path.to_string_lossy().contains("BankFrameTemplates.lua"))
    );
    let env = new_blizzard_addon_env(&ui);
    wow_ui_sim::xml::register_intrinsic_templates();
    let loaded = load_blizzard_addon_closure_into_env(
        &env,
        &ui,
        &["Blizzard_FrameXML", "Blizzard_UIPanels_Game"],
        &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_UIPanels_Game"));
    env
}

#[test]
fn approved_native_defaults_bank_selected_tab_consumer() {
    let env = load_bank();
    env.exec(r#"
        local source = debug.getinfo(BankPanelMixin.OnEvent, 'S').source:gsub('\\', '/')
        assert(source:find('Mainline/BankFrameTemplates.lua', 1, true))
        local panel = {selectedTabID=7, dirty=0, MarkDirty=function(self) self.dirty=self.dirty+1 end}
        BankPanelMixin.OnEvent(panel, 'BAG_UPDATE', 9)
        assert(panel.dirty == 0)
        BankPanelMixin.OnEvent(panel, 'BAG_UPDATE', 7)
        assert(panel.dirty == 1)
        assert(C_Bank.ShouldUsePlayerBagsInBank() == false)
    "#).expect("original bank event consumes the approved default");
}
