#![cfg(feature = "retail-12-0-5")]

use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon, load_addon_from_toc};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::AuraInfo;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn retail_cache() -> PathBuf {
    dirs::home_dir()
        .expect("home directory for retail vendor fixtures")
        .join(".cache/wow-ui-sim/blizzard-ui/retail/AddOns")
}

fn load_game_ui(include_aura_header: bool, spec_index: i32) -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create vendor-path environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![retail_cache()];
        state.player.class_index = 2;
        state.player.active_spec_index = spec_index;
        let spec_id = match spec_index {
            1 => 65,
            2 => 66,
            3 => 70,
            _ => panic!("invalid Paladin fixture specialization"),
        };
        state.talents.switch_to_spec(spec_id);
    }
    wow_ui_sim::xml::register_intrinsic_templates();
    for (name, toc) in discover_blizzard_addons_for_screen(&retail_cache(), ScreenKind::Game) {
        if include_aura_header && name == "Blizzard_RestrictedAddOnEnvironment" {
            load_restricted_environment_with_aura_header(&env, &toc);
        } else {
            load_addon(&env.loader_env(), &toc)
                .unwrap_or_else(|error| panic!("load cached {name}: {error}"));
        }
    }
    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

fn load_restricted_environment_with_aura_header(env: &WowLuaEnv, toc: &std::path::Path) {
    // Extend the real TOC before its first load. Reloading the same folder
    // after startup is a silent already-loaded no-op, regardless of Title.
    let source = std::fs::read_to_string(toc).expect("read restricted environment TOC");
    let source = format!("{source}\nSecureAuraHeader.lua\nSecureAuraHeader.xml\n");
    let probe_toc = TocFile::parse(toc.parent().expect("TOC directory"), &source);
    let loaded = load_addon_from_toc(&env.loader_env(), &probe_toc)
        .expect("load actual secure aura files with restricted environment");
    assert!(loaded.lua_files > 0 && loaded.xml_files > 0);
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
}

fn aura(id: i32, duration: f64, expiration_time: f64) -> AuraInfo {
    AuraInfo {
        name: format!("Header aura {id}"),
        spell_id: 98000 + id,
        icon: 134973,
        duration,
        expiration_time,
        applications: 1,
        source_unit: "player".into(),
        is_helpful: true,
        is_raid: false,
        is_nameplate_only: false,
        is_stealable: false,
        can_apply_aura: true,
        is_from_player_or_player_pet: true,
        dispel_type: None,
        aura_instance_id: id,
    }
}

