#![cfg(feature = "retail-12-0-7")]

use std::path::Path;
use wow_ui_sim::lua_api::WowLuaEnv;

// Cumulative gate: Cargo.toml retail-12-0-7 -> retail-12-0-5; 12.1 inherits 12.0.7.
// Proposed pre-12.0.7 cfg/closure-argument paths are retained, not executed here.
// B31/B32 have no older native producer to retain; that historical gap stays open.
const DEPRECATED_FILES: &[&str] = &[
    "Blizzard_Deprecated/Mainline/Deprecated_12_0_7.lua",
    "Blizzard_DeprecatedBattleNet/Deprecated_BattleNet.lua",
    "Blizzard_DeprecatedPartyInfo/Deprecated_PartyInfo.lua",
    "Blizzard_DeprecatedAutoComplete/Deprecated_AutoComplete.lua",
];

fn read_cached_lua(relative_path: &str) -> String {
    let addons = wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )));
    let path = addons.join(relative_path);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

fn load_cached_lua(env: &WowLuaEnv, relative_path: &str) {
    // Same profile-cache read + exec boundary as tests/mists_class_colors.rs.
    let source = read_cached_lua(relative_path);
    env.exec(&source)
        .unwrap_or_else(|error| panic!("execute {relative_path}: {error}"));
}

fn assert_native_absent(env: &WowLuaEnv, symbol: &str) {
    let (namespace, member) = symbol.split_once('.').unwrap_or(("_G", symbol));
    env.exec(&format!(
        "assert(rawget({namespace}, '{member}') == nil, 'native {symbol} survived')"
    ))
    .unwrap();
}

fn assert_minimap_method_absent(env: &WowLuaEnv, method: &str) {
    env.exec(&format!(
        r#"
        local minimap = CreateFrame('Minimap')
        local mt = getmetatable(minimap)
        assert(rawget(mt, '{method}') == nil, 'advertised removed method')
        if type(mt.__index) == 'table' then
            assert(rawget(mt.__index, '{method}') == nil, 'dispatch table retained method')
        end
        assert(minimap['{method}'] == nil, 'actual lookup retained method')
        local ok = pcall(function() minimap['{method}'](minimap, 'fixture-texture') end)
        assert(not ok, 'removed method remained callable')
        minimap:SetZoom(2)
        assert(minimap:GetZoom() == 2, 'unrelated zoom API regressed')
        "#
    ))
    .unwrap();
}

#[test]
fn native_b_n_invite_friend_is_absent_before_deprecated_load() {
    let env = WowLuaEnv::new().unwrap();
    assert_native_absent(&env, "BNInviteFriend");
}

#[test]
fn native_c_click_bindings_get_string_from_modifiers_is_absent_before_deprecated_load() {
    let env = WowLuaEnv::new().unwrap();
    assert_native_absent(&env, "C_ClickBindings.GetStringFromModifiers");
}

#[test]
fn native_c_click_bindings_make_modifiers_is_absent_before_deprecated_load() {
    let env = WowLuaEnv::new().unwrap();
    assert_native_absent(&env, "C_ClickBindings.MakeModifiers");
}

#[test]
fn native_c_spell_get_maw_power_border_atlas_by_spell_i_d_is_absent_before_deprecated_load() {
    let env = WowLuaEnv::new().unwrap();
    assert_native_absent(&env, "C_Spell.GetMawPowerBorderAtlasBySpellID");
}

#[test]
fn native_confirm_ready_check_is_absent_before_deprecated_load() {
    let env = WowLuaEnv::new().unwrap();
    assert_native_absent(&env, "ConfirmReadyCheck");
}

#[test]
fn native_demote_assistant_is_absent_before_deprecated_load() {
    let env = WowLuaEnv::new().unwrap();
    assert_native_absent(&env, "DemoteAssistant");
}

#[test]
fn native_do_ready_check_is_absent_before_deprecated_load() {
    let env = WowLuaEnv::new().unwrap();
    assert_native_absent(&env, "DoReadyCheck");
}

#[test]
fn native_get_merchant_currencies_is_absent_before_deprecated_load() {
    let env = WowLuaEnv::new().unwrap();
    assert_native_absent(&env, "GetMerchantCurrencies");
}

