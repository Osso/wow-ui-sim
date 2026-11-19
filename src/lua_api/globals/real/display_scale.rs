//! Display scale getters backed by host display model inputs.

use crate::lua_api::methods::borrow_state;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

fn default_scale(lua: &mut LuaState) -> LuaResult<u32> {
    let value = borrow_state(lua)?.display_scale_inputs.0;
    lua.push(Val::Num(value));
    Ok(1)
}

fn min_render_scale(lua: &mut LuaState) -> LuaResult<u32> {
    let value = borrow_state(lua)?.display_scale_inputs.1;
    lua.push(Val::Num(value));
    Ok(1)
}

fn max_render_scale(lua: &mut LuaState) -> LuaResult<u32> {
    let value = borrow_state(lua)?.display_scale_inputs.2;
    lua.push(Val::Num(value));
    Ok(1)
}

pub(crate) fn register(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.register_function("GetDefaultScale", default_scale)?;
    lua.register_function("GetMinRenderScale", min_render_scale)?;
    lua.register_function("GetMaxRenderScale", max_render_scale)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_display_scale_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                if GetDefaultScale() ~= 1 then return "default" end
                if GetMinRenderScale() ~= 0.5 then return "min" end
                if GetMaxRenderScale() ~= 1.0 then return "max" end
                return "ok"
                "#,
            )
            .expect("display scale defaults probe should run");

        assert_eq!(result, "ok");
    }
}
