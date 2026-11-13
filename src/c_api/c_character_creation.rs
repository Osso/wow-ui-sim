//! Glue-only character-creation mode and facing inputs.

use crate::c_api::ensure_namespace;
use crate::c_api::helpers::bad_argument;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::WowLuaEnv;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_set_static,
};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use crate::screen::ScreenKind;
use rilua::table_security::unwrap_secret;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaApiMut, LuaResult, Val};

/// Host-supplied data for the normal starting-zone choice.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StartingZoneChoiceInput {
    pub zone_name: String,
    pub zone_image_atlas: Option<String>,
}

/// Host inputs for GetStartingZoneChoices. Defaults represent no supplied data.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StartingZoneChoicesInput {
    /// None means no matching normal-zone record was supplied.
    pub normal_choice: Option<StartingZoneChoiceInput>,
    /// Applied only when a normal-zone record exists; None means no signal.
    pub restricted_to_npe: Option<bool>,
}

const SET_IN_CHARACTER_CREATE_USAGE: &str =
    "Usage: C_CharacterCreation.SetInCharacterCreate(inCharacterCreate)";
const SET_VIEWING_ALTERED_FORM_USAGE: &str =
    "Usage: C_CharacterCreation.SetViewingAlteredForm(isViewingAlteredForm)";
const SET_BLUR_ENABLED_USAGE: &str = "Usage: C_CharacterCreation.SetBlurEnabled(enabled)";
const SET_CAMERA_ZOOM_LEVEL_USAGE: &str =
    "Usage: C_CharacterCreation.SetCameraZoomLevel(zoomLevel [, keepCustomZoom])";
const SET_MODEL_DRESS_STATE_USAGE: &str =
    "Usage: C_CharacterCreation.SetModelDressState(dressedState)";
const SET_SELECTED_PREVIEW_GEAR_TYPE_USAGE: &str =
    "Usage: C_CharacterCreation.SetSelectedPreviewGearType(gearType)";
const MAX_CHARACTER_CREATE_CAMERA_ZOOM: i32 = 100;
const CHARACTER_CREATE_BYTE_MODULUS: f64 = 256.0;
const NEW_CHAR_GEAR_START: u8 = 0;
const NEW_CHAR_GEAR_PREVIEW: u8 = 1;
const SET_CHARACTER_CREATE_FACING_USAGE: &str =
    "Usage: C_CharacterCreation.SetCharacterCreateFacing(degrees)";
// R36 marks the displayed bounds as inferred; tests check only the stable prefix and usage.
const SET_CHARACTER_CREATE_FACING_RANGE_ERROR: &str = concat!(
    "outside of expected range -3.402823e+38 to 3.402823e+38 - ",
    "Usage: C_CharacterCreation.SetCharacterCreateFacing(degrees)"
);
const SET_CHARACTER_CREATE_FACING_FINITE_ERROR: &str =
    "must be a finite number - Usage: C_CharacterCreation.SetCharacterCreateFacing(degrees)";
const DEGREES_PER_HALF_TURN: f32 = 180.0;

/// Update Glue-only character-creation members for the selected screen.
pub(crate) fn update_screen_registration(env: &WowLuaEnv, screen: ScreenKind) {
    if !matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    ) {
        return;
    }

    if screen == ScreenKind::CharacterCreate
        && matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever)
    {
        env.state.borrow_mut().character_create_viewing_altered_form = false;
    }

    let mut lua = env.lua.borrow_mut();
    update_registration(lua.state_mut(), screen.is_glue())
        .expect("register Glue-only C_CharacterCreation members for the selected screen");
}

