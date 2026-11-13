//! `C_UIFileAsset` path/fileDataID helpers.
//!
//! Under 12.0.7, GetFileID resolves bundled paths while known/loose queries
//! read explicit host catalogs. Loaded-addon path registration is INFERRED;
//! complete native catalog acquisition and table identity remain UNKNOWN.
//! Earlier epochs retain their separate loader-selected filesystem contract.

use crate::c_api::helpers::ensure_namespace;
use crate::lua_api::methods::{borrow_state, val_to_string};
#[cfg(not(feature = "retail-12-0-7"))]
use crate::lua_bridge::FromStack;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
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

/// INFERRED: scanning an addon's directory seeds known loose-path markers before its Lua loads.
#[cfg(feature = "retail-12-0-7")]
pub(crate) fn register_loaded_addon_files(
    state: &mut crate::lua_api::state::SimState,
    addon_name: &str,
    addon_root: &Path,
) -> std::io::Result<()> {
    let entries = match std::fs::read_dir(addon_root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    let mut asset_paths = Vec::new();
    append_addon_asset_paths(addon_root, Path::new(""), entries, &mut asset_paths)?;
    let addon_prefix = Path::new("Interface").join("AddOns").join(addon_name);
    for relative_path in asset_paths {
        let path = addon_prefix.join(relative_path);
        let path = path.to_str().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "loaded addon asset path is not UTF-8",
            )
        })?;
        state
            .known_loose_asset_paths
            .insert(normalize_loose_asset_path(path));
    }
    Ok(())
}

#[cfg(feature = "retail-12-0-7")]
fn append_addon_asset_paths(
    addon_root: &Path,
    relative_directory: &Path,
    entries: std::fs::ReadDir,
    asset_paths: &mut Vec<PathBuf>,
) -> std::io::Result<()> {
    for entry in entries {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let relative_path = relative_directory.join(entry.file_name());
        if file_type.is_dir() {
            let child_entries = std::fs::read_dir(addon_root.join(&relative_path))?;
            append_addon_asset_paths(addon_root, &relative_path, child_entries, asset_paths)?;
        } else if file_type.is_file() {
            asset_paths.push(relative_path);
        }
    }
    Ok(())
}

/// INFERRED: the result is public and authenticated extra arguments are ignored.
#[cfg(feature = "retail-12-0-7")]
fn c_ui_file_asset_get_file_id(state: &mut LuaState) -> LuaResult<u32> {
    let asset = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    for value in state.stack.iter().take(state.top).skip(state.base + 1) {
        rilua::table_security::unwrap_secret(state, *value)?;
    }
    let file_id = resolve_authenticated_asset_id(state, asset)?;
    state.push(file_id.map_or(Val::Nil, |id| Val::Num(id as f64)));
    Ok(1)
}

#[cfg(feature = "retail-12-0-7")]
fn resolve_authenticated_asset_id(state: &LuaState, asset: Val) -> LuaResult<Option<u32>> {
    // INFERRED positive integral u32 domain, nil for numeric boundary misses.
    let file_id = match asset {
        Val::Num(number) => {
            let is_integer = number.is_finite() && number.fract() == 0.0;
            let in_domain = (1.0..=u32::MAX as f64).contains(&number);
            (is_integer && in_domain).then_some(number as u32)
        }
        Val::Str(_) => {
            let path = val_to_string(state, asset).ok_or_else(|| {
                rilua::runtime_error("C_UIFileAsset.GetFileID: invalid asset string")
            })?;
            crate::limited_listfile::lookup_path(&path)
        }
        _ => {
            return Err(rilua::runtime_error(
                "C_UIFileAsset.GetFileID: asset must be a number or path string",
            ));
        }
    };
    Ok(file_id)
}

