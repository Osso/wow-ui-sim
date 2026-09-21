use super::permanent_shims::{c_fog_of_war, c_nameplate};
use super::{
    c_allied_races, c_ardenweald_gardening, c_arrow_callout_manager, c_artifact_relic_forge_ui,
    c_artifact_ui, c_azerite_empowered_item, c_azerite_essence, c_azerite_item, c_barber_shop,
    c_cursor, c_major_factions, c_map, c_map_exploration_info, c_paper_doll_info,
    c_player_interaction_manager, c_spell, c_spell_diminish, c_widget,
};

use rilua::LuaResult;
use rilua::vm::state::LuaState;

pub(crate) fn register_spell_and_widget_tables(state: &mut LuaState) -> LuaResult<()> {
    #[cfg(feature = "client-wowforever")]
    super::c_input_interface_style::register(state)?;
    #[cfg(feature = "client-wowforever")]
    super::c_gamepad_ui::register(state)?;
    #[cfg(feature = "client-wowforever")]
    super::c_edit_mode::register(state)?;
    #[cfg(feature = "client-wowforever")]
    super::gamepad_action_bar_constants::register(state);
    #[cfg(feature = "client-wowforever")]
    super::forever_finite_constants::register(state);
    c_spell::register_c_spell_surface(state)?;
    c_spell_diminish::register_c_spell_diminish_surface(state)?;
    c_widget::register_c_widget_surface(state)
}

pub(crate) fn register_item_power_tables(state: &mut LuaState) -> LuaResult<()> {
    super::weapon_enchants::register(state)?;
    c_paper_doll_info::register_c_paper_doll_info_surface(state)?;
    c_artifact_ui::register_c_artifact_ui_surface(state)?;
    c_artifact_relic_forge_ui::register_c_artifact_relic_forge_ui_surface(state)?;
    c_azerite_item::register_c_azerite_item_surface(state)?;
    c_azerite_essence::register_c_azerite_essence_surface(state)?;
    c_azerite_empowered_item::register_c_azerite_empowered_item_surface(state)
}

pub(crate) fn register_character_progression_tables(state: &mut LuaState) -> LuaResult<()> {
    c_barber_shop::register_c_barber_shop_surface(state)?;
    c_cursor::register_c_cursor_surface(state)?;
    c_major_factions::register_c_major_factions_surface(state)?;
    c_allied_races::register_c_allied_races_surface(state)
}

pub(crate) fn register_interaction_tables(state: &mut LuaState) -> LuaResult<()> {
    #[cfg(feature = "client-wowforever")]
    super::c_combat_log::register_publication(state)?;
    #[cfg(feature = "retail-12-0-0")]
    super::c_combat_audio_alert::register(state)?;
    #[cfg(feature = "retail-12-0-0")]
    super::c_combat_text::register(state)?;
    super::c_neighborhood_initiative::register(state)?;
    #[cfg(feature = "retail-12-0-0")]
    super::c_transmog_sets::register(state)?;
    #[cfg(feature = "retail-12-0-0")]
    super::c_transmog_outfit_info::register(state)?;
    c_ardenweald_gardening::register_c_ardenweald_gardening_surface(state)?;
    c_arrow_callout_manager::register_c_arrow_callout_manager_surface(state)?;
    c_player_interaction_manager::register_c_player_interaction_manager_surface(state)
}

pub(crate) fn register_map_prefix_tables(state: &mut LuaState) -> LuaResult<()> {
    c_map::register_c_map_surface(state)
}

pub(crate) fn register_map_environment_tables(state: &mut LuaState) -> LuaResult<()> {
    c_fog_of_war::register_fog_of_war_surface(state)?;
    c_map_exploration_info::register_c_map_exploration_info_surface(state)
}

pub(crate) fn register_nameplate_tables(state: &mut LuaState) -> LuaResult<()> {
    c_nameplate::register_c_nameplate(state)
}
