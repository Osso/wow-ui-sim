#![cfg(feature = "client-wowforever")]

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, new_blizzard_addon_env, new_blizzard_addon_glue_env,
};
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::{fire_startup_events_for_screen, settle_headless_startup};
use wow_ui_sim::toc::TocFile;

const SELECTED_NAME_PROBE: &str = r#"
    assert(GetNumCharacters() >= 2, 'two resolved host characters are required')
    local first = GetBasicCharacterInfo(GetCharacterGUID(1))
    local second = GetBasicCharacterInfo(GetCharacterGUID(2))
    assert(first and second and first.fullName ~= second.fullName,
        'the two selected characters must have distinct names')
    SelectCharacter(2)
    assert(GetCharacterSelection() == 2, 'second character selection must reach the host model')
    assert(CharSelectCharacterName:GetText() == second.fullName,
        'original UPDATE_SELECTED_CHARACTER consumer must publish the second name')
    SelectCharacter(1)
    assert(GetCharacterSelection() == 1, 'first character selection must reach the host model')
    assert(CharSelectCharacterName:GetText() == first.fullName,
        'original UPDATE_SELECTED_CHARACTER consumer must publish the first name')
    assert(rawget(_G, '__wow_character_select_selected_name_patched') == nil,
        'the retired selected-name guard must not install on Forever')
    return first.fullName, second.fullName
"#;

const TOKEN_TRACKER_PROBE: &str = r#"
    local tracker = ContainerFrameSettingsManager.TokenTracker
    assert(tracker and tracker == BackpackTokenFrame, 'original startup must publish the XML tracker')
    assert(tracker.Border and tracker.tokenPool, 'original XML children and OnLoad pool must exist')
    assert(tracker:GetScript('OnLoad'), 'original XML OnLoad binding must remain installed')
    assert(tracker.GetMaxTokensWatched == BackpackTokenFrameMixin.GetMaxTokensWatched,
        'original tracker must retain its capacity consumer')
    local template = C_XMLUtil.GetTemplateInfo('BackpackTokenTemplate')
    assert(template and template.width == 50, 'pinned original token template width')
    for _, slots in ipairs({2, 5, 2}) do
        tracker:SetWidth(template.width * slots)
        assert(tracker:GetMaxTokensWatched() == slots, 'capacity must follow the original width input')
    end
    assert(tracker.tokenWidth == template.width, 'original cached template width must remain stable')
    assert(ContainerFrameSettingsManager.TokenTracker == tracker, 'tracker identity must remain stable')
    return tracker:GetMaxTokensWatched()
"#;

const MOUNT_POPUP_PROBE: &str = r#"
    assert(rawget(_G, '__wow_mount_journal_dynamic_flight_popup_patched') == nil,
        'the retired mount-journal guard must not install on Forever')
    local button = MountJournal.ToggleDynamicFlightFlyoutButton
    local popup = button:GetPopup()
    assert(popup == MountJournal.DynamicFlightFlyoutPopup, 'selected Mount Journal must own its popup')
    assert(button.UnspentGlyphsAnim, 'selected flyout button must own its animation')
    assert(popup.OpenDynamicFlightSkillTreeButton.UnspentGlyphsAnim,
        'selected popup button must own its animation')
    assert(not button:IsPopupOpen(), 'selected popup must start closed')
    button:Click()
    assert(button:IsPopupOpen() and popup:IsShown(), 'real button click must open the original popup')
    button:Click()
    assert(not button:IsPopupOpen() and not popup:IsShown(),
        'second real button click must close the original popup')
    return true
"#;

const MAP_POSITION_PROBE: &str = r#"
    local provider
    for candidate in pairs(FlightMapFrame.dataProviders) do
        if candidate.MapLabel then
            provider = candidate
            break
        end
    end
    assert(provider, 'selected FlightMap must register ClickToZoomDataProviderMixin')
    local function observe(mapID)
        FlightMapFrame:SetMapID(mapID)
        return C_Map.GetMapArtHelpTextPosition(mapID), select('#', provider.MapLabel:GetPoint())
    end
    local presentValue, presentPoints = observe(1411)
    local missingValue, missingPoints = observe(14)
    return presentValue, missingValue, presentPoints, missingPoints
"#;

fn load_character_select() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_glue_env(&ui);
    let toc = find_toc_file(&ui.join("Blizzard_GlueXML")).unwrap();
    let parsed = TocFile::from_file(&toc).unwrap();
    assert!(parsed.files.iter().any(|path| path.ends_with("Mainline/CharacterSelect.lua")));
    assert!(parsed.files.iter().any(|path| path.ends_with("Mainline/CharacterSelect.xml")));
    let addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::CharacterSelect);
    assert!(addons.iter().any(|(name, path)| name == "Blizzard_GlueXML" && path == &toc));
    for (name, path) in addons {
        load_addon(&env.loader_env(), &path).unwrap_or_else(|error| panic!("load {name}: {error}"));
    }
    env.apply_post_load_workarounds();
    settle_headless_startup(&env);
    env
}

fn load_game_ui() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    wow_ui_sim::xml::register_intrinsic_templates();
    let addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    for (name, path) in addons {
        load_addon(&env.loader_env(), &path).unwrap_or_else(|error| panic!("load {name}: {error}"));
    }
    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

#[test]
fn forever_selected_name_consumer_survives_guard_retirement() {
    let env = load_character_select();
    let names: (String, String) = env.exec(SELECTED_NAME_PROBE).unwrap();
    assert_ne!(names.0, names.1);
}

#[test]
fn forever_token_tracker_startup_owns_original_xml_frame() {
    let env = load_game_ui();
    let slots: i32 = env.eval(TOKEN_TRACKER_PROBE).unwrap();
    assert_eq!(slots, 2);
}

#[test]
fn forever_mount_journal_popup_click_uses_original_animation_children() {
    let env = load_game_ui();
    env.exec("assert(C_AddOns.LoadAddOn('Blizzard_Collections'))").unwrap();
    let errors_before = env.state().borrow().lua_errors.len();
    let clicked: bool = env.eval(MOUNT_POPUP_PROBE).unwrap();
    assert!(clicked);
    assert_eq!(env.state().borrow().lua_errors.len(), errors_before);
}

#[test]
fn forever_map_help_position_consumer_reads_present_and_missing_rows() {
    let env = load_game_ui();
    env.exec("assert(C_AddOns.LoadAddOn('Blizzard_FlightMap'))").unwrap();
    let values: (i32, i32, i32, i32) = env.eval(MAP_POSITION_PROBE).unwrap();
    assert_eq!(values, (0, 0, 0, 0));
}
