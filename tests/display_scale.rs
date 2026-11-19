#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use crate::common::{blizzard_ui_dir, load_blizzard_addon_lua_files};
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn original_apply_default_scale_reads_host_display_input() {
    let env = WowLuaEnv::new().unwrap();
    load_blizzard_addon_lua_files(
        &env,
        &blizzard_ui_dir(),
        "Blizzard_SharedXMLBase",
        &["FrameUtil.lua"],
    );
    env.set_display_scale_inputs(0.75, 0.625, 1.25).unwrap();
    let scale: f64 = env
        .eval(
            "local frame = CreateFrame('Frame'); ApplyDefaultScale(frame); return frame:GetScale()",
        )
        .unwrap();
    assert_eq!(scale, 0.75, "original ApplyDefaultScale ignored host input");
    let bounds: (f64, f64) = env
        .eval("return GetMinRenderScale(), GetMaxRenderScale()")
        .unwrap();
    assert_eq!(bounds, (0.625, 1.25));
}
