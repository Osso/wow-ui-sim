//! `C_TransmogCollection` probe surface backed by `WorldState.transmog_appearances`
//! and `WorldState.collected_transmogs`.

use super::ensure_namespace;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_set, table_set_num,
    val_to_string,
};
use crate::lua_api::state_types::{TransmogAppearance, character_world::WorldState};
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const DEFAULT_CLASS_FILTER: i32 = 2;
type StaticLuaFn = fn(&mut LuaState) -> LuaResult<u32>;
type StaticFunction = (&'static str, StaticLuaFn);

const APPEARANCE_QUERY_FUNCTIONS: &[StaticFunction] = &[
    ("GetAppearanceSources", get_appearance_sources),
    (
        "GetValidAppearanceSourcesForClass",
        get_valid_appearance_sources_for_class,
    ),
    ("GetSourceInfo", get_source_info),
    ("PlayerHasTransmog", player_has_transmog),
    (
        "PlayerHasTransmogByItemInfo",
        player_has_transmog_by_item_info,
    ),
    (
        "PlayerHasTransmogItemModifiedAppearance",
        player_has_transmog_item_modified_appearance,
    ),
    ("GetNumTransmogSources", get_num_transmog_sources),
    ("GetAllAppearanceSources", get_all_appearance_sources),
];

pub(super) fn register_transmog_collection_surface(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_TransmogCollection")?;
    register_transmog_collection_appearance_queries(state, table_ref)?;
    register_transmog_collection_category_queries(state, table_ref)?;
    register_transmog_collection_flags(state, table_ref)?;
    register_transmog_collection_outfits(state, table_ref)?;
    #[cfg(feature = "retail-12-0-0")]
    crate::c_api::c_transmog_collection::register(state, table_ref)?;
    Ok(())
}

fn register_transmog_collection_appearance_queries(
    state: &mut LuaState,
    table_ref: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    register_static_functions(state, table_ref, APPEARANCE_QUERY_FUNCTIONS)
}

fn register_transmog_collection_category_queries(
    state: &mut LuaState,
    table_ref: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetCategoryAppearances",
        get_category_appearances,
    )?;
    table_set_rust_fn_static(state, table_ref, "GetCategoryInfo", get_category_info)?;
    table_set_rust_fn_static(state, table_ref, "PlayerKnowsSource", player_knows_source)?;
    Ok(())
}

fn register_transmog_collection_flags(
    state: &mut LuaState,
    table_ref: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    register_transmog_filter_defaults(state, table_ref)?;
    register_transmog_source_filters(state, table_ref)?;
    register_transmog_filter_counts(state, table_ref)?;
    register_static_functions(
        state,
        table_ref,
        &[
            ("IsSearchInProgress", return_false),
            ("IsAppearanceHiddenVisual", is_appearance_hidden_visual),
            (
                "GetShowMissingSourceInItemTooltips",
                get_show_missing_source_in_item_tooltips,
            ),
        ],
    )
}

fn register_transmog_filter_defaults(
    state: &mut LuaState,
    table_ref: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    register_static_functions(
        state,
        table_ref,
        &[
            ("GetClassFilter", get_class_filter),
            ("SetClassFilter", set_class_filter),
            ("SetSearchAndFilterCategory", set_search_and_filter_category),
            ("GetCollectedShown", get_collected_shown),
            ("GetUncollectedShown", get_uncollected_shown),
            ("SetCollectedShown", set_collected_shown),
            ("SetUncollectedShown", set_uncollected_shown),
            ("GetAllFactionsShown", get_all_factions_shown),
            ("GetAllRacesShown", get_all_races_shown),
            ("SetAllFactionsShown", set_all_factions_shown),
            ("SetAllRacesShown", set_all_races_shown),
            ("SetAllSourceTypeFilters", set_all_source_type_filters),
            ("SetSourceTypeFilter", set_source_type_filter),
            ("IsUsingDefaultFilters", is_using_default_filters),
            ("SetDefaultFilters", set_default_filters),
            ("SetSearch", set_search),
            ("ClearSearch", clear_search),
            ("EndSearch", end_search),
            ("SearchSize", search_size),
            ("SearchProgress", search_progress),
            ("IsSearchDBLoading", return_false),
        ],
    )
}