fn update_registration(state: &mut LuaState, is_glue: bool) -> LuaResult<()> {
    update_altered_form_model_helper(state, is_glue)?;
    let namespace = match LuaApiMut::get_global_val(state, "C_CharacterCreation") {
        Val::Table(namespace) => namespace,
        _ if is_glue => ensure_namespace(state, "C_CharacterCreation")?,
        _ => return Ok(()),
    };

    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    super::c_character_creation_submit::update_registration(state, namespace, is_glue)?;
    #[cfg(feature = "client-ptr")]
    super::c_character_creation_submit_ptr::update_registration(state, namespace, is_glue)?;
    update_default_facing_registration(state, namespace, is_glue)?;
    update_retail_and_forever_registration(state, namespace, is_glue)?;
    update_starting_zone_choices_registration(state, namespace, is_glue)
}

fn update_altered_form_model_helper(state: &mut LuaState, is_glue: bool) -> LuaResult<()> {
    let global = state.global;
    if is_glue && matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever) {
        table_set_rust_fn_static(
            state,
            global,
            "__wow_character_create_has_altered_model",
            selected_race_has_altered_model,
        )
    } else {
        table_set_static(
            state,
            Val::Table(global),
            "__wow_character_create_has_altered_model",
            Val::Nil,
        );
        Ok(())
    }
}

fn update_starting_zone_choices_registration(
    state: &mut LuaState,
    namespace: GcRef<Table>,
    is_glue: bool,
) -> LuaResult<()> {
    if !matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    ) {
        return Ok(());
    }

    if is_glue {
        table_set_rust_fn_static(
            state,
            namespace,
            "GetStartingZoneChoices",
            get_starting_zone_choices,
        )
    } else {
        table_set_static(
            state,
            Val::Table(namespace),
            "GetStartingZoneChoices",
            Val::Nil,
        );
        Ok(())
    }
}

fn update_default_facing_registration(
    state: &mut LuaState,
    namespace: GcRef<Table>,
    is_glue: bool,
) -> LuaResult<()> {
    if is_glue {
        table_set_rust_fn_static(
            state,
            namespace,
            "GetDefaultCharacterCreateFacing",
            read_default_character_create_facing,
        )
    } else {
        table_set_static(
            state,
            Val::Table(namespace),
            "GetDefaultCharacterCreateFacing",
            Val::Nil,
        );
        Ok(())
    }
}

fn update_retail_and_forever_registration(
    state: &mut LuaState,
    namespace: GcRef<Table>,
    is_glue: bool,
) -> LuaResult<()> {
    if !matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever) {
        return Ok(());
    }

    if is_glue {
        register_retail_and_forever_glue_apis(state, namespace)
    } else {
        for member in [
            "SetInCharacterCreate",
            "GetCharacterCreateFacing",
            "SetCharacterCreateFacing",
            "SetViewingAlteredForm",
            "IsViewingAlteredForm",
            "SetBlurEnabled",
            "GetCurrentCameraZoom",
            "SetCameraZoomLevel",
            "SetModelDressState",
            "SetSelectedPreviewGearType",
        ] {
            table_set_static(state, Val::Table(namespace), member, Val::Nil);
        }
        Ok(())
    }
}

fn register_retail_and_forever_glue_apis(
    state: &mut LuaState,
    namespace: GcRef<Table>,
) -> LuaResult<()> {
    register_character_create_mode_apis(state, namespace)?;
    register_character_create_altered_form_apis(state, namespace)?;
    register_character_create_preview_apis(state, namespace)
}

fn register_character_create_mode_apis(
    state: &mut LuaState,
    namespace: GcRef<Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        namespace,
        "SetInCharacterCreate",
        set_in_character_create,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetCharacterCreateFacing",
        get_character_create_facing,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "SetCharacterCreateFacing",
        set_character_create_facing,
    )
}

fn register_character_create_altered_form_apis(
    state: &mut LuaState,
    namespace: GcRef<Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        namespace,
        "SetViewingAlteredForm",
        set_viewing_altered_form,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "IsViewingAlteredForm",
        is_viewing_altered_form,
    )
}

