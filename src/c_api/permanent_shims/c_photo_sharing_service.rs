//! Permanent `C_PhotoSharing` service members.
//!
//! These talk to an external photo-sharing service: a third-party OAuth flow,
//! a screenshot capture, and an upload. The simulator has no network service
//! or world capture, so the flow calls are accepted and never progress (no
//! authorization, screenshot or upload-status event fires), the auth URL is
//! empty and the crop ratio is a fixed value. Account-link state itself
//! (`IsAuthorized`, `IsEnabled`, `GetStatus`, `ClearAuthorization`) is modeled
//! in `c_api::c_photo_sharing`.

use crate::lua_api::methods::create_string;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

/// INFERRED 16:9 preview crop; `Blizzard_PhotoSharing` sizes its preview
/// width as a fixed height times this ratio.
const CROP_RATIO: f64 = 16.0 / 9.0;

pub(crate) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    let functions: &[(&str, rilua::RustFn)] = &[
        ("BeginAuthorizationFlow", accept_service_request),
        ("CompleteAuthorizationFlow", accept_service_request),
        ("TakePhoto", accept_service_request),
        ("UploadPhotoToService", accept_service_request),
        ("SetScreenshotPreviewTexture", accept_service_request),
        ("GetPhotoSharingAuthURL", get_auth_url),
        ("GetCropRatio", get_crop_ratio),
    ];
    for &(name, function) in functions {
        table_set_rust_fn_static(state, namespace, name, function)?;
    }
    Ok(())
}

fn accept_service_request(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn get_auth_url(state: &mut LuaState) -> LuaResult<u32> {
    let url = create_string(state, "");
    state.push(url);
    Ok(1)
}

fn get_crop_ratio(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(CROP_RATIO));
    Ok(1)
}
