//! Main register_globals function and core utilities.
//!
//! This module contains the main registration function that orchestrates
//! registering all WoW API globals, plus core Lua utilities like print,
//! type, ipairs, pairs, getmetatable, and setmetatable.

use super::super::SimState;
use super::super::env::WowLuaAppData;
use super::super::hot_literals::HotLiteralRegistry;
use super::super::methods::publish_frame_ref_cache_alias;
use crate::lua_api::methods::borrow_state;
use rilua::LuaApiMut;
use std::cell::RefCell;
use std::rc::Rc;

/// Pre-intern the Track 1 whitelist via `intern_string_static` and stash
/// the resulting [`HotLiteralHandles`] on the Lua app-data so later
/// consumers can fetch an already-interned handle without re-hashing.
/// Must run at the top of the bootstrap pass, before any other registrar
/// touches the string arena.
fn prewarm_hot_literal_registry(lua: &mut rilua::Lua) {
    let handles = HotLiteralRegistry::install(lua.state_mut());
    let app_data = lua
        .state_mut()
        .app_data_mut::<WowLuaAppData>()
        .expect("WowLuaEnv rilua app_data should always exist");
    app_data.hot_literals = Some(handles);
}

/// Register the live rilua global surface.
///
/// This native registrar owns the split-module wiring so `env_init` can use
/// one entry point for the current global surface again.
pub fn register_globals(lua: &mut rilua::Lua, _state: Rc<RefCell<SimState>>) -> crate::Result<()> {
    // Bootstrap allocates monotonically — frames/globals/metatables/bytecode
    // stay live through startup. Pause the collector so the mark phase
    // doesn't walk the growing `_G` and frame trees on every threshold
    // trigger. Caller must match with full_gc() + gc_restart() once
    // bootstrap (and, in the binary, addon loading) completes.
    lua.gc_stop();
    register_bootstrap_globals(lua)?;
    register_frame_globals(lua)?;
    register_tail_globals(lua)?;
    // Keep the old debug/test alias available. The cache itself must remain
    // traversable because addon code can attach Lua functions/tables to
    // frame refs via Mixin or direct field assignment.
    publish_frame_ref_cache_alias(lua.state_mut());
    Ok(())
}

fn register_bootstrap_globals(lua: &mut rilua::Lua) -> crate::Result<()> {
    prewarm_hot_literal_registry(lua);
    LuaApiMut::register_function(lua, "GetTime", get_time)?;
    LuaApiMut::register_function(lua, "GetTimePreciseSec", get_time)?;
    LuaApiMut::register_function(lua, "GetServerTime", get_server_time)?;
    LuaApiMut::register_function(lua, "GetRaidDifficultyID", get_raid_difficulty_id)?;
    LuaApiMut::register_function(
        lua,
        "GetLegacyRaidDifficultyID",
        get_legacy_raid_difficulty_id,
    )?;
    LuaApiMut::register_function(lua, "GetTickTime", get_tick_time)?;
    super::strings::register_all_ui_strings(lua)?;
    super::security::register_all(lua)?;
    super::keybindings::register_all(lua)?;
    super::permanent_shims::register_all(lua.state_mut());
    super::action_bar_api::register_all(lua)?;
    LuaApiMut::register_function(lua, "UpdateUIParentPosition", update_ui_parent_position)?;
    // Must run after stubs so the fixture aura data overrides the
    // stub_nil registrations for C_UnitAuras.GetAuraSlots & friends.
    super::auras::register_all(lua.state_mut());
    #[cfg(feature = "aura-instance-enumeration")]
    crate::c_api::c_unit_auras::register(lua.state_mut())?;
    Ok(())
}

fn register_frame_globals(lua: &mut rilua::Lua) -> crate::Result<()> {
    register_frame_foundation_globals(lua)?;
    register_frame_context_globals(lua)?;
    Ok(())
}

