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
fn approved_native_defaults_quick_keybind_consumer() {
    let ui = blizzard_ui_dir();
    let toc =
        TocFile::from_file(&find_toc_file(&ui.join("Blizzard_QuickKeybind")).unwrap()).unwrap();
    assert!(toc.allows_screen(ScreenKind::Game));
    assert!(
        toc.files
            .iter()
            .any(|path| path.to_string_lossy().contains("QuickKeybind.xml"))
    );
    let env = new_blizzard_addon_env(&ui);
    wow_ui_sim::xml::register_intrinsic_templates();
    let loaded = load_blizzard_addon_closure_into_env(
        &env,
        &ui,
        &["Blizzard_FrameXML", "Blizzard_QuickKeybind"],
        &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_QuickKeybind"));
    env.exec(r#"
        local source = debug.getinfo(QuickKeybindFrameMixin.OnLoad, 'S').source:gsub('\\', '/')
        assert(source:find('QuickKeybind.lua', 1, true))
        local function button() return {SetText=function() end, SetScript=function() end} end
        local checkbox = {hidden=false, Hide=function(self) self.hidden=true end}
        local frame = setmetatable({CancelButton=button(), OkayButton=button(), DefaultsButton=button(), UseCharacterBindingsButton=checkbox}, {__index=QuickKeybindFrameMixin})
        QuickKeybindFrameMixin.OnLoad(frame)
        assert(checkbox.hidden)
        assert(C_GameRules.IsAccountKeyBindingsEnabled() == false)
    "#).expect("original keybind OnLoad consumes the approved default");
}

#[test]
fn approved_native_defaults_minimap_event_consumer() {
    let ui = blizzard_ui_dir();
    let toc = TocFile::from_file(&find_toc_file(&ui.join("Blizzard_Minimap")).unwrap()).unwrap();
    assert!(toc.allows_screen(ScreenKind::Game));
    assert!(toc.files.iter().any(|p| {
        p.to_string_lossy()
            .contains("ExternalEventURLMinimapButton.lua")
    }));
    let env = new_blizzard_addon_env(&ui);
    wow_ui_sim::xml::register_intrinsic_templates();
    let loaded = load_blizzard_addon_closure_into_env(
        &env,
        &ui,
        &["Blizzard_FrameXML", "Blizzard_Minimap"],
        &[],
    );
    assert!(loaded.iter().any(|n| n == "Blizzard_Minimap"));
    env.exec(
        r#"
        local f = ExternalEventURLMinimapButtonMixin.Refresh
        local source = debug.getinfo(f, 'S').source:gsub('\\', '/')
        assert(source:find('Shared/ExternalEventURLMinimapButton.lua', 1, true))
        local button = {shown=true, effect=true,
            SetShown=function(self,v) self.shown=v end,
            IsShown=function(self) return self.shown end,
            SetEffectShown=function(self,v) self.effect=v end}
        f(button)
        assert(not button.shown and not button.effect)
    "#,
    )
    .expect("original minimap refresh hides absent event");
}

#[test]
fn approved_native_defaults_guild_locale_consumer() {
    let ui = blizzard_ui_dir();
    let toc =
        TocFile::from_file(&find_toc_file(&ui.join("Blizzard_Communities")).unwrap()).unwrap();
    assert!(toc.allows_screen(ScreenKind::Game));
    assert!(toc.files.iter().any(|path| {
        path.to_string_lossy()
            .contains("GuildPreferredPlaySettings.lua")
    }));
    let env = new_blizzard_addon_env(&ui);
    wow_ui_sim::xml::register_intrinsic_templates();
    let loaded = load_blizzard_addon_closure_into_env(
        &env,
        &ui,
        &["Blizzard_FrameXML", "Blizzard_Communities"],
        &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_Communities"));
    env.exec(
        r#"
        local f = CommunitiesGuildPreferredPlaySettingsFrameMixin.InitializeLocaleDropdown
        local source = debug.getinfo(f, 'S').source:gsub('\\', '/')
        assert(source:find('GuildPreferredPlaySettings.lua', 1, true))
        local root = {count=0, CreateButton=function(self) self.count=self.count+1 end}
        local dropdown = {SetupMenu=function(self, generator) generator(self, root) end}
        f({LocaleDropdown=dropdown})
        assert(root.count == 0)
        local ids, names = C_AccountServices.GetPreferredLocales()
        assert(type(ids) == 'table' and #ids == 0 and type(names) == 'table' and #names == 0)
    "#,
    )
    .expect("original guild locale menu consumes two empty catalogs");
}

#[test]
fn approved_native_defaults_voice_transcription_consumer() {
    let ui = blizzard_ui_dir();
    let toc = TocFile::from_file(&find_toc_file(&ui.join("Blizzard_ChatFrame")).unwrap()).unwrap();
    assert!(toc.allows_screen(ScreenKind::Game));
    assert!(toc.files.iter().any(|path| {
        path.to_string_lossy()
            .contains("VoiceChatTranscriptionFrame.lua")
    }));
    let env = new_blizzard_addon_env(&ui);
    wow_ui_sim::xml::register_intrinsic_templates();
    let loaded = load_blizzard_addon_closure_into_env(
        &env,
        &ui,
        &["Blizzard_FrameXML", "Blizzard_ChatFrame"],
        &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_ChatFrame"));
    env.exec(r#"
        local source = debug.getinfo(VoiceTranscriptionFrame_UpdateVisibility, 'S').source:gsub('\\', '/')
        assert(source:find('Shared/VoiceChatTranscriptionFrame.lua', 1, true))
        local frame = CreateFrame('Frame')
        frame:SetID(1)
        VoiceTranscriptionFrame_UpdateVisibility(frame)
        assert(C_VoiceChat.GetActiveVoiceProviderID() == Enum.VoiceProviderID.Legacy)
    "#).expect("original transcription consumer accepts the approved provider");
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