fn load_aura_header() -> WowLuaEnv {
    let env = load_game_ui(true, 3);
    // Isolate existing whole-UI startup errors, never errors from the probe.
    env.state().borrow_mut().lua_errors.clear();
    env.exec(
        r#"
        assert(type(SecureAuraHeader_Update) == 'function', 'cached aura Lua must load')
        HeaderProbe = CreateFrame('Frame', 'HeaderProbe', UIParent, 'SecureAuraHeaderTemplate')
        assert(type(HeaderProbe:GetScript('OnShow')) == 'function', 'cached aura XML must load')
        assert(not HeaderProbe:IsShown(), 'header template starts hidden')
        HeaderProbe:SetAttribute('unit', 'player')
        HeaderProbe:SetAttribute('filter', 'HELPFUL')
        HeaderProbe:SetAttribute('sortMethod', 'TIME')
        HeaderProbe:SetAttribute('sortDirection', '+')
        HeaderProbe:SetAttribute('separateOwn', 0)
        HeaderProbe:SetAttribute('template', 'SecureAuraButtonTemplate')
        HeaderProbe:SetAttribute('point', 'TOPLEFT')
        HeaderProbe:SetAttribute('xOffset', 20)
        HeaderProbe:SetAttribute('minWidth', 100)
        HeaderProbe:SetAttribute('minHeight', 20)
        HeaderProbe:SetPoint('TOPLEFT', UIParent, 'TOPLEFT', 0, 0)
        function HeaderOrder()
            local result = {}
            local position = 1
            while true do
                local child = HeaderProbe:GetAttribute('child' .. position)
                if not child then break end
                if child:IsShown() then
                    local index = child:GetAttribute('index')
                    assert(child:GetID() == index)
                    local record = C_UnitAuras.GetAuraDataByIndex('player', index, child:GetAttribute('filter'))
                    assert(record, 'child must reference a live aura')
                    result[#result + 1] = record.auraInstanceID
                end
                position = position + 1
            end
            return table.concat(result, ',')
        end
        "#,
    )
    .expect("construct real cached header template");
    env
}

fn header_order(env: &WowLuaEnv) -> String {
    env.eval("return HeaderOrder()")
        .expect("read observable header child order")
}

fn assert_no_lua_errors(env: &WowLuaEnv) {
    let errors = env.state().borrow().lua_errors.clone();
    assert!(errors.is_empty(), "vendor-path Lua errors: {errors:?}");
}

#[test]
fn cached_secure_aura_header_sorts_finite_before_permanent() {
    let env = load_aura_header();
    let now: f64 = env.eval("return GetTime()").expect("current aura clock");
    let permanent = aura(301, 0.0, 0.0);
    let long = aura(302, 3600.0, now + 3600.0);
    let short = aura(303, 30.0, now + 30.0);
    let second_permanent = aura(304, 0.0, 0.0);
    env.state().borrow_mut().player.buffs = vec![
        permanent.clone(),
        long.clone(),
        short.clone(),
        second_permanent.clone(),
    ];
    env.exec("HeaderProbe:Show(); assert(HeaderProbe:IsVisible()); assert(HeaderProbe:IsEventRegistered('UNIT_AURA'))")
        .expect("real template OnShow updates aura children");
    let first_order = header_order(&env);
    assert!(
        first_order == "303,302,301,304" || first_order == "303,302,304,301",
        "TIME ascending must place both permanents last: {first_order}"
    );

    env.state().borrow_mut().player.buffs = vec![short, second_permanent, long, permanent];
    env.fire_event_with_args("UNIT_AURA", &[env.lua_string("player")])
        .expect("real template OnEvent updates a shuffled aura stream");
    let shuffled_order = header_order(&env);
    assert!(
        shuffled_order == "303,302,301,304" || shuffled_order == "303,302,304,301",
        "ordering must not depend on producer insertion order: {shuffled_order}"
    );
    assert_no_lua_errors(&env);
}

#[test]
fn cached_secure_aura_header_reorders_and_hides_stale_children() {
    let env = load_aura_header();
    let now: f64 = env.eval("return GetTime()").expect("current aura clock");
    env.state().borrow_mut().player.buffs = vec![
        aura(311, 0.0, 0.0),
        aura(312, 3600.0, now + 3600.0),
        aura(313, 30.0, now + 30.0),
    ];
    env.exec("HeaderProbe:Show(); assert(HeaderProbe:IsVisible()); assert(HeaderProbe:IsEventRegistered('UNIT_AURA'))")
        .expect("populate real header children");
    assert_eq!(header_order(&env), "313,312,311");
    env.exec("OriginalFirstChild = HeaderProbe:GetAttribute('child1')")
        .expect("retain original child identity");

    {
        let mut state = env.state().borrow_mut();
        state.player.buffs[2].duration = 0.0;
        state.player.buffs[2].expiration_time = 0.0;
    }
    env.fire_event_with_args("UNIT_AURA", &[env.lua_string("player")])
        .expect("finite-to-permanent update");
    let order = header_order(&env);
    assert!(order == "312,311,313" || order == "312,313,311", "{order}");
    env.exec("assert(HeaderProbe:GetAttribute('child1') == OriginalFirstChild)")
        .expect("vendor header reuses its positional child");

    env.state().borrow_mut().player.buffs.remove(0);
    env.fire_event_with_args("UNIT_AURA", &[env.lua_string("player")])
        .expect("aura removal update");
    assert_eq!(header_order(&env), "312,313");
    env.exec("assert(not HeaderProbe:GetAttribute('child3'):IsShown())")
        .expect("removed aura leaves no stale shown child");
    env.state()
        .borrow_mut()
        .player
        .buffs
        .push(aura(311, 20.0, now + 20.0));
    env.fire_event_with_args("UNIT_AURA", &[env.lua_string("player")])
        .expect("aura readdition update");
    assert_eq!(header_order(&env), "311,312,313");
    assert_no_lua_errors(&env);
}