fn register_transmog_source_filters(
    state: &mut LuaState,
    table_ref: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    register_static_functions(
        state,
        table_ref,
        &[
            ("IsValidTransmogSource", is_valid_transmog_source),
            ("IsSourceTypeFilterChecked", is_source_type_filter_checked),
        ],
    )
}

fn register_transmog_filter_counts(
    state: &mut LuaState,
    table_ref: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    register_static_functions(
        state,
        table_ref,
        &[
            (
                "GetFilteredCategoryCollectedCount",
                get_filtered_category_collected_count,
            ),
            ("GetFilteredCategoryTotal", get_filtered_category_total),
        ],
    )
}

fn register_transmog_collection_outfits(
    state: &mut LuaState,
    table_ref: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(state, table_ref, "GetIllusions", get_illusions)?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetAppearanceCameraID",
        get_appearance_camera_id,
    )?;
    Ok(())
}

fn register_static_functions(
    state: &mut LuaState,
    table_ref: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
    functions: &[StaticFunction],
) -> LuaResult<()> {
    for (name, function) in functions {
        table_set_rust_fn_static(state, table_ref, name, *function)?;
    }
    Ok(())
}

fn get_appearance_sources(state: &mut LuaState) -> LuaResult<u32> {
    let visual_id = i32::from_stack(state, 1)?;
    push_appearance_sources(state, visual_id)
}

fn get_valid_appearance_sources_for_class(state: &mut LuaState) -> LuaResult<u32> {
    let visual_id = i32::from_stack(state, 1)?;
    let _class_id = i32::from_stack(state, 2)?;
    push_appearance_sources(state, visual_id)
}

fn push_appearance_sources(state: &mut LuaState, visual_id: i32) -> LuaResult<u32> {
    let appearances = transmog_appearances(state, Some(visual_id), None);
    let array = create_table(state);
    let Val::Table(array_ref) = array else {
        state.push(Val::Nil);
        return Ok(1);
    };

    for (index, appearance) in appearances.iter().enumerate() {
        let row = appearance_row(state, appearance, None);
        table_set_num(state, array_ref, (index + 1) as f64, row);
    }

    state.push(array);
    Ok(1)
}

fn get_source_info(state: &mut LuaState) -> LuaResult<u32> {
    let source_id = i32::from_stack(state, 1)?;
    let appearance = transmog_appearance(state, source_id).or_else(|| {
        if has_collected_transmog(state, source_id) {
            Some(TransmogAppearance {
                source_id,
                visual_id: 0,
                category_id: 0,
                item_id: source_id,
                is_collected: true,
                source_type: 0,
                item_mod_id: 0,
            })
        } else {
            None
        }
    });

    let Some(appearance) = appearance else {
        state.push(Val::Nil);
        return Ok(1);
    };

    let row = appearance_row(state, &appearance, None);
    state.push(row);
    Ok(1)
}

fn player_has_transmog(state: &mut LuaState) -> LuaResult<u32> {
    let source_id = i32::from_stack(state, 1)?;
    state.push(Val::Bool(has_collected_transmog(state, source_id)));
    Ok(1)
}

fn player_has_transmog_by_item_info(state: &mut LuaState) -> LuaResult<u32> {
    let Some(item_id) = parse_item_id_from_val(state, stack_val(state, 1)) else {
        state.push(Val::Bool(false));
        return Ok(1);
    };
    state.push(Val::Bool(has_collected_transmog(state, item_id as i32)));
    Ok(1)
}

fn player_has_transmog_item_modified_appearance(state: &mut LuaState) -> LuaResult<u32> {
    let appearance_id = i32::from_stack(state, 1)?;
    state.push(Val::Bool(has_collected_transmog(state, appearance_id)));
    Ok(1)
}

fn get_num_transmog_sources(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(7.0));
    Ok(1)
}

fn get_all_appearance_sources(state: &mut LuaState) -> LuaResult<u32> {
    let array = empty_array(state);
    state.push(array);
    Ok(1)
}

