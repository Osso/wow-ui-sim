//! Unchanged cached Wrath consumers; bounded compatibility, not native-client proof.
#![cfg(feature = "client-wrath")]

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use sha2::{Digest, Sha256};
use wow_ui_sim::loader::{LoadTiming, create_frame_from_xml, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::render::font::{DEFAULT_WOW_FONT, WowFontSystem};
use wow_ui_sim::xml::{XmlElement, parse_xml};

const ORIGINALS: &[(&str, &str)] = &[
    (
        "GameTime.lua",
        "86878a2b74491350afb3f5f120d0fb8525efa6179ca4ba23579262bfa998e434",
    ),
    (
        "FloatingChatFrame.xml",
        "38565a526fde29e0d47ff9f7837d35697d53ee93bcd4d8a1f3166b4922f71a98",
    ),
    (
        "Fonts.xml",
        "cf08457a8fd1e1d1788532bb10c2de7e05280291c11f6a00294709f104d26757",
    ),
    (
        "FontStyles.xml",
        "618ec8ba7234c5725576c31b6d97f30588e8653474292d3783edf602d0d31d8a",
    ),
    (
        "UIPanelTemplates.lua",
        "d975f94a6d7deb8895a8ab47a2b6975cca4fd9e45f67cc15c54c8e100b37427e",
    ),
    (
        "UIPanelTemplates.xml",
        "e3cb10077e9296a5067ec32e33655b01c8930bee4019db37196ac292f602b91c",
    ),
    (
        "OptionsFrameTemplates.lua",
        "ec6cc8f756a05a4f1233beaa918da66eee5e98de5e65bcd8ccdf8319decca771",
    ),
    (
        "OptionsFrameTemplates.xml",
        "098d545546b4f7e29f9589118b4a3e4c797d23f403b201a55a3fb519919d46c6",
    ),
    (
        "AudioOptionsFrame.lua",
        "5634ffa5aaf789240dd437dc7c2bba40701de35cff6583165bd9a796bed34d2f",
    ),
    (
        "AudioOptionsFrame.xml",
        "2c315d636e2a95f7f03752b8d8d003629f5d9b82bf0c35f7def2e4c5af0405d6",
    ),
];

fn read_original(name: &str) -> (PathBuf, String) {
    let toc = wow_ui_sim::client_profile::blizzard_ui_framexml_toc()
        .expect("SETUP FAILURE: populate the pinned Wrath runtime cache first");
    let path = toc.parent().expect("FrameXML TOC parent").join(name);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("SETUP FAILURE: {}: {error}", path.display()));
    let expected = ORIGINALS
        .iter()
        .find(|(file, _)| *file == name)
        .expect("SETUP FAILURE: consumer must have a pinned digest")
        .1;
    let digest = format!("{:x}", Sha256::digest(&bytes));
    assert_eq!(
        digest,
        expected,
        "SETUP FAILURE: changed original {}",
        path.display()
    );
    eprintln!("Wrath original={} sha256={digest}", path.display());
    (
        path,
        String::from_utf8(bytes).expect("SETUP FAILURE: UTF-8 source"),
    )
}

fn create_env() -> WowLuaEnv {
    let fonts = WowFontSystem::new_without_casc();
    assert!(
        fonts.font_system.db().faces().next().is_some(),
        "SETUP FAILURE: font face"
    );
    assert!(
        fonts.family_name(Some(DEFAULT_WOW_FONT)).is_some(),
        "SETUP FAILURE: font alias"
    );
    let env = WowLuaEnv::new().expect("SETUP FAILURE: Wrath environment");
    env.set_screen_size(1600.0, 1200.0);
    env.set_font_system(Rc::new(RefCell::new(fonts)));
    env.exec("PanelErrors = {}; seterrorhandler(function(message) PanelErrors[#PanelErrors + 1] = tostring(message) end)").unwrap();
    env
}

fn assert_no_errors(env: &WowLuaEnv) {
    env.exec("assert(#PanelErrors == 0, table.concat(PanelErrors, '\\n'))")
        .expect("original callback errors must not be swallowed");
    assert!(
        env.state().borrow().lua_errors.is_empty(),
        "{:?}",
        env.state().borrow().lua_errors
    );
}

fn exec_and_audit(env: &WowLuaEnv, code: &str) {
    let result = env.exec(code);
    assert_no_errors(env);
    result.unwrap();
}

