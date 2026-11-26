//! Temporary GlueParent/UIParent attribute repair.
//!
//! The simulator currently loads GlueParent outside the real glue-screen-only
//! context, so its `UIParent = self` alias can hide the panel-manager
//! attributes normally defined by UIParent XML.

use crate::lua_api::LoaderEnv;
#[cfg(test)]
use crate::lua_api::WowLuaEnv;

const GLUEPARENT_UIPARENT_ATTRIBUTES_LUA: &str = r#"
if UIParent and type(UIParent.SetAttribute) == "function" then
    UIParent:SetAttribute("DEFAULT_FRAME_WIDTH", 384)
    UIParent:SetAttribute("TOP_OFFSET", -116)
    UIParent:SetAttribute("LEFT_OFFSET", 16)
    UIParent:SetAttribute("CENTER_OFFSET", 384)
    UIParent:SetAttribute("RIGHT_OFFSET", 768)
    UIParent:SetAttribute("RIGHT_OFFSET_BUFFER", 80)
    UIParent:SetAttribute("PANEl_SPACING_X", 32)
end
"#;

pub(crate) fn patch(env: &LoaderEnv<'_>) -> Result<(), crate::Error> {
    env.exec(GLUEPARENT_UIPARENT_ATTRIBUTES_LUA)
}

#[cfg(test)]
fn patch_env(env: &WowLuaEnv) {
    env.exec(GLUEPARENT_UIPARENT_ATTRIBUTES_LUA)
        .expect("GlueParent UIParent attribute patch should install");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restores_panel_manager_attributes_on_ui_parent_alias() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            UIParent = {
                attrs = {},
                SetAttribute = function(self, key, value)
                    self.attrs[key] = value
                end,
            }
            "#,
        )
        .expect("UIParent fixture should install");

        patch_env(&env);

        let attrs: (i64, i64, i64, i64, i64, i64, i64) = env
            .eval(
                r#"
                return UIParent.attrs.DEFAULT_FRAME_WIDTH,
                    UIParent.attrs.TOP_OFFSET,
                    UIParent.attrs.LEFT_OFFSET,
                    UIParent.attrs.CENTER_OFFSET,
                    UIParent.attrs.RIGHT_OFFSET,
                    UIParent.attrs.RIGHT_OFFSET_BUFFER,
                    UIParent.attrs.PANEl_SPACING_X
                "#,
            )
            .expect("UIParent attributes should be restored");

        assert_eq!(attrs, (384, -116, 16, 384, 768, 80, 32));
    }

    #[test]
    fn ignores_missing_ui_parent() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec("UIParent = nil")
            .expect("UIParent fixture should be removable");

        patch_env(&env);

        let ui_parent_type: String = env
            .eval("return type(UIParent)")
            .expect("missing UIParent should be untouched");

        assert_eq!(ui_parent_type, "nil");
    }
}