fn register_frame_foundation_globals(lua: &mut rilua::Lua) -> crate::Result<()> {
    super::create_frame::register_all(lua)?;
    super::font_strings_collection::register_all(lua)?;
    super::utility_system_spell::register_all(lua)?;
    super::real::frame_level_helpers::register_all(lua)?;
    #[cfg(feature = "retail-12-0-0")]
    {
        LuaApiMut::register_function(
            lua,
            "RegisterEventCallback",
            super::real::event_callbacks::register_event_callback,
        )?;
        LuaApiMut::register_function(
            lua,
            "UnregisterEventCallback",
            super::real::event_callbacks::unregister_event_callback,
        )?;
        LuaApiMut::register_function(
            lua,
            "RegisterUnitEventCallback",
            super::real::event_callbacks::register_unit_event_callback,
        )?;
        LuaApiMut::register_function(
            lua,
            "UnregisterUnitEventCallback",
            super::real::event_callbacks::unregister_unit_event_callback,
        )?;
    }
    #[cfg(any(feature = "retail-12-1-5", feature = "client-wowforever"))]
    super::real::math_extensions::register_all(lua)?;
    #[cfg(any(feature = "retail-12-1-5", feature = "client-wowforever"))]
    super::real::table_extensions::register_all(lua)?;
    #[cfg(feature = "retail-12-1-0")]
    super::real::table_freeze::register_all(lua)?;
    #[cfg(feature = "retail-12-1-5")]
    super::real::string_extensions::register_all(lua)?;
    super::real::net_stats::register_all(lua)?;
    super::store_frame::register_all(lua)?;
    Ok(())
}

fn register_frame_context_globals(lua: &mut rilua::Lua) -> crate::Result<()> {
    super::unit_probes::register_all(lua)?;
    super::real::unit_interaction::register_all(lua)?;
    super::real::preferred_interact::register_all(lua)?;
    super::unit_misc::register_all(lua)?;
    #[cfg(feature = "aura-containers")]
    super::real::unit_relationships::register_all(lua)?;
    super::inventory_slot::register_all(lua)?;
    super::zone_text::register_all(lua)?;
    super::real::modifier_keys::register_all(lua)?;
    super::real::guild_logo::register_all(lua)?;
    super::guild_control::register_all(lua)?;
    super::targeting_verbs::register_all(lua)?;
    super::game_rules::register_all(lua)?;
    super::guild_info::register_all(lua)?;
    super::housing::register_all(lua)?;
    super::transmog_outfit_info::register_all(lua)?;
    super::pet_battles::register_all(lua)?;
    super::photo_sharing::register_all(lua)?;
    super::wowlabs::register_all(lua)?;
    super::adventure_map::register_all(lua)?;
    Ok(())
}

fn register_tail_globals(lua: &mut rilua::Lua) -> crate::Result<()> {
    register_core_surfaces(lua)?;
    register_action_verbs(lua)?;
    register_state_probes(lua)?;
    register_compat_and_admin(lua)?;
    super::super::timer_layout::register_all(lua)?;
    Ok(())
}

fn register_core_surfaces(lua: &mut rilua::Lua) -> crate::Result<()> {
    super::lfg_list::register_all(lua)?;
    super::real::locale_info::register_all(lua)?;
    super::missing_surface::register_all(lua)?;
    super::quest_surface::register_all(lua)?;
    super::missing_surface::register_quest_log_overrides(lua)?;
    super::lua_duration_object::register_lua_duration_object(lua)?;
    Ok(())
}

fn register_action_verbs(lua: &mut rilua::Lua) -> crate::Result<()> {
    register_action_and_item_verbs(lua)?;
    register_social_and_world_verbs(lua)?;
    register_ui_action_verbs(lua)?;
    Ok(())
}

fn register_action_and_item_verbs(lua: &mut rilua::Lua) -> crate::Result<()> {
    super::combat_verbs::register_all(lua)?;
    super::inventory_verbs::register_all(lua)?;
    super::mail_verbs::register_all(lua)?;
    Ok(())
}

fn register_social_and_world_verbs(lua: &mut rilua::Lua) -> crate::Result<()> {
    super::group_verbs::register_all(lua)?;
    super::guild_verbs::register_all(lua)?;
    super::quest_verbs::register_all(lua)?;
    super::close_frames::register_all(lua)?;
    super::battlefield_verbs::register_all(lua)?;
    super::channel_verbs::register_all(lua)?;
    super::spell_macro_verbs::register_all(lua)?;
    super::chat_window_verbs::register_all(lua)?;
    super::offer_verbs::register_all(lua)?;
    super::trade_verbs::register_all(lua)?;
    super::movement_verbs::register_all(lua)?;
    Ok(())
}

fn register_ui_action_verbs(lua: &mut rilua::Lua) -> crate::Result<()> {
    super::panel_toggle_verbs::register_all(lua)?;
    super::voice_chat_verbs::register_all(lua)?;
    super::message_verbs::register_all(lua)?;
    super::chat_frame_util::register_all(lua)?;
    super::set_cvar_verb::register_all(lua)?;
    super::ui_visibility::register_all(lua)?;
    super::session_exit::register_all(lua)?;
    Ok(())
}