fn register_character_create_preview_apis(
    state: &mut LuaState,
    namespace: GcRef<Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "SetBlurEnabled", set_blur_enabled)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetCurrentCameraZoom",
        get_current_camera_zoom,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "SetCameraZoomLevel",
        set_camera_zoom_level,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "SetModelDressState",
        set_model_dress_state,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "SetSelectedPreviewGearType",
        set_selected_preview_gear_type,
    )
}

/// VERIFIED for Retail 12.1.5.70688 and Forever 70707. PTR applicability is
/// INFERRED from Retail behavior; no PTR-specific result assertions are made.
fn get_starting_zone_choices(state: &mut LuaState) -> LuaResult<u32> {
    let inputs = borrow_state(state)?
        .character_create_starting_zone_choices
        .clone();
    let retail_projection = matches!(ACTIVE, ClientProfile::Retail | ClientProfile::Ptr);
    let normal_choice_available = inputs.normal_choice.is_some();
    let is_npe_restricted =
        retail_projection && normal_choice_available && inputs.restricted_to_npe.unwrap_or(false);

    let first_choice = if is_npe_restricted {
        starting_zone_choice_table(state, "", None, true)
    } else if let Some(normal_choice) = inputs.normal_choice.as_ref() {
        starting_zone_choice_table(
            state,
            &normal_choice.zone_name,
            normal_choice.zone_image_atlas.as_deref(),
            false,
        )
    } else {
        starting_zone_choice_table(state, "", None, false)
    };
    state.push(first_choice);

    let second_choice = if retail_projection && normal_choice_available && !is_npe_restricted {
        starting_zone_choice_table(state, "", None, true)
    } else {
        Val::Nil
    };
    state.push(second_choice);
    Ok(2)
}

fn starting_zone_choice_table(
    state: &mut LuaState,
    zone_name: &str,
    zone_image_atlas: Option<&str>,
    is_npe: bool,
) -> Val {
    let stack_top = state.top;
    let choice = create_table(state);
    state.push(choice);
    let zone_name = create_string(state, zone_name);
    table_set_static(state, choice, "zoneName", zone_name);
    let atlas = zone_image_atlas.map_or(Val::Nil, |name| create_string(state, name));
    table_set_static(state, choice, "zoneImageAtlas", atlas);
    table_set_static(state, choice, "isNPE", Val::Bool(is_npe));
    state.top = stack_top;
    choice
}

fn set_blur_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let argument = stack_val(state, 1);
    if matches!(argument, Val::Nil) {
        return Err(bad_argument(1, "SetBlurEnabled", SET_BLUR_ENABLED_USAGE));
    }

    borrow_state_mut(state)?.character_create_blur_enabled = !matches!(argument, Val::Bool(false));
    Ok(0)
}

fn get_current_camera_zoom(state: &mut LuaState) -> LuaResult<u32> {
    let zoom = borrow_state(state)?.character_create_camera_zoom;
    state.push(Val::Num(f64::from(zoom)));
    Ok(1)
}

// R83b/R83c (#305/#320): PTR shares this verified signed32/clamp and
// changed-only marker update with Retail/Forever. Its Lua installation and
// ordinary return/second-argument truthiness policy remain separately inferred.
pub(super) fn set_camera_zoom_level(state: &mut LuaState) -> LuaResult<u32> {
    let zoom = crate::c_api::helpers::read_untainted_signed32_argument(
        state,
        1,
        "SetCameraZoomLevel",
        SET_CAMERA_ZOOM_LEVEL_USAGE,
    )?;
    let keep_custom_zoom = !matches!(stack_val(state, 2), Val::Nil | Val::Bool(false));
    let clamped_zoom = zoom.clamp(0, MAX_CHARACTER_CREATE_CAMERA_ZOOM);
    let mut sim = borrow_state_mut(state)?;
    if sim.character_create_camera_zoom_is_custom && keep_custom_zoom {
        return Ok(0);
    }
    if sim.character_create_camera_zoom != clamped_zoom {
        sim.character_create_camera_zoom = clamped_zoom;
        sim.character_create_camera_zoom_is_custom = false;
    }
    Ok(0)
}