// TOC entries are relative; XML <Script>/<Include> paths still resolve beside
// the unchanged original cache document, not beside a copied fixture.
fn load_original_files(env: &WowLuaEnv, names: &[&str]) {
    let root = tempfile::tempdir().unwrap();
    let mut toc = String::from("## Interface: 30300\n## Title: WrathPanelBoundary\n");
    for name in names {
        let (path, _) = read_original(name);
        let common = root
            .path()
            .ancestors()
            .find(|base| path.starts_with(base))
            .unwrap();
        let mut relative = PathBuf::new();
        for _ in root.path().strip_prefix(common).unwrap().components() {
            relative.push("..");
        }
        relative.push(path.strip_prefix(common).unwrap());
        toc.push_str(relative.to_str().expect("SETUP FAILURE: cache path UTF-8"));
        toc.push('\n');
    }
    let path = root.path().join("WrathPanelBoundary.toc");
    std::fs::write(&path, toc).unwrap();
    let result = load_addon(&env.loader_env(), &path);
    assert_no_errors(env);
    let loaded = result.expect("load unchanged original files");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
}

#[test]
fn original_wrath_gametime_captures_pi_and_formats_explicit_times() {
    let env = create_env();
    exec_and_audit(
        &env,
        r#"
        TIMEMANAGER_TICKER_24HOUR = '%02d:%02d'
        TIMEMANAGER_TICKER_12HOUR = '%d:%02d'
        TIME_TWELVEHOURAM = '%d:%02d AM'
        TIME_TWELVEHOURPM = '%d:%02d PM'
    "#,
    );
    let (_, source) = read_original("GameTime.lua");
    let result = env.loader_env().exec(&source);
    assert_no_errors(&env);
    result.expect("complete original GameTime.lua must load, including PI * 2");
    exec_and_audit(
        &env,
        r#"
        assert(PI == math.pi)
        SetCVar('timeMgrUseMilitaryTime', '1')
        assert(GameTime_GetFormattedTime(13, 7, false) == '13:07')
        SetCVar('timeMgrUseMilitaryTime', '0')
        assert(GameTime_GetFormattedTime(13, 7, true) == '1:07 PM')
        assert(GameTime_GetFormattedTime(0, 5, true) == '12:05 AM')
    "#,
    );
}

#[test]
fn original_wrath_floating_chat_fontstring_creates_and_updates_friend_count() {
    let env = create_env();
    load_original_files(&env, &["Fonts.xml", "FontStyles.xml"]);
    // Inputs outside the XML spelling boundary: a chat anchor and friend counts.
    // GetNumFriends observes declarative text before the ORIGINAL OnLoad sets it.
    exec_and_audit(
        &env,
        r#"
        DEFAULT_CHAT_FRAME = CreateFrame('Frame', 'PanelChatAnchor', UIParent)
        DEFAULT_CHAT_FRAME.buttonFrame = CreateFrame('Frame', nil, DEFAULT_CHAT_FRAME)
        function BNGetNumFriends() return 7, 2 end
        function GetNumFriends()
            assert(FriendsMicroButtonCount:GetText() == '0')
            return 9, 3
        end
    "#,
    );
    let (_, source) = read_original("FloatingChatFrame.xml");
    // Parse the entire original document: the historical failure aborted it all.
    let ui = parse_xml(&source).expect("complete original FloatingChatFrame.xml must parse");
    let frame = ui
        .elements
        .iter()
        .find_map(|element| match element {
            XmlElement::Button(frame) if frame.name.as_deref() == Some("FriendsMicroButton") => {
                Some(frame)
            }
            _ => None,
        })
        .expect("SETUP FAILURE: original FriendsMicroButton subtree");
    // No XML rewrite, extracted Lua body, or substitute OnLoad.
    let result = create_frame_from_xml(
        &env.loader_env(),
        frame,
        "Button",
        None,
        None,
        None,
        &mut LoadTiming::default(),
    );
    assert_no_errors(&env);
    assert_eq!(result.unwrap().as_deref(), Some("FriendsMicroButton"));
    exec_and_audit(
        &env,
        r#"
        assert(FriendsMicroButtonCount:GetObjectType() == 'FontString')
        assert(FriendsMicroButtonCount:GetParent() == FriendsMicroButton)
        assert(FriendsMicroButtonCount:GetText() == '5')
    "#,
    );
}

