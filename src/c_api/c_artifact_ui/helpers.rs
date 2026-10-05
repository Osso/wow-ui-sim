//! Tuple/table builders shared between the action-bar and panel
//! `C_ArtifactUI` methods.

use crate::lua_api::methods::{
    call_function_state, create_string, create_table, create_table_with_fields, table_set,
    table_set_num,
};
use crate::lua_api::sim_substates::{
    ArtifactAppearanceInfo, ArtifactArtInfo, ArtifactPowerInfo, ColorRgb, RelicSlotInfo,
};
use rilua::Val;
use rilua::vm::state::LuaState;

/// Push the 13-value `ArtifactInfo` tuple shared by `GetArtifactInfo`
/// and `GetEquippedArtifactInfo`. Order matches the doc:
/// `(itemID, altItemID, name, icon, xp, pointsSpent, quality,
/// artifactAppearanceID, appearanceModID, itemAppearanceID,
/// altItemAppearanceID, altOnTop, tier)`.
pub(super) fn push_artifact_info_tuple(
    state: &mut LuaState,
    info: &crate::lua_api::state::ArtifactInfo,
) {
    let name_val = create_string(state, &info.name);
    let icon_val = create_string(state, &info.icon);
    state.push(Val::Num(info.item_id as f64));
    state.push(Val::Num(info.alt_item_id as f64));
    state.push(name_val);
    state.push(icon_val);
    state.push(Val::Num(info.total_xp as f64));
    state.push(Val::Num(info.points_spent as f64));
    state.push(Val::Num(info.quality as f64));
    state.push(Val::Num(info.artifact_appearance_id as f64));
    state.push(Val::Num(info.appearance_mod_id as f64));
    state.push(Val::Num(info.item_appearance_id as f64));
    state.push(Val::Num(info.alt_item_appearance_id as f64));
    state.push(Val::Bool(info.alt_on_top));
    state.push(Val::Num(info.tier as f64));
}

/// 13-value tuple returned by `GetAppearanceInfo` (and the tail of
/// `GetAppearanceInfoByID`, prefixed by `artifactAppearanceSetID`).
pub(super) fn push_appearance_info_tuple(
    state: &mut LuaState,
    appearance: &ArtifactAppearanceInfo,
) {
    let name_val = create_string(state, &appearance.name);
    let failure_val = match &appearance.failure_description {
        Some(s) => create_string(state, s),
        None => Val::Nil,
    };
    let alt_camera_val = match appearance.alt_hand_camera_id {
        Some(id) => Val::Num(id as f64),
        None => Val::Nil,
    };
    state.push(Val::Num(appearance.appearance_id as f64));
    state.push(name_val);
    state.push(Val::Num(appearance.display_index as f64));
    state.push(Val::Bool(appearance.unlocked));
    state.push(failure_val);
    state.push(Val::Num(appearance.ui_camera_id as f64));
    state.push(alt_camera_val);
    state.push(Val::Num(appearance.swatch_color.r as f64));
    state.push(Val::Num(appearance.swatch_color.g as f64));
    state.push(Val::Num(appearance.swatch_color.b as f64));
    state.push(Val::Num(appearance.model_opacity as f64));
    state.push(Val::Num(appearance.model_saturation as f64));
    state.push(Val::Bool(appearance.obtainable));
}

/// 4-value relic tuple returned by `GetRelicInfo` and
/// `GetRelicInfoByItemID`: `(name, icon, slotTypeName, link)`.
pub(super) fn push_relic_info_tuple(state: &mut LuaState, slot: &RelicSlotInfo) {
    let name_val = create_string(state, &slot.name);
    let icon_val = create_string(state, &slot.icon);
    let slot_type_val = create_string(state, &slot.slot_type);
    let link_val = create_string(state, &slot.link);
    state.push(name_val);
    state.push(icon_val);
    state.push(slot_type_val);
    state.push(link_val);
}

/// Push every integer in `ids` as a separate Lua return value.
pub(super) fn push_int_multireturn(state: &mut LuaState, ids: &[i32]) {
    for id in ids {
        state.push(Val::Num(*id as f64));
    }
}

