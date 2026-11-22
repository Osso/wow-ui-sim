//! Original cached Wrath consumers; scoped callbacks, not a clean startup claim.
#![cfg(feature = "client-wrath")]

use sha2::{Digest, Sha256};
use wow_ui_sim::loader::{LoadTiming, create_frame_from_xml};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::PartyMember;
use wow_ui_sim::xml::{FrameChildElement, ScriptsXml, XmlElement, parse_xml};

const ORIGINALS: &[(&str, &str)] = &[
    (
        "UIParent.lua",
        "a48687367d05ba9965bdd7a43bdecb92666c8ba74873bd7d9da08b205b34e177",
    ),
    (
        "UnitPopup.lua",
        "ed3f60d1152d7054fb846afce1febf3249494c89ebff42fe88e5c09b6be40209",
    ),
    (
        "MainMenuBarMicroButtons.lua",
        "40737acac438f610b2b2436ac55866e2c37cc1e76272732504fbebb378be43ca",
    ),
    (
        "MainMenuBar.xml",
        "e2796237e8b8716484579a9da5fe6a9ff0d5bf3e377b49c2527b8eef8484b99a",
    ),
    (
        "VehicleMenuBar.xml",
        "6699e853665cae62318de917ca4bf195ec480be0d88648291dc79460d3ca97da",
    ),
    (
        "VehicleMenuBar.lua",
        "2e04588f023d50a528c7b3fef4897adc37e8678b84d33b7478a6fd320e3e99a0",
    ),
    (
        "ActionButtonTemplate.xml",
        "f1d376a849fca3867c2e1b3cb6dfeb4cd21db7838914482a16e48d5ce95c3ffa",
    ),
];

fn read_original(name: &str) -> String {
    let pin = ORIGINALS.iter().find(|(file, _)| *file == name).unwrap().1;
    let toc = wow_ui_sim::client_profile::blizzard_ui_framexml_toc()
        .expect("SETUP FAILURE: pinned Wrath runtime cache required");
    let path = toc.parent().unwrap().join(name);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("SETUP FAILURE: {}: {error}", path.display()));
    let digest = format!("{:x}", Sha256::digest(&bytes));
    assert_eq!(
        digest,
        pin,
        "SETUP FAILURE: changed original {}",
        path.display()
    );
    eprintln!("Wrath consumer {} sha256={digest}", path.display());
    String::from_utf8(bytes).expect("SETUP FAILURE: original UTF-8")
}

fn assert_no_errors(env: &WowLuaEnv) {
    env.exec("assert(#C3Errors == 0, table.concat(C3Errors, '\\n'))")
        .expect("script errors must not be swallowed");
    assert!(
        env.state().borrow().lua_errors.is_empty(),
        "{:?}",
        env.state().borrow().lua_errors
    );
}

fn exec_and_audit(env: &WowLuaEnv, source: &str) {
    let result = env.exec(source);
    assert_no_errors(env);
    result.expect("original consumer and observable assertions");
}

fn consumer_env(files: &[&str]) -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("SETUP FAILURE: Wrath environment");
    env.exec("C3Errors = {}; seterrorhandler(function(message) C3Errors[#C3Errors + 1] = tostring(message) end)")
        .unwrap();
    for file in files {
        env.loader_env()
            .exec(&read_original(file))
            .unwrap_or_else(|error| panic!("SETUP FAILURE: complete unchanged {file}: {error}"));
        assert_no_errors(&env);
    }
    env
}

fn seed_party(env: &WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    state.party_group_active = true;
    state.party_members = vec![PartyMember {
        map_position: None,
        faction_group: None,
        #[cfg(feature = "client-retail")]
        pvp: None,
        name: "RafFriend".into(),
        name_cached: true,
        connected: true,
        class_index: 2,
        level: 60,
        health: 100,
        health_max: 100,
        power: 100,
        power_max: 100,
        power_type: 0,
        power_type_name: "MANA".into(),
        is_leader: false,
        dead_since: None,
        buffs: vec![],
        debuffs: vec![],
    }];
}

