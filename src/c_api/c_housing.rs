//! `C_Housing` and `C_HousingBlueprint` probes backed by `SimState.housing`.
//!
//! The simulator does not model the full War Within housing service yet, but it
//! already keeps house-favor display state. These 12.1 probes expose a small,
//! deterministic local contract: tests or future service glue may mark the
//! player as being inside an owned house and/or plot, `ResetHouse` clears local
//! housing/favor state, and blueprint import/export calls produce simulator
//! share codes that can be round-tripped in tests.

#[cfg(feature = "retail-12-0-0")]
mod basic_mode;

use crate::c_api::helpers::ensure_namespace;
#[cfg(feature = "retail-12-1-0")]
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_set,
};
#[cfg(feature = "retail-12-1-0")]
use crate::lua_api::state::HousingState;
#[cfg(feature = "retail-12-1-0")]
use crate::lua_bridge::FromStack;
#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::LuaResult;
#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
use rilua::Val;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;

type NamespaceTable = GcRef<Table>;

#[cfg(feature = "retail-12-1-0")]
const BLUEPRINT_CODE_PREFIX: &str = "wow-ui-sim:blueprint:";
#[cfg(feature = "retail-12-1-0")]
const ROOM_BLUEPRINT_CODE_PREFIX: &str = "wow-ui-sim:room-blueprint:";
#[cfg(feature = "retail-12-1-0")]
const BLUEPRINT_TYPE_HOUSE: i32 = 1;
#[cfg(feature = "retail-12-1-0")]
const BLUEPRINT_TYPE_ROOM: i32 = 2;

pub(crate) fn register_c_housing_surface(state: &mut LuaState) -> LuaResult<()> {
    #[cfg(feature = "retail-12-0-0")]
    basic_mode::register(state)?;
    let housing = ensure_namespace(state, "C_Housing")?;
    let blueprints = ensure_namespace(state, "C_HousingBlueprint")?;
    let house_editor = ensure_namespace(state, "C_HouseEditor")?;
    let customize_mode = ensure_namespace(state, "C_HousingCustomizeMode")?;
    let decor = ensure_namespace(state, "C_HousingDecor")?;
    let layout = ensure_namespace(state, "C_HousingLayout")?;
    register_patch_12_0_7_housing_surface(state, customize_mode, layout)?;
    register_patch_12_1_c_housing_surface(
        state,
        housing,
        blueprints,
        house_editor,
        customize_mode,
        decor,
        layout,
    )
}

#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
fn register_patch_12_0_7_housing_surface(
    state: &mut LuaState,
    customize_mode: NamespaceTable,
    layout: NamespaceTable,
) -> LuaResult<()> {
    register_catalog_methods(state)?;
    register_patch_12_0_7_customize_mode_methods(state, customize_mode)?;
    register_patch_12_0_7_layout_methods(state, layout)
}

#[cfg(not(any(feature = "retail-12-0-7", feature = "retail-12-1-0")))]
fn register_patch_12_0_7_housing_surface(
    _state: &mut LuaState,
    _customize_mode: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
    _layout: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    Ok(())
}

#[cfg(feature = "retail-12-1-0")]
fn register_patch_12_1_c_housing_surface(
    state: &mut LuaState,
    housing: NamespaceTable,
    blueprints: NamespaceTable,
    house_editor: NamespaceTable,
    customize_mode: NamespaceTable,
    decor: NamespaceTable,
    layout: NamespaceTable,
) -> LuaResult<()> {
    register_housing_methods(state, housing)?;
    register_blueprint_methods(state, blueprints)?;
    register_house_editor_methods(state, house_editor)?;
    register_customize_mode_methods(state, customize_mode)?;
    register_decor_methods(state, decor)?;
    register_layout_methods(state, layout)
}

#[cfg(not(feature = "retail-12-1-0"))]
fn register_patch_12_1_c_housing_surface(
    _state: &mut LuaState,
    _housing: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
    _blueprints: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
    _house_editor: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
    _customize_mode: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
    _decor: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
    _layout: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    Ok(())
}

