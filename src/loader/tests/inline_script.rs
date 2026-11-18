use super::super::LoadTiming;
use super::super::addon::AddonContext;
use super::super::lua_file::load_lua_file;
use super::super::xml_file::load_xml_file;
use crate::loader::load_addon;
use crate::lua_api::WowLuaEnv;
use crate::xml::parse_xml_file;
use rilua::LuaApi;
const INLINE_SCRIPT_ERROR_XML: &str = r#"<Ui>
            <Script>
                ScriptErrorTestInit = "initialized"
            </Script>
            <Script>
                error("intentional error")
            </Script>
            <Script>
                ScriptErrorTestAfter = "still running"
            </Script>
        </Ui>"#;

use std::path::{Path, PathBuf};

const SCOPED_FILE_LUA: &str = r#"
        local addonName, addonEnv = ...
        _G.ScopedFileUsesAddonEnv = getfenv(1) == addonEnv
        _G.ScopedFileGlobalFallbackWorks = type(CreateFrame) == "function"
        "#;
const SCOPED_MODIFIER_XML: &str = r#"<Ui>
            <ScopedModifier scriptsUseGivenEnv="true" hideFromGlobalEnv="true">
                <Script>
                    local addonName, addonEnv = ...
                    _G.ScopedInlineUsesAddonEnv = getfenv(1) == addonEnv
                    _G.ScopedInlineGlobalFallbackWorks = type(CreateFrame) == "function"
                    ScopedLocalMixin = {
                        DescribeScope = function(self)
                            return "scoped"
                        end,
                    }
                </Script>
                <Script file="scoped-file.lua"/>
                <Frame name="ScopedHiddenFrame" mixin="ScopedLocalMixin">
                    <Scripts>
                        <OnLoad>
                            _G.ScopedMixinResult = self:DescribeScope()
                        </OnLoad>
                    </Scripts>
                </Frame>
                <Script>
                    local addonName, addonEnv = ...
                    setmetatable(addonEnv, { __index = { ScopedCustomFallback = "custom", _G = _G } })
                </Script>
                <ScopedModifier scriptsUseGivenEnv="true">
                    <Script>
                        _G.ScopedNestedCustomFallback = ScopedCustomFallback
                    </Script>
                </ScopedModifier>
            </ScopedModifier>
            <ScopedModifier addToSecureEnv="true">
                <Frame name="ScopedSecureFrame"/>
            </ScopedModifier>
            <ScopedModifier addToSecureEnv="true" hideFromGlobalEnv="true">
                <Frame name="ScopedSecureOnlyFrame"/>
            </ScopedModifier>
        </Ui>"#;
const SCOPED_SECURE_FALLBACK_XML: &str = r#"<Ui>
            <ScopedModifier scriptsUseGivenEnv="true">
                <Script>
                    assert(ScopedSecureEnvOnlyValue == "secure-fallback")
                </Script>
            </ScopedModifier>
        </Ui>"#;
const SECURE_RETAIL_XML: &str = r#"<Ui>
            <ScopedModifier scriptsUseGivenEnv="true">
                <Frame name="ScopedRetailSecureHandlerFrame">
                    <Scripts>
                        <OnLoad>
                            self:SetAttribute("ObservedHandlerRan", true)
                            self:SetAttribute("ObservedAddonValue", ScopedAddonValue or "__nil")
                            self:SetAttribute("ObservedSecureValue", ScopedSecureOnlyValue or "__nil")
                        </OnLoad>
                    </Scripts>
                </Frame>
            </ScopedModifier>
        </Ui>"#;