#[test]
fn native_is_g_u_i_d_in_group_is_absent_before_deprecated_load() {
    let env = WowLuaEnv::new().unwrap();
    assert_native_absent(&env, "IsGUIDInGroup");
}

#[test]
fn native_promote_to_assistant_is_absent_before_deprecated_load() {
    let env = WowLuaEnv::new().unwrap();
    assert_native_absent(&env, "PromoteToAssistant");
}

#[test]
fn native_promote_to_leader_is_absent_before_deprecated_load() {
    let env = WowLuaEnv::new().unwrap();
    assert_native_absent(&env, "PromoteToLeader");
}

#[test]
fn native_set_everyone_is_assistant_is_absent_before_deprecated_load() {
    let env = WowLuaEnv::new().unwrap();
    assert_native_absent(&env, "SetEveryoneIsAssistant");
}

#[test]
fn native_uninvite_unit_is_absent_before_deprecated_load() {
    let env = WowLuaEnv::new().unwrap();
    assert_native_absent(&env, "UninviteUnit");
}

#[test]
fn native_get_auto_complete_presence_i_d_is_absent_before_deprecated_load() {
    let env = WowLuaEnv::new().unwrap();
    assert_native_absent(&env, "GetAutoCompletePresenceID");
}

#[test]
fn native_get_auto_complete_results_is_absent_before_deprecated_load() {
    let env = WowLuaEnv::new().unwrap();
    assert_native_absent(&env, "GetAutoCompleteResults");
}

#[test]
fn native_get_auto_complete_realms_is_absent_before_deprecated_load() {
    let env = WowLuaEnv::new().unwrap();
    assert_native_absent(&env, "GetAutoCompleteRealms");
}

#[test]
fn native_is_recognized_name_is_absent_before_deprecated_load() {
    let env = WowLuaEnv::new().unwrap();
    assert_native_absent(&env, "IsRecognizedName");
}

#[test]
fn minimap_set_blip_texture_is_absent_before_deprecated_load() {
    let env = WowLuaEnv::new().unwrap();
    assert_minimap_method_absent(&env, "SetBlipTexture");
}

#[test]
fn minimap_set_corpse_p_o_i_arrow_texture_is_absent_before_deprecated_load() {
    let env = WowLuaEnv::new().unwrap();
    assert_minimap_method_absent(&env, "SetCorpsePOIArrowTexture");
}

#[test]
fn minimap_set_icon_texture_is_absent_before_deprecated_load() {
    let env = WowLuaEnv::new().unwrap();
    assert_minimap_method_absent(&env, "SetIconTexture");
}

#[test]
fn minimap_set_p_o_i_arrow_texture_is_absent_before_deprecated_load() {
    let env = WowLuaEnv::new().unwrap();
    assert_minimap_method_absent(&env, "SetPOIArrowTexture");
}

#[test]
fn minimap_set_player_texture_is_absent_before_deprecated_load() {
    let env = WowLuaEnv::new().unwrap();
    assert_minimap_method_absent(&env, "SetPlayerTexture");
}

#[test]
fn minimap_set_static_p_o_i_arrow_texture_is_absent_before_deprecated_load() {
    let env = WowLuaEnv::new().unwrap();
    assert_minimap_method_absent(&env, "SetStaticPOIArrowTexture");
}