fn load_options_prerequisites(env: &WowLuaEnv) {
    // Pin Lua files referenced by original XML before that XML loads them.
    read_original("OptionsFrameTemplates.lua");
    read_original("AudioOptionsFrame.lua");
    load_original_files(
        env,
        &[
            "Fonts.xml",
            "FontStyles.xml",
            "UIPanelTemplates.lua",
            "UIPanelTemplates.xml",
            "OptionsFrameTemplates.xml",
        ],
    );
    exec_and_audit(
        env,
        r#"
        if not UIErrorsFrame then UIErrorsFrame = CreateFrame('Frame', 'UIErrorsFrame', UIParent) end
        PanelLoads = {}
        local function observe(name)
            local original = _G[name]
            assert(type(original) == 'function')
            _G[name] = function(self, ...)
                local key = self:GetName() .. ':' .. name
                PanelLoads[key] = (PanelLoads[key] or 0) + 1
                return original(self, ...)
            end
        end
        observe('OptionsFrame_OnLoad')
        observe('OptionsList_OnLoad')
        observe('OptionsListButton_OnLoad')
    "#,
    );
}

fn assert_options_buttons_and_categories(env: &WowLuaEnv) {
    exec_and_audit(
        env,
        r#"
        local function assert_left(text, offset)
            for index = 1, text:GetNumPoints() do
                local point, _, _, x = text:GetPoint(index)
                if point == 'LEFT' then
                    assert(x == offset)
                    return
                end
            end
            error('missing LEFT anchor')
        end
        local category = AudioOptionsFrameCategoryFrame
        assert(PanelLoads['AudioOptionsFrame:OptionsFrame_OnLoad'] == 1)
        assert(PanelLoads[category:GetName() .. ':OptionsList_OnLoad'] == 1)
        assert(#category.buttons >= 2)
        for _, button in ipairs(category.buttons) do
            assert(button.text == _G[button:GetName() .. 'Text'])
            assert(button.text:GetObjectType() == 'FontString')
            assert(button:IsShown(), 'hidden XML root must not hide public dynamic buttons')
            assert(button.toggle == _G[button:GetName() .. 'Toggle'])
            assert(PanelLoads[button:GetName() .. ':OptionsListButton_OnLoad'] == 1)
        end
        local root = CreateFrame('Frame', 'PanelSoundCategory', UIParent)
        root.name = 'Sound'
        OptionsFrame_AddCategory(AudioOptionsFrame, root)
        assert(category.buttons[1].element == root)
        assert(category.buttons[1].text:GetText() == 'Sound')
        assert_left(category.buttons[1].text, 8)
        local child = CreateFrame('Frame', 'PanelSoundChild', UIParent)
        child.name = 'Effects'
        child.parent = 'Sound'
        OptionsFrame_AddCategory(AudioOptionsFrame, child)
        root.collapsed = false
        child.hidden = false
        category:update()
        assert(category.buttons[1].toggle:IsShown())
        assert(category.buttons[2].element == child)
        assert(category.buttons[2].text:GetText() == 'Effects')
        assert_left(category.buttons[2].text, 16)
    "#,
    );
}

#[test]
fn original_wrath_audio_xml_nested_options_buttons_run_onload_once() {
    let env = create_env();
    load_options_prerequisites(&env);
    // Essential: original XML construction -> inherited category OnLoad -> public
    // CreateFrame. An idle Lua CreateFrame call cannot reproduce this ordering.
    load_original_files(&env, &["AudioOptionsFrame.xml"]);
    assert_options_buttons_and_categories(&env);
}

#[test]
fn original_wrath_audio_explicit_parent_nested_buttons_run_onload_once() {
    let env = create_env();
    load_options_prerequisites(&env);
    load_original_files(&env, &["AudioOptionsFrame.lua"]);
    let (_, source) = read_original("AudioOptionsFrame.xml");
    let ui = parse_xml(&source).unwrap();
    let frame = ui
        .elements
        .iter()
        .find_map(|element| match element {
            XmlElement::Frame(frame) if frame.name.as_deref() == Some("AudioOptionsFrame") => {
                Some(frame)
            }
            _ => None,
        })
        .expect("SETUP FAILURE: original AudioOptionsFrame subtree");
    // Same original subtree; loader-supplied explicit parent exercises direct
    // construction instead of the generated CreateFrame Lua path.
    let result = create_frame_from_xml(
        &env.loader_env(),
        frame,
        "Frame",
        Some("UIParent"),
        None,
        None,
        &mut LoadTiming::default(),
    );
    assert_no_errors(&env);
    assert_eq!(result.unwrap().as_deref(), Some("AudioOptionsFrame"));
    assert_options_buttons_and_categories(&env);
}
