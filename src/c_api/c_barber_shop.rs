//! `C_BarberShop` — character-customization surface read by
//! `Blizzard_BarbershopUI`. Backed by `state.barber_shop`.
//!
//! Covers `Blizzard_BarbershopUI/Mainline/Blizzard_BarberShopUI.lua`
//! and the altered-form availability query from `BarberShopDocumentation.lua`.
//! Availability follows the host's current-character alternate-race snapshot;
//! it is independent of which form is currently being viewed.
//! `GetCurrentCost` remains unmodeled.
//!
//! Event semantics:
//! - `Cancel()` → fires `BARBER_SHOP_RESULT(false)` so the addon's
//!   close path treats the cancel as a non-success.
//! - `ApplyCustomizationChoices()` → folds preview choices, fires
//!   `BARBER_SHOP_RESULT(true)` then `BARBER_SHOP_APPEARANCE_APPLIED`,
//!   returns `true`.
//! - `SetCustomizationChoice` → fires `BARBER_SHOP_COST_UPDATE` (drives
//!   the Accept/Reset button enable state via `UpdateButtons`).
//! - `ResetCustomizationChoices` and `RandomizeCustomizationChoices`
//!   fire `BARBER_SHOP_FORCE_CUSTOMIZATIONS_UPDATE` so the addon
//!   re-pulls the customization tree.
//! - `SetViewingAlteredForm`, `SetViewingShapeshiftForm`,
//!   `SetViewingChrModel`, `RotateCamera`, `ResetCameraRotation`, and
//!   `SetSelectedSex` fire `BARBER_SHOP_CAMERA_VALUES_UPDATED` so the
//!   addon's one-shot listener resets the model rotation.

use crate::c_api::ensure_namespace;
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, create_table_with_fields,
    table_set_num,
};
use crate::lua_api::state_types::{
    BarberShopAlternateFormRace, BarberShopCategory, BarberShopCharacterData, BarberShopOption,
};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

type BarberShopMethod = fn(&mut LuaState) -> LuaResult<u32>;

const BARBER_SHOP_METHODS: &[(&str, BarberShopMethod)] = &[
    ("HasCustomizationFeature", has_customization_feature),
    ("GetCurrentCharacterData", get_current_character_data),
    ("IsViewingAlteredForm", is_viewing_altered_form),
    ("HasAlteredForm", has_altered_form),
    ("GetViewingChrModel", get_viewing_chr_model),
    ("Cancel", cancel),
    ("ResetCustomizationChoices", reset_customization_choices),
    ("ApplyCustomizationChoices", apply_customization_choices),
    ("HasAnyChanges", has_any_changes),
    ("GetAvailableCustomizations", get_available_customizations),
    ("SetCustomizationChoice", set_customization_choice),
    ("ClearPreviewChoices", clear_preview_choices),
    ("PreviewCustomizationChoice", preview_customization_choice),
    (
        "MarkCustomizationChoiceAsSeen",
        mark_customization_choice_as_seen,
    ),
    (
        "MarkCustomizationOptionAsSeen",
        mark_customization_option_as_seen,
    ),
    ("SaveSeenChoices", save_seen_choices),
    ("GetCurrentCameraZoom", get_current_camera_zoom),
    ("SetCameraZoomLevel", set_camera_zoom_level),
    ("ZoomCamera", zoom_camera),
    ("RotateCamera", rotate_camera),
    ("ResetCameraRotation", reset_camera_rotation),
    ("SetViewingAlteredForm", set_viewing_altered_form),
    ("SetViewingShapeshiftForm", set_viewing_shapeshift_form),
    ("SetViewingChrModel", set_viewing_chr_model),
    ("SetModelDressState", set_model_dress_state),
    ("SetCameraDistanceOffset", set_camera_distance_offset),
    (
        "RandomizeCustomizationChoices",
        randomize_customization_choices,
    ),
    ("SetSelectedSex", set_selected_sex),
];

