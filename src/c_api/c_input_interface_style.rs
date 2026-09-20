//! Forever input style. Keyboard/mouse is the simulator's initial policy.
use crate::event::{Event, EventArg};
use crate::lua_api::methods::{borrow_state, table_set_static};
use crate::lua_api::state::SimState;
use crate::lua_bridge::table_set_rust_fn_static;
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
    let values = crate::lua_api::methods::create_table(state);
    table_set_static(state, values, "Mkb", Val::Num(0.0));
    table_set_static(state, values, "Gamepad", Val::Num(1.0));
    table_set_static(state, Val::Table(enums), "InputDeviceInterfaceType", values);
    Ok(())
}

fn get_current_style(state: &mut LuaState) -> LuaResult<u32> {
    let style = borrow_state(state)?.input_interface_style;
    state.push(Val::Num(style as u8 as f64));
    Ok(1)
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