#[cfg(feature = "retail-12-1-0")]
fn register_housing_methods(state: &mut LuaState, housing: NamespaceTable) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        housing,
        "HouseFinderIgnoreNeighborhood",
        house_finder_ignore_neighborhood,
    )?;
    table_set_rust_fn_static(
        state,
        housing,
        "IsInsideOwnedHouseOrPlot",
        is_inside_owned_house_or_plot,
    )?;
    table_set_rust_fn_static(state, housing, "IsInsideOwnedHouse", is_inside_owned_house)?;
    table_set_rust_fn_static(state, housing, "IsInsideOwnedPlot", is_inside_owned_plot)?;
    table_set_rust_fn_static(state, housing, "ResetHouse", reset_house)
}

#[cfg(feature = "retail-12-1-0")]
fn register_blueprint_methods(state: &mut LuaState, blueprints: NamespaceTable) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        blueprints,
        "CanImportTypeFromCurrentLocation",
        can_import_type_from_current_location,
    )?;
    table_set_rust_fn_static(state, blueprints, "DeleteBlueprint", delete_blueprint)?;
    table_set_rust_fn_static(state, blueprints, "ExportBlueprint", export_blueprint)?;
    table_set_rust_fn_static(
        state,
        blueprints,
        "ExportRoomBlueprint",
        export_room_blueprint,
    )?;
    table_set_rust_fn_static(
        state,
        blueprints,
        "GetBlueprintHyperlink",
        get_blueprint_hyperlink,
    )?;
    table_set_rust_fn_static(
        state,
        blueprints,
        "GetBlueprintTypeForCode",
        get_blueprint_type_for_code,
    )?;
    table_set_rust_fn_static(
        state,
        blueprints,
        "GetExportAvailability",
        get_export_availability,
    )?;
    table_set_rust_fn_static(
        state,
        blueprints,
        "GetFeatureAvailability",
        get_feature_availability,
    )?;
    table_set_rust_fn_static(
        state,
        blueprints,
        "GetImportAvailability",
        get_import_availability,
    )?;
    table_set_rust_fn_static(state, blueprints, "ImportBlueprint", import_blueprint)?;
    table_set_rust_fn_static(state, blueprints, "IsShareCodeValid", is_share_code_valid)?;
    table_set_rust_fn_static(state, blueprints, "RenameBlueprint", rename_blueprint)?;
    table_set_rust_fn_static(
        state,
        blueprints,
        "RequestBlueprintCollection",
        request_blueprint_collection,
    )?;
    table_set_rust_fn_static(
        state,
        blueprints,
        "RequestBlueprintContents",
        request_blueprint_contents,
    )?;
    table_set_rust_fn_static(
        state,
        blueprints,
        "RequestBlueprintContentsForContext",
        request_blueprint_contents_for_context,
    )?;
    table_set_rust_fn_static(
        state,
        blueprints,
        "StartImportRoomBlueprint",
        start_import_room_blueprint,
    )
}

#[cfg(feature = "retail-12-1-0")]
fn register_house_editor_methods(
    state: &mut LuaState,
    house_editor: NamespaceTable,
) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        house_editor,
        "GetActiveHouseEditorMode",
        get_active_house_editor_mode,
    )?;
    table_set_rust_fn_static(
        state,
        house_editor,
        "GetHouseEditorPlayerType",
        get_house_editor_player_type,
    )
}

#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
fn register_catalog_methods(state: &mut LuaState) -> LuaResult<()> {
    let catalog = ensure_namespace(state, "C_HousingCatalog")?;
    table_set_rust_fn_static(
        state,
        catalog,
        "GetCatalogCategoryAndSubcategoryNames",
        get_catalog_category_and_subcategory_names,
    )
}

#[cfg(not(any(feature = "retail-12-0-7", feature = "retail-12-1-0")))]
fn register_catalog_methods(_state: &mut LuaState) -> LuaResult<()> {
    Ok(())
}