#[test]
fn cached_1207_shim_forwards_modifiers_atlas_and_currency_tuple() {
    let env = WowLuaEnv::new().unwrap();
    for symbol in [
        "C_ClickBindings.MakeModifiers",
        "C_ClickBindings.GetStringFromModifiers",
        "C_Spell.GetMawPowerBorderAtlasBySpellID",
        "GetMerchantCurrencies",
    ] {
        assert_native_absent(&env, symbol);
    }
    env.exec(
        r#"
        SetCVar('loadDeprecationFallbacks', '1')
        MenuUtil = MenuUtil or {}
        function MakeModifiers(...) assert(select('#', ...) == 0); return 5, 'mod-tail' end
        function GetStringFromModifiers(value, ...)
            assert(value == 5 and select('#', ...) == 0)
            return 'SHIFT-ALT', 'string-tail'
        end
        function C_Spell.GetMawPowerRarityInfoBySpellID(value, ...)
            assert(value == 101 and select('#', ...) == 0)
            return 3, 'fixture-rare-border', 'ignored-third'
        end
        function C_MerchantFrame.GetMerchantCurrencies(...) 
            assert(select('#', ...) == 0)
            return {17, 23, 41}
        end
        function exact(...)
            local values = {...}
            return select('#', ...), values
        end
    "#,
    )
    .unwrap();
    load_cached_lua(&env, DEPRECATED_FILES[0]);
    env.exec(
        r#"
        local n, v = exact(C_ClickBindings.MakeModifiers('discarded'))
        assert(n == 2 and v[1] == 5 and v[2] == 'mod-tail')
        n, v = exact(C_ClickBindings.GetStringFromModifiers(5, 'discarded'))
        assert(n == 2 and v[1] == 'SHIFT-ALT' and v[2] == 'string-tail')
        n, v = exact(C_Spell.GetMawPowerBorderAtlasBySpellID(101, 'discarded'))
        assert(n == 1 and v[1] == 'fixture-rare-border')
        C_Spell.GetMawPowerRarityInfoBySpellID = function() return 2, nil end
        n, v = exact(C_Spell.GetMawPowerBorderAtlasBySpellID(101))
        assert(n == 1 and v[1] == nil)
        n, v = exact(GetMerchantCurrencies('discarded'))
        assert(n == 3 and v[1] == 17 and v[2] == 23 and v[3] == 41)
        C_MerchantFrame.GetMerchantCurrencies = function() return {} end
        assert(select('#', GetMerchantCurrencies()) == 0)
    "#,
    )
    .unwrap();
}

#[test]
fn cached_battle_net_shim_preserves_one_input_and_return_tuple() {
    let env = WowLuaEnv::new().unwrap();
    assert_native_absent(&env, "BNInviteFriend");
    env.exec(
        r#"
        SetCVar('loadDeprecationFallbacks', '1')
        function C_BattleNet.InviteFriend(id, ...)
            assert(id == 701 and select('#', ...) == 0)
            return 19, nil, 'invite-tail'
        end
    "#,
    )
    .unwrap();
    load_cached_lua(&env, DEPRECATED_FILES[1]);
    env.exec(
        r#"
        assert(type(rawget(_G, 'BNInviteFriend')) == 'function')
        local function check(...)
            assert(select('#', ...) == 3)
            local first, second, third = ...
            assert(first == 19 and second == nil and third == 'invite-tail')
        end
        check(BNInviteFriend(701, 'discarded'))
    "#,
    )
    .unwrap();
}

