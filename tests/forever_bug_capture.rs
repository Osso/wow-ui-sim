//! Current local !BugGrabber/BugSack sources, not an identified Forever archive.
//! Uses their embedded libraries and cached Blizzard dependency closure unchanged.
#![cfg(feature = "client-wowforever")]

use std::path::Path;

use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;

const MARKER: &str = "FOREVER_BUG_CAPTURE_MARKER";
const PRODUCER: &str = "ForeverBugCaptureProducer";
// Settings initializes BugSack's configuration; Minimap is used by embedded
// LibDBIcon at file load and PLAYER_LOGIN. Both pull their actual TOC deps.
const BLIZZARD_ROOTS: &[&str] = &["Blizzard_Settings_Shared", "Blizzard_Minimap"];

fn load_bug_capture_addons() -> WowLuaEnv {
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path()
        .expect("Forever Blizzard cache must already exist; this test does not download it");
    for root in BLIZZARD_ROOTS {
        assert!(ui.join(root).join(format!("{root}.toc")).is_file());
    }
    let (env, _) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui,
        BLIZZARD_ROOTS,
        &[],
    );
    let addons = Path::new(env!("CARGO_MANIFEST_DIR")).join("Interface/AddOns");
    env.state()
        .borrow_mut()
        .addon_base_paths
        .push(addons.clone());
    for name in ["!BugGrabber", "BugSack"] {
        load_local_addon_and_fire_loaded(&env, &addons, name);
    }
    env.fire_event("PLAYER_LOGIN")
        .expect("normal login must install BugGrabber's handler and BugSack's icon");
    assert!(
        env.state().borrow().lua_errors.is_empty(),
        "unexpected startup errors: {:?}",
        env.state().borrow().lua_errors
    );
    env
}

fn load_local_addon_and_fire_loaded(env: &WowLuaEnv, addons: &Path, name: &str) {
    let toc = addons.join(name).join(format!("{name}.toc"));
    let result = load_addon(&env.loader_env(), &toc)
        .unwrap_or_else(|error| panic!("real local {name} TOC must load: {error}"));
    assert!(result.lua_files > 0, "{name} did not load any Lua files");
    assert!(result.warnings.is_empty(), "{name}: {:?}", result.warnings);
    assert!(
        result.missing_requirements.is_empty(),
        "{name}: {:?}",
        result.missing_requirements
    );
    // load_addon itself does not emit ADDON_LOADED; the executable does this
    // after loading each TOC. Exercise that same event, not addon init methods.
    env.fire_event_with_args("ADDON_LOADED", &[env.lua_string(name)])
        .unwrap_or_else(|error| panic!("{name} ADDON_LOADED dispatch failed: {error}"));
}

#[test]
fn forever_bug_capture_script_errors_reach_store_counter_and_bugsack_display() {
    let env = load_bug_capture_addons();
    env.exec(
        r#"
        assert(BugGrabber:GetSessionId() == 1, 'ADDON_LOADED did not initialize BugGrabber')
        assert(BugGrabber:GetDB() == BugGrabberDB.errors)
        assert(#BugGrabber:GetDB() == 0, 'unexpected pre-producer stored error')
        assert(BugSack.db == BugSackDB, 'ADDON_LOADED did not initialize BugSack')
        local broker = LibStub('LibDataBroker-1.1'):GetDataObjectByName('BugSack')
        assert(broker, 'BugSack display did not register')
        assert(tonumber(broker.text) == 0)
        assert(broker.icon == 'Interface\\AddOns\\BugSack\\Media\\icon')
        local icon = LibStub('LibDBIcon-1.0'):GetMinimapButton('BugSack')
        assert(icon and icon:IsShown(), 'PLAYER_LOGIN did not register the minimap display')
        assert(icon.icon:GetTexture() == broker.icon, 'minimap texture did not initialize')
        ForeverBugCaptureCalls = 0
        ForeverBugCaptureProducer = CreateFrame('Frame', 'ForeverBugCaptureProducer', UIParent)
        ForeverBugCaptureProducer:SetScript('OnUpdate', function()
            ForeverBugCaptureCalls = ForeverBugCaptureCalls + 1
            error('FOREVER_BUG_CAPTURE_MARKER', 0)
        end)
        "#,
    )
    .expect("real lifecycle must initialize both addons before constructing the producer");
    let producer_id = env
        .state()
        .borrow()
        .widgets
        .get_id_by_name(PRODUCER)
        .expect("producer frame must exist");
    assert!(env.has_script_handler(producer_id, "OnUpdate"));

    for counter in 1..=2 {
        env.fire_script_handler(producer_id, "OnUpdate", vec![])
            .expect("normal dispatch records and routes the intentional Lua failure");
        assert_intentional_error_boundary(&env, counter);
        env.exec(&format!(
            r#"
            assert(ForeverBugCaptureCalls == {counter}, 'producer did not run exactly once per dispatch')
            local errors = BugGrabber:GetDB()
            assert(#errors == 1, 'marker must be the only stored error')
            local captured = errors[1]
            local headline = captured.message:match('^([^\n]+)')
            assert(headline:match(': ([^:]+)$') == '{MARKER}', captured.message)
            assert(captured.counter == {counter}, 'repeat did not increment the stored counter')
            assert(captured.session == BugGrabber:GetSessionId())
            local displayed = BugSack:GetErrors(BugGrabber:GetSessionId())
            assert(#displayed == 1 and displayed[1] == captured)
            local broker = LibStub('LibDataBroker-1.1'):GetDataObjectByName('BugSack')
            assert(tonumber(broker.text) == 1, 'normal error callback did not refresh display count')
            assert(broker.icon == 'Interface\\AddOns\\BugSack\\Media\\icon_red',
                'normal error callback did not turn the display red')
            local icon = LibStub('LibDBIcon-1.0'):GetMinimapButton('BugSack')
            assert(icon:IsShown() and icon.icon:GetTexture() == broker.icon,
                'data-broker update did not reach the actual minimap texture')
            "#
        ))
        .expect("BugGrabber storage and public BugSack display must reflect the dispatched error");
        assert_intentional_error_boundary(&env, counter);
    }
}

fn assert_intentional_error_boundary(env: &WowLuaEnv, occurrences: usize) {
    let state = env.state().borrow();
    assert_eq!(
        state.lua_errors.len(),
        occurrences,
        "{:?}",
        state.lua_errors
    );
    assert_eq!(state.lua_error_records.len(), occurrences);
    assert_eq!(state.lua_error_counts.len(), 1);
    assert_eq!(state.lua_error_counts.values().next(), Some(&occurrences));
    for error in &state.lua_errors {
        let headline = error.lines().next().expect("error must have a headline");
        assert!(
            headline.starts_with(&format!("[OnUpdate] frame={PRODUCER} addon=__BuiltIn")),
            "unexpected error boundary: {error}"
        );
        assert_eq!(
            headline.rsplit_once(": ").map(|(_, message)| message),
            Some(MARKER),
            "unexpected error at the producer boundary: {error}"
        );
    }
}