#[cfg(feature = "retail-12-1-0")]
fn register_customize_mode_methods(
    state: &mut LuaState,
    customize_mode: NamespaceTable,
) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        customize_mode,
        "ApplyPetToSelectedDecor",
        apply_pet_to_selected_decor,
    )?;
    table_set_rust_fn_static(
        state,
        customize_mode,
        "GetSelectedDecorPetInfo",
        get_selected_decor_pet_info,
    )?;
    register_patch_12_0_7_customize_mode_methods(state, customize_mode)
}

#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
fn register_patch_12_0_7_customize_mode_methods(
    state: &mut LuaState,
    customize_mode: NamespaceTable,
) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        customize_mode,
        "RoomConnectionSupportsDoorType",
        room_connection_supports_door_type,
    )
}

#[cfg(not(any(feature = "retail-12-0-7", feature = "retail-12-1-0")))]
fn register_patch_12_0_7_customize_mode_methods(
    _state: &mut LuaState,
    _customize_mode: NamespaceTable,
) -> LuaResult<()> {
    Ok(())
}

#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
fn register_patch_12_0_7_layout_methods(
    state: &mut LuaState,
    layout: NamespaceTable,
) -> LuaResult<()> {
    table_set_rust_fn_static(state, layout, "CanSetViewedFloor", can_set_viewed_floor)
}

#[cfg(not(any(feature = "retail-12-0-7", feature = "retail-12-1-0")))]
fn register_patch_12_0_7_layout_methods(
    _state: &mut LuaState,
    _layout: NamespaceTable,
) -> LuaResult<()> {
    Ok(())
}

#[cfg(feature = "retail-12-1-0")]
fn register_decor_methods(state: &mut LuaState, decor: NamespaceTable) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        decor,
        "AnyDecorPlacedInRoom",
        any_decor_placed_in_room,
    )?;
    table_set_rust_fn_static(
        state,
        decor,
        "GetBothMaxPlacementBudgets",
        get_both_max_placement_budgets,
    )?;
    table_set_rust_fn_static(
        state,
        decor,
        "GetBothSpentPlacementBudgets",
        get_both_spent_placement_budgets,
    )?;
    table_set_rust_fn_static(
        state,
        decor,
        "GetDecorAssignedPetName",
        get_decor_assigned_pet_name,
    )?;
    table_set_rust_fn_static(
        state,
        decor,
        "GetDecorCanAttachPet",
        get_decor_can_attach_pet,
    )?;
    table_set_rust_fn_static(
        state,
        decor,
        "GetMaxPetPlacementBudget",
        get_max_pet_placement_budget,
    )?;
    table_set_rust_fn_static(
        state,
        decor,
        "GetSpentPetPlacementBudget",
        get_spent_pet_placement_budget,
    )
}

#[cfg(feature = "retail-12-1-0")]
fn register_layout_methods(state: &mut LuaState, layout: NamespaceTable) -> LuaResult<()> {
    table_set_rust_fn_static(state, layout, "GetBaseRoomFloor", get_base_room_floor)?;
    table_set_rust_fn_static(state, layout, "GetRoomPlayerIsIn", get_room_player_is_in)?;
    table_set_rust_fn_static(
        state,
        layout,
        "GetSelectedBlueprintFloorplan",
        get_selected_blueprint_floorplan,
    )?;
    table_set_rust_fn_static(
        state,
        layout,
        "HasSelectedBlueprintFloorplan",
        has_selected_blueprint_floorplan,
    )
}

#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
fn get_catalog_category_and_subcategory_names(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Nil);
    Ok(1)
}