fn get_category_appearances(state: &mut LuaState) -> LuaResult<u32> {
    let category_id = i32::from_stack(state, 1)?;
    let appearances = filtered_transmog_appearances(state, None, Some(category_id));
    let array = create_table(state);
    let Val::Table(array_ref) = array else {
        state.push(Val::Nil);
        return Ok(1);
    };

    for (index, appearance) in appearances.iter().enumerate() {
        let row = appearance_row(state, appearance, Some((index + 1) as i32));
        table_set_num(state, array_ref, (index + 1) as f64, row);
    }

    state.push(array);
    Ok(1)
}

fn get_category_info(state: &mut LuaState) -> LuaResult<u32> {
    let category_id = i32::from_stack(state, 1)?;
    let (name, is_weapon, can_enchant, can_main_hand, can_off_hand) = category_info(category_id);
    let category_name = create_string(state, name);
    state.push(category_name);
    state.push(Val::Bool(is_weapon));
    state.push(Val::Bool(can_enchant));
    state.push(Val::Bool(can_main_hand));
    state.push(Val::Bool(can_off_hand));
    Ok(5)
}

fn player_knows_source(state: &mut LuaState) -> LuaResult<u32> {
    let _source_id = i32::from_stack(state, 1)?;
    state.push(Val::Bool(false));
    Ok(1)
}

fn is_appearance_hidden_visual(state: &mut LuaState) -> LuaResult<u32> {
    let _visual_id = i32::from_stack(state, 1)?;
    state.push(Val::Bool(false));
    Ok(1)
}

fn is_source_type_filter_checked(state: &mut LuaState) -> LuaResult<u32> {
    let source_type = i32::from_stack(state, 1).unwrap_or_default();
    let checked = borrow_state(state)
        .map(|sim| source_type_filter_enabled(&sim.world, source_type))
        .unwrap_or(false);
    state.push(Val::Bool(checked));
    Ok(1)
}

fn source_type_filter_enabled(world: &WorldState, source_type: i32) -> bool {
    world.transmog_source_type_filters.contains(&source_type)
}

fn is_valid_transmog_source(state: &mut LuaState) -> LuaResult<u32> {
    let source_type = i32::from_stack(state, 1).unwrap_or_default();
    state.push(Val::Bool((1..=7).contains(&source_type)));
    Ok(1)
}