#[test]
fn cached_party_shim_preserves_defaults_exact_match_and_return_policy() {
    let env = WowLuaEnv::new().unwrap();
    for symbol in [
        "ConfirmReadyCheck",
        "DemoteAssistant",
        "DoReadyCheck",
        "PromoteToAssistant",
        "PromoteToLeader",
        "SetEveryoneIsAssistant",
        "UninviteUnit",
        "IsGUIDInGroup",
    ] {
        assert_native_absent(&env, symbol);
    }
    env.exec(
        r#"
        SetCVar('loadDeprecationFallbacks', '1')
        partyInputs = {}
        function C_PartyInfo.ConfirmReadyCheck(value, ...)
            assert(select('#', ...) == 0)
            partyInputs.ready = value
            return 'must-be-discarded'
        end
        function C_PartyInfo.DemoteAssistant(name, exact, ...)
            assert(name == 'Ada-Realm' and exact == true and select('#', ...) == 0)
            partyInputs.demoted = name
            return 'must-be-discarded'
        end
        function C_PartyInfo.DoReadyCheck(...)
            assert(select('#', ...) == 0)
            partyInputs.started = true
            return 'must-be-discarded'
        end
        function C_PartyInfo.PromoteToAssistant(name, exact, ...)
            assert(name == 'Ada-Realm' and exact == false and select('#', ...) == 0)
            partyInputs.assistant = name
            return 'must-be-discarded'
        end
        function C_PartyInfo.PromoteToLeader(name, exact, ...)
            assert(name == 'Bea-Realm' and exact == true and select('#', ...) == 0)
            partyInputs.leader = name
            return 'must-be-discarded'
        end
        function C_PartyInfo.SetEveryoneIsAssistant(value, ...)
            assert(value == false and select('#', ...) == 0)
            partyInputs.everyone = value
            return 7, nil, 'assistant-tail'
        end
        function C_PartyInfo.UninviteUnit(unit, reason, exact, ...)
            assert(unit == 'party1' and reason == 'fixture reason' and exact == true)
            assert(select('#', ...) == 0)
            partyInputs.removed = unit
            return 'must-be-discarded'
        end
        function C_PartyInfo.IsGUIDInGroup(guid, category, ...)
            assert(guid == 'Player-1-ABC' and category == 2 and select('#', ...) == 0)
            return true, 'guid-tail'
        end
    "#,
    )
    .unwrap();
    load_cached_lua(&env, DEPRECATED_FILES[2]);
    env.exec(
        r#"
        assert(select('#', ConfirmReadyCheck(nil)) == 0 and partyInputs.ready == false)
        assert(select('#', ConfirmReadyCheck(true)) == 0 and partyInputs.ready == true)
        assert(select('#', DemoteAssistant('Ada-Realm', true, 'discarded')) == 0)
        assert(partyInputs.demoted == 'Ada-Realm')
        assert(select('#', DoReadyCheck('discarded')) == 0 and partyInputs.started)
        assert(select('#', PromoteToAssistant('Ada-Realm', false, 'discarded')) == 0)
        assert(partyInputs.assistant == 'Ada-Realm')
        assert(select('#', PromoteToLeader('Bea-Realm', true, 'discarded')) == 0)
        assert(partyInputs.leader == 'Bea-Realm')
        local function checkAssistant(...)
            assert(select('#', ...) == 3)
            local a, b, c = ...
            assert(a == 7 and b == nil and c == 'assistant-tail')
        end
        checkAssistant(SetEveryoneIsAssistant(false, 'discarded'))
        assert(partyInputs.everyone == false)
        assert(select('#', UninviteUnit('party1', 'fixture reason', true, 'discarded')) == 0)
        assert(partyInputs.removed == 'party1')
        local function checkGuid(...)
            assert(select('#', ...) == 2)
            local a, b = ...
            assert(a == true and b == 'guid-tail')
        end
        checkGuid(IsGUIDInGroup('Player-1-ABC', 2, 'discarded'))
    "#,
    )
    .unwrap();
}