#[cfg(not(feature = "retail-12-0-7"))]
fn c_ui_file_asset_get_file_id(state: &mut LuaState) -> LuaResult<u32> {
    match file_id_from_asset_arg(state) {
        Some(file_id) => state.push(Val::Num(file_id as f64)),
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn c_ui_file_asset_is_known_file(state: &mut LuaState) -> LuaResult<u32> {
    let asset = query_asset(state)?;
    state.push(Val::Bool(asset.is_known));
    Ok(1)
}

fn c_ui_file_asset_is_loose_file(state: &mut LuaState) -> LuaResult<u32> {
    let asset = query_asset(state)?;
    state.push(Val::Bool(asset.is_loose));
    Ok(1)
}

#[derive(Default)]
struct AssetStatus {
    is_known: bool,
    is_loose: bool,
}

/// INFERRED public boolean, positive integral u32 domain and ignored authenticated extras.
#[cfg(feature = "retail-12-0-7")]
fn query_asset(state: &LuaState) -> LuaResult<AssetStatus> {
    let asset = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    for value in state.stack.iter().take(state.top).skip(state.base + 1) {
        rilua::table_security::unwrap_secret(state, *value)?;
    }
    // No validation can mask secret denial in any original argument or extra.
    if !matches!(asset, Val::Str(_)) {
        let file_id = resolve_authenticated_asset_id(state, asset)?;
        let sim = borrow_state(state)?;
        let is_known =
            file_id.is_some_and(|id| id > 0 && sim.known_shipped_asset_ids.contains(&id));
        return Ok(AssetStatus {
            is_known,
            is_loose: false,
        });
    }

    let path = val_to_string(state, asset)
        .ok_or_else(|| rilua::runtime_error("C_UIFileAsset.GetFileID: invalid asset string"))?;
    let sim = borrow_state(state)?;
    let normalized_path = normalize_native_asset_path(&path);
    for candidate in [
        normalized_path.clone(),
        retry_blp_asset_path(&normalized_path),
    ] {
        let normalized_loose_path = normalize_loose_asset_path(&candidate);
        let is_loose = sim.known_loose_asset_paths.contains(&normalized_loose_path);
        let file_id = crate::limited_listfile::lookup_path(&candidate);
        if let Some(file_id) = file_id {
            if file_id == 0 {
                return Ok(AssetStatus::default());
            }
            let is_known_shipped = sim.known_shipped_asset_ids.contains(&file_id);
            return Ok(AssetStatus {
                is_known: is_known_shipped || is_loose,
                is_loose,
            });
        }
        if is_loose {
            return Ok(AssetStatus {
                is_known: true,
                is_loose: true,
            });
        }
    }
    Ok(AssetStatus::default())
}

#[cfg(feature = "retail-12-0-7")]
fn normalize_native_asset_path(path: &str) -> String {
    path.replace('/', "\\").to_ascii_uppercase()
}

#[cfg(feature = "retail-12-0-7")]
fn normalize_loose_asset_path(path: &str) -> String {
    path.replace('\\', "/").to_ascii_lowercase()
}

#[cfg(feature = "retail-12-0-7")]
fn retry_blp_asset_path(path: &str) -> String {
    let file_name_start = path.rfind('\\').map_or(0, |index| index + 1);
    let file_name = &path[file_name_start..];
    if let Some(extension_start) = file_name.rfind('.') {
        format!("{}.BLP", &path[..file_name_start + extension_start])
    } else {
        format!("{path}.BLP")
    }
}

#[cfg(not(feature = "retail-12-0-7"))]
fn query_asset(state: &LuaState) -> LuaResult<AssetStatus> {
    if file_id_from_asset_arg(state).is_some() {
        return Ok(AssetStatus {
            is_known: true,
            is_loose: false,
        });
    }
    let Some(path) = val_to_string(state, stack_val(state, 1)) else {
        return Ok(AssetStatus::default());
    };
    let is_loose = query_selected_addon_file(state, &path);
    Ok(AssetStatus {
        is_known: is_loose,
        is_loose,
    })
}

#[cfg(not(feature = "retail-12-0-7"))]
fn file_id_from_asset_arg(state: &LuaState) -> Option<u32> {
    if let Some(file_id) = numeric_file_id_arg(state) {
        return Some(file_id);
    }
    let path = val_to_string(state, stack_val(state, 1))?;
    crate::limited_listfile::lookup_path(&path)
}

#[cfg(not(feature = "retail-12-0-7"))]
fn numeric_file_id_arg(state: &LuaState) -> Option<u32> {
    let file_id = u32::from_stack(state, 1).ok()?;
    (file_id > 0).then_some(file_id)
}

#[cfg(not(feature = "retail-12-0-7"))]
fn query_selected_addon_file(state: &LuaState, path: &str) -> bool {
    let normalized = path.replace('\\', "/");
    let Some(parts) = valid_addon_parts(&normalized) else {
        return false;
    };
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
    texture_candidates(&parts[3..])
        .iter()
        .any(|candidate| read_selected_addon_file(&root, &canonical_root, candidate))
}

#[cfg(not(feature = "retail-12-0-7"))]
fn valid_addon_parts(path: &str) -> Option<Vec<&str>> {
    let parts: Vec<_> = path.split('/').collect();
    let valid_prefix = parts.len() >= 4
        && parts[0].eq_ignore_ascii_case("Interface")
        && parts[1].eq_ignore_ascii_case("AddOns");
    let valid_components = parts
        .iter()
        .all(|part| !part.is_empty() && *part != "." && *part != ".." && !part.contains(':'));
    (valid_prefix && valid_components).then_some(parts)
}

#[cfg(not(feature = "retail-12-0-7"))]
fn texture_candidates(relative: &[&str]) -> Vec<String> {
    let base = relative.join("/");
    let mut candidates = vec![base.clone()];
    if !relative.last().is_some_and(|name| name.contains('.')) {
        candidates.extend(["blp", "tga", "png"].map(|ext| format!("{base}.{ext}")));
    }
    candidates
}

#[cfg(not(feature = "retail-12-0-7"))]
fn read_selected_addon_file(root: &Path, canonical_root: &Path, candidate: &str) -> bool {
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
        #[cfg(feature = "retail-12-0-7")]
        env.state()
            .borrow_mut()
            .known_shipped_asset_ids
            .insert(136243);
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

    #[cfg(feature = "retail-12-0-7")]
    #[test]
    fn known_file_normalizes_paths_and_retries_blp() {
        assert_eq!(
            crate::limited_listfile::lookup_path("Interface/Icons/Trade_Engineering.tga"),
            None
        );
        assert_eq!(
            crate::limited_listfile::lookup_path("Interface/Icons/Trade_Engineering.blp"),
            Some(136243)
        );

        let env = WowLuaEnv::new().expect("env");
        env.state()
            .borrow_mut()
            .known_shipped_asset_ids
            .insert(136243);
        let result: String = env
            .eval(
                r#"
                if C_UIFileAsset.IsKnownFile("iNterFace\\iCons\\TRADE_Engineering.blp") ~= true then return "normalization" end
                if C_UIFileAsset.IsKnownFile("Interface/Icons/Trade_Engineering.tga") ~= true then return "blp-retry" end
                return "ok"
                "#,
            )
            .expect("queries should run");

        assert_eq!(result, "ok");
    }
}