fn register_state_probes(lua: &mut rilua::Lua) -> crate::Result<()> {
    register_actor_state_probes(lua)?;
    register_action_state_probes(lua)?;
    register_progression_state_probes(lua)?;
    register_world_state_probes(lua)?;
    register_social_state_probes(lua)?;
    Ok(())
}

fn register_actor_state_probes(lua: &mut rilua::Lua) -> crate::Result<()> {
    super::real::combat_probes::register_all(lua)?;
    super::spell_state_probes::register_all(lua)?;
    super::pvp_probes::register_all(lua)?;
    super::real::player_probes::register_all(lua)?;
    super::real::player_identity::register_all(lua)?;
    super::unit_stats::register_all(lua)?;
    super::real::combat_stats::register_all(lua)?;
    super::real::pet_stats::register_all(lua)?;
    super::cooldown_probes::register_all(lua)?;
    Ok(())
}

fn register_action_state_probes(lua: &mut rilua::Lua) -> crate::Result<()> {
    super::inventory_probes::register_all(lua)?;
    super::inventory_counts::register_all(lua)?;
    super::real::action_highlights::register_all(lua)?;
    super::real::shapeshift::register_all(lua)?;
    super::real::pet_bar::register_all(lua)?;
    super::real::vehicle_possession::register_all(lua)?;
    super::real::glyph_state::register_all(lua)?;
    super::real::action_bar_state::register_all(lua)?;
    Ok(())
}

fn register_progression_state_probes(lua: &mut rilua::Lua) -> crate::Result<()> {
    super::real::xp_honor_rest::register_all(lua)?;
    super::real::spell_tabs::register_all(lua)?;
    super::battlefield_lfg_probes::register_all(lua)?;
    super::real::loot_method::register_all(lua)?;
    Ok(())
}

fn register_world_state_probes(lua: &mut rilua::Lua) -> crate::Result<()> {
    super::real::mouse_probes::register_all(lua)?;
    super::movement_probes::register_all(lua)?;
    super::faction_probes::register_all(lua)?;
    super::real::gossip_probes::register_all(lua)?;
    super::torghast::register_all(lua)?;
    super::instance_info::register_all(lua)?;
    super::state_backed_queries::register_all(lua)?;
    super::archaeology::register_all(lua)?;
    Ok(())
}

fn register_social_state_probes(lua: &mut rilua::Lua) -> crate::Result<()> {
    super::social_probes::register_all(lua)?;
    super::guild_probes::register_all(lua)?;
    super::mail_probes::register_all(lua)?;
    super::real::voice_chat_probes::register_all(lua)?;
    Ok(())
}

fn register_compat_and_admin(lua: &mut rilua::Lua) -> crate::Result<()> {
    super::compat_overrides::register_all(lua)?;
    super::bank_storage_verbs::register_guild_tabard_files(lua)?;
    #[cfg(feature = "client-mists")]
    super::auction_verbs::register_all(lua)?;
    #[cfg(feature = "client-mists")]
    super::bank_storage_verbs::register_all(lua)?;
    super::debug_api::register_all(lua)?;
    super::admin::register_all(lua)?;
    Ok(())
}

fn update_ui_parent_position(_state: &mut rilua::vm::state::LuaState) -> rilua::LuaResult<u32> {
    Ok(0)
}

fn get_time(state: &mut rilua::vm::state::LuaState) -> rilua::LuaResult<u32> {
    let elapsed = {
        let sim = borrow_state(state)?;
        sim.start_time.elapsed().as_secs_f64()
    };
    state.push(rilua::Val::Num(elapsed));
    Ok(1)
}

fn get_server_time(state: &mut rilua::vm::state::LuaState) -> rilua::LuaResult<u32> {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs_f64())
        .unwrap_or(0.0);
    state.push(rilua::Val::Num(seconds));
    Ok(1)
}

fn get_raid_difficulty_id(state: &mut rilua::vm::state::LuaState) -> rilua::LuaResult<u32> {
    let difficulty_id = {
        let sim = borrow_state(state)?;
        match sim.world.instance_difficulty {
            id if id > 0 => id,
            _ => 14,
        }
    };
    state.push(rilua::Val::Num(f64::from(difficulty_id)));
    Ok(1)
}

