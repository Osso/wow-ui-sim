//! Minimal `C_EncounterWarnings` surface for Edit Mode warning previews.

use super::ensure_namespace;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::LuaResult;
use rilua::vm::state::LuaState;

pub(super) fn register_encounter_warnings_surface(state: &mut LuaState) -> LuaResult<()> {
    let ns = ensure_namespace(state, "C_EncounterWarnings")?;
    #[cfg(feature = "retail-12-1-5")]
    crate::c_api::c_encounter_warnings::register_preview(state, ns)?;
    #[cfg(not(feature = "retail-12-1-5"))]
    table_set_rust_fn_static(
        state,
        ns,
        "GetEditModeWarningInfo",
        legacy::get_edit_mode_warning_info,
    )?;
    table_set_rust_fn_static(state, ns, "PlaySound", play_sound)?;
    Ok(())
}

fn play_sound(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

#[cfg(not(feature = "retail-12-1-5"))]
mod legacy {
    use super::*;
    use crate::lua_api::globals::font_strings_collection::colors::make_rilua_color_table;
    use crate::lua_api::methods::{create_table, table_set};
    use crate::lua_bridge::FromStack;
    use rilua::Val;

    pub(super) fn get_edit_mode_warning_info(state: &mut LuaState) -> LuaResult<u32> {
        let severity = f64::from_stack(state, 1).ok().unwrap_or(0.0);
        let info = create_table(state);
        let text = preview_text(state, severity);
        let caster_name = preview_text(state, "");
        let color = preview_color(state, severity)?;
        table_set(state, info, "severity", Val::Num(severity));
        table_set(state, info, "text", text);
        table_set(state, info, "iconFileID", Val::Num(136122.0));
        table_set(state, info, "isDeadly", Val::Bool(severity >= 3.0));
        table_set(state, info, "duration", Val::Num(30.0));
        table_set(state, info, "shouldShowWarning", Val::Bool(true));
        table_set(state, info, "shouldPlaySound", Val::Bool(false));
        table_set(state, info, "shouldShowChatMessage", Val::Bool(false));
        table_set(state, info, "casterName", caster_name);
        table_set(state, info, "color", color);
        state.push(info);
        Ok(1)
    }

    fn preview_text(state: &mut LuaState, severity: impl Into<PreviewText>) -> Val {
        let text = match severity.into() {
            PreviewText::Severity(severity) if severity >= 3.0 => "Critical Encounter Warning",
            PreviewText::Severity(severity) if severity >= 2.0 => "Important Encounter Warning",
            PreviewText::Severity(_) => "Encounter Warning",
            PreviewText::Literal(text) => text,
        };
        crate::lua_api::methods::create_string(state, text)
    }

    fn preview_color(state: &mut LuaState, severity: f64) -> LuaResult<Val> {
        if severity >= 3.0 {
            return make_rilua_color_table(state, 1.0, 0.15, 0.05, 1.0);
        }
        if severity >= 2.0 {
            return make_rilua_color_table(state, 1.0, 0.75, 0.1, 1.0);
        }
        make_rilua_color_table(state, 1.0, 1.0, 1.0, 1.0)
    }

    enum PreviewText {
        Severity(f64),
        Literal(&'static str),
    }

    impl From<f64> for PreviewText {
        fn from(value: f64) -> Self {
            Self::Severity(value)
        }
    }

    impl From<&'static str> for PreviewText {
        fn from(value: &'static str) -> Self {
            Self::Literal(value)
        }
    }
}

#[cfg(all(test, not(feature = "retail-12-1-5")))]
mod tests {
    use super::*;
    use crate::lua_api::WowLuaEnv;
    use rilua::LuaApiMut;

    #[test]
    fn edit_mode_warning_info_has_preview_shape() {
        let env = WowLuaEnv::new().expect("env");
        {
            let mut lua = env.rilua_mut();
            let state = lua.state_mut();
            register_encounter_warnings_surface(state).expect("register surface");
        }

        let (text, r, should_show): (String, f64, bool) = env
            .eval(
                r#"
                local info = C_EncounterWarnings.GetEditModeWarningInfo(3)
                local r = info.color:GetRGB()
                return info.text, r, info.shouldShowWarning
                "#,
            )
            .expect("warning info");

        assert_eq!(text, "Critical Encounter Warning");
        assert_eq!(r, 1.0);
        assert!(should_show);
    }
}