pub(crate) fn register_c_barber_shop_surface(state: &mut LuaState) -> LuaResult<()> {
    let ns = ensure_namespace(state, "C_BarberShop")?;
    for &(name, func) in BARBER_SHOP_METHODS {
        table_set_rust_fn_static(state, ns, name, func)?;
    }
    Ok(())
}

fn has_customization_feature(state: &mut LuaState) -> LuaResult<u32> {
    let mask = i32::from_stack(state, 1)?;
    let flags = borrow_state(state)?.barber_shop.feature_flags;
    state.push(Val::Bool((flags & mask) != 0));
    Ok(1)
}

fn get_current_character_data(state: &mut LuaState) -> LuaResult<u32> {
    let Some(data) = borrow_state(state)?.barber_shop.current_character.clone() else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let table = build_character_data_table(state, &data);
    state.push(table);
    Ok(1)
}

fn is_viewing_altered_form(state: &mut LuaState) -> LuaResult<u32> {
    let viewing = borrow_state(state)?.barber_shop.viewing_altered_form;
    state.push(Val::Bool(viewing));
    Ok(1)
}

fn has_altered_form(state: &mut LuaState) -> LuaResult<u32> {
    let has_altered_form = borrow_state(state)?
        .barber_shop
        .current_character
        .as_ref()
        .is_some_and(|character| character.alternate_form_race.is_some());
    state.push(Val::Bool(has_altered_form));
    Ok(1)
}

fn get_viewing_chr_model(state: &mut LuaState) -> LuaResult<u32> {
    let chr_model = borrow_state(state)?.barber_shop.viewing_chr_model;
    state.push(optional_id_val(chr_model));
    Ok(1)
}

fn cancel(state: &mut LuaState) -> LuaResult<u32> {
    dispatch_event_now(state, "BARBER_SHOP_RESULT", &[Val::Bool(false)])?;
    Ok(0)
}

fn reset_customization_choices(state: &mut LuaState) -> LuaResult<u32> {
    {
        let mut sim = borrow_state_mut(state)?;
        sim.barber_shop.choices.clear();
        sim.barber_shop.preview_choices.clear();
        sim.barber_shop.has_changes = false;
    }
    dispatch_event_now(state, "BARBER_SHOP_FORCE_CUSTOMIZATIONS_UPDATE", &[])?;
    Ok(0)
}

fn apply_customization_choices(state: &mut LuaState) -> LuaResult<u32> {
    {
        let mut sim = borrow_state_mut(state)?;
        let preview = std::mem::take(&mut sim.barber_shop.preview_choices);
        for (option_id, choice_id) in preview {
            sim.barber_shop.choices.insert(option_id, choice_id);
        }
        sim.barber_shop.has_changes = false;
    }
    dispatch_event_now(state, "BARBER_SHOP_RESULT", &[Val::Bool(true)])?;
    dispatch_event_now(state, "BARBER_SHOP_APPEARANCE_APPLIED", &[])?;
    state.push(Val::Bool(true));
    Ok(1)
}

fn has_any_changes(state: &mut LuaState) -> LuaResult<u32> {
    let has_changes = borrow_state(state)?.barber_shop.has_changes;
    state.push(Val::Bool(has_changes));
    Ok(1)
}

fn get_available_customizations(state: &mut LuaState) -> LuaResult<u32> {
    let Some(categories) = borrow_state(state)?
        .barber_shop
        .available_customizations
        .clone()
    else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let table_val = create_table(state);
    let Val::Table(table_ref) = table_val else {
        unreachable!("create_table must return a table");
    };
    for (index, category) in categories.iter().enumerate() {
        let entry = build_category_table(state, category);
        table_set_num(state, table_ref, (index + 1) as f64, entry);
    }
    state.push(table_val);
    Ok(1)
}

