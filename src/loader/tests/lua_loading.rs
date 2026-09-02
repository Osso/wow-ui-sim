use super::*;

const MULTI_FILE_WIDGETS_LUA: &str = r#"
    local _, addon = ...
    local function updateKeyDirection(self) return "updated: " .. tostring(self) end
    local function onCVarUpdate(self, cvar)
        if cvar == "TestCVar" then
            if not updateKeyDirection then error("updateKeyDirection is nil!") end
            self.result = updateKeyDirection(self)
        end
    end
    function addon:CreateButton(name)
        local button = { name = name }
        onCVarUpdate(button, "TestCVar")
        return button
    end
"#;

const MULTI_FILE_BUTTON_LUA: &str = r#"
    local _, addon = ...
    function addon:CreateExtraButton(name) return addon:CreateButton(name .. "_extra") end
"#;

const MULTI_FILE_ADDON_LUA: &str = r#"
    local _, addon = ...
    local button = addon:CreateExtraButton("test")
    addon.testButton = button
"#;

/// Load multiple Lua files in sequence with a shared addon table.
fn load_test_lua_files(
    dir_suffix: &str,
    addon_name: &str,
    files: &[(&'static str, &str)],
) -> (TestCtx, Val) {
    let env = WowLuaEnv::new().unwrap();
    let temp_dir = std::env::temp_dir().join(format!("wow-sim-{}", dir_suffix));
    std::fs::create_dir_all(&temp_dir).unwrap();

    let addon_table = env.create_addon_table().unwrap();
    let ctx = AddonContext {
        name: addon_name,
        table: addon_table.clone(),
        addon_root: &temp_dir,
        use_secure_env: false,
        taint: false,
    };

    for (filename, content) in files {
        let path = temp_dir.join(filename);
        std::fs::write(&path, content).unwrap();
        load_lua_file(&env.loader_env(), &path, &ctx, &mut LoadTiming::default())
            .unwrap_or_else(|e| panic!("{} should load: {}", filename, e));
    }

    (TestCtx { env, temp_dir }, addon_table)
}

#[test]
fn test_multi_file_closures() {
    let (t, addon_table) = load_test_lua_files(
        "test-multifile",
        "MultiFileTest",
        &[
            ("widgets.lua", MULTI_FILE_WIDGETS_LUA),
            ("button.lua", MULTI_FILE_BUTTON_LUA),
            ("addon.lua", MULTI_FILE_ADDON_LUA),
        ],
    );

    let test_button = table_get(&t.env, addon_table, "testButton");
    let result = val_to_rust_string(&t.env, table_get(&t.env, test_button, "result"));
    assert!(
        result.starts_with("updated:"),
        "updateKeyDirection should have been called, got: {}",
        result
    );
}

#[test]
fn test_text_to_speech_checkload_recovers_clobbered_dropdown_globals() {
    let env = WowLuaEnv::new().unwrap();
    let temp_dir = std::env::temp_dir().join("wow-sim-test-tts-frame");
    std::fs::create_dir_all(&temp_dir).unwrap();
    let lua_path = temp_dir.join("TextToSpeechFrame.lua");
    std::fs::write(
        &lua_path,
        r#"
        Enum = { TtsVoiceType = { Standard = "standard", Alternate = "alternate" } }
        CALLS = {}

        function SetupVoiceMenu(_, voiceType)
            table.insert(CALLS, voiceType)
        end

        function TextToSpeechFrame_SetupVoiceDropdown(self)
            SetupVoiceMenu(self.PanelContainer.TtsVoiceDropdown, Enum.TtsVoiceType.Standard);
        end

        function TextToSpeechFrame_SetupAlternateVoiceDropdown(self)
            SetupVoiceMenu(self.PanelContainer.TtsVoiceAlternateDropdown, Enum.TtsVoiceType.Alternate);
        end

        function IsReadyToLoad()
            return true
        end

        function TextToSpeechFrame_CheckLoad(self)
            if not self.loaded and IsReadyToLoad(self.loadedEvents) then
                self.loaded = true;

                TextToSpeechFrame_SetupVoiceDropdown(self);
                TextToSpeechFrame_SetupAlternateVoiceDropdown(self);
            end
        end
        "#,
    )
    .unwrap();

    let addon_table = env.create_addon_table().unwrap();
    let ctx = AddonContext {
        name: "TestAddon",
        table: addon_table,
        addon_root: &temp_dir,
        use_secure_env: false,
        taint: false,
    };
    load_lua_file(
        &env.loader_env(),
        &lua_path,
        &ctx,
        &mut LoadTiming::default(),
    )
    .unwrap();

    env.exec(
        r#"
        TextToSpeechFrame_SetupVoiceDropdown = true
        TextToSpeechFrame_SetupAlternateVoiceDropdown = false
        TTS_TEST_FRAME = {
            loaded = false,
            loadedEvents = {},
            PanelContainer = {
                TtsVoiceDropdown = {},
                TtsVoiceAlternateDropdown = {},
            },
        }
        TextToSpeechFrame_CheckLoad(TTS_TEST_FRAME)
        "#,
    )
    .unwrap();

    let (voice_ty, alt_ty): (String, String) = env
        .eval(
            "return type(TextToSpeechFrame_SetupVoiceDropdown), type(TextToSpeechFrame_SetupAlternateVoiceDropdown)",
        )
        .unwrap();
    assert_eq!(voice_ty, "function");
    assert_eq!(alt_ty, "function");

    let calls: (String, String) = env.eval("return CALLS[1], CALLS[2]").unwrap();
    assert_eq!(calls.0, "standard");
    assert_eq!(calls.1, "alternate");

    let loaded: bool = env.eval("return TTS_TEST_FRAME.loaded").unwrap();
    assert!(
        loaded,
        "TextToSpeechFrame_CheckLoad should still mark the frame loaded"
    );

    std::fs::remove_file(&lua_path).ok();
    std::fs::remove_dir_all(&temp_dir).ok();
}

#[test]
fn third_party_bootstrap_files_load_in_normal_order() {
    let env = WowLuaEnv::new().unwrap();
    let temp_root = std::env::temp_dir().join("wow-sim-third-party-bootstrap-order-test");
    let addon_dir = temp_root.join("ThirdPartyBootstrapProbe");
    std::fs::create_dir_all(&addon_dir).unwrap();
    std::fs::write(
        addon_dir.join("Before.lua"),
        r#"ThirdPartyBootstrapProbeEvents = { "before" }"#,
    )
    .unwrap();
    std::fs::write(
        addon_dir.join("Bootstrap.lua"),
        r#"table.insert(ThirdPartyBootstrapProbeEvents, "bootstrap")"#,
    )
    .unwrap();
    std::fs::write(
        addon_dir.join("After.lua"),
        r#"table.insert(ThirdPartyBootstrapProbeEvents, "after")"#,
    )
    .unwrap();

    let toc = crate::toc::TocFile::parse(
        &addon_dir,
        r#"
## Title: ThirdPartyBootstrapProbe
Before.lua
Bootstrap.lua [Bootstrap]
After.lua
"#,
    );

    load_addon_from_toc(&env.loader_env(), &toc).unwrap();
    let events: String = env
        .eval("return table.concat(ThirdPartyBootstrapProbeEvents, ',')")
        .unwrap();
    assert_eq!(events, "before,bootstrap,after");

    std::fs::remove_dir_all(&temp_root).ok();
}

#[test]
fn bootstrap_self_load_addon_is_noop_without_recursive_normal_file_load() {
    let env = WowLuaEnv::new().unwrap();
    let temp_root = std::env::temp_dir().join("wow-sim-bootstrap-self-load-test");
    let addon_dir = temp_root.join("Blizzard_BootstrapProbe");
    std::fs::create_dir_all(&addon_dir).unwrap();
    std::fs::write(
        addon_dir.join("Blizzard_BootstrapProbe.toc"),
        r#"
## Title: Blizzard_BootstrapProbe
## LoadOnDemand: 1
Bootstrap.lua [Bootstrap]
Normal.lua
"#,
    )
    .unwrap();
    std::fs::write(
        addon_dir.join("Bootstrap.lua"),
        r#"
        local _, private = ...
        private.bootstrapSeen = true
        BootstrapProbeEvents = BootstrapProbeEvents or {}
        table.insert(BootstrapProbeEvents, "bootstrap start")
        local loaded, reason = C_AddOns.LoadAddOn("Blizzard_BootstrapProbe")
        table.insert(BootstrapProbeEvents, "self load " .. tostring(loaded) .. ":" .. tostring(reason))
        table.insert(BootstrapProbeEvents, private.normalSeen and "normal ran early" or "normal not early")
        "#,
    )
    .unwrap();
    std::fs::write(
        addon_dir.join("Normal.lua"),
        r#"
        local _, private = ...
        private.normalSeen = true
        BootstrapProbeEvents = BootstrapProbeEvents or {}
        table.insert(BootstrapProbeEvents, private.bootstrapSeen and "normal sees bootstrap" or "normal missing bootstrap")
        "#,
    )
    .unwrap();

    env.state().borrow_mut().addon_base_paths = vec![temp_root.clone()];
    let (loaded, reason): (bool, Option<String>) = env
        .eval(r#"return C_AddOns.LoadAddOn("Blizzard_BootstrapProbe")"#)
        .unwrap();
    assert!(loaded, "outer LoadAddOn should load addon: {reason:?}");
    assert_eq!(reason, None);

    let (events, loaded_state, lod): (String, bool, bool) = env
        .eval(
            r#"
            return table.concat(BootstrapProbeEvents, ","),
                   C_AddOns.IsAddOnLoaded("Blizzard_BootstrapProbe"),
                   C_AddOns.IsAddOnLoadOnDemand("Blizzard_BootstrapProbe")
            "#,
        )
        .unwrap();
    assert_eq!(
        events,
        "bootstrap start,self load true:nil,normal not early,normal sees bootstrap"
    );
    assert!(loaded_state, "outer load should mark LoD addon loaded");
    assert!(
        lod,
        "addon registration should preserve LoadOnDemand metadata"
    );

    std::fs::remove_dir_all(&temp_root).ok();
}

#[test]
fn nested_runtime_addon_cycle_is_guarded() {
    let env = WowLuaEnv::new().unwrap();
    let temp_root = std::env::temp_dir().join("wow-sim-nested-runtime-addon-cycle-test");
    let addon_a_dir = temp_root.join("RuntimeCycleA");
    let addon_b_dir = temp_root.join("RuntimeCycleB");
    std::fs::create_dir_all(&addon_a_dir).unwrap();
    std::fs::create_dir_all(&addon_b_dir).unwrap();

    std::fs::write(
        addon_a_dir.join("RuntimeCycleA.toc"),
        r#"
## Title: RuntimeCycleA
## LoadOnDemand: 1
## Dependencies: RuntimeCycleB
RuntimeCycleA.lua
"#,
    )
    .unwrap();
    std::fs::write(
        addon_a_dir.join("RuntimeCycleA.lua"),
        r#"
        RuntimeCycleEvents = RuntimeCycleEvents or {}
        table.insert(RuntimeCycleEvents, "A:start")
        table.insert(RuntimeCycleEvents, "A:end")
        "#,
    )
    .unwrap();
    std::fs::write(
        addon_b_dir.join("RuntimeCycleB.toc"),
        r#"
## Title: RuntimeCycleB
## LoadOnDemand: 1
RuntimeCycleB.lua
"#,
    )
    .unwrap();
    std::fs::write(
        addon_b_dir.join("RuntimeCycleB.lua"),
        r#"
        RuntimeCycleEvents = RuntimeCycleEvents or {}
        table.insert(RuntimeCycleEvents, "B:start")
        RuntimeCycleAWasLoading, RuntimeCycleAWasLoaded = C_AddOns.IsAddOnLoaded("RuntimeCycleA")
        local loaded, reason = C_AddOns.LoadAddOn("RuntimeCycleA")
        table.insert(RuntimeCycleEvents, "B:load-a:" .. tostring(loaded) .. ":" .. tostring(reason))
        table.insert(RuntimeCycleEvents, "B:end")
        "#,
    )
    .unwrap();

    env.state().borrow_mut().addon_base_paths = vec![temp_root.clone()];
    let (loaded, reason): (bool, Option<String>) = env
        .eval(r#"return C_AddOns.LoadAddOn("RuntimeCycleA")"#)
        .unwrap();
    assert!(loaded, "outer LoadAddOn should load addon: {reason:?}");
    assert_eq!(reason, None);

    let (events, addon_a_loaded, addon_b_loaded, a_was_loading, a_was_loaded): (
        String,
        bool,
        bool,
        bool,
        bool,
    ) = env
        .eval(
            r#"
            local _, addonALoaded = C_AddOns.IsAddOnLoaded("RuntimeCycleA")
            local _, addonBLoaded = C_AddOns.IsAddOnLoaded("RuntimeCycleB")
            return table.concat(RuntimeCycleEvents, ","), addonALoaded, addonBLoaded,
                   RuntimeCycleAWasLoading, RuntimeCycleAWasLoaded
            "#,
        )
        .unwrap();
    assert_eq!(events, "B:start,B:load-a:true:nil,B:end,A:start,A:end");
    assert!(addon_a_loaded, "RuntimeCycleA should be loaded");
    assert!(addon_b_loaded, "RuntimeCycleB should be loaded");
    assert!(
        a_was_loading,
        "RuntimeCycleA should report loading inside B"
    );
    assert!(
        !a_was_loaded,
        "RuntimeCycleA should not report loaded before its outer load commits"
    );

    std::fs::remove_dir_all(&temp_root).ok();
}

#[test]
fn blizzard_lua_files_replay_into_secure_environment() {
    let env = WowLuaEnv::new().unwrap();
    let temp_root = std::env::temp_dir().join("wow-sim-secure-replay-test");
    let addon_dir = temp_root.join("Blizzard_SharedXMLBase");
    std::fs::create_dir_all(&addon_dir).unwrap();
    std::fs::write(
        addon_dir.join("Core.lua"),
        r#"ReplayLibraryValue = { marker = "shared" }"#,
    )
    .unwrap();
    std::fs::write(
        addon_dir.join("GlobalOnly.lua"),
        r#"ReplayGlobalOnlyValue = "global-only""#,
    )
    .unwrap();

    let toc = crate::toc::TocFile::parse(
        &addon_dir,
        r#"
## Title: Blizzard_SharedXMLBase
## AllowLoad: Game
Core.lua
GlobalOnly.lua [AllowLoadEnvironment Global]
"#,
    );

    let result = load_addon_from_toc(&env.loader_env(), &toc).unwrap();
    assert_eq!(result.lua_files, 3);

    let (global_marker, secure_marker, global_only_type): (String, String, String) = env
        .eval(
            r#"
            return _G.ReplayLibraryValue.marker,
                   __secureenv.ReplayLibraryValue.marker,
                   type(rawget(__secureenv, "ReplayGlobalOnlyValue"))
            "#,
        )
        .unwrap();
    assert_eq!(global_marker, "shared");
    assert_eq!(secure_marker, "shared");
    assert_eq!(global_only_type, "nil");

    std::fs::remove_dir_all(&temp_root).ok();
}

#[test]
fn blizzard_frame_xml_util_replays_aura_comparators_into_secure_environment() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("AuraUtil = { bootstrap = true }").unwrap();

    let temp_root = std::env::temp_dir().join("wow-sim-framexmlutil-secure-replay-test");
    let addon_dir = temp_root.join("Blizzard_FrameXMLUtil");
    std::fs::create_dir_all(&addon_dir).unwrap();
    std::fs::write(
        addon_dir.join("AuraUtil.lua"),
        r#"
        AuraUtil = {}
        AuraUtil.DefaultAuraCompare = function() return "default" end
        AuraUtil.UnitFrameDebuffComparator = function() return "debuff" end
        "#,
    )
    .unwrap();

    let toc = crate::toc::TocFile::parse(
        &addon_dir,
        r#"
## Title: Blizzard_FrameXMLUtil
## AllowLoad: Game
AuraUtil.lua
"#,
    );

    load_addon_from_toc(&env.loader_env(), &toc).unwrap();

    let comparator_types: (String, String, String, String) = env
        .eval(
            r#"
            return type(rawget(_G, "AuraUtil").DefaultAuraCompare),
                   type(rawget(__secureenv, "AuraUtil").DefaultAuraCompare),
                   type(rawget(_G, "AuraUtil").UnitFrameDebuffComparator),
                   type(rawget(__secureenv, "AuraUtil").UnitFrameDebuffComparator)
            "#,
        )
        .unwrap();
    assert_eq!(
        comparator_types,
        (
            "function".into(),
            "function".into(),
            "function".into(),
            "function".into()
        )
    );

    std::fs::remove_dir_all(&temp_root).ok();
}

#[test]
fn blizzard_game_tooltip_lua_replays_into_secure_environment() {
    let env = WowLuaEnv::new().unwrap();
    let temp_root = std::env::temp_dir().join("wow-sim-game-tooltip-secure-replay-test");
    let addon_dir = temp_root.join("Blizzard_GameTooltip");
    std::fs::create_dir_all(&addon_dir).unwrap();
    std::fs::write(
        addon_dir.join("GameTooltip.lua"),
        r#"GameTooltip_OnLoad = function() return "secure-visible" end"#,
    )
    .unwrap();

    let toc = crate::toc::TocFile::parse(
        &addon_dir,
        r#"
## Title: Blizzard_GameTooltip
## AllowLoad: Both
GameTooltip.lua
"#,
    );

    let result = load_addon_from_toc(&env.loader_env(), &toc).unwrap();
    assert_eq!(result.lua_files, 2);

    let (global_type, secure_result): (String, String) = env
        .eval(
            r#"
            return type(_G.GameTooltip_OnLoad),
                   __secureenv.GameTooltip_OnLoad()
            "#,
        )
        .unwrap();
    assert_eq!(global_type, "function");
    assert_eq!(secure_result, "secure-visible");

    std::fs::remove_dir_all(&temp_root).ok();
}

#[test]
fn blizzard_async_request_lua_replays_into_secure_environment() {
    let env = WowLuaEnv::new().unwrap();
    let temp_root = std::env::temp_dir().join("wow-sim-async-request-secure-replay-test");
    let addon_dir = temp_root.join("Blizzard_AsyncRequest");
    std::fs::create_dir_all(&addon_dir).unwrap();
    std::fs::write(
        addon_dir.join("Blizzard_AsyncRequest.lua"),
        r#"AsyncRequests = { marker = "secure-visible" }"#,
    )
    .unwrap();

    let toc = crate::toc::TocFile::parse(
        &addon_dir,
        r#"
## Title: Blizzard_AsyncRequest
## AllowLoad: Both
Blizzard_AsyncRequest.lua
"#,
    );

    let result = load_addon_from_toc(&env.loader_env(), &toc).unwrap();
    assert_eq!(result.lua_files, 2);

    let (global_marker, secure_marker): (String, String) = env
        .eval(
            r#"
            return _G.AsyncRequests.marker,
                   __secureenv.AsyncRequests.marker
            "#,
        )
        .unwrap();
    assert_eq!(global_marker, "secure-visible");
    assert_eq!(secure_marker, "secure-visible");

    std::fs::remove_dir_all(&temp_root).ok();
}

#[test]
fn secure_addon_load_into_environment_global_overrides_default_secure_environment() {
    let env = WowLuaEnv::new().unwrap();
    let temp_root = std::env::temp_dir().join("wow-sim-secure-global-override-test");
    let addon_dir = temp_root.join("Blizzard_AuraContainerProbe");
    std::fs::create_dir_all(&addon_dir).unwrap();
    std::fs::write(addon_dir.join("Secure.lua"), r#"SecureOnly = "secure""#).unwrap();
    std::fs::write(addon_dir.join("Inbound.lua"), r#"InboundOnly = "global""#).unwrap();
    std::fs::write(
        addon_dir.join("Frame.xml"),
        r#"<Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Frame name="GlobalOverrideXmlFrame"/>
        </Ui>"#,
    )
    .unwrap();

    let toc = crate::toc::TocFile::parse(
        &addon_dir,
        r#"
## Title: Blizzard_AuraContainerProbe
## UseSecureEnvironment: 1
Secure.lua
Frame.xml [LoadIntoEnvironment global]
Inbound.lua [LoadIntoEnvironment global]
"#,
    );

    let result = load_addon_from_toc(&env.loader_env(), &toc).unwrap();
    assert_eq!(result.lua_files, 2);
    assert_eq!(result.xml_files, 1);

    let probe: (String, String, String, String, String, String) = env
        .eval(
            r#"
            return type(rawget(_G, "SecureOnly")),
                   rawget(__secureenv, "SecureOnly"),
                   rawget(_G, "InboundOnly"),
                   type(rawget(__secureenv, "InboundOnly")),
                   type(rawget(_G, "GlobalOverrideXmlFrame")),
                   type(rawget(__secureenv, "GlobalOverrideXmlFrame"))
            "#,
        )
        .unwrap();

    assert_eq!(probe.0, "nil");
    assert_eq!(probe.1, "secure");
    assert_eq!(probe.2, "global");
    assert_eq!(probe.3, "nil");
    assert_eq!(probe.4, "table");
    assert_eq!(probe.5, "nil");

    std::fs::remove_dir_all(&temp_root).ok();
}

#[test]
fn secretunwrap_remains_callable_in_secure_environment_after_global_cleanup() {
    let env = WowLuaEnv::new().unwrap();
    let before_cleanup: (String, String, bool, bool) = env
        .eval(
            r#"
            local value = {}
            return type(_G.secretunwrap),
                   type(__secureenv.secretunwrap),
                   __secureenv.secretunwrap(value) == value,
                   __secureenv.secretunwrap() == nil
            "#,
        )
        .unwrap();
    assert_eq!(
        before_cleanup,
        ("function".into(), "function".into(), true, true)
    );

    env.exec("secretunwrap = nil").unwrap();
    let after_cleanup: (String, String, f64) = env
        .eval(
            r#"
            return type(_G.secretunwrap),
                   type(__secureenv.secretunwrap),
                   __secureenv.secretunwrap(42)
            "#,
        )
        .unwrap();
    assert_eq!(after_cleanup, ("nil".into(), "function".into(), 42.0));
}

#[test]
fn secure_xml_named_frames_bind_into_secure_environment() {
    let env = WowLuaEnv::new().unwrap();
    let temp_root = std::env::temp_dir().join("wow-sim-secure-xml-frame-test");
    let addon_dir = temp_root.join("SecureXmlAddon");
    std::fs::create_dir_all(&addon_dir).unwrap();
    std::fs::write(
        addon_dir.join("Frame.xml"),
        r#"<Ui xmlns="http://www.blizzard.com/wow/ui/">
            <GameTooltip name="SecureXmlTooltip"/>
        </Ui>"#,
    )
    .unwrap();

    let toc = crate::toc::TocFile::parse(
        &addon_dir,
        r#"
## Title: SecureXmlAddon
## UseSecureEnvironment: 1
Frame.xml
"#,
    );

    let result = load_addon_from_toc(&env.loader_env(), &toc).unwrap();
    assert_eq!(result.xml_files, 1);

    let (global_type, secure_same): (String, bool) = env
        .eval(
            r#"
            return type(_G.SecureXmlTooltip),
                   __secureenv.SecureXmlTooltip == _G.SecureXmlTooltip
            "#,
        )
        .unwrap();
    assert_eq!(global_type, "table");
    assert!(secure_same);

    std::fs::remove_dir_all(&temp_root).ok();
}

#[test]
fn blizzard_shared_xml_lua_replays_into_secure_environment() {
    let env = WowLuaEnv::new().unwrap();
    let temp_root = std::env::temp_dir().join("wow-sim-sharedxml-secure-replay-test");
    let addon_dir = temp_root.join("Blizzard_SharedXML");
    std::fs::create_dir_all(&addon_dir).unwrap();
    std::fs::write(
        addon_dir.join("LoopingSoundEffect.lua"),
        r#"CreateLoopingSoundEffectEmitter = function() return "secure-visible" end"#,
    )
    .unwrap();

    let toc = crate::toc::TocFile::parse(
        &addon_dir,
        r#"
## Title: Blizzard_SharedXML
## AllowLoad: Game
LoopingSoundEffect.lua
"#,
    );

    let result = load_addon_from_toc(&env.loader_env(), &toc).unwrap();
    assert_eq!(result.lua_files, 2);

    let (global_type, secure_result): (String, String) = env
        .eval(
            r#"
            return type(_G.CreateLoopingSoundEffectEmitter),
                   __secureenv.CreateLoopingSoundEffectEmitter()
            "#,
        )
        .unwrap();
    assert_eq!(global_type, "function");
    assert_eq!(secure_result, "secure-visible");

    std::fs::remove_dir_all(&temp_root).ok();
}

#[test]
fn blizzard_combat_log_base_lua_replays_into_secure_environment() {
    let env = WowLuaEnv::new().unwrap();
    let temp_root = std::env::temp_dir().join("wow-sim-combat-log-base-secure-replay-test");
    let addon_dir = temp_root.join("Blizzard_CombatLogBase");
    std::fs::create_dir_all(&addon_dir).unwrap();
    std::fs::write(
        addon_dir.join("CombatLogUtil.lua"),
        r#"CombatLogUtil = { marker = "secure-visible" }"#,
    )
    .unwrap();

    let toc = crate::toc::TocFile::parse(
        &addon_dir,
        r#"
## Title: Blizzard_CombatLogBase
## LoadOnDemand: 1
CombatLogUtil.lua
"#,
    );

    let result = load_addon_from_toc(&env.loader_env(), &toc).unwrap();
    assert_eq!(result.lua_files, 2);

    let (global_marker, secure_marker): (String, String) = env
        .eval(
            r#"
            return _G.CombatLogUtil.marker,
                   __secureenv.CombatLogUtil.marker
            "#,
        )
        .unwrap();
    assert_eq!(global_marker, "secure-visible");
    assert_eq!(
        secure_marker, "secure-visible",
        "secure Blizzard_CombatLogProcessor reads CombatLogUtil from this library"
    );

    std::fs::remove_dir_all(&temp_root).ok();
}
