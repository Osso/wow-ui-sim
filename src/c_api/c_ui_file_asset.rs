//! `C_UIFileAsset` path/fileDataID helpers.
//!
//! Patch 12.0.7 exposes a small lookup surface for UI file assets. The
//! simulator recognizes shipped IDs from the bundled listfile and loose files
//! within the loader-selected addon directory.

use crate::c_api::helpers::ensure_namespace;
use crate::lua_api::methods::{borrow_state, val_to_string};
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};
use std::path::{Path, PathBuf};

pub(crate) fn register_c_ui_file_asset(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_UIFileAsset")?;
    table_set_rust_fn_static(state, table_ref, "GetFileID", c_ui_file_asset_get_file_id)?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "IsKnownFile",
        c_ui_file_asset_is_known_file,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "IsLooseFile",
        c_ui_file_asset_is_loose_file,
    )
}

fn c_ui_file_asset_get_file_id(state: &mut LuaState) -> LuaResult<u32> {
    match classify_asset(state) {
        Asset::FileId(file_id) => state.push(Val::Num(file_id as f64)),
        Asset::Loose | Asset::Missing => state.push(Val::Nil),
    }
    Ok(1)
}

fn c_ui_file_asset_is_known_file(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(!matches!(classify_asset(state), Asset::Missing)));
    Ok(1)
}

fn c_ui_file_asset_is_loose_file(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(matches!(classify_asset(state), Asset::Loose)));
    Ok(1)
}

#[derive(Clone, Copy)]
enum Asset {
    FileId(u32),
    Loose,
    Missing,
}

fn classify_asset(state: &LuaState) -> Asset {
    if let Some(file_id) = numeric_file_id_arg(state) {
        return Asset::FileId(file_id);
    }
    let Some(path) = val_to_string(state, stack_val(state, 1)) else {
        return Asset::Missing;
    };
    if let Some(file_id) = crate::limited_listfile::lookup_path(&path) {
        return Asset::FileId(file_id);
    }
    if is_selected_addon_file(state, &path) {
        Asset::Loose
    } else {
        Asset::Missing
    }
}

fn numeric_file_id_arg(state: &LuaState) -> Option<u32> {
    let file_id = u32::from_stack(state, 1).ok()?;
    (file_id > 0).then_some(file_id)
}

fn is_selected_addon_file(state: &LuaState, path: &str) -> bool {
    let normalized = path.replace('\\', "/");
    let parts: Vec<_> = normalized.split('/').collect();
    if parts.len() < 4
        || !parts[0].eq_ignore_ascii_case("Interface")
        || !parts[1].eq_ignore_ascii_case("AddOns")
        || parts
            .iter()
            .any(|part| part.is_empty() || *part == "." || *part == ".." || part.contains(':'))
    {
        return false;
    }
    let root = {
        let Ok(sim) = borrow_state(state) else {
            return false;
        };
        sim.addons
            .iter()
            .find(|addon| addon.folder_name.eq_ignore_ascii_case(parts[2]))
            .and_then(|addon| addon.addon_dir.clone())
    };
    let Some(root) = root else {
        return false;
    };
    let Ok(canonical_root) = root.canonicalize() else {
        return false;
    };
    let relative = &parts[3..];
    let base = relative.join("/");
    let mut candidates = vec![base.clone()];
    if !relative.last().is_some_and(|name| name.contains('.')) {
        candidates.extend(["blp", "tga", "png"].map(|ext| format!("{base}.{ext}")));
    }
    candidates
        .iter()
        .any(|candidate| selected_file_exists(&root, &canonical_root, candidate))
}

fn selected_file_exists(root: &Path, canonical_root: &Path, candidate: &str) -> bool {
    let mut file = PathBuf::from(root);
    for part in candidate.split('/') {
        let Ok(entries) = std::fs::read_dir(&file) else {
            return false;
        };
        let Some(entry) = entries.filter_map(Result::ok).find(|entry| {
            entry
                .file_name()
                .to_str()
                .is_some_and(|name| name.eq_ignore_ascii_case(part))
        }) else {
            return false;
        };
        file.push(entry.file_name());
    }
    file.canonicalize()
        .is_ok_and(|resolved| resolved.starts_with(canonical_root) && resolved.is_file())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn ui_file_asset_uses_limited_listfile_paths() {
        let env = WowLuaEnv::new().expect("env");
        let result: String = env
            .eval(
                r#"
                if C_UIFileAsset.GetFileID(123) ~= 123 then return "numeric" end
                if C_UIFileAsset.GetFileID("Interface\\Icons\\Trade_Engineering.blp") ~= 136243 then return "path" end
                if C_UIFileAsset.IsKnownFile("Interface/Icons/Trade_Engineering.blp") ~= true then return "known" end
                if C_UIFileAsset.GetFileID("Interface/Unknown") ~= nil then return "unknown-id" end
                if C_UIFileAsset.IsKnownFile("Interface/Unknown") ~= false then return "unknown-known" end
                if C_UIFileAsset.IsLooseFile("Interface/Icons/Trade_Engineering.blp") ~= false then return "loose" end
                return "ok"
                "#,
            )
            .expect("probe should run");
        assert_eq!(result, "ok");
    }
}
