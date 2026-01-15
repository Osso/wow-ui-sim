//! Lua API bindings implementing WoW's addon API.

mod frame_methods;
mod globals;

use crate::event::{EventQueue, ScriptRegistry};
use crate::widget::WidgetRegistry;
use crate::Result;
use mlua::{Lua, MultiValue, Value};
use std::cell::RefCell;
use std::rc::Rc;

/// The WoW Lua environment.
pub struct WowLuaEnv {
    lua: Lua,
    state: Rc<RefCell<SimState>>,
}

/// Shared simulator state accessible from Lua.
#[derive(Debug, Default)]
pub struct SimState {
    pub widgets: WidgetRegistry,
    pub events: EventQueue,
    pub scripts: ScriptRegistry,
    /// Console output from Lua print() calls.
    pub console_output: Vec<String>,
}

impl WowLuaEnv {
    /// Create a new WoW Lua environment with the API initialized.
    pub fn new() -> Result<Self> {
        // Use unsafe_new to get full standard library including debug
        // This is safe for our simulator since we control the Lua code
        let lua = unsafe { Lua::unsafe_new() };
        let state = Rc::new(RefCell::new(SimState::default()));

        // Create UIParent (the root frame) - must have screen dimensions for layout
        {
            let mut s = state.borrow_mut();
            let mut ui_parent = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("UIParent".to_string()),
                None,
            );
            // Set UIParent to screen size (reference coordinate system)
            ui_parent.width = 500.0;
            ui_parent.height = 375.0;
            let ui_parent_id = ui_parent.id;
            s.widgets.register(ui_parent);

            // Create Minimap (built-in UI element)
            let minimap = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("Minimap".to_string()),
                Some(ui_parent_id),
            );
            s.widgets.register(minimap);
        }

        // Register global functions
        globals::register_globals(&lua, Rc::clone(&state))?;

        Ok(Self { lua, state })
    }

    /// Execute Lua code.
    pub fn exec(&self, code: &str) -> Result<()> {
        self.lua.load(code).exec()?;
        Ok(())
    }

    /// Execute Lua code with a custom chunk name (for better error messages and debugstack).
    pub fn exec_named(&self, code: &str, name: &str) -> Result<()> {
        self.lua.load(code).set_name(name).exec()?;
        Ok(())
    }

    /// Execute Lua code with varargs (like WoW addon loading).
    /// In WoW, each addon file receives (addonName, addonTable) as varargs.
    pub fn exec_with_varargs(
        &self,
        code: &str,
        name: &str,
        addon_name: &str,
        addon_table: mlua::Table,
    ) -> Result<()> {
        let chunk = self.lua.load(code).set_name(name);
        let func: mlua::Function = chunk.into_function()?;
        func.call::<()>((addon_name.to_string(), addon_table))?;
        Ok(())
    }

    /// Create a new empty table for addon private storage.
    pub fn create_addon_table(&self) -> Result<mlua::Table> {
        let table = self.lua.create_table()?;
        Ok(table)
    }

    /// Execute Lua code and return the result.
    pub fn eval<T: mlua::FromLuaMulti>(&self, code: &str) -> Result<T> {
        let result = self.lua.load(code).eval()?;
        Ok(result)
    }

    /// Fire an event to all registered frames.
    pub fn fire_event(&self, event: &str) -> Result<()> {
        self.fire_event_with_args(event, &[])
    }

    /// Fire an event with arguments to all registered frames.
    pub fn fire_event_with_args(&self, event: &str, args: &[Value]) -> Result<()> {
        let listeners = {
            let state = self.state.borrow();
            state.widgets.get_event_listeners(event)
        };

        for widget_id in listeners {
            // Get the handler function from our scripts table
            let scripts_table: Option<mlua::Table> = self.lua.globals().get("__scripts").ok();

            if let Some(table) = scripts_table {
                let frame_key = format!("{}_OnEvent", widget_id);
                let handler: Option<mlua::Function> = table.get(frame_key.as_str()).ok();

                if let Some(handler) = handler {
                    // Get the frame userdata
                    let frame_ref_key = format!("__frame_{}", widget_id);
                    let frame: Value = self.lua.globals().get(frame_ref_key.as_str())?;

                    // Build arguments: (self, event, ...args)
                    let mut call_args = vec![frame, Value::String(self.lua.create_string(event)?)];
                    call_args.extend(args.iter().cloned());

                    handler.call::<()>(MultiValue::from_vec(call_args))?;
                }
            }
        }

        Ok(())
    }

    /// Fire a script handler for a specific widget.
    /// handler_name is like "OnClick", "OnEnter", etc.
    /// extra_args are passed after the frame (self) argument.
    pub fn fire_script_handler(
        &self,
        widget_id: u64,
        handler_name: &str,
        extra_args: Vec<Value>,
    ) -> Result<()> {
        let scripts_table: Option<mlua::Table> = self.lua.globals().get("__scripts").ok();

        if let Some(table) = scripts_table {
            let frame_key = format!("{}_{}", widget_id, handler_name);
            let handler: Option<mlua::Function> = table.get(frame_key.as_str()).ok();

            if let Some(handler) = handler {
                // Get the frame userdata
                let frame_ref_key = format!("__frame_{}", widget_id);
                let frame: Value = self.lua.globals().get(frame_ref_key.as_str())?;

                // Build arguments: (self, ...extra_args)
                let mut call_args = vec![frame];
                call_args.extend(extra_args);

                handler.call::<()>(MultiValue::from_vec(call_args))?;
            }
        }

        Ok(())
    }

    /// Dispatch a slash command (e.g., "/wa options").
    /// Returns Ok(true) if a handler was found and called, Ok(false) if no handler matched.
    pub fn dispatch_slash_command(&self, input: &str) -> Result<bool> {
        let input = input.trim();
        if !input.starts_with('/') {
            return Ok(false);
        }

        // Parse command and message: "/wa options" -> cmd="/wa", msg="options"
        let (cmd, msg) = match input.find(' ') {
            Some(pos) => (&input[..pos], input[pos + 1..].trim()),
            None => (input, ""),
        };
        let cmd_lower = cmd.to_lowercase();

        // Scan globals for SLASH_* variables to find a matching command
        let globals = self.lua.globals();
        let slash_cmd_list: mlua::Table = globals.get("SlashCmdList")?;

        // Iterate through all globals looking for SLASH_* patterns
        for pair in globals.pairs::<String, Value>() {
            let (key, value) = pair?;

            // Look for SLASH_NAME1, SLASH_NAME2, etc.
            if !key.starts_with("SLASH_") {
                continue;
            }

            // Extract the command name (e.g., "SLASH_WEAKAURAS1" -> "WEAKAURAS")
            let suffix = &key[6..]; // Skip "SLASH_"
            let name = suffix.trim_end_matches(|c: char| c.is_ascii_digit());
            if name.is_empty() {
                continue;
            }

            // Check if this SLASH_ variable matches our command
            if let Value::String(slash_str) = value {
                if slash_str.to_str()?.to_lowercase() == cmd_lower {
                    // Found a match! Look up the handler in SlashCmdList
                    let handler: Option<mlua::Function> = slash_cmd_list.get(name).ok();
                    if let Some(handler) = handler {
                        let msg_value = self.lua.create_string(msg)?;
                        handler.call::<()>(msg_value)?;
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    /// Get access to the Lua state.
    pub fn lua(&self) -> &Lua {
        &self.lua
    }

    /// Get access to the simulator state.
    pub fn state(&self) -> &Rc<RefCell<SimState>> {
        &self.state
    }

    /// Dump all frame positions for debugging.
    /// Returns a formatted string similar to iced-debug output.
    pub fn dump_frames(&self) -> String {
        let state = self.state.borrow();
        let screen_width = 500.0_f32;
        let screen_height = 375.0_f32;

        let mut output = String::new();
        output.push_str(&format!(
            "[WoW Frames: {}x{}]\n\n",
            screen_width, screen_height
        ));

        // Collect and sort frames by strata/level
        let mut frames: Vec<_> = state.widgets.all_ids().into_iter().collect();
        frames.sort_by(|&a, &b| {
            let fa = state.widgets.get(a);
            let fb = state.widgets.get(b);
            match (fa, fb) {
                (Some(fa), Some(fb)) => fa
                    .frame_strata
                    .cmp(&fb.frame_strata)
                    .then_with(|| fa.frame_level.cmp(&fb.frame_level)),
                _ => std::cmp::Ordering::Equal,
            }
        });

        for id in frames {
            let frame = match state.widgets.get(id) {
                Some(f) => f,
                None => continue,
            };

            // Compute position
            let rect = compute_frame_rect(&state.widgets, id, screen_width, screen_height);

            // Format: Name [Type] (x,y w×h) visible/hidden
            let name = frame.name.as_deref().unwrap_or("(anon)");
            let vis = if frame.visible { "" } else { " HIDDEN" };
            let mouse = if frame.mouse_enabled { " mouse" } else { "" };

            // Indentation based on parent depth
            let depth = get_parent_depth(&state.widgets, id);
            let indent = "  ".repeat(depth);

            // Get parent name for context
            let parent_name = frame.parent_id
                .and_then(|pid| state.widgets.get(pid))
                .and_then(|p| p.name.as_deref())
                .unwrap_or("(root)");

            output.push_str(&format!(
                "{}{} [{}] ({:.0},{:.0} {:.0}x{:.0}){}{} parent={}\n",
                indent,
                name,
                frame.widget_type.as_str(),
                rect.x,
                rect.y,
                rect.width,
                rect.height,
                vis,
                mouse,
                parent_name,
            ));

            // Show anchor info
            if !frame.anchors.is_empty() {
                let anchor = &frame.anchors[0];
                output.push_str(&format!(
                    "{}  └─ {:?} -> {:?} offset ({:.0},{:.0})\n",
                    indent, anchor.point, anchor.relative_point, anchor.x_offset, anchor.y_offset
                ));
            } else {
                output.push_str(&format!("{}  └─ (no anchors - centered)\n", indent));
            }
        }

        output
    }
}

/// Get depth in parent hierarchy (for indentation).
fn get_parent_depth(registry: &crate::widget::WidgetRegistry, id: u64) -> usize {
    let mut depth = 0;
    let mut current = id;
    while let Some(frame) = registry.get(current) {
        if let Some(parent_id) = frame.parent_id {
            depth += 1;
            current = parent_id;
        } else {
            break;
        }
    }
    depth
}

/// Compute frame rect for debugging (same algorithm as renderer).
fn compute_frame_rect(
    registry: &crate::widget::WidgetRegistry,
    id: u64,
    screen_width: f32,
    screen_height: f32,
) -> LayoutRect {
    let frame = match registry.get(id) {
        Some(f) => f,
        None => return LayoutRect::default(),
    };

    let width = frame.width;
    let height = frame.height;

    // If no anchors, default to center of parent
    if frame.anchors.is_empty() {
        let parent_rect = if let Some(parent_id) = frame.parent_id {
            compute_frame_rect(registry, parent_id, screen_width, screen_height)
        } else {
            LayoutRect {
                x: 0.0,
                y: 0.0,
                width: screen_width,
                height: screen_height,
            }
        };

        return LayoutRect {
            x: parent_rect.x + (parent_rect.width - width) / 2.0,
            y: parent_rect.y + (parent_rect.height - height) / 2.0,
            width,
            height,
        };
    }

    let anchor = &frame.anchors[0];

    let parent_rect = if let Some(parent_id) = frame.parent_id {
        compute_frame_rect(registry, parent_id, screen_width, screen_height)
    } else {
        LayoutRect {
            x: 0.0,
            y: 0.0,
            width: screen_width,
            height: screen_height,
        }
    };

    let (parent_anchor_x, parent_anchor_y) = anchor_position(
        anchor.relative_point,
        parent_rect.x,
        parent_rect.y,
        parent_rect.width,
        parent_rect.height,
    );

    let target_x = parent_anchor_x + anchor.x_offset;
    // WoW uses Y-up coordinate system, screen uses Y-down
    let target_y = parent_anchor_y - anchor.y_offset;

    let (frame_x, frame_y) =
        frame_position_from_anchor(anchor.point, target_x, target_y, width, height);

    LayoutRect {
        x: frame_x,
        y: frame_y,
        width,
        height,
    }
}

fn anchor_position(
    point: crate::widget::AnchorPoint,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) -> (f32, f32) {
    use crate::widget::AnchorPoint;
    match point {
        AnchorPoint::TopLeft => (x, y),
        AnchorPoint::Top => (x + w / 2.0, y),
        AnchorPoint::TopRight => (x + w, y),
        AnchorPoint::Left => (x, y + h / 2.0),
        AnchorPoint::Center => (x + w / 2.0, y + h / 2.0),
        AnchorPoint::Right => (x + w, y + h / 2.0),
        AnchorPoint::BottomLeft => (x, y + h),
        AnchorPoint::Bottom => (x + w / 2.0, y + h),
        AnchorPoint::BottomRight => (x + w, y + h),
    }
}

fn frame_position_from_anchor(
    point: crate::widget::AnchorPoint,
    anchor_x: f32,
    anchor_y: f32,
    w: f32,
    h: f32,
) -> (f32, f32) {
    use crate::widget::AnchorPoint;
    match point {
        AnchorPoint::TopLeft => (anchor_x, anchor_y),
        AnchorPoint::Top => (anchor_x - w / 2.0, anchor_y),
        AnchorPoint::TopRight => (anchor_x - w, anchor_y),
        AnchorPoint::Left => (anchor_x, anchor_y - h / 2.0),
        AnchorPoint::Center => (anchor_x - w / 2.0, anchor_y - h / 2.0),
        AnchorPoint::Right => (anchor_x - w, anchor_y - h / 2.0),
        AnchorPoint::BottomLeft => (anchor_x, anchor_y - h),
        AnchorPoint::Bottom => (anchor_x - w / 2.0, anchor_y - h),
        AnchorPoint::BottomRight => (anchor_x - w, anchor_y - h),
    }
}

/// Simple layout rect for frame positioning.
#[derive(Debug, Default, Clone, Copy)]
struct LayoutRect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}