fn assert_raf_visibility(env: &WowLuaEnv, expected: i32) {
    exec_and_audit(
        env,
        &format!(
            r#"
        UIDROPDOWNMENU_MENU_LEVEL = 1
        UIDROPDOWNMENU_INIT_MENU = CreateFrame('Frame')
        UIDROPDOWNMENU_INIT_MENU.which = 'PARTY'
        UIDROPDOWNMENU_INIT_MENU.unit = 'party1'
        UIDROPDOWNMENU_INIT_MENU.name = 'RafFriend'
        UnitPopup_HideButtons()
        local found = 0
        for index, name in ipairs(UnitPopupMenus.PARTY) do
            if name == 'RAF_SUMMON' or name == 'RAF_GRANT_LEVEL' then
                assert(UnitPopupShown[1][index] == {expected}, name)
                found = found + 1
            end
        end
        assert(found == 2)
    "#
        ),
    );
}

#[test]
fn original_wrath_party_menu_hides_unlinked_raf_actions() {
    let env = consumer_env(&["UIParent.lua", "UnitPopup.lua"]);
    seed_party(&env);
    assert_raf_visibility(&env, 0);
}

#[test]
fn original_wrath_party_menu_tracks_explicit_raf_link_changes() {
    let env = consumer_env(&["UIParent.lua", "UnitPopup.lua"]);
    seed_party(&env);
    let guid: String = env.eval("return UnitGUID('party1')").unwrap();
    assert!(!guid.is_empty());
    env.state()
        .borrow_mut()
        .refer_a_friend_linked_guids
        .insert(guid.clone());
    assert_raf_visibility(&env, 1);
    env.state()
        .borrow_mut()
        .refer_a_friend_linked_guids
        .remove(&guid);
    assert_raf_visibility(&env, 0);
}

// Passive geometry fixture. Copy the original parsed OnLoad declaration, not a
// rewritten handler. Child creation still happens after script installation.
fn create_original_onload_scope(env: &WowLuaEnv, xml_file: &str, frame_name: &str, child: &str) {
    let original = parse_xml(&read_original(xml_file)).expect("SETUP FAILURE: original XML");
    let original_frame = original
        .elements
        .iter()
        .find_map(|element| match element {
            XmlElement::Frame(frame) if frame.name.as_deref() == Some(frame_name) => Some(frame),
            _ => None,
        })
        .expect("SETUP FAILURE: original frame declaration");
    let mut scope = parse_xml(&format!(
        r#"
        <Ui><Frame name="C3BindingScope" parent="UIParent" frameLevel="10">
            <Frames><Frame name="{child}" frameLevel="20"/></Frames>
        </Frame></Ui>
    "#
    ))
    .unwrap();
    let XmlElement::Frame(frame) = &mut scope.elements[0] else {
        panic!("scope frame")
    };
    frame.children.push(FrameChildElement::Scripts(ScriptsXml {
        on_load: original_frame.scripts().unwrap().on_load.clone(),
        ..Default::default()
    }));
    exec_and_audit(env, &format!("assert(rawget(_G, '{child}') == nil)"));
    let result = create_frame_from_xml(
        &env.loader_env(),
        frame,
        "Frame",
        None,
        None,
        None,
        &mut LoadTiming::default(),
    );
    assert_no_errors(env);
    result.expect("original OnLoad must see its child at invocation time");
}

#[test]
fn original_wrath_mainmenu_onload_resolves_late_child_and_rebound_global() {
    let env = consumer_env(&["UIParent.lua"]);
    create_original_onload_scope(
        &env,
        "MainMenuBar.xml",
        "MainMenuBar",
        "MainMenuBarArtFrame",
    );
    exec_and_audit(
        &env,
        r#"
        assert(MainMenuBarArtFrame:GetFrameLevel() == 21)
        local first = MainMenuBarArtFrame
        local replacement = CreateFrame('Frame', nil, UIParent)
        replacement:SetFrameLevel(40)
        MainMenuBarArtFrame = replacement
        C3BindingScope:GetScript('OnLoad')(C3BindingScope)
        assert(replacement:GetFrameLevel() == 41)
        assert(first:GetFrameLevel() == 21)
    "#,
    );
}