#[test]
fn cached_auto_complete_shim_coerces_full_match_and_preserves_results() {
    let env = WowLuaEnv::new().unwrap();
    for symbol in [
        "GetAutoCompletePresenceID",
        "GetAutoCompleteResults",
        "GetAutoCompleteRealms",
        "IsRecognizedName",
    ] {
        assert_native_absent(&env, symbol);
    }
    env.exec(r#"
        SetCVar('loadDeprecationFallbacks', '1')
        function C_AutoComplete.GetAutoCompletePresenceID(name, ...)
            assert(name == 'Ada-Realm' and select('#', ...) == 0)
            return 701, 'presence-tail'
        end
        function C_AutoComplete.GetAutoCompleteResults(name, count, cursor, full, include, exclude, ...)
            assert(name == 'Ad' and count == 3 and cursor == 2 and type(full) == 'boolean')
            assert(include == 5 and exclude == 8 and select('#', ...) == 0)
            return {'Ada-Realm', 'Adam-Realm'}, full, 'results-tail'
        end
        function C_AutoComplete.GetAutoCompleteRealms(...)
            assert(select('#', ...) == 0)
            return {'Realm-A', 'Realm-B'}, 'realms-tail'
        end
        function C_AutoComplete.IsRecognizedName(name, include, exclude, ...)
            assert(name == 'Ada-Realm' and include == 5 and exclude == 8)
            assert(select('#', ...) == 0)
            return true, 'recognized-tail'
        end
    "#).unwrap();
    load_cached_lua(&env, DEPRECATED_FILES[3]);
    env.exec(
        r#"
        local function checkPresence(...)
            assert(select('#', ...) == 2)
            local a, b = ...
            assert(a == 701 and b == 'presence-tail')
        end
        checkPresence(GetAutoCompletePresenceID('Ada-Realm', 'discarded'))
        local function checkResults(expected, ...)
            assert(select('#', ...) == 3)
            local values, full, tail = ...
            assert(#values == 2 and values[1] == 'Ada-Realm' and values[2] == 'Adam-Realm')
            assert(full == expected and tail == 'results-tail')
        end
        checkResults(true, GetAutoCompleteResults('Ad', 3, 2, 0, 5, 8, 'discarded'))
        checkResults(false, GetAutoCompleteResults('Ad', 3, 2, nil, 5, 8))
        checkResults(false, GetAutoCompleteResults('Ad', 3, 2, false, 5, 8))
        local function checkRealms(...)
            assert(select('#', ...) == 2)
            local values, tail = ...
            assert(#values == 2 and values[1] == 'Realm-A' and values[2] == 'Realm-B')
            assert(tail == 'realms-tail')
        end
        checkRealms(GetAutoCompleteRealms('discarded'))
        local function checkRecognized(...)
            assert(select('#', ...) == 2)
            local a, b = ...
            assert(a == true and b == 'recognized-tail')
        end
        checkRecognized(IsRecognizedName('Ada-Realm', 5, 8, 'discarded'))
    "#,
    )
    .unwrap();
}

#[test]
fn disabled_cached_shims_do_not_resurrect_any_removed_native_symbol() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("SetCVar('loadDeprecationFallbacks', '0')")
        .unwrap();
    for relative_path in DEPRECATED_FILES {
        load_cached_lua(&env, relative_path);
    }
    for symbol in REMOVED_NATIVE_SYMBOLS {
        assert_native_absent(&env, symbol);
    }
    env.apply_post_load_workarounds();
    for symbol in REMOVED_NATIVE_SYMBOLS {
        assert_native_absent(&env, symbol);
    }
}

#[test]
fn minimap_removals_survive_cached_deprecation_load() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("SetCVar('loadDeprecationFallbacks', '1'); MenuUtil = MenuUtil or {}")
        .unwrap();
    for relative_path in DEPRECATED_FILES {
        load_cached_lua(&env, relative_path);
    }
    env.apply_post_load_workarounds();
    for method in REMOVED_MINIMAP_METHODS {
        assert_minimap_method_absent(&env, method);
    }
}

#[test]
fn minimap_removals_survive_full_cached_game_ui_startup() {
    use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
    use wow_ui_sim::screen::ScreenKind;
    use wow_ui_sim::startup::fire_startup_events_for_screen;

    let env = WowLuaEnv::new().unwrap();
    let addons = wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )));
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    env.state().borrow_mut().addon_base_paths = vec![addons.clone()];
    wow_ui_sim::xml::register_intrinsic_templates();
    for (name, toc) in discover_blizzard_addons_for_screen(&addons, ScreenKind::Game) {
        load_addon(&env.loader_env(), &toc).unwrap_or_else(|error| panic!("load {name}: {error}"));
    }
    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    for method in REMOVED_MINIMAP_METHODS {
        assert_minimap_method_absent(&env, method);
    }
}

const REMOVED_NATIVE_SYMBOLS: &[&str] = &[
    "BNInviteFriend",
    "C_ClickBindings.GetStringFromModifiers",
    "C_ClickBindings.MakeModifiers",
    "C_Spell.GetMawPowerBorderAtlasBySpellID",
    "ConfirmReadyCheck",
    "DemoteAssistant",
    "DoReadyCheck",
    "GetMerchantCurrencies",
    "IsGUIDInGroup",
    "PromoteToAssistant",
    "PromoteToLeader",
    "SetEveryoneIsAssistant",
    "UninviteUnit",
    "GetAutoCompletePresenceID",
    "GetAutoCompleteResults",
    "GetAutoCompleteRealms",
    "IsRecognizedName",
];

const REMOVED_MINIMAP_METHODS: &[&str] = &[
    "SetBlipTexture",
    "SetCorpsePOIArrowTexture",
    "SetIconTexture",
    "SetPOIArrowTexture",
    "SetPlayerTexture",
    "SetStaticPOIArrowTexture",
];