fn get_filtered_category_collected_count(state: &mut LuaState) -> LuaResult<u32> {
    let category_id = i32::from_stack(state, 1)?;
    let count = filtered_transmog_appearances(state, None, Some(category_id))
        .iter()
        .filter(|appearance| appearance.is_collected)
        .count();
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn get_filtered_category_total(state: &mut LuaState) -> LuaResult<u32> {
    let category_id = i32::from_stack(state, 1)?;
    let count = filtered_transmog_appearances(state, None, Some(category_id)).len();
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn get_class_filter(state: &mut LuaState) -> LuaResult<u32> {
    let class_filter = borrow_state(state)
        .map(|sim| sim.world.transmog_class_filter)
        .unwrap_or(DEFAULT_CLASS_FILTER);
    state.push(Val::Num(class_filter as f64));
    Ok(1)
}

fn set_class_filter(state: &mut LuaState) -> LuaResult<u32> {
    let class_filter = i32::from_stack(state, 1).unwrap_or(DEFAULT_CLASS_FILTER);
    borrow_state_mut(state)?.world.transmog_class_filter = class_filter;
    Ok(0)
}

fn get_collected_shown(state: &mut LuaState) -> LuaResult<u32> {
    let shown = borrow_state(state)
        .map(|sim| sim.world.transmog_collected_shown)
        .unwrap_or(true);
    state.push(Val::Bool(shown));
    Ok(1)
}

fn get_uncollected_shown(state: &mut LuaState) -> LuaResult<u32> {
    let shown = borrow_state(state)
        .map(|sim| sim.world.transmog_uncollected_shown)
        .unwrap_or(true);
    state.push(Val::Bool(shown));
    Ok(1)
}

fn set_collected_shown(state: &mut LuaState) -> LuaResult<u32> {
    let shown = bool::from_stack(state, 1).unwrap_or(true);
    borrow_state_mut(state)?.world.transmog_collected_shown = shown;
    Ok(0)
}

fn set_uncollected_shown(state: &mut LuaState) -> LuaResult<u32> {
    let shown = bool::from_stack(state, 1).unwrap_or(true);
    borrow_state_mut(state)?.world.transmog_uncollected_shown = shown;
    Ok(0)
}

fn get_all_factions_shown(state: &mut LuaState) -> LuaResult<u32> {
    let shown = borrow_state(state)
        .map(|sim| sim.world.transmog_all_factions_shown)
        .unwrap_or(false);
    state.push(Val::Bool(shown));
    Ok(1)
}

fn get_all_races_shown(state: &mut LuaState) -> LuaResult<u32> {
    let shown = borrow_state(state)
        .map(|sim| sim.world.transmog_all_races_shown)
        .unwrap_or(false);
    state.push(Val::Bool(shown));
    Ok(1)
}

fn set_all_factions_shown(state: &mut LuaState) -> LuaResult<u32> {
    let shown = bool::from_stack(state, 1).unwrap_or(false);
    borrow_state_mut(state)?.world.transmog_all_factions_shown = shown;
    Ok(0)
}

fn set_all_races_shown(state: &mut LuaState) -> LuaResult<u32> {
    let shown = bool::from_stack(state, 1).unwrap_or(false);
    borrow_state_mut(state)?.world.transmog_all_races_shown = shown;
    Ok(0)
}

fn set_all_source_type_filters(state: &mut LuaState) -> LuaResult<u32> {
    let checked = bool::from_stack(state, 1).unwrap_or(true);
    let mut sim = borrow_state_mut(state)?;
    sim.world.transmog_source_type_filters.clear();
    if checked {
        sim.world.transmog_source_type_filters.extend(1..=7);
    }
    Ok(0)
}

fn set_source_type_filter(state: &mut LuaState) -> LuaResult<u32> {
    let source_type = i32::from_stack(state, 1).unwrap_or_default();
    let checked = bool::from_stack(state, 2).unwrap_or(true);
    let mut sim = borrow_state_mut(state)?;
    if checked {
        sim.world.transmog_source_type_filters.insert(source_type);
    } else {
        sim.world.transmog_source_type_filters.remove(&source_type);
    }
    Ok(0)
}

fn is_using_default_filters(state: &mut LuaState) -> LuaResult<u32> {
    let is_default = borrow_state(state)
        .map(|sim| {
            sim.world.transmog_collected_shown
                && sim.world.transmog_uncollected_shown
                && !sim.world.transmog_all_factions_shown
                && !sim.world.transmog_all_races_shown
                && sim.world.transmog_class_filter == DEFAULT_CLASS_FILTER
                && (1..=7).all(|source| sim.world.transmog_source_type_filters.contains(&source))
                && sim.world.transmog_source_type_filters.len() == 7
        })
        .unwrap_or(false);
    state.push(Val::Bool(is_default));
    Ok(1)
}

fn set_default_filters(state: &mut LuaState) -> LuaResult<u32> {
    let mut sim = borrow_state_mut(state)?;
    sim.world.transmog_collected_shown = true;
    sim.world.transmog_uncollected_shown = true;
    sim.world.transmog_all_factions_shown = false;
    sim.world.transmog_all_races_shown = false;
    sim.world.transmog_class_filter = DEFAULT_CLASS_FILTER;
    sim.world.transmog_source_type_filters.clear();
    sim.world.transmog_source_type_filters.extend(1..=7);
    Ok(0)
}

fn set_search_and_filter_category(state: &mut LuaState) -> LuaResult<u32> {
    let _category_id = i32::from_stack(state, 1).unwrap_or_default();
    Ok(0)
}

fn set_search(state: &mut LuaState) -> LuaResult<u32> {
    let search_type = i32::from_stack(state, 1).unwrap_or_default();
    let text = String::from_stack(state, 2).unwrap_or_default();
    borrow_state_mut(state)?
        .world
        .transmog_search_text
        .insert(search_type, text);
    state.push(Val::Bool(true));
    Ok(1)
}

fn clear_search(state: &mut LuaState) -> LuaResult<u32> {
    let search_type = i32::from_stack(state, 1).unwrap_or_default();
    borrow_state_mut(state)?
        .world
        .transmog_search_text
        .remove(&search_type);
    Ok(0)
}

fn end_search(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.world.transmog_search_text.clear();
    Ok(0)
}

fn search_size(state: &mut LuaState) -> LuaResult<u32> {
    let _search_type = i32::from_stack(state, 1).unwrap_or_default();
    let count = filtered_transmog_appearances(state, None, None).len();
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn search_progress(state: &mut LuaState) -> LuaResult<u32> {
    search_size(state)
}

fn return_false(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    Ok(1)
}

fn get_show_missing_source_in_item_tooltips(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(true));
    Ok(1)
}

fn get_illusions(state: &mut LuaState) -> LuaResult<u32> {
    let array = empty_array(state);
    state.push(array);
    Ok(1)
}

fn get_appearance_camera_id(state: &mut LuaState) -> LuaResult<u32> {
    let _appearance_id = i32::from_stack(state, 1)?;
    state.push(Val::Num(0.0));
    Ok(1)
}

fn appearance_row(
    state: &mut LuaState,
    appearance: &TransmogAppearance,
    ui_order: Option<i32>,
) -> Val {
    let row = create_table(state);
    set_number_field(state, row, "sourceID", appearance.source_id);
    set_number_field(state, row, "visualID", appearance.visual_id);
    set_number_field(state, row, "categoryID", appearance.category_id);
    set_number_field(state, row, "itemID", appearance.item_id);
    set_bool_field(state, row, "isCollected", appearance.is_collected);
    set_bool_field(state, row, "playerCanCollect", true);
    set_bool_field(state, row, "canDisplayOnPlayer", true);
    set_bool_field(state, row, "isUsable", true);
    set_bool_field(state, row, "isValidSourceForPlayer", true);
    set_bool_field(state, row, "isHideVisual", false);
    set_bool_field(state, row, "isFavorite", false);
    set_number_field(state, row, "sourceType", appearance.source_type);
    set_number_field(state, row, "itemModID", appearance.item_mod_id);
    set_number_field(state, row, "quality", 4);
    set_appearance_name_field(state, row, appearance.item_id);
    if let Some(ui_order) = ui_order {
        set_number_field(state, row, "uiOrder", ui_order);
    }
    row
}

fn set_appearance_name_field(state: &mut LuaState, row: Val, item_id: i32) {
    let name = create_string(state, &format!("Item {}", item_id));
    table_set(state, row, "name", name);
}

fn set_number_field(state: &mut LuaState, table: Val, key: &str, value: i32) {
    table_set(state, table, key, Val::Num(value as f64));
}

fn set_bool_field(state: &mut LuaState, table: Val, key: &str, value: bool) {
    table_set(state, table, key, Val::Bool(value));
}

fn empty_array(state: &mut LuaState) -> Val {
    create_table(state)
}

fn has_collected_transmog(state: &LuaState, id: i32) -> bool {
    borrow_state(state)
        .map(|sim| sim.world.collected_transmogs.contains(&id))
        .unwrap_or(false)
}

fn transmog_appearance(state: &LuaState, source_id: i32) -> Option<TransmogAppearance> {
    borrow_state(state)
        .ok()?
        .world
        .transmog_appearances
        .iter()
        .find(|appearance| appearance.source_id == source_id)
        .cloned()
}

fn filtered_transmog_appearances(
    state: &LuaState,
    visual_id: Option<i32>,
    category_id: Option<i32>,
) -> Vec<TransmogAppearance> {
    collect_transmog_appearances(state, visual_id, category_id, true)
}

fn transmog_appearances(
    state: &LuaState,
    visual_id: Option<i32>,
    category_id: Option<i32>,
) -> Vec<TransmogAppearance> {
    collect_transmog_appearances(state, visual_id, category_id, false)
}

fn collect_transmog_appearances(
    state: &LuaState,
    visual_id: Option<i32>,
    category_id: Option<i32>,
    use_active_filters: bool,
) -> Vec<TransmogAppearance> {
    borrow_state(state)
        .ok()
        .map(|sim| {
            matching_transmog_appearances(&sim.world, visual_id, category_id)
                .filter(|appearance| {
                    !use_active_filters || active_filters_allow(&sim.world, appearance)
                })
                .cloned()
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn matching_transmog_appearances(
    world: &WorldState,
    visual_id: Option<i32>,
    category_id: Option<i32>,
) -> impl Iterator<Item = &TransmogAppearance> {
    world
        .transmog_appearances
        .iter()
        .filter(move |appearance| visual_id.is_none_or(|id| appearance.visual_id == id))
        .filter(move |appearance| category_id.is_none_or(|id| appearance.category_id == id))
}

fn active_filters_allow(world: &WorldState, appearance: &TransmogAppearance) -> bool {
    collection_filter_allows(world, appearance)
        && source_filter_allows(world, appearance)
        && search_filter_allows(world, appearance)
}

fn collection_filter_allows(world: &WorldState, appearance: &TransmogAppearance) -> bool {
    if appearance.is_collected {
        world.transmog_collected_shown
    } else {
        world.transmog_uncollected_shown
    }
}

fn source_filter_allows(world: &WorldState, appearance: &TransmogAppearance) -> bool {
    world
        .transmog_source_type_filters
        .contains(&appearance.source_type)
}

fn search_filter_allows(world: &WorldState, appearance: &TransmogAppearance) -> bool {
    let Some(search_text) = active_transmog_search_text(world) else {
        return true;
    };
    let needle = search_text.to_ascii_lowercase();
    [
        appearance.source_id.to_string(),
        appearance.visual_id.to_string(),
        appearance.item_id.to_string(),
        appearance.category_id.to_string(),
    ]
    .iter()
    .any(|value| value.to_ascii_lowercase().contains(&needle))
}

fn active_transmog_search_text(world: &WorldState) -> Option<&str> {
    world
        .transmog_search_text
        .values()
        .find(|text| !text.trim().is_empty())
        .map(String::as_str)
}

fn category_info(category_id: i32) -> (&'static str, bool, bool, bool, bool) {
    match category_id {
        1 => ("Head", false, false, false, false),
        2 => ("Shoulder", false, false, false, false),
        3 => ("Back", false, false, false, false),
        4 => ("Chest", false, false, false, false),
        5 => ("Shirt", false, false, false, false),
        6 => ("Tabard", false, false, false, false),
        7 => ("Wrist", false, false, false, false),
        8 => ("Hands", false, false, false, false),
        9 => ("Waist", false, false, false, false),
        10 => ("Legs", false, false, false, false),
        11 => ("Feet", false, false, false, false),
        14 => ("One-Handed Swords", true, true, true, true),
        18 => ("Shield", true, false, false, true),
        23 => ("Staff", true, true, true, false),
        _ => ("", false, false, false, false),
    }
}

fn parse_item_id_from_val(state: &LuaState, value: Val) -> Option<u32> {
    match value {
        Val::Num(number) if number > 0.0 => Some(number as u32),
        Val::Str(_) => {
            let text = val_to_string(state, value)?;
            parse_prefixed_id(&text, "item").or_else(|| text.parse().ok())
        }
        _ => None,
    }
}

fn parse_prefixed_id(value: &str, prefix: &str) -> Option<u32> {
    let prefixed = format!("|H{prefix}:");
    if let Some(start) = value.find(&prefixed) {
        let digits: String = value[start + prefixed.len()..]
            .chars()
            .take_while(|ch| ch.is_ascii_digit())
            .collect();
        return digits.parse().ok();
    }

    let bare = format!("{prefix}:");
    if let Some(start) = value.find(&bare) {
        let digits: String = value[start + bare.len()..]
            .chars()
            .take_while(|ch| ch.is_ascii_digit())
            .collect();
        return digits.parse().ok();
    }

    None
}