fn get_legacy_raid_difficulty_id(state: &mut rilua::vm::state::LuaState) -> rilua::LuaResult<u32> {
    state.push(rilua::Val::Num(3.0));
    Ok(1)
}

fn get_tick_time(state: &mut rilua::vm::state::LuaState) -> rilua::LuaResult<u32> {
    state.push(rilua::Val::Num(1.0 / 60.0));
    Ok(1)
}

#[cfg(test)]
mod tests {
    use super::register_globals;
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn register_globals_is_idempotent_and_keeps_core_surface_live() {
        let env = WowLuaEnv::new().expect("failed to create Lua environment");
        {
            let mut lua = env.rilua_mut();
            register_globals(&mut lua, env.state().clone()).expect("failed to re-register globals");
        }

        let result: (bool, bool, bool, bool) = env
            .eval(
                r#"
                return type(CreateFrame) == "function",
                       type(strsplit) == "function",
                       type(C_Timer) == "table",
                       type(OKAY) == "string"
                "#,
            )
            .expect("failed to probe globals");

        assert!(result.0, "CreateFrame should remain registered");
        assert!(result.1, "strsplit should remain registered");
        assert!(result.2, "C_Timer should remain registered");
        assert!(result.3, "UI strings should remain registered");
    }

    #[test]
    #[cfg(all(feature = "profile-retail", feature = "retail-12-1-0"))]
    fn live_retail_12_1_global_strings_match_probe() {
        const PRESENT: &[(&str, &str)] = &[
            (
                "BLIZZARD_STORE_VAS_ERROR_BOOST_THROTTLE",
                "Maximum number of character boosts reached for the day. Please try again tomorrow.",
            ),
            (
                "HOUSING_FIXTURE_ATTACHED_DECOR_CONFIRMATION",
                "This change will affect Decor currently attached to your house.\n\nWould you prefer to put the attached Decor into storage or leave it detached?",
            ),
            (
                "HOUSING_FIXTURE_ATTACHED_DECOR_CONFIRMATION_DETACH",
                "Detach",
            ),
            ("HOUSING_FIXTURE_ATTACHED_DECOR_CONFIRMATION_STORE", "Store"),
            (
                "HUD_EDIT_MODE_PERSONAL_RESOURCE_DISPLAY_DISABLED_TOOLTIP",
                "Displays health, power, and class resources. The Personal Resource Display is currently disabled. Enable it in: Combat>Personal Resource Display",
            ),
            (
                "HUD_EDIT_MODE_SETTING_DAMAGE_METER_VISIBILITY_IN_GROUP",
                "In Group",
            ),
            (
                "HUD_EDIT_MODE_SETTING_ENCOUNTER_EVENTS_TOOLTIPS_NONE",
                "None",
            ),
            (
                "HUD_EDIT_MODE_SETTING_PERSONAL_RESOURCE_DISPLAY_BAR_WIDTH",
                "Bar Width",
            ),
            (
                "HUD_EDIT_MODE_SETTING_PERSONAL_RESOURCE_DISPLAY_HEALTH_BAR_HEIGHT",
                "Health Bar Height",
            ),
            (
                "HUD_EDIT_MODE_SETTING_PERSONAL_RESOURCE_DISPLAY_HIDE_ALT_POWER_BAR",
                "Hide Alternate Power Bar",
            ),
            (
                "HUD_EDIT_MODE_SETTING_PERSONAL_RESOURCE_DISPLAY_HIDE_CLASS_INFO",
                "Hide Class Resources",
            ),
            (
                "HUD_EDIT_MODE_SETTING_PERSONAL_RESOURCE_DISPLAY_HIDE_CLASS_INFO_ON_PLAYER_FRAME",
                "Hide Class Resources On Player Frame",
            ),
            (
                "HUD_EDIT_MODE_SETTING_PERSONAL_RESOURCE_DISPLAY_HIDE_HEALTH_BAR",
                "Hide Health Bar",
            ),
            (
                "HUD_EDIT_MODE_SETTING_PERSONAL_RESOURCE_DISPLAY_HIDE_POWER_BAR",
                "Hide Power Bar",
            ),
            (
                "HUD_EDIT_MODE_SETTING_PERSONAL_RESOURCE_DISPLAY_OPACITY",
                "Opacity",
            ),
            (
                "HUD_EDIT_MODE_SETTING_PERSONAL_RESOURCE_DISPLAY_PADDING",
                "Padding",
            ),
            (
                "HUD_EDIT_MODE_SETTING_PERSONAL_RESOURCE_DISPLAY_POWER_BAR_HEIGHT",
                "Power Bar Height",
            ),
            (
                "HUD_EDIT_MODE_SETTING_PERSONAL_RESOURCE_DISPLAY_SHOW_BAR_TEXT",
                "Show Bar Text",
            ),
            (
                "HUD_EDIT_MODE_SETTING_PERSONAL_RESOURCE_DISPLAY_SIZE",
                "Size",
            ),
            (
                "HUD_EDIT_MODE_SETTING_PERSONAL_RESOURCE_DISPLAY_VISIBLE_SETTING",
                "Visibility",
            ),
            (
                "HUD_EDIT_MODE_SETTING_PERSONAL_RESOURCE_DISPLAY_VISIBLE_SETTING_ALWAYS",
                "Always",
            ),
            (
                "HUD_EDIT_MODE_SETTING_PERSONAL_RESOURCE_DISPLAY_VISIBLE_SETTING_HIDDEN",
                "Hidden",
            ),
            (
                "HUD_EDIT_MODE_SETTING_PERSONAL_RESOURCE_DISPLAY_VISIBLE_SETTING_IN_COMBAT",
                "In Combat",
            ),
            ("HUD_EDIT_MODE_SETTING_STATUS_TACKING_BAR_SIZE", "Size"),
            (
                "HUD_EDIT_MODE_SETTING_UNIT_FRAME_BIGDEFENSIVE_AURA_ICON_SIZE",
                "Big Defensive Size",
            ),
            ("HUD_EDIT_MODE_TOTEM_ACTION_BAR_LABEL", "Totem Bar"),
            ("MIDNIGHT_LANDING_PAGE_TITLE", "Omnium Folio"),
            (
                "MIDNIGHT_LANDING_PAGE_TOOLTIP",
                "Contains important information on Midnight features and powers.",
            ),
            (
                "PLAYER_DIFFICULTY_MYTHIC_FLEXIBLE",
                "Mythic (Flexible Raiding)",
            ),
            ("QUEST_HUB_TOOLTIP_TRAVEL_HEADER", "Travel"),
            ("WORLD_TIER_HEROIC", "Heroic"),
            ("WORLD_TIER_MYTHIC", "Mythic"),
        ];
        const ABSENT: &[&str] = &[
            "BLOCK_REDUCED",
            "CONFIRM_TALENT_WIPE",
            "EDIT_MODE_OVERRIDE_LAYOUTS",
            "EDIT_MODE_OVERRIDE_LAYOUT_MAP",
            "LOCALE_koKR",
            "LOCALE_ruRU",
            "LOCALE_zhCN",
            "LOCALE_zhTW",
            "NEWBIE_TOOLTIP_COMMUNITIESTAB",
            "OK",
            "SLASH_TEXTTOSPEECH_HELP_GUILD_ANNOUNCE",
            "WOWHACK_ACCOUNT_STORE_TITLE",
        ];

        let env = WowLuaEnv::new().expect("failed to create Lua environment");
        let read_global = |name: &str| -> (String, String) {
            env.eval(&format!(
                r#"
                local value = rawget(_G, {name:?})
                return type(value), type(value) == "string" and value or tostring(value)
                "#
            ))
            .unwrap_or_else(|error| panic!("failed to read global {name}: {error}"))
        };

        for &(name, expected) in PRESENT {
            let (actual_type, actual_value) = read_global(name);
            assert_eq!(
                actual_type, "string",
                "global {name}: expected string {expected:?}, actual type {actual_type:?} value {actual_value:?}"
            );
            assert_eq!(
                actual_value, expected,
                "global {name}: expected {expected:?}, actual {actual_value:?}"
            );
        }

        for &name in ABSENT {
            let (actual_type, actual_value) = read_global(name);
            assert_eq!(
                actual_type, "nil",
                "global {name}: expected nil, actual type {actual_type:?} value {actual_value:?}"
            );
        }
    }

    #[test]
    fn get_server_time_returns_unix_seconds_for_addon_date_calls() {
        let env = WowLuaEnv::new().expect("failed to create Lua environment");

        let result: (String, bool) = env
            .eval(
                r#"
                local now = GetServerTime()
                return type(now), date("*t", now).year >= 2024
                "#,
            )
            .expect("GetServerTime should be date-compatible");

        assert_eq!(result.0, "number");
        assert!(result.1, "GetServerTime should return current Unix seconds");
    }
}
