//! Numeric script-object update modes from SimpleFrameScriptObjectConstants.

use rilua::{LuaResult, Val, runtime_error};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum OnUpdateMode {
    Disabled = 0,
    #[default]
    RunWhenVisible = 1,
    RunWhenVisibleOnce = 2,
    RunOnce = 3,
    RunAlways = 4,
}

impl OnUpdateMode {
    pub(crate) fn from_value(value: Val) -> LuaResult<Self> {
        match value {
            Val::Num(0.0) => Ok(Self::Disabled),
            Val::Num(1.0) => Ok(Self::RunWhenVisible),
            Val::Num(2.0) => Ok(Self::RunWhenVisibleOnce),
            Val::Num(3.0) => Ok(Self::RunOnce),
            Val::Num(4.0) => Ok(Self::RunAlways),
            _ => Err(runtime_error(
                "invalid OnUpdateMode: expected an integer from 0 to 4",
            )),
        }
    }

    pub(crate) fn from_stored_value(value: Val) -> LuaResult<Self> {
        if matches!(value, Val::Nil) {
            Ok(Self::default())
        } else {
            Self::from_value(value)
        }
    }

    pub(crate) fn from_xml_name(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "disabled" => Some(Self::Disabled),
            "runwhenvisible" => Some(Self::RunWhenVisible),
            "runwhenvisibleonce" => Some(Self::RunWhenVisibleOnce),
            "runonce" => Some(Self::RunOnce),
            "runalways" => Some(Self::RunAlways),
            _ => None,
        }
    }

    pub(crate) fn value(self) -> Val {
        Val::Num(self as u8 as f64)
    }

    pub(crate) fn is_one_shot(self) -> bool {
        matches!(self, Self::RunWhenVisibleOnce | Self::RunOnce)
    }

    pub(crate) fn requires_visibility(self) -> bool {
        matches!(self, Self::RunWhenVisible | Self::RunWhenVisibleOnce)
    }
}

#[cfg(feature = "on-update-modes")]
pub(crate) fn register(state: &mut rilua::vm::state::LuaState) {
    use crate::lua_api::methods::{create_table, table_set};

    let enums = super::helpers::ensure_global_table(state, "Enum");
    let modes = create_table(state);
    for (name, mode) in [
        ("Disabled", OnUpdateMode::Disabled),
        ("RunWhenVisible", OnUpdateMode::RunWhenVisible),
        ("RunWhenVisibleOnce", OnUpdateMode::RunWhenVisibleOnce),
        ("RunOnce", OnUpdateMode::RunOnce),
        ("RunAlways", OnUpdateMode::RunAlways),
    ] {
        table_set(state, modes, name, mode.value());
    }
    table_set(state, enums, "OnUpdateMode", modes);
    let metadata = create_table(state);
    for (name, value) in [("MinValue", 0), ("MaxValue", 4), ("NumValues", 5)] {
        table_set(state, metadata, name, Val::Num(f64::from(value)));
    }
    table_set(state, enums, "OnUpdateModeMeta", metadata);
}