const SECURE_RETAIL_HANDLER_PROBE: &str = r#"
            local setupValue = rawget(__secureenv, "ScopedSetupObserved")
            local setupTable = rawget(__secureenv, "ScopedSetupTable")
            local setupIsSecure = setupValue == "addon-only"
                and rawget(_G, "ScopedSetupObserved") == nil
            local frame = rawget(_G, "ScopedRetailSecureHandlerFrame")
                or rawget(__secureenv, "ScopedRetailSecureHandlerFrame")
            if not frame then
                return setupValue or "", setupIsSecure, false, false, "", ""
            end
            local handler = frame:GetScript("OnLoad")
            local handlerEnvIsAddon = type(handler) == "function"
                and rawequal(getfenv(handler), setupTable)
            return setupValue or "", setupIsSecure,
                frame:GetAttribute("ObservedHandlerRan") == true,
                handlerEnvIsAddon,
                frame:GetAttribute("ObservedAddonValue") or "",
                frame:GetAttribute("ObservedSecureValue") or ""
            "#;

#[test]
fn test_parse_wowless_test_xml() {
    let path = std::path::Path::new("Interface/TestAddOns/Wowless/test.xml");
    if !path.exists() {
        return; // Skip if Wowless addon not present
    }
    let result = parse_xml_file(path);
    assert!(
        result.is_ok(),
        "Wowless test.xml should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_xml_inline_script_error_continues() {
    // In WoW, a Lua error inside a <Script> element does not abort the XML file.
    // Errors are caught by the error handler and processing continues.
    // This is how Wowless test.xml works: it sets WowlessXmlErrors = {} in the
    // first <Script>, then later elements may error, but the global persists.
    let env = WowLuaEnv::new().unwrap();
    let temp_dir = std::env::temp_dir().join("wow-sim-test-inline-script-error");
    let xml_path = write_inline_script_error_xml(&temp_dir);
    let addon_table = env.create_addon_table().unwrap();
    let ctx =
        AddonContext::new(env.lua(), "TestAddon", addon_table, &temp_dir, false, false).unwrap();
    let before_errors = env.state().borrow().lua_errors.len();
    let result = load_xml_file(
        &env.loader_env(),
        &xml_path,
        &ctx,
        &mut LoadTiming::default(),
        &mut Vec::new(),
    );
    assert!(
        result.is_ok(),
        "inline script error should not abort XML file: {:?}",
        result.err()
    );
    // First script should have run
    assert_inline_scripts_ran(&env);
    // Third script should also run despite second erroring
    assert_inline_error_collected(&env, before_errors);
    let _ = std::fs::remove_dir_all(&temp_dir);
}

fn write_inline_script_error_xml(temp_dir: &Path) -> PathBuf {
    std::fs::create_dir_all(temp_dir).unwrap();
    let xml_path = temp_dir.join("test.xml");
    std::fs::write(&xml_path, INLINE_SCRIPT_ERROR_XML).unwrap();
    xml_path
}

fn assert_inline_scripts_ran(env: &WowLuaEnv) {
    let init: String = env.eval("return ScriptErrorTestInit").unwrap();
    assert_eq!(init, "initialized");
    let after: String = env.eval("return ScriptErrorTestAfter").unwrap();
    assert_eq!(after, "still running");
}

fn assert_inline_error_collected(env: &WowLuaEnv, before_errors: usize) {
    let state = env.state().borrow();
    let new_errors = &state.lua_errors[before_errors..];
    assert!(
        new_errors
            .iter()
            .any(|msg| msg.contains("intentional error")),
        "inline XML script error should be collected in state.lua_errors: {new_errors:?}"
    );
}

#[test]
fn test_load_lua_file_runtime_error_collects_lua_error() {
    let env = WowLuaEnv::new().unwrap();
    let temp_dir = std::env::temp_dir().join("wow-sim-test-load-lua-file-runtime-error");
    std::fs::create_dir_all(&temp_dir).unwrap();
    let lua_path = temp_dir.join("test.lua");
    std::fs::write(&lua_path, r#"error("load lua failed")"#).unwrap();

    let addon_table = env.create_addon_table().unwrap();
    let ctx =
        AddonContext::new(env.lua(), "TestAddon", addon_table, &temp_dir, false, false).unwrap();

    let before_errors = env.state().borrow().lua_errors.len();
    let result = load_lua_file(
        &env.loader_env(),
        &lua_path,
        &ctx,
        &mut LoadTiming::default(),
    );
    assert!(result.is_err(), "runtime error should fail load_lua_file");

    let state = env.state().borrow();
    let new_errors = &state.lua_errors[before_errors..];
    assert!(
        new_errors.iter().any(|msg| msg.contains("load lua failed")),
        "load_lua_file runtime errors should be collected in state.lua_errors: {new_errors:?}"
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_collect_lua_error_tracks_seen_message_counts() {
    let env = WowLuaEnv::new().unwrap();

    let first_seen = crate::lua_api::script_helpers::collect_lua_error(
        env.rilua().state(),
        "runtime error: repeated boom\nstack traceback:\n\t[C]: in function 'error'",
    );
    let second_seen = crate::lua_api::script_helpers::collect_lua_error(
        env.rilua().state(),
        "runtime error: repeated boom\nstack traceback:\n\t[C]: in function 'error'",
    );
    let third_seen =
        crate::lua_api::script_helpers::collect_lua_error(env.rilua().state(), "different boom");

    let state = env.state().borrow();
    assert_eq!(state.lua_errors.len(), 3);
    assert_eq!(state.lua_error_counts.get("repeated boom"), Some(&2));
    assert_eq!(state.lua_error_counts.get("different boom"), Some(&1));
    assert!(first_seen, "first occurrence should be reported");
    assert!(!second_seen, "repeat occurrence should be suppressible");
    assert!(third_seen, "new message should be reported");
}

#[test]
fn test_suppressed_lua_error_summary_lines_report_repeat_counts() {
    let env = WowLuaEnv::new().unwrap();

    crate::lua_api::script_helpers::collect_lua_error(
        env.rilua().state(),
        "runtime error: repeated boom\nstack traceback:\n\t[C]: in function 'error'",
    );
    crate::lua_api::script_helpers::collect_lua_error(
        env.rilua().state(),
        "runtime error: repeated boom\nstack traceback:\n\t[C]: in function 'error'",
    );
    crate::lua_api::script_helpers::collect_lua_error(env.rilua().state(), "different boom");

    let state = env.state().borrow();
    let summary = crate::lua_errors::suppressed_error_summary_lines(&state);
    assert_eq!(summary.len(), 1);
    assert_eq!(
        summary[0],
        "Lua error suppressed 1 additional times: repeated boom\nstack traceback:\n\t[C]: in function 'error'"
    );
}

#[test]
fn test_collect_lua_error_records_loading_addon_name() {
    let env = WowLuaEnv::new().unwrap();
    super::register_loading_test_addon(&env);

    crate::lua_api::script_helpers::collect_lua_error(env.rilua().state(), "boom");

    let state = env.state().borrow();
    assert_eq!(state.lua_error_records.len(), 1);
    assert_eq!(
        state.lua_error_records[0].addon_name.as_deref(),
        Some("TestAddon")
    );
}

#[test]
fn test_collect_lua_error_prefers_executing_addon_name() {
    let env = WowLuaEnv::new().unwrap();
    env.register_addon(crate::lua_api::AddonInfo {
        folder_name: "LoadingAddon".to_string(),
        title: "LoadingAddon".to_string(),
        enabled: true,
        loaded: true,
        ..Default::default()
    });
    env.register_addon(crate::lua_api::AddonInfo {
        folder_name: "ExecutingAddon".to_string(),
        title: "ExecutingAddon".to_string(),
        enabled: true,
        loaded: true,
        ..Default::default()
    });
    let (loading_idx, executing_idx) = {
        let state = env.state().borrow();
        let loading_idx = state
            .addons
            .iter()
            .position(|addon| addon.folder_name == "LoadingAddon")
            .expect("LoadingAddon should be registered");
        let executing_idx = state
            .addons
            .iter()
            .position(|addon| addon.folder_name == "ExecutingAddon")
            .expect("ExecutingAddon should be registered");
        (loading_idx as u16, executing_idx as u16)
    };
    {
        let mut state = env.state().borrow_mut();
        state.loading_addon_index = Some(loading_idx);
        state.executing_addon_index = Some(executing_idx);
    }

    crate::lua_api::script_helpers::collect_lua_error(env.rilua().state(), "boom");

    let state = env.state().borrow();
    assert_eq!(state.lua_error_records.len(), 1);
    assert_eq!(
        state.lua_error_records[0].addon_name.as_deref(),
        Some("ExecutingAddon")
    );
}

#[test]
fn secure_env_xml_animation_groups_attach_to_named_frames() {
    const ADDON_NAME: &str = "SecureAnimAddon";

    let env = WowLuaEnv::new().unwrap();
    env.register_addon(crate::lua_api::AddonInfo {
        folder_name: ADDON_NAME.to_string(),
        title: ADDON_NAME.to_string(),
        enabled: true,
        loaded: true,
        use_secure_env: true,
        ..Default::default()
    });
    super::set_loading_addon_index(&env, ADDON_NAME);

    let temp_dir = std::env::temp_dir().join("wow-sim-secure-env-xml-animations");
    std::fs::create_dir_all(&temp_dir).unwrap();
    let xml_path = temp_dir.join("test.xml");
    std::fs::write(
        &xml_path,
        r#"<Ui>
            <Frame name="SecureAnimFrame" parent="UIParent">
                <Animations>
                    <AnimationGroup parentKey="Pulse">
                        <Animation duration="1" order="1" />
                    </AnimationGroup>
                </Animations>
            </Frame>
        </Ui>"#,
    )
    .unwrap();

    let addon_table = env.create_addon_table().unwrap();
    let ctx =
        AddonContext::new(env.lua(), ADDON_NAME, addon_table, &temp_dir, true, false).unwrap();

    load_xml_file(
        &env.loader_env(),
        &xml_path,
        &ctx,
        &mut LoadTiming::default(),
        &mut Vec::new(),
    )
    .unwrap();

    drop(ctx);
    let test_ctx = super::TestCtx { env, temp_dir };
    let has_animation_group: bool = test_ctx
        .env
        .eval("return SecureAnimFrame.Pulse ~= nil")
        .expect("animation parentKey should be readable");
    assert!(
        has_animation_group,
        "secure XML animation setup should resolve the frame outside the secure fenv"
    );
}

#[test]
fn scoped_modifier_scripts_use_given_env_for_scripts_and_mixins() {
    let env = WowLuaEnv::new().unwrap();
    let temp_dir = std::env::temp_dir().join("wow-sim-test-scoped-modifier-env");
    let xml_path = write_scoped_modifier_fixture(&temp_dir);
    load_scoped_modifier_fixture(&env, &temp_dir, &xml_path);
    let results = read_scoped_modifier_results(&env);
    assert_scoped_environment_results(&results);
    assert_scoped_frame_results(&results);
}

fn write_scoped_modifier_fixture(temp_dir: &Path) -> PathBuf {
    std::fs::create_dir_all(temp_dir).unwrap();
    let xml_path = temp_dir.join("test.xml");
    let file_script = temp_dir.join("scoped-file.lua");
    std::fs::write(&file_script, SCOPED_FILE_LUA).unwrap();
    std::fs::write(&xml_path, SCOPED_MODIFIER_XML).unwrap();
    xml_path
}

fn load_scoped_modifier_fixture(env: &WowLuaEnv, temp_dir: &Path, xml_path: &Path) {
    let addon_table = env.create_addon_table().unwrap();
    let ctx =
        AddonContext::new(env.lua(), "TestAddon", addon_table, temp_dir, false, false).unwrap();
    load_xml_file(
        &env.loader_env(),
        xml_path,
        &ctx,
        &mut LoadTiming::default(),
        &mut Vec::new(),
    )
    .unwrap();
}

struct ScopedModifierResults {
    inline_uses_env: bool,
    inline_fallback: bool,
    file_uses_env: bool,
    file_fallback: bool,
    nested_custom_fallback: String,
    mixin_result: String,
    frame_hidden_from_global: bool,
    frame_added_to_secure_env: bool,
    frame_without_policy_absent_from_secure_env: bool,
    secure_only_frame_added_to_secure_env: bool,
    secure_only_frame_hidden_from_global: bool,
}

fn read_scoped_modifier_results(env: &WowLuaEnv) -> ScopedModifierResults {
    ScopedModifierResults {
        inline_uses_env: env.eval("return ScopedInlineUsesAddonEnv == true").unwrap(),
        inline_fallback: env
            .eval("return ScopedInlineGlobalFallbackWorks == true")
            .unwrap(),
        file_uses_env: env.eval("return ScopedFileUsesAddonEnv == true").unwrap(),
        file_fallback: env
            .eval("return ScopedFileGlobalFallbackWorks == true")
            .unwrap(),
        nested_custom_fallback: env.eval("return ScopedNestedCustomFallback").unwrap(),
        mixin_result: env.eval("return ScopedMixinResult").unwrap(),
        frame_hidden_from_global: env.eval("return ScopedHiddenFrame == nil").unwrap(),
        frame_added_to_secure_env: env
            .eval("return __secureenv.ScopedSecureFrame == ScopedSecureFrame")
            .unwrap(),
        frame_without_policy_absent_from_secure_env: env
            .eval("return __secureenv.ScopedHiddenFrame == nil")
            .unwrap(),
        secure_only_frame_added_to_secure_env: env
            .eval("return __secureenv.ScopedSecureOnlyFrame ~= nil")
            .unwrap(),
        secure_only_frame_hidden_from_global: env
            .eval("return ScopedSecureOnlyFrame == nil")
            .unwrap(),
    }
}

fn assert_scoped_environment_results(results: &ScopedModifierResults) {
    let inline_uses_env = results.inline_uses_env;
    let inline_fallback = results.inline_fallback;
    let file_uses_env = results.file_uses_env;
    let file_fallback = results.file_fallback;
    let nested_custom_fallback = results.nested_custom_fallback.as_str();
    let mixin_result = results.mixin_result.as_str();
    assert!(inline_uses_env, "inline XML script should run in addon env");
    assert!(inline_fallback, "scoped env should fall back to _G");
    assert!(file_uses_env, "file XML script should run in addon env");
    assert!(file_fallback, "file scoped env should fall back to _G");
    assert_eq!(nested_custom_fallback, "custom");
    assert_eq!(mixin_result, "scoped");
}

fn assert_scoped_frame_results(results: &ScopedModifierResults) {
    let frame_hidden_from_global = results.frame_hidden_from_global;
    let frame_added_to_secure_env = results.frame_added_to_secure_env;
    let frame_without_policy_absent_from_secure_env =
        results.frame_without_policy_absent_from_secure_env;
    let secure_only_frame_added_to_secure_env = results.secure_only_frame_added_to_secure_env;
    let secure_only_frame_hidden_from_global = results.secure_only_frame_hidden_from_global;
    assert!(
        frame_hidden_from_global,
        "hideFromGlobalEnv should keep named scoped frames out of _G"
    );
    assert!(
        frame_added_to_secure_env,
        "addToSecureEnv should copy named scoped frames into secureenv"
    );
    assert!(
        frame_without_policy_absent_from_secure_env,
        "frames without addToSecureEnv should stay out of secureenv"
    );
    assert!(
        secure_only_frame_added_to_secure_env,
        "addToSecureEnv should copy named frames into secureenv even when hidden from _G"
    );
    assert!(
        secure_only_frame_hidden_from_global,
        "hideFromGlobalEnv should still hide names when addToSecureEnv is also set"
    );
}

#[test]
fn scoped_modifier_scripts_use_secure_env_fallback_for_secure_addons() {
    let env = WowLuaEnv::new().unwrap();
    // INFERRED — R88g.3.q2: this pre-construction write is visible through the
    // shared secure-table reference used by the inline chunk. Evidence: pr46-49-decomp-answers.md, sections R88g.3.q2.
    env.exec("__secureenv.ScopedSecureEnvOnlyValue = 'secure-fallback'")
        .unwrap();
    let temp_dir = std::env::temp_dir().join("wow-sim-test-scoped-modifier-secure-env");
    std::fs::create_dir_all(&temp_dir).unwrap();
    let xml_path = temp_dir.join("test.xml");
    std::fs::write(&xml_path, SCOPED_SECURE_FALLBACK_XML).unwrap();
    let addon_table = env.create_addon_table().unwrap();
    let ctx =
        AddonContext::new(env.lua(), "TestAddon", addon_table, &temp_dir, true, false).unwrap();
    load_xml_file(
        &env.loader_env(),
        &xml_path,
        &ctx,
        &mut LoadTiming::default(),
        &mut Vec::new(),
    )
    .unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
fn secure_retail_scoped_handler_uses_only_the_given_environment() {
    if !crate::client_profile::is_active_retail_12_1_5() {
        return;
    }
    let env = WowLuaEnv::new().unwrap();
    env.exec("__secureenv.ScopedSecureOnlyValue = 'secure-only'")
        .unwrap();
    let temp_dir = tempfile::tempdir().unwrap();
    let addon_dir = temp_dir.path().join("TestSecureAddon");
    std::fs::create_dir(&addon_dir).unwrap();
    let toc_path = write_secure_retail_addon(&addon_dir);
    let load_result = load_addon(&env.loader_env(), &toc_path).unwrap();
    assert_eq!(load_result.lua_files, 1, "the setup file must load");
    assert_eq!(load_result.xml_files, 1, "the XML file must load");
    assert!(
        load_result.warnings.is_empty(),
        "{:?}",
        load_result.warnings
    );
    let observed = read_secure_retail_handler_state(&env);
    assert_secure_retail_handler_state(observed);
    assert!(env.state().borrow().lua_errors.is_empty());
}

fn write_secure_retail_addon(addon_dir: &Path) -> PathBuf {
    let toc_path = addon_dir.join("TestSecureAddon.toc");
    std::fs::write(
        &toc_path,
        format!(
            "## Interface: {}\n## UseSecureEnvironment: 1\nSetup.lua\nTestSecureAddon.xml\n",
            crate::client_profile::RETAIL_API_INTERFACE_VERSION
        ),
    )
    .unwrap();
    std::fs::write(
        addon_dir.join("Setup.lua"),
        "local _, addon = ...\naddon.ScopedAddonValue = 'addon-only'\nScopedSetupObserved = addon.ScopedAddonValue\nScopedSetupTable = addon\n",
    )
    .unwrap();
    std::fs::write(addon_dir.join("TestSecureAddon.xml"), SECURE_RETAIL_XML).unwrap();
    toc_path
}

type SecureRetailHandlerState = (String, bool, bool, bool, String, String);

fn read_secure_retail_handler_state(env: &WowLuaEnv) -> SecureRetailHandlerState {
    env.eval(SECURE_RETAIL_HANDLER_PROBE).unwrap()
}

fn assert_secure_retail_handler_state(observed: SecureRetailHandlerState) {
    let (
        setup_value,
        setup_is_secure,
        handler_ran,
        handler_env_is_addon,
        addon_value,
        secure_value,
    ) = observed;
    assert_eq!(
        setup_value, "addon-only",
        "the TOC Lua file must populate the add-on table"
    );
    assert!(
        setup_is_secure,
        "the setup file should use the shared secure environment"
    );
    assert!(handler_ran, "the XML OnLoad handler must run");
    assert!(
        handler_env_is_addon,
        "the XML OnLoad handler must use the add-on's given environment"
    );
    assert_eq!(addon_value, "addon-only");
    assert_eq!(
        secure_value, "__nil",
        "the handler must not read __secureenv"
    );
}
