//! Profile-specific publication of the removed C_TableUtil function.

use crate::lua_api::WowLuaEnv;

#[test]
fn patch_12_1_5_table_util_removal() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    let kind: String = env
        .eval("return type(C_TableUtil.FindIndexedMismatch)")
        .expect("function lookup should evaluate");
    #[cfg(feature = "retail-12-1-5")]
    assert_eq!(kind, "nil");
    #[cfg(not(feature = "retail-12-1-5"))]
    {
        assert_eq!(kind, "function");
        let mismatch: i32 = env
            .eval("return C_TableUtil.FindIndexedMismatch({10, 20, 30}, {10, 99, 30})")
            .expect("existing mismatch comparison should evaluate");
        assert_eq!(mismatch, 2);
    }
}