#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
fn room_connection_supports_door_type(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn house_finder_ignore_neighborhood(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

#[cfg(feature = "retail-12-1-0")]
fn is_inside_owned_house_or_plot(state: &mut LuaState) -> LuaResult<u32> {
    let is_inside = {
        let sim = borrow_state(state)?;
        sim.housing.inside_owned_house || sim.housing.inside_owned_plot
    };
    state.push(Val::Bool(is_inside));
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn is_inside_owned_house(state: &mut LuaState) -> LuaResult<u32> {
    let inside_owned_house = { borrow_state(state)?.housing.inside_owned_house };
    state.push(Val::Bool(inside_owned_house));
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn is_inside_owned_plot(state: &mut LuaState) -> LuaResult<u32> {
    let inside_owned_plot = { borrow_state(state)?.housing.inside_owned_plot };
    state.push(Val::Bool(inside_owned_plot));
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn reset_house(state: &mut LuaState) -> LuaResult<u32> {
    let mut sim = borrow_state_mut(state)?;
    sim.housing = HousingState::default();
    Ok(0)
}

#[cfg(feature = "retail-12-1-0")]
fn can_import_type_from_current_location(state: &mut LuaState) -> LuaResult<u32> {
    is_inside_owned_house_or_plot(state)
}

#[cfg(feature = "retail-12-1-0")]
fn delete_blueprint(state: &mut LuaState) -> LuaResult<u32> {
    let blueprint_id = Option::<String>::from_stack(state, 1)?;
    borrow_state_mut(state)?.housing.last_deleted_blueprint_id = blueprint_id;
    Ok(0)
}

#[cfg(feature = "retail-12-1-0")]
fn export_blueprint(state: &mut LuaState) -> LuaResult<u32> {
    let blueprint_id = string_arg_or_empty(state, 1)?;
    borrow_state_mut(state)?.housing.last_exported_blueprint_id = Some(blueprint_id.clone());
    push_string(state, &blueprint_code(BLUEPRINT_CODE_PREFIX, &blueprint_id));
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn export_room_blueprint(state: &mut LuaState) -> LuaResult<u32> {
    let blueprint_id = string_arg_or_empty(state, 1)?;
    borrow_state_mut(state)?
        .housing
        .last_exported_room_blueprint_id = Some(blueprint_id.clone());
    push_string(
        state,
        &blueprint_code(ROOM_BLUEPRINT_CODE_PREFIX, &blueprint_id),
    );
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn get_blueprint_hyperlink(state: &mut LuaState) -> LuaResult<u32> {
    let code = Option::<String>::from_stack(state, 1)?;
    match code.filter(|code| is_valid_share_code(code)) {
        Some(code) => push_string(
            state,
            &format!("|Hhousingblueprint:{code}|h[Housing Blueprint]|h"),
        ),
        None => state.push(Val::Nil),
    }
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn get_blueprint_type_for_code(state: &mut LuaState) -> LuaResult<u32> {
    let blueprint_type =
        Option::<String>::from_stack(state, 1)?.and_then(|code| blueprint_type_for_code(&code));
    match blueprint_type {
        Some(blueprint_type) => state.push(Val::Num(f64::from(blueprint_type))),
        None => state.push(Val::Nil),
    }
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn import_blueprint(state: &mut LuaState) -> LuaResult<u32> {
    let code = Option::<String>::from_stack(state, 1)?.filter(|code| is_valid_share_code(code));
    borrow_state_mut(state)?
        .housing
        .last_imported_blueprint_code = code;
    Ok(0)
}

#[cfg(feature = "retail-12-1-0")]
fn is_share_code_valid(state: &mut LuaState) -> LuaResult<u32> {
    let valid =
        Option::<String>::from_stack(state, 1)?.is_some_and(|code| is_valid_share_code(&code));
    state.push(Val::Bool(valid));
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn rename_blueprint(state: &mut LuaState) -> LuaResult<u32> {
    let blueprint_id = string_arg_or_empty(state, 1)?;
    let name = string_arg_or_empty(state, 2)?;
    borrow_state_mut(state)?.housing.last_renamed_blueprint = Some((blueprint_id, name));
    Ok(0)
}

#[cfg(feature = "retail-12-1-0")]
fn request_blueprint_collection(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?
        .housing
        .requested_blueprint_collection = true;
    Ok(0)
}

#[cfg(feature = "retail-12-1-0")]
fn request_blueprint_contents(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?
        .housing
        .last_requested_blueprint_contents_id = Option::<String>::from_stack(state, 1)?;
    Ok(0)
}

#[cfg(feature = "retail-12-1-0")]
fn request_blueprint_contents_for_context(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?
        .housing
        .requested_blueprint_context_contents = true;
    Ok(0)
}

#[cfg(feature = "retail-12-1-0")]
fn start_import_room_blueprint(state: &mut LuaState) -> LuaResult<u32> {
    let code =
        Option::<String>::from_stack(state, 1)?.filter(|code| is_valid_room_blueprint_code(code));
    borrow_state_mut(state)?
        .housing
        .last_imported_room_blueprint_code = code;
    Ok(0)
}

#[cfg(feature = "retail-12-1-0")]
fn get_export_availability(state: &mut LuaState) -> LuaResult<u32> {
    let availability = { borrow_state(state)?.housing.blueprint_export_availability };
    state.push(Val::Num(f64::from(availability)));
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn get_feature_availability(state: &mut LuaState) -> LuaResult<u32> {
    let availability = { borrow_state(state)?.housing.blueprint_feature_availability };
    state.push(Val::Num(f64::from(availability)));
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn get_import_availability(state: &mut LuaState) -> LuaResult<u32> {
    let availability = { borrow_state(state)?.housing.blueprint_import_availability };
    state.push(Val::Num(f64::from(availability)));
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn get_active_house_editor_mode(state: &mut LuaState) -> LuaResult<u32> {
    let mode = { borrow_state(state)?.housing.active_house_editor_mode };
    state.push(Val::Num(f64::from(mode)));
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn get_house_editor_player_type(state: &mut LuaState) -> LuaResult<u32> {
    let player_type = { borrow_state(state)?.housing.house_editor_player_type };
    push_optional_i32(state, player_type);
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn apply_pet_to_selected_decor(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.housing.selected_decor_pet_guid =
        Option::<String>::from_stack(state, 1)?;
    Ok(0)
}

#[cfg(feature = "retail-12-1-0")]
fn get_selected_decor_pet_info(state: &mut LuaState) -> LuaResult<u32> {
    let pet_info = {
        let sim = borrow_state(state)?;
        sim.housing
            .selected_decor_pet_guid
            .clone()
            .map(|guid| (guid, sim.housing.selected_decor_pet_name.clone()))
    };
    match pet_info {
        Some((guid, name)) => {
            let table = selected_pet_table(state, &guid, name.as_deref());
            state.push(table);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn any_decor_placed_in_room(state: &mut LuaState) -> LuaResult<u32> {
    let room_id = i32::from_stack(state, 1)?;
    let has_decor = borrow_state(state)?
        .housing
        .rooms_with_decor
        .contains(&room_id);
    state.push(Val::Bool(has_decor));
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn get_both_max_placement_budgets(state: &mut LuaState) -> LuaResult<u32> {
    let budgets = {
        let sim = borrow_state(state)?;
        (
            sim.housing.max_indoor_placement_budget,
            sim.housing.max_outdoor_placement_budget,
        )
    };
    push_optional_pair(state, budgets)
}

#[cfg(feature = "retail-12-1-0")]
fn get_both_spent_placement_budgets(state: &mut LuaState) -> LuaResult<u32> {
    let budgets = {
        let sim = borrow_state(state)?;
        (
            sim.housing.spent_indoor_placement_budget,
            sim.housing.spent_outdoor_placement_budget,
        )
    };
    push_optional_pair(state, budgets)
}

#[cfg(feature = "retail-12-1-0")]
fn get_decor_assigned_pet_name(state: &mut LuaState) -> LuaResult<u32> {
    let decor_guid = string_arg_or_empty(state, 1)?;
    let pet_name = borrow_state(state)?
        .housing
        .decor_pet_names
        .get(&decor_guid)
        .cloned();
    push_optional_string(state, pet_name.as_deref());
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn get_decor_can_attach_pet(state: &mut LuaState) -> LuaResult<u32> {
    let decor_guid = string_arg_or_empty(state, 1)?;
    let can_attach = borrow_state(state)?
        .housing
        .pet_attachable_decor_guids
        .contains(&decor_guid);
    state.push(Val::Bool(can_attach));
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn get_max_pet_placement_budget(state: &mut LuaState) -> LuaResult<u32> {
    let budget = { borrow_state(state)?.housing.max_pet_placement_budget };
    push_optional_i32(state, budget);
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn get_spent_pet_placement_budget(state: &mut LuaState) -> LuaResult<u32> {
    let budget = { borrow_state(state)?.housing.spent_pet_placement_budget };
    push_optional_i32(state, budget);
    Ok(1)
}

#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
fn can_set_viewed_floor(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn get_base_room_floor(state: &mut LuaState) -> LuaResult<u32> {
    let room_id = i32::from_stack(state, 1)?;
    let floor = borrow_state(state)?
        .housing
        .base_room_floors
        .get(&room_id)
        .copied();
    push_optional_i32(state, floor);
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn get_room_player_is_in(state: &mut LuaState) -> LuaResult<u32> {
    let room_id = { borrow_state(state)?.housing.room_player_is_in };
    push_optional_i32(state, room_id);
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn get_selected_blueprint_floorplan(state: &mut LuaState) -> LuaResult<u32> {
    let floorplan = { borrow_state(state)?.housing.selected_blueprint_floorplan };
    push_optional_i32(state, floorplan);
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn has_selected_blueprint_floorplan(state: &mut LuaState) -> LuaResult<u32> {
    let has_floorplan = borrow_state(state)?
        .housing
        .selected_blueprint_floorplan
        .is_some();
    state.push(Val::Bool(has_floorplan));
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn string_arg_or_empty(state: &mut LuaState, index: i32) -> LuaResult<String> {
    Ok(Option::<String>::from_stack(state, index)?.unwrap_or_default())
}

#[cfg(feature = "retail-12-1-0")]
fn push_string(state: &mut LuaState, value: &str) {
    let value = create_string(state, value);
    state.push(value);
}

#[cfg(feature = "retail-12-1-0")]
fn push_optional_string(state: &mut LuaState, value: Option<&str>) {
    match value {
        Some(value) => push_string(state, value),
        None => state.push(Val::Nil),
    }
}

#[cfg(feature = "retail-12-1-0")]
fn push_optional_i32(state: &mut LuaState, value: Option<i32>) {
    match value {
        Some(value) => state.push(Val::Num(f64::from(value))),
        None => state.push(Val::Nil),
    }
}

#[cfg(feature = "retail-12-1-0")]
fn push_optional_pair(state: &mut LuaState, pair: (Option<i32>, Option<i32>)) -> LuaResult<u32> {
    let Some(first) = pair.0 else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let Some(second) = pair.1 else {
        state.push(Val::Nil);
        return Ok(1);
    };
    state.push(Val::Num(f64::from(first)));
    state.push(Val::Num(f64::from(second)));
    Ok(2)
}

#[cfg(feature = "retail-12-1-0")]
fn selected_pet_table(state: &mut LuaState, guid: &str, name: Option<&str>) -> Val {
    let table = create_table(state);
    let guid = create_string(state, guid);
    table_set(state, table, "petGUID", guid);
    if let Some(name) = name {
        let name = create_string(state, name);
        table_set(state, table, "petName", name);
    }
    table
}

#[cfg(feature = "retail-12-1-0")]
fn blueprint_code(prefix: &str, blueprint_id: &str) -> String {
    format!("{prefix}{blueprint_id}")
}

#[cfg(feature = "retail-12-1-0")]
fn is_valid_share_code(code: &str) -> bool {
    blueprint_type_for_code(code).is_some()
}

#[cfg(feature = "retail-12-1-0")]
fn is_valid_room_blueprint_code(code: &str) -> bool {
    code.strip_prefix(ROOM_BLUEPRINT_CODE_PREFIX)
        .is_some_and(|id| !id.is_empty())
}

#[cfg(feature = "retail-12-1-0")]
fn blueprint_type_for_code(code: &str) -> Option<i32> {
    if code
        .strip_prefix(BLUEPRINT_CODE_PREFIX)
        .is_some_and(|id| !id.is_empty())
    {
        Some(BLUEPRINT_TYPE_HOUSE)
    } else if is_valid_room_blueprint_code(code) {
        Some(BLUEPRINT_TYPE_ROOM)
    } else {
        None
    }
}