fn set_model_dress_state(state: &mut LuaState) -> LuaResult<u32> {
    let argument = stack_val(state, 1);
    if matches!(argument, Val::Nil) {
        return Err(bad_argument(
            1,
            "SetModelDressState",
            SET_MODEL_DRESS_STATE_USAGE,
        ));
    }

    // The value selects unsupported 3D model work, so only validation is observable here.
    Ok(0)
}

fn set_selected_preview_gear_type(state: &mut LuaState) -> LuaResult<u32> {
    validate_selected_preview_gear_type(state)?;
    Ok(0)
}

fn validate_selected_preview_gear_type(state: &LuaState) -> LuaResult<()> {
    let argument = stack_val(state, 1);
    if matches!(argument, Val::Nil) {
        return Err(bad_argument(
            1,
            "SetSelectedPreviewGearType",
            SET_SELECTED_PREVIEW_GEAR_TYPE_USAGE,
        ));
    }

    let selected_gear = selected_preview_gear_byte(state, argument);
    if matches!(selected_gear, NEW_CHAR_GEAR_START | NEW_CHAR_GEAR_PREVIEW) {
        return Ok(());
    }

    Err(bad_argument(
        1,
        "SetSelectedPreviewGearType",
        SET_SELECTED_PREVIEW_GEAR_TYPE_USAGE,
    ))
}

fn selected_preview_gear_byte(state: &LuaState, argument: Val) -> u8 {
    let number = match argument {
        Val::Num(number) => Some(number),
        Val::Str(reference) => state
            .gc
            .string_arena
            .get(reference)
            .and_then(|string| crate::c_api::helpers::parse_numeric_string(string.data())),
        _ => None,
    }
    .unwrap_or(0.0);
    if number.is_finite() {
        number.trunc().rem_euclid(CHARACTER_CREATE_BYTE_MODULUS) as u8
    } else {
        NEW_CHAR_GEAR_START
    }
}

fn get_character_create_facing(state: &mut LuaState) -> LuaResult<u32> {
    let radians = borrow_state(state)?.character_create_facing_radians;
    let degrees = radians * DEGREES_PER_HALF_TURN / std::f32::consts::PI;
    state.push(Val::Num(f64::from(degrees)));
    Ok(1)
}

fn set_viewing_altered_form(state: &mut LuaState) -> LuaResult<u32> {
    let argument = stack_val(state, 1);
    if matches!(argument, Val::Nil) {
        return Err(bad_argument(
            1,
            "SetViewingAlteredForm",
            SET_VIEWING_ALTERED_FORM_USAGE,
        ));
    }

    let has_altered_model = selected_race_has_altered_model_value(state)?;
    let viewing = has_altered_model && !matches!(argument, Val::Bool(false));
    let mut sim = borrow_state_mut(state)?;
    if sim.character_create_viewing_altered_form != viewing {
        sim.character_create_viewing_altered_form = viewing;
    }
    Ok(0)
}

fn is_viewing_altered_form(state: &mut LuaState) -> LuaResult<u32> {
    let has_altered_model = selected_race_has_altered_model_value(state)?;
    let viewing = has_altered_model && borrow_state(state)?.character_create_viewing_altered_form;
    state.push(Val::Bool(viewing));
    Ok(1)
}

fn selected_race_has_altered_model(state: &mut LuaState) -> LuaResult<u32> {
    let has_altered_model = selected_race_has_altered_model_value(state)?;
    state.push(Val::Bool(has_altered_model));
    Ok(1)
}

fn selected_race_has_altered_model_value(state: &mut LuaState) -> LuaResult<bool> {
    let Some(race_id) = read_selected_character_create_id(state, "__wow_selected_race_id") else {
        return Ok(false);
    };
    Ok(borrow_state(state)?
        .character_create_altered_form_races
        .contains(&race_id))
}

