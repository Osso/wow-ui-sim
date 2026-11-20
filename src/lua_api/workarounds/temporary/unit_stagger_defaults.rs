//! Unit stagger compatibility for profiles without modeled combat state.

const UNIT_STAGGER_DEFAULTS_LUA: &str = r#"
if UnitStagger == nil then
  function UnitStagger(_unit)
    return 0
  end
end

C_PaperDollInfo = C_PaperDollInfo or __wow_namespace()

if rawget(C_PaperDollInfo, "GetStaggerPercentage") == nil then
  function C_PaperDollInfo.GetStaggerPercentage(_unit)
    return 0
  end
end
"#;

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    rilua::LuaApiMut::register_function(lua, "UnitStagger", unit_stagger)?;
    Ok(())
}

#[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(UNIT_STAGGER_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn unit_stagger(state: &mut rilua::vm::state::LuaState) -> rilua::LuaResult<u32> {
    use crate::lua_bridge::FromStack;

    let unit = String::from_stack(state, 1)?;
    let amount = if matches!(unit.as_str(), "player" | "self") {
        crate::lua_api::methods::borrow_state(state)?
            .player
            .stagger_amount
    } else {
        0.0
    };
    state.push(rilua::Val::Num(amount));
    Ok(1)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) fn set_player_stagger(state: &mut rilua::vm::state::LuaState) -> rilua::LuaResult<u32> {
    use crate::lua_bridge::FromStack;

    let amount = f64::from_stack(state, 1)?;
    crate::lua_api::methods::borrow_state_mut(state)?
        .player
        .stagger_amount = amount;
    Ok(0)
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
    #[test]
    fn installs_unit_stagger_default() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let stagger: i32 = env
            .eval(r#"return UnitStagger("player")"#)
            .expect("UnitStagger probe should run");

        assert_eq!(stagger, 0);
    }

    #[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
    #[test]
    fn installs_paper_doll_stagger_default() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let (stagger, against_target_is_nil): (f64, bool) = env
            .eval(
                r#"
                local stagger, staggerAgainstTarget = C_PaperDollInfo.GetStaggerPercentage("player")
                return stagger, staggerAgainstTarget == nil
                "#,
            )
            .expect("PaperDoll stagger default should be callable");

        assert_eq!(stagger, 0.0);
        assert!(against_target_is_nil);
    }

    #[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
    #[test]
    fn preserves_existing_paper_doll_stagger_provider() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            C_PaperDollInfo = C_PaperDollInfo or __wow_namespace()

            function C_PaperDollInfo.GetStaggerPercentage(_unit)
                return 42, 17
            end
            "#,
        )
        .expect("fixture should install existing PaperDoll stagger provider");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let result: (i32, i32) = env
            .eval(r#"return C_PaperDollInfo.GetStaggerPercentage("player")"#)
            .expect("existing PaperDoll stagger provider should remain callable");

        assert_eq!(result, (42, 17));
    }
}
