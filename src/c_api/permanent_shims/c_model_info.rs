//! C_ModelInfo permanent shim — 3D model scene rendering is out of scope.

use crate::lua_api::methods::{create_string, create_table, table_set_static};
use crate::lua_bridge::{FromStack, table_set_rust_fn, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

use crate::c_api::set_global_val;

pub fn register_c_model_info(state: &mut LuaState) -> LuaResult<()> {
    let t = create_table(state);
    let Val::Table(t_ref) = t else {
        unreachable!("create_table must return a table");
    };
    register_model_scene_stubs(state, t_ref)?;
    table_set_rust_fn_static(
        state,
        t_ref,
        "GetModelSceneInfoByID",
        c_model_info_get_model_scene_info_by_id,
    )?;
    table_set_rust_fn_static(
        state,
        t_ref,
        "GetModelSceneActorInfoByID",
        c_model_info_get_model_scene_actor_info_by_id,
    )?;
    table_set_rust_fn_static(
        state,
        t_ref,
        "GetModelSceneCameraInfoByID",
        c_model_info_get_model_scene_camera_info_by_id,
    )?;
    set_global_val(state, "C_ModelInfo", t);
    Ok(())
}

fn register_model_scene_stubs(state: &mut LuaState, t_ref: GcRef<Table>) -> LuaResult<()> {
    const NOOPS: &[&str] = &[
        "AddActiveModelScene",
        "AddActiveModelSceneActor",
        "ClearActiveModelScene",
        "ClearActiveModelSceneActor",
    ];
    const EMPTY_TABLE_GETTERS: &[&str] = &["GetModelSceneActorDisplayInfoByID"];
    for name in NOOPS {
        table_set_rust_fn_static(state, t_ref, name, |_state| Ok(0))?;
    }
    for name in EMPTY_TABLE_GETTERS {
        table_set_rust_fn(state, t_ref, name, empty_table_result)?;
    }
    Ok(())
}

fn c_model_info_get_model_scene_info_by_id(state: &mut LuaState) -> LuaResult<u32> {
    let scene_id = i64::from_stack(state, 1)?;
    let Some(&(_, scene_type)) = sim_data::model_scenes::SCENES
        .iter()
        .find(|(id, _)| *id == scene_id)
    else {
        state.push(Val::Nil);
        return Ok(1);
    };

    let camera_ids = create_table(state);
    let actor_ids = create_table(state);
    for (index, &(id, _)) in sim_data::model_scenes::CAMERAS
        .iter()
        .filter(|(_, scene)| *scene == scene_id)
        .enumerate()
    {
        set_array_number(state, camera_ids, index + 1, id as f64);
    }
    for (index, &(id, _, _)) in sim_data::model_scenes::ACTORS
        .iter()
        .filter(|(_, scene, _)| *scene == scene_id)
        .enumerate()
    {
        set_array_number(state, actor_ids, index + 1, id as f64);
    }

    state.push(Val::Num(scene_type as f64));
    state.push(camera_ids);
    state.push(actor_ids);
    state.push(Val::Num(0.0));
    Ok(4)
}

fn c_model_info_get_model_scene_actor_info_by_id(state: &mut LuaState) -> LuaResult<u32> {
    let actor_id = i64::from_stack(state, 1)?;
    let Some(&(_, _, tag)) = sim_data::model_scenes::ACTORS
        .iter()
        .find(|(id, _, _)| *id == actor_id)
    else {
        state.push(Val::Nil);
        return Ok(1);
    };

    let info = create_table(state);
    let tag = create_string(state, tag);
    table_set_static(state, info, "scriptTag", tag);
    table_set_static(state, info, "modelActorID", Val::Num(actor_id as f64));
    state.push(info);
    Ok(1)
}

fn c_model_info_get_model_scene_camera_info_by_id(state: &mut LuaState) -> LuaResult<u32> {
    let _camera_id = i64::from_stack(state, 1)?;
    state.push(Val::Nil);
    Ok(1)
}

fn set_array_number(state: &mut LuaState, table: Val, index: usize, value: f64) {
    let Val::Table(table_ref) = table else { return };
    let Some(table) = state.gc.tables.get_mut(table_ref) else {
        return;
    };
    let _ = table.raw_set(
        Val::Num(index as f64),
        Val::Num(value),
        &state.gc.string_arena,
    );
    state.gc.barrier_back(table_ref);
}

fn empty_table_result(state: &mut LuaState) -> LuaResult<u32> {
    let table = create_table(state);
    state.push(table);
    Ok(1)
}
