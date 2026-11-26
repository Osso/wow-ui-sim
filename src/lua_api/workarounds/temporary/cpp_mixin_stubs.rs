//! Temporary simulator behavior for mixins whose native implementation is absent from Lua.
//!
//! ModelSceneControlButtonMixin.OnLoad is inferred native behavior, and the
//! PetActionBar update guard preserves simulator behavior for empty action bars.
//! Apply after each loaded Lua file so later XML sees the required methods.

use crate::lua_api::LoaderEnv;

const CPP_MIXIN_STUBS_LUA: &str = r#"
local ModelSceneControlButtonMixin = rawget(_G, "ModelSceneControlButtonMixin")
if ModelSceneControlButtonMixin and not ModelSceneControlButtonMixin.OnLoad then
    ModelSceneControlButtonMixin.OnLoad = function() end
end

local PetActionBarMixin = rawget(_G, "PetActionBarMixin")
if PetActionBarMixin and PetActionBarMixin.Update and not PetActionBarMixin._update_guarded then
    PetActionBarMixin._update_guarded = true
    local origUpdate = PetActionBarMixin.Update
    PetActionBarMixin.Update = function(self)
        if not self.actionButtons or #self.actionButtons == 0 then return end
        return origUpdate(self)
    end
end
"#;

pub(crate) fn patch_after_lua_file(env: &LoaderEnv<'_>) -> Result<(), crate::Error> {
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::WowForever
    ) {
        return Ok(());
    }
    env.exec(CPP_MIXIN_STUBS_LUA)
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    use super::*;

    #[test]
    fn installs_missing_model_scene_onload_stubs() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            ModelSceneControlButtonMixin = {}
            "#,
        )
        .expect("fixture should install");

        patch_after_lua_file(&env.loader_env()).expect("patch should apply");

        let model_scene_onload_is_callable: bool = env
            .eval("return type(ModelSceneControlButtonMixin.OnLoad) == 'function'")
            .expect("ModelSceneControlButtonMixin.OnLoad should be readable");
        assert!(model_scene_onload_is_callable);
    }

    #[test]
    fn guards_pet_action_bar_update_until_buttons_exist() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            PetActionBarMixin = {
                calls = 0,
                Update = function(self)
                    self.calls = self.calls + 1
                    return "updated"
                end,
            }
            "#,
        )
        .expect("fixture should install");

        patch_after_lua_file(&env.loader_env()).expect("patch should apply");

        let result: (i32, String, i32, bool) = env
            .eval(
                r#"
                local emptyBar = { actionButtons = {} }
                setmetatable(emptyBar, { __index = PetActionBarMixin })
                emptyBar:Update()

                local activeBar = { actionButtons = { "button" } }
                setmetatable(activeBar, { __index = PetActionBarMixin })
                local activeResult = activeBar:Update()

                local alreadyGuarded = PetActionBarMixin._update_guarded
                return emptyBar.calls or 0, activeResult, activeBar.calls, alreadyGuarded
                "#,
            )
            .expect("guarded update should run");
        assert_eq!(result, (0, "updated".to_string(), 1, true));
    }
}
