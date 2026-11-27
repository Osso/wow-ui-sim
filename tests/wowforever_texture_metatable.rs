//! Forever texture metatable access through the actual Blizzard consumer.

#[test]
#[cfg(feature = "client-wowforever")]
fn wowforever_texture_metatable_loads_unit_frame_util() {
    use crate::common::blizzard_addon_harness::{
        blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
    };
    // UnitFrameUtil.lua:327 reads CVarCallbackRegistry, published by the
    // Blizzard_SharedXMLBase TOC dependency (CvarUtil.lua:114).
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_SharedXMLBase"], &[]);
    let root = ui.join("Blizzard_UnitFrame");
    let style =
        std::fs::read_to_string(root.join("Camelot/UnitFramePvPIndicatorStyle.lua")).unwrap();
    let source = std::fs::read_to_string(root.join("Shared/UnitFrameUtil.lua")).unwrap();
    // Match the Forever TOC order and pass the same addon-local table to both files.
    env.exec(&format!(
        r#"
        local addonTable = {{}}
        assert(loadstring([==[{style}]==], '@Blizzard_UnitFrame/Camelot/UnitFramePvPIndicatorStyle.lua'))('Blizzard_UnitFrame', addonTable)
        assert(loadstring([==[{source}]==], '@Blizzard_UnitFrame/Shared/UnitFrameUtil.lua'))('Blizzard_UnitFrame', addonTable)
        "#,
    ))
    .unwrap();
    env.exec(
        r#"
        local texture = CreateFrame("Frame"):CreateTexture()
        local meta = GetTextureMetatable()
        assert(meta == getmetatable(texture), "factory texture metatable identity")
        assert(meta == GetTextureMetatable(), "repeated metatable identity")
        assert(meta == GetTextureMetatable(texture), "explicit texture identity")
        assert(meta.__index.IsObjectType(texture, "Texture"), "real texture type")
        assert(not meta.__index.IsObjectType(texture, "FontString"), "non-font type")
        meta.__index.SetTexture(texture, 134400)
        assert(texture:GetTexture() == 134400, "texture method mutation")
        local copied = CopyTable(meta.__index)
        assert(copied ~= meta.__index, "independent copied index")
        assert(copied.IsObjectType(texture, "Texture"), "copied method dispatch")
        "#,
    )
    .unwrap();
}

#[test]
#[cfg(not(any(feature = "retail-12-1-5", feature = "client-wowforever")))]
fn wowforever_texture_metatable_is_not_published_elsewhere() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec("assert(GetTextureMetatable == nil)").unwrap();
}
