//! Migrate only legacy simulator output at the persisted CVar boundary.
#![cfg(feature = "client-wowforever")]

use std::path::Path;
use wow_ui_sim::cvars::CVarStorage;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

const KEY: &str = "lastlockedtieredentrancecompanionabilities";
const LEGACY: &str = r#"{"9101":{"9301":1.0,"9302":1.0,"9303":1.0}}"#;
// RFC 8949 map {9101: {9301: 1, 9302: 1, 9303: 1}} as standard Base64.
const ENCODED: &str = "oRkjjaMZJFUBGSRWARkkVwE=";

fn seed(root: &Path, value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    let text = format!(r#"{{"{KEY}":"{escaped}","unrelated":"{escaped}"}}"#);
    std::fs::write(root.join("cvars.json"), &text).unwrap();
    text
}

#[test]
fn forever_delves_cvar_load_migrates_the_legacy_mapping_without_writing() {
    let fixture = tempfile::tempdir().unwrap();
    let original = seed(fixture.path(), LEGACY);
    let cvars = CVarStorage::with_session_wtf_path(fixture.path());
    assert_eq!(cvars.get(KEY).as_deref(), Some(ENCODED));
    assert_eq!(cvars.get("unrelated").as_deref(), Some(LEGACY));
    assert_eq!(cvars.get_default(KEY).as_deref(), Some(""));
    assert_eq!(
        std::fs::read_to_string(fixture.path().join("cvars.json")).unwrap(),
        original
    );
}

#[test]
fn forever_delves_cvar_load_leaves_encoded_and_unrecognized_values_unchanged() {
    let fixture = tempfile::tempdir().unwrap();
    for value in [
        ENCODED,
        "",
        "{",
        "[]",
        r#"{"9101":[]}"#,
        r#"{"09101":{"9301":1}}"#,
        r#"{"9101":{"0":1}}"#,
        r#"{"4294967296":{"9301":1}}"#,
        r#"{"9101":{"9301":2}}"#,
        r#"{"9101":{"9301":true}}"#,
    ] {
        let original = seed(fixture.path(), value);
        let cvars = CVarStorage::with_session_wtf_path(fixture.path());
        assert_eq!(cvars.get(KEY).as_deref(), Some(value));
        assert_eq!(
            std::fs::read_to_string(fixture.path().join("cvars.json")).unwrap(),
            original
        );
    }
}

#[test]
fn forever_delves_cvar_selected_cvar_util_reads_default_and_migrated_numeric_maps() {
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let addon = ui.join("Blizzard_SharedXMLBase");
    let toc_path = wow_ui_sim::loader::find_toc_file(&addon).unwrap();
    let toc = TocFile::from_file(&toc_path).unwrap();
    assert!(toc.allows_screen(ScreenKind::Game) && !toc.is_game_type_restricted());
    let index = toc
        .files
        .iter()
        .position(|path| path == Path::new("CvarUtil.lua"))
        .unwrap();
    assert!(toc.file_allows_screen(index, ScreenKind::Game));
    let source = std::fs::read_to_string(addon.join(&toc.files[index])).unwrap();
    let definitions = source
        .split_once("CVarCallbackRegistry = CreateFromMixins(CallbackRegistryMixin);")
        .expect("original selected file contains its standalone CVar helper definitions")
        .0;
    assert!(definitions.contains("DeserializeCBOR(C_EncodingUtil.DecodeBase64(GetCVar(name)))"));

    let default_fixture = tempfile::tempdir().unwrap();
    let env = WowLuaEnv::with_session_wtf_path(default_fixture.path().to_path_buf()).unwrap();
    env.exec(definitions)
        .expect("load unchanged selected CVar helper definitions");
    env.exec(
        r#"
        local name = 'lastLockedTieredEntranceCompanionAbilities'
        assert(GetCVar(name) == '')
        assert(next(GetCVarTable(name)) == nil)
        assert(GetCVarTableValue(name, 9101, 'missing') == 'missing')
        "#,
    )
    .expect("original CVarUtil must consume the native empty-string default");

    let legacy_fixture = tempfile::tempdir().unwrap();
    let original = seed(legacy_fixture.path(), LEGACY);
    let env = WowLuaEnv::with_session_wtf_path(legacy_fixture.path().to_path_buf()).unwrap();
    env.exec(definitions)
        .expect("load unchanged selected CVar helper definitions");
    env.exec(
        r#"
        local name = 'lastLockedTieredEntranceCompanionAbilities'
        local nodes = GetCVarTableValue(name, 9101)
        assert(nodes[9301] == 1 and nodes[9302] == 1 and nodes[9303] == 1)
        assert(nodes['9301'] == nil and GetCVarTableValue(name, '9101') == nil)
        assert(not pcall(C_EncodingUtil.DecodeBase64, '{"9101":{}}'))
        "#,
    )
    .expect("original CVarUtil must consume migrated numeric keys without relaxing Base64");
    assert_eq!(
        std::fs::read_to_string(legacy_fixture.path().join("cvars.json")).unwrap(),
        original
    );
}
