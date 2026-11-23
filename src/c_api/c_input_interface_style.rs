//! PTR and Forever input style backed by the shared environment-local input state.
//! Keyboard/mouse is existing simulator policy, not a documented native default.
use crate::event::{Event, EventArg};
use crate::lua_api::methods::borrow_state;
#[cfg(feature = "client-wowforever")]
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_api::state::SimState;
use crate::lua_bridge::table_set_rust_fn_static;
#[cfg(feature = "client-wowforever")]
use crate::lua_bridge::{FromStack, TableBuilder};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputInterfaceStyle {
    #[default]
    Mkb = 0,
    Gamepad = 1,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_InputInterfaceStyle")?;
    table_set_rust_fn_static(state, namespace, "GetCurrentStyle", get_current_style)?;
    let enums = super::ensure_namespace(state, "Enum")?;
    let values =
        crate::c_api::ensure_namespace_table_field(state, enums, "InputDeviceInterfaceType")?;
    super::set_namespace_member(state, Val::Table(values), "Mkb", Val::Num(0.0))?;
    super::set_namespace_member(state, Val::Table(values), "Gamepad", Val::Num(1.0))?;
    Ok(())
}

fn get_current_style(state: &mut LuaState) -> LuaResult<u32> {
    let style = borrow_state(state)?.input_interface_style;
    state.push(Val::Num(style as u8 as f64));
    Ok(1)
}

#[cfg(feature = "client-wowforever")]
pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("SetInputInterfaceStyle", set_input_interface_style_admin)
}

#[cfg(feature = "client-wowforever")]
fn set_input_interface_style_admin(state: &mut LuaState) -> LuaResult<u32> {
    let style = match i32::from_stack(state, 1)? {
        0 => InputInterfaceStyle::Mkb,
        1 => InputInterfaceStyle::Gamepad,
        _ => {
            return Err(rilua::runtime_error(
                "A_Admin.SetInputInterfaceStyle requires Mkb (0) or Gamepad (1)",
            ));
        }
    };
    borrow_state_mut(state)?.set_input_interface_style(style);
    Ok(0)
}

impl SimState {
    /// Queue the source-documented new/old payload after updating model state.
    /// Re-selecting the current style is a simulator no-transition policy.
    pub fn set_input_interface_style(&mut self, style: InputInterfaceStyle) {
        let old = self.input_interface_style;
        if old == style {
            return;
        }
        self.input_interface_style = style;
        self.events.push(Event {
            name: "INPUT_DEVICE_INTERFACE_TRANSITION".to_owned(),
            args: vec![
                EventArg::Number(style as u8 as f64),
                EventArg::Number(old as u8 as f64),
            ],
        });
    }
}