fn set_customization_choice(state: &mut LuaState) -> LuaResult<u32> {
    let option_id = i32::from_stack(state, 1)?;
    let choice_id = i32::from_stack(state, 2)?;
    {
        let mut sim = borrow_state_mut(state)?;
        sim.barber_shop.choices.insert(option_id, choice_id);
        sim.barber_shop.has_changes = true;
    }
    dispatch_event_now(state, "BARBER_SHOP_COST_UPDATE", &[])?;
    Ok(0)
}

fn clear_preview_choices(state: &mut LuaState) -> LuaResult<u32> {
    let clear_saved = bool::from_stack(state, 1).unwrap_or(false);
    let mut sim = borrow_state_mut(state)?;
    sim.barber_shop.preview_choices.clear();
    if clear_saved {
        sim.barber_shop.choices.clear();
        sim.barber_shop.has_changes = false;
    }
    Ok(0)
}

fn preview_customization_choice(state: &mut LuaState) -> LuaResult<u32> {
    let option_id = i32::from_stack(state, 1)?;
    let choice_id = i32::from_stack(state, 2)?;
    let mut sim = borrow_state_mut(state)?;
    sim.barber_shop.preview_choices.insert(option_id, choice_id);
    Ok(0)
}

fn mark_customization_choice_as_seen(state: &mut LuaState) -> LuaResult<u32> {
    let choice_id = i32::from_stack(state, 1)?;
    borrow_state_mut(state)?
        .barber_shop
        .seen_choices
        .insert(choice_id);
    Ok(0)
}

fn mark_customization_option_as_seen(state: &mut LuaState) -> LuaResult<u32> {
    let option_id = i32::from_stack(state, 1)?;
    borrow_state_mut(state)?
        .barber_shop
        .seen_options
        .insert(option_id);
    Ok(0)
}