#[test]
fn original_wrath_vehicle_onload_raises_child_and_initializes_action_buttons() {
    let env = consumer_env(&["UIParent.lua", "VehicleMenuBar.lua"]);
    // NormalTexture geometry from the inherited ActionButtonTemplate.xml:74-85.
    // Ordinary widgets only; no replacement VehicleMenuBar functions or methods.
    read_original("ActionButtonTemplate.xml");
    exec_and_audit(
        &env,
        r#"
        for index = 1, 6 do
            local name = 'VehicleMenuBarActionButton'..index
            local button = CreateFrame('Button', name, UIParent)
            button:SetNormalTexture('Interface\\Buttons\\UI-Panel-Button-Up')
            local texture = button:GetNormalTexture()
            texture:ClearAllPoints()
            texture:SetPoint('CENTER', button, 'CENTER', 0, -1)
            texture:SetSize(66, 66)
            button:CreateFontString(name..'HotKey', 'OVERLAY')
        end
    "#,
    );
    create_original_onload_scope(
        &env,
        "VehicleMenuBar.xml",
        "VehicleMenuBar",
        "VehicleMenuBarArtFrame",
    );
    exec_and_audit(
        &env,
        r#"
        assert(VehicleMenuBarArtFrame:GetFrameLevel() == 21, 'vehicle child level: '..VehicleMenuBarArtFrame:GetFrameLevel())
        assert(C3BindingScope:IsEventRegistered('UNIT_ENTERED_VEHICLE'), 'vehicle OnLoad must register UNIT_ENTERED_VEHICLE')
        assert(C3BindingScope:IsEventRegistered('UNIT_DISPLAYPOWER'), 'vehicle OnLoad must register UNIT_DISPLAYPOWER')
        for index = 1, 6 do
            local texture = _G['VehicleMenuBarActionButton'..index]:GetNormalTexture()
            assert(texture:GetHeight() == 105 and texture:GetWidth() == 105,
                'vehicle texture '..index..': '..texture:GetWidth()..'x'..texture:GetHeight())
        end
    "#,
    );
}

#[test]
fn original_wrath_character_binding_uses_nil_key_label_fallback_on_load_and_event() {
    let env = consumer_env(&["UIParent.lua", "MainMenuBarMicroButtons.lua"]);
    exec_and_audit(
        &env,
        r#"
        assert(GetBindingKey('TOGGLECHARACTER0') == 'C')
        assert(rawget(_G, 'KEY_C') == nil and rawget(_G, 'C_KEY_TEXT') == nil)
        CharacterMicroButton = CreateFrame('Button', 'CharacterMicroButton', UIParent)
        CharacterMicroButton_OnLoad(CharacterMicroButton)
        assert(string.find(CharacterMicroButton.tooltipText, '(C)', 1, true))
        assert(C_KEY_TEXT == nil and rawget(_G, 'C_KEY_TEXT') == nil)
        assert(GetBindingText('C', 'KEY_') == 'C')
        CharacterMicroButton:SetScript('OnEvent', CharacterMicroButton_OnEvent)
        CharacterMicroButton.tooltipText = 'before UPDATE_BINDINGS'
        assert(type(C_CVar.GetCVar('Sound_EnableAllSound')) == 'string')
    "#,
    );
    let result = env.fire_event("UPDATE_BINDINGS");
    assert_no_errors(&env);
    result.unwrap();
    exec_and_audit(
        &env,
        r#"
        assert(string.find(CharacterMicroButton.tooltipText, '(C)', 1, true))
        assert(GetBindingText('C', 'KEY_') == 'C')
        assert(C_KEY_TEXT == nil and rawget(_G, 'C_KEY_TEXT') == nil)
    "#,
    );
}