fn set_character_create_facing(state: &mut LuaState) -> LuaResult<u32> {
    let degrees = read_character_create_facing_degrees(state)?;
    let radians = wrap_character_create_facing(degrees);
    borrow_state_mut(state)?.character_create_facing_radians = radians;
    Ok(0)
}

fn read_character_create_facing_degrees(state: &LuaState) -> LuaResult<f32> {
    let degrees = read_character_create_facing_argument(state)?;
    if !degrees.is_finite() {
        return Err(character_create_facing_argument_error(
            SET_CHARACTER_CREATE_FACING_FINITE_ERROR,
        ));
    }
    if degrees < f64::from(f32::MIN) || degrees > f64::from(f32::MAX) {
        return Err(character_create_facing_argument_error(
            SET_CHARACTER_CREATE_FACING_RANGE_ERROR,
        ));
    }
    Ok(degrees as f32)
}

fn read_character_create_facing_argument(state: &LuaState) -> LuaResult<f64> {
    let value = stack_val(state, 1);
    if matches!(value, Val::Nil) {
        return Err(character_create_facing_argument_error(
            SET_CHARACTER_CREATE_FACING_USAGE,
        ));
    }

    let number = match value {
        Val::Num(number) => Some(number),
        Val::Str(reference) => state
            .gc
            .string_arena
            .get(reference)
            .and_then(|string| crate::c_api::helpers::parse_numeric_string(string.data())),
        _ => None,
    };
    number.ok_or_else(|| {
        character_create_facing_argument_error(SET_CHARACTER_CREATE_FACING_RANGE_ERROR)
    })
}

fn character_create_facing_argument_error(detail: &'static str) -> rilua::LuaError {
    crate::c_api::helpers::bad_argument(1, "SetCharacterCreateFacing", detail)
}

fn wrap_character_create_facing(degrees: f32) -> f32 {
    let pi = std::f32::consts::PI;
    let full_turn = 2.0 * pi;
    let remainder = (degrees * pi / DEGREES_PER_HALF_TURN) % full_turn;
    if remainder > pi {
        remainder - full_turn
    } else if remainder < -pi {
        remainder + full_turn
    } else {
        remainder
    }
}

fn read_default_character_create_facing(state: &mut LuaState) -> LuaResult<u32> {
    let race_id = read_selected_character_create_id(state, "__wow_selected_race_id");
    let sex_id = read_selected_character_create_id(state, "__wow_selected_sex_id");
    let facing = match (race_id, sex_id) {
        (Some(race_id), Some(sex_id)) => borrow_state(state)?
            .character_create_default_facing_by_race_sex
            .get(&(race_id, sex_id))
            .copied()
            .unwrap_or(0.0),
        _ => 0.0,
    };
    state.push(Val::Num(facing));
    Ok(1)
}

fn read_selected_character_create_id(state: &mut LuaState, name: &str) -> Option<i64> {
    let Val::Num(value) = LuaApiMut::get_global_val(state, name) else {
        return None;
    };
    if value.is_finite() && value.fract() == 0.0 && (0.0..=f64::from(u32::MAX)).contains(&value) {
        Some(value as i64)
    } else {
        None
    }
}

pub(crate) fn set_in_character_create(state: &mut LuaState) -> LuaResult<u32> {
    let value = unwrap_secret(state, stack_val(state, 1))?;
    if matches!(value, Val::Nil) {
        return Err(rilua::runtime_error(SET_IN_CHARACTER_CREATE_USAGE));
    }

    let entering = !matches!(value, Val::Bool(false));
    let mut sim = borrow_state_mut(state)?;
    if entering
        && !sim.in_character_create
        && matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever)
    {
        sim.character_create_viewing_altered_form = false;
    }
    sim.in_character_create = entering;
    Ok(0)
}