/// Build a 1-based Lua sequence table from a slice of integers.
pub(super) fn create_int_sequence_table(state: &mut LuaState, ids: &[i32]) -> Val {
    let table_val = create_table(state);
    let Val::Table(table_ref) = table_val else {
        unreachable!("create_table must return a table");
    };
    for (index, id) in ids.iter().enumerate() {
        table_set_num(state, table_ref, (index + 1) as f64, Val::Num(*id as f64));
    }
    table_val
}

pub(super) fn build_artifact_art_info_table(state: &mut LuaState, art: &ArtifactArtInfo) -> Val {
    let saved_top = state.top;
    let table = create_table(state);
    state.push(table);
    let texture_kit_val = create_string(state, &art.texture_kit);
    let title_name_val = create_string(state, &art.title_name);
    table_set(state, table, "textureKit", texture_kit_val);
    table_set(state, table, "titleName", title_name_val);
    // Attach each callback result before the next CreateColor can collect it.
    for (key, color) in [
        ("titleColor", &art.title_color),
        ("barConnectedColor", &art.bar_connected_color),
        ("barDisconnectedColor", &art.bar_disconnected_color),
    ] {
        let value = create_color_mixin(state, color);
        table_set(state, table, key, value);
    }
    table_set(
        state,
        table,
        "uiModelSceneID",
        Val::Num(art.ui_model_scene_id as f64),
    );
    table_set(
        state,
        table,
        "spellVisualKitID",
        Val::Num(art.spell_visual_kit_id as f64),
    );
    state.top = saved_top;
    table
}

pub(super) fn build_power_info_table(state: &mut LuaState, power: &ArtifactPowerInfo) -> Val {
    let position_val = create_vector2_mixin(state, power.position.0, power.position.1);
    let offset_val = match power.offset {
        Some((x, y)) => create_vector2_mixin(state, x, y),
        None => Val::Nil,
    };
    let linear_index_val = match power.linear_index {
        Some(idx) => Val::Num(idx as f64),
        None => Val::Nil,
    };
    create_table_with_fields(
        state,
        &[
            ("spellID", Val::Num(power.spell_id as f64)),
            ("cost", Val::Num(power.cost as f64)),
            ("currentRank", Val::Num(power.current_rank as f64)),
            ("maxRank", Val::Num(power.max_rank as f64)),
            ("bonusRanks", Val::Num(power.bonus_ranks as f64)),
            (
                "numMaxRankBonusFromTier",
                Val::Num(power.num_max_rank_bonus_from_tier as f64),
            ),
            ("prereqsMet", Val::Bool(power.prereqs_met)),
            ("isStart", Val::Bool(power.is_start)),
            ("isGoldMedal", Val::Bool(power.is_gold_medal)),
            ("isFinal", Val::Bool(power.is_final)),
            ("tier", Val::Num(power.tier as f64)),
            ("position", position_val),
            ("offset", offset_val),
            ("linearIndex", linear_index_val),
        ],
    )
}

fn create_color_mixin(state: &mut LuaState, color: &ColorRgb) -> Val {
    let create_color_key = state.gc.intern_string(b"CreateColor");
    let create_color = state
        .gc
        .tables
        .get(state.global)
        .map(|globals| globals.get_str(create_color_key, &state.gc.string_arena))
        .unwrap_or(Val::Nil);
    let args = [
        Val::Num(color.r as f64),
        Val::Num(color.g as f64),
        Val::Num(color.b as f64),
        Val::Num(1.0),
    ];
    match call_function_state(state, create_color, &args) {
        Ok(v) => v,
        Err(_) => fallback_color_table(state, color),
    }
}

fn fallback_color_table(state: &mut LuaState, color: &ColorRgb) -> Val {
    create_table_with_fields(
        state,
        &[
            ("r", Val::Num(color.r as f64)),
            ("g", Val::Num(color.g as f64)),
            ("b", Val::Num(color.b as f64)),
            ("a", Val::Num(1.0)),
        ],
    )
}

fn create_vector2_mixin(state: &mut LuaState, x: f32, y: f32) -> Val {
    let table = create_table(state);
    let Val::Table(_) = table else {
        unreachable!("create_table must return a table");
    };
    table_set(state, table, "x", Val::Num(x as f64));
    table_set(state, table, "y", Val::Num(y as f64));
    table
}
