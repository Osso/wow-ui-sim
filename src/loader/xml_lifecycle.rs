//! Lifecycle script firing for XML-created frames (OnLoad, OnShow).

use crate::lua_api::LoaderEnv;

use super::helpers::lua_global_ref;

/// Fire OnLoad and OnShow lifecycle scripts after the frame is fully configured.
pub fn fire_lifecycle_scripts(env: &LoaderEnv<'_>, name: &str) {
    fire_lifecycle_onload(env, name);
    fire_lifecycle_onshow(env, name);
}

fn fire_lifecycle_onload(env: &LoaderEnv<'_>, name: &str) {
    let frame_ref = lua_global_ref(name);
    let code = format!(
        r#"
        local frame = {frame_ref}
        if type(frame.OnLoad_Intrinsic) == "function" then
            local ok, err = pcall(frame.OnLoad_Intrinsic, frame)
            if not ok then
                print("[OnLoad_Intrinsic] " .. tostring(err))
            end
        end
        local handler = frame:GetScript("OnLoad")
        if handler then
            local ok, err = pcall(handler, frame)
            if not ok then
                print("[OnLoad] " .. (frame.GetName and frame:GetName() or "{name}") .. ": " .. tostring(err))
            end
        end
        "#
    );
    if let Err(e) = env.exec(&code) {
        eprintln!("[OnLoad] {} error: {}", name, e);
    }
}

fn fire_lifecycle_onshow(env: &LoaderEnv<'_>, name: &str) {
    let frame_ref = lua_global_ref(name);
    let code = format!(
        r#"
        local frame = {frame_ref}
        if frame:IsVisible() then
            local handler = frame:GetScript("OnShow")
            if handler then
                local ok, err = pcall(handler, frame)
                if not ok then
                    print("[OnShow] " .. (frame.GetName and frame:GetName() or "{name}") .. ": " .. tostring(err))
                end
            end
            if type(frame.OnShow_Intrinsic) == "function" then
                local ok, err = pcall(frame.OnShow_Intrinsic, frame)
                if not ok then
                    print("[OnShow_Intrinsic] " .. tostring(err))
                end
            end
        end
        "#
    );
    if let Err(e) = env.exec(&code) {
        eprintln!("[OnShow] {} error: {}", name, e);
    }
}
