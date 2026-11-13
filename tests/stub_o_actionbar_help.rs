//! O startup-navigation retirement: original actionbar ownership and Help timer.
//! Loaded-owner cases are preservation; bootstrap absence is retirement RED.
//! Help observer measures scheduling only, never native ticket response parity.
#![cfg(all(feature = "optional-tests", feature = "client-retail"))]

use crate::common::blizzard_addon_harness::{
    load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::lua_api::WowLuaEnv;

fn original_owner(root: &str) -> WowLuaEnv {
    let cache = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let env = new_blizzard_addon_env(&cache);
    wow_ui_sim::xml::register_intrinsic_templates();
    // Existing original dependency fixture, not install_lua_harness_stubs.
    crate::common::panel_fixtures::load_panel_addons(&env);
    let loaded = load_blizzard_addon_closure_into_env(&env, &cache, &[root], &[]);
    assert!(loaded.iter().any(|name| name == root), "{root}: {loaded:?}");
    println!("O actionbar/help selected closure: {loaded:?}");
    env.apply_post_load_workarounds();
    for error in &env.state().borrow().lua_errors {
        eprintln!("O actionbar/help original owner error: {error:?}");
    }
    env
}

#[test]
fn actionbars_are_not_fabricated_before_xml_owner() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        for _, name in ipairs({'MainActionBar', 'MultiBarBottomLeft', 'MultiBarBottomRight', 'MultiBarRight', 'MultiBarLeft'}) do
            assert(rawget(_G, name) == nil, 'pre-owner actionbar placeholder remains: ' .. name)
        end
    "#).expect("retire five bootstrap frame placeholders");
}

#[test]
fn original_actionbar_xml_owns_buttons_and_dirty_cache_invalidation() {
    let env = original_owner("Blizzard_ActionBar");
    env.exec(r#"
        local function source(fn, suffix)
            local info = debug.getinfo(fn, 'S')
            local path = info.source:gsub('\\', '/')
            assert(path:find(suffix, 1, true), path)
        end
        source(EditModeSystemMixin.MarkAllSettingsDirty,
            'Blizzard_EditMode/Shared/EditModeSystemTemplates.lua')
        for _, name in ipairs({'MainActionBar', 'MultiBarBottomLeft', 'MultiBarBottomRight', 'MultiBarRight', 'MultiBarLeft'}) do
            local frame = assert(_G[name], 'missing selected XML frame ' .. name)
            source(frame.ActionBar_OnLoad, 'Blizzard_ActionBar/Shared/ActionBar.lua')
            source(frame.MarkAllSettingsDirty, 'Blizzard_EditMode/Shared/EditModeSystemTemplates.lua')
            assert(frame.MarkAllSettingsDirty == EditModeSystemMixin.MarkAllSettingsDirty,
                name .. ': no-op instance injection masks original mixin')
            assert(type(frame.actionButtons) == 'table' and #frame.actionButtons > 0,
                name .. ': original ActionBar_OnLoad did not produce buttons')
            assert(#frame.actionButtons == frame.numButtons, name .. ': incomplete template button population')
            for _, button in ipairs(frame.actionButtons) do
                assert(button.bar == frame and button.container:GetParent() == frame,
                    name .. ': original button/container linkage missing')
            end
            local registered = false
            for _, system in ipairs(EditModeManagerFrame.registeredSystemFrames) do
                if system == frame then registered = true end
            end
            assert(registered, name .. ': original OnSystemLoad registration missing')
            -- Explicit Lua cache state, NOT invented systemInfo/DB2/native fields.
            -- Even an empty valid cache must be invalidated to nil; a no-op fails.
            -- This is method-contract preservation, not full layout-model proof.
            frame.settingMap = {}
            frame:MarkAllSettingsDirty()
            assert(frame.settingMap == nil, name .. ': original dirty operation did not invalidate cache')
        end
    "#).expect("actual actionbar template/mixin ownership and cache invalidation");
}

#[test]
fn help_timer_helper_is_not_published_before_original_owner() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("assert(rawget(_G, 'HelpOpenWebTicketButton_OnUpdate') == nil, 'bootstrap Help OnUpdate no-op remains')")
        .unwrap();
}

#[test]
fn original_help_xml_onupdate_schedules_ticket_queries_preservation() {
    let env = original_owner("Blizzard_HelpFrame");
    env.exec(r#"
        local path = debug.getinfo(HelpOpenWebTicketButton_OnUpdate, 'S').source:gsub('\\', '/')
        assert(path:find('Blizzard_HelpFrame/HelpFrame.lua', 1, true), path)
        local button = assert(HelpOpenWebTicketButton, 'actual Help XML button missing')
        local update = assert(button:GetScript('OnUpdate'), 'actual XML OnUpdate binding missing')
        assert(type(button.refreshTime) == 'number', 'original XML OnLoad timer missing')
        assert(type(GMTICKET_CHECK_INTERVAL) == 'number' and GMTICKET_CHECK_INTERVAL > 0)
        local originalQuery = assert(GetWebTicket)
        local queries = 0
        -- Native boundary observer, not a replacement Lua owner or response stub.
        -- Delegate unchanged arguments/results. Current native query is separately
        -- unmodeled; this test claims scheduling only, no ticket/event semantics.
        GetWebTicket = function(...)
            queries = queries + 1
            return originalQuery(...)
        end
        local ok, err = pcall(function()
            button.refreshTime = 2 -- explicit scheduler state, no native ticket data
            update(button, 0.5)
            assert(button.refreshTime == 1.5 and queries == 0, 'timer must count down without premature query')
            update(button, 1.5)
            assert(button.refreshTime == GMTICKET_CHECK_INTERVAL and queries == 1,
                'due timer must reset and request once')
            update(button, 0.5)
            assert(button.refreshTime == GMTICKET_CHECK_INTERVAL - 0.5 and queries == 1)
            button.refreshTime = nil
            update(button, GMTICKET_CHECK_INTERVAL * 2)
            assert(queries == 1 and button.refreshTime == nil, 'disabled timer must not poll')
        end)
        GetWebTicket = originalQuery
        assert(ok, err)
    "#).expect("original Help timer scheduling with delegating query observer only");
}

#[test]
fn settings_panel_has_no_invented_dirty_method_after_bootstrap_or_cleanup() {
    let env = WowLuaEnv::new().unwrap();
    for restore in [false, true] {
        if restore {
            env.restore_post_cleanup_globals();
        }
        env.exec("assert(SettingsPanel); assert(rawget(SettingsPanel, 'MarkAllSettingsDirty') == nil, 'synthetic Settings dirty method remains')").unwrap();
    }
}