fn save_seen_choices(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn get_current_camera_zoom(state: &mut LuaState) -> LuaResult<u32> {
    let zoom = borrow_state(state)?.barber_shop.camera_zoom;
    state.push(Val::Num(zoom as f64));
    Ok(1)
}

fn set_camera_zoom_level(state: &mut LuaState) -> LuaResult<u32> {
    let zoom = f32::from_stack(state, 1)?;
    borrow_state_mut(state)?.barber_shop.camera_zoom = zoom;
    Ok(0)
}

fn zoom_camera(state: &mut LuaState) -> LuaResult<u32> {
    let delta = f32::from_stack(state, 1)?;
    borrow_state_mut(state)?.barber_shop.camera_zoom += delta;
    Ok(0)
}

fn rotate_camera(state: &mut LuaState) -> LuaResult<u32> {
    dispatch_event_now(state, "BARBER_SHOP_CAMERA_VALUES_UPDATED", &[])?;
    Ok(0)
}

fn reset_camera_rotation(state: &mut LuaState) -> LuaResult<u32> {
    dispatch_event_now(state, "BARBER_SHOP_CAMERA_VALUES_UPDATED", &[])?;
    Ok(0)
}

fn set_viewing_altered_form(state: &mut LuaState) -> LuaResult<u32> {
    let viewing = bool::from_stack(state, 1)?;
    borrow_state_mut(state)?.barber_shop.viewing_altered_form = viewing;
    dispatch_event_now(state, "BARBER_SHOP_FORCE_CUSTOMIZATIONS_UPDATE", &[])?;
    Ok(0)
}

fn set_viewing_shapeshift_form(state: &mut LuaState) -> LuaResult<u32> {
    let form_id = optional_int_from_stack(state, 1);
    borrow_state_mut(state)?.barber_shop.viewing_shapeshift_form = form_id;
    dispatch_event_now(state, "BARBER_SHOP_CAMERA_VALUES_UPDATED", &[])?;
    Ok(0)
}

fn set_viewing_chr_model(state: &mut LuaState) -> LuaResult<u32> {
    let chr_model_id = optional_int_from_stack(state, 1);
    borrow_state_mut(state)?.barber_shop.viewing_chr_model = chr_model_id;
    dispatch_event_now(state, "BARBER_SHOP_CAMERA_VALUES_UPDATED", &[])?;
    Ok(0)
}

fn set_model_dress_state(state: &mut LuaState) -> LuaResult<u32> {
    let dressed = bool::from_stack(state, 1)?;
    borrow_state_mut(state)?.barber_shop.model_dressed = dressed;
    Ok(0)
}

fn set_camera_distance_offset(state: &mut LuaState) -> LuaResult<u32> {
    let offset = f32::from_stack(state, 1)?;
    borrow_state_mut(state)?.barber_shop.camera_distance_offset = offset;
    Ok(0)
}

fn randomize_customization_choices(state: &mut LuaState) -> LuaResult<u32> {
    dispatch_event_now(state, "BARBER_SHOP_FORCE_CUSTOMIZATIONS_UPDATE", &[])?;
    Ok(0)
}

fn set_selected_sex(state: &mut LuaState) -> LuaResult<u32> {
    let sex_id = i32::from_stack(state, 1)?;
    {
        let mut sim = borrow_state_mut(state)?;
        if let Some(character) = sim.barber_shop.current_character.as_mut() {
            character.sex = sex_id;
        }
    }
    dispatch_event_now(state, "BARBER_SHOP_CAMERA_VALUES_UPDATED", &[])?;
    Ok(0)
}

fn build_character_data_table(state: &mut LuaState, data: &BarberShopCharacterData) -> Val {
    let name_val = create_string(state, &data.name);
    let file_name_val = create_string(state, &data.file_name);
    let icon_atlas_val = create_string(state, &data.create_screen_icon_atlas);
    let alternate_form_val = match &data.alternate_form_race {
        Some(form) => build_alternate_form_table(state, form),
        None => Val::Nil,
    };
    create_table_with_fields(
        state,
        &[
            ("name", name_val),
            ("fileName", file_name_val),
            ("alternateFormRaceData", alternate_form_val),
            ("createScreenIconAtlas", icon_atlas_val),
            ("sex", Val::Num(data.sex as f64)),
        ],
    )
}

fn build_alternate_form_table(state: &mut LuaState, form: &BarberShopAlternateFormRace) -> Val {
    let name_val = create_string(state, &form.name);
    let file_name_val = create_string(state, &form.file_name);
    let icon_atlas_val = create_string(state, &form.create_screen_icon_atlas);
    create_table_with_fields(
        state,
        &[
            ("raceID", Val::Num(form.race_id as f64)),
            ("name", name_val),
            ("fileName", file_name_val),
            ("createScreenIconAtlas", icon_atlas_val),
        ],
    )
}

fn build_category_table(state: &mut LuaState, category: &BarberShopCategory) -> Val {
    let name_val = create_string(state, &category.name);
    let options_val = build_options_table(state, &category.options);
    create_table_with_fields(state, &[("name", name_val), ("options", options_val)])
}

fn build_options_table(state: &mut LuaState, options: &[BarberShopOption]) -> Val {
    let table_val = create_table(state);
    let Val::Table(table_ref) = table_val else {
        unreachable!("create_table must return a table");
    };
    for (index, option) in options.iter().enumerate() {
        let entry = build_option_table(state, option);
        table_set_num(state, table_ref, (index + 1) as f64, entry);
    }
    table_val
}

fn build_option_table(state: &mut LuaState, option: &BarberShopOption) -> Val {
    let name_val = create_string(state, &option.name);
    create_table_with_fields(
        state,
        &[
            ("optionID", Val::Num(option.option_id as f64)),
            ("name", name_val),
            ("currentChoice", optional_id_val(option.current_choice_id)),
        ],
    )
}

fn optional_int_from_stack(state: &LuaState, slot: i32) -> Option<i32> {
    match i32::from_stack(state, slot) {
        Ok(value) => Some(value),
        Err(_) => None,
    }
}

fn optional_id_val(value: Option<i32>) -> Val {
    match value {
        Some(n) => Val::Num(n as f64),
        None => Val::Nil,
    }
}
