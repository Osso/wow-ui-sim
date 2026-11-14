//! R88n.6 (Retail 70688): ordinary descriptor/object readers and result shape.
//! Native callback publication and full guarded failures are outside this model.

mod model;

use crate::c_api::helpers::{bad_argument, ensure_namespace, parse_numeric_string};
use crate::c_api::item_spell::{item_guid_for_bag_slot, parse_item_guid};
use crate::lua_api::methods::{
    borrow_state, create_string, create_table, ensure_not_fully_locked_down,
    frame_id_from_stack_unrestricted, table_get, table_set, table_set_num, val_to_string,
};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use crate::widget::WidgetType;
use rilua::{LuaResult, Val, table_security::unwrap_secret, vm::state::LuaState};

const COMPARE_USAGE: &str =
    "Usage: C_TooltipComparison.CompareItem(comparisonItem, tooltip [, anchorFrame])";
const INFO_USAGE: &str =
    "Usage: local info = C_TooltipComparison.GetItemComparisonInfo(comparisonItem)";
const DELTA_USAGE: &str = "Usage: local lines = C_TooltipComparison.GetItemComparisonDelta(comparisonItem, equippedItem [, pairedItem, addPairedStats])";

#[derive(Debug, Clone, Copy)]
enum Api {
    Compare,
    Info,
    Delta,
}

impl Api {
    fn name(self) -> &'static str {
        match self {
            Self::Compare => "CompareItem",
            Self::Info => "GetItemComparisonInfo",
            Self::Delta => "GetItemComparisonDelta",
        }
    }

    fn usage(self) -> &'static str {
        match self {
            Self::Compare => COMPARE_USAGE,
            Self::Info => INFO_USAGE,
            Self::Delta => DELTA_USAGE,
        }
    }

    fn argument_error(self, index: i32, reason: Option<&str>) -> rilua::LuaError {
        match reason {
            Some(reason) => {
                bad_argument(index, self.name(), &format!("{reason} - {}", self.usage()))
            }
            None => bad_argument(index, self.name(), self.usage()),
        }
    }

    fn readable(self, state: &LuaState, value: Val, index: i32) -> LuaResult<Val> {
        unwrap_secret(state, value).map_err(|_| {
            bad_argument(
                index,
                self.name(),
                &format!(
                    "{}. Secret values are only allowed during untainted execution for this argument.",
                    self.usage()
                ),
            )
        })
    }
}

#[derive(Debug)]
enum ItemIdentity {
    Guid { bag: i32, slot: i32, item_id: u32 },
    Hyperlink(String),
}

#[derive(Debug)]
struct ComparisonItem {
    identity: ItemIdentity,
    override_level: Option<i32>,
}

fn read_item(state: &mut LuaState, index: i32, api: Api) -> LuaResult<ComparisonItem> {
    let table = api.readable(state, stack_val(state, index), index)?;
    if !matches!(table, Val::Table(_)) {
        return Err(api.argument_error(index, None));
    }

    let guid_value = table_get(state, table, "guid");
    let guid_value = api.readable(state, guid_value, index)?;
    let guid = val_to_string(state, guid_value);
    let identity = comparison_guid(guid.as_deref());
    let identity = match identity {
        Some(identity) => identity,
        None => {
            let link_value = table_get(state, table, "hyperlink");
            let link_value = api.readable(state, link_value, index)?;
            let Some(link) = val_to_string(state, link_value).filter(|s| s.contains("item:"))
            else {
                return Err(api.argument_error(index, None));
            };
            ItemIdentity::Hyperlink(link)
        }
    };

    let level = table_get(state, table, "overrideItemLevel");
    let level = api.readable(state, level, index)?;
    Ok(ComparisonItem {
        identity,
        override_level: comparison_level(state, level),
    })
}

/// The simulator publishes canonical Item-bag-slot-id GUIDs. Syntactic
/// conversion intentionally does not check current inventory occupancy. The
/// complete native Item GUID grammar and other GUID families are not modeled.
fn comparison_guid(guid: Option<&str>) -> Option<ItemIdentity> {
    let guid = guid?;
    let (bag, slot, item_id) = parse_item_guid(guid)?;
    if slot <= 0 || item_id == 0 || item_guid_for_bag_slot(bag, slot, item_id) != guid {
        return None;
    }
    Some(ItemIdentity::Guid { bag, slot, item_id })
}

fn comparison_level(state: &LuaState, level: Val) -> Option<i32> {
    let number = match level {
        Val::Num(number) => Some(number),
        Val::Str(s) => state
            .gc
            .string_arena
            .get(s)
            .and_then(|s| parse_numeric_string(s.data())),
        _ => None,
    }?;
    (f64::from(i32::MIN)..=f64::from(i32::MAX))
        .contains(&number)
        .then(|| number.trunc() as i32)
}

fn read_frame(state: &LuaState, index: i32, tooltip: bool) -> LuaResult<()> {
    let api = Api::Compare;
    let value = stack_val(state, index);
    if value.is_nil() {
        return Err(api.argument_error(index, None));
    }
    if !matches!(value, Val::Table(_)) {
        return Err(api.argument_error(
            index,
            Some("Attempt to find 'this' in non-table object (used '.' instead of ':' ?)"),
        ));
    }
    let id = frame_id_from_stack_unrestricted(state, index)
        .map_err(|_| api.argument_error(index, Some("Wrong object type for function")))?;
    {
        let sim = borrow_state(state)?;
        let Some(frame) = sim.widgets.get(id) else {
            return Err(api.argument_error(
                index,
                Some("Attempt to find 'this' in non-framescript object"),
            ));
        };
        let class_valid = if tooltip {
            frame.widget_type == WidgetType::GameTooltip
        } else {
            !matches!(
                frame.widget_type,
                WidgetType::FontString
                    | WidgetType::Texture
                    | WidgetType::Line
                    | WidgetType::WorldFrame
            )
        };
        if !class_valid {
            return Err(api.argument_error(index, Some("Wrong object type for function")));
        }
    }
    crate::c_api::unit_aura_access::ensure_script_object_accessible(state, id)?;
    ensure_not_fully_locked_down(state, id)
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Retail {
        return Ok(());
    }
    let ns = ensure_namespace(state, "C_TooltipComparison")?;
    table_set_rust_fn_static(state, ns, Api::Compare.name(), compare_item)?;
    table_set_rust_fn_static(state, ns, Api::Info.name(), comparison_info)?;
    table_set_rust_fn_static(state, ns, Api::Delta.name(), comparison_delta)
}

fn compare_item(state: &mut LuaState) -> LuaResult<u32> {
    read_item(state, 1, Api::Compare)?;
    read_frame(state, 2, true)?;
    if !stack_val(state, 3).is_nil() {
        read_frame(state, 3, false)?;
    }
    // VERIFIED when no native callback is installed: zero results. Do not
    // dispatch a guessed FrameScript event or intercept the original Lua manager.
    Ok(0)
}

fn comparison_info(state: &mut LuaState) -> LuaResult<u32> {
    let source = read_item(state, 1, Api::Info)?;
    let Some(selection) = model::select(state, &source)? else {
        return Ok(0);
    };
    publish_info(state, selection);
    Ok(1)
}

fn publish_info(state: &mut LuaState, selection: model::Selection) {
    let record = create_table(state);
    state.push(record);
    // VERIFIED: Single=0; the other native weapon methods are not inferred.
    table_set(state, record, "method", Val::Num(0.0));
    publish_hyperlink_field(state, &selection.item, |state, item| {
        table_set(state, record, "item", item);
    });
    let additional = create_table(state);
    table_set(state, record, "additionalItems", additional);
    if let Val::Table(additional) = additional {
        for (index, link) in selection.additional_items.into_iter().enumerate() {
            publish_hyperlink_field(state, &link, |state, item| {
                table_set_num(state, additional, (index + 1) as f64, item);
            });
        }
    }
}

fn publish_hyperlink_field(
    state: &mut LuaState,
    link: &str,
    attach: impl FnOnce(&mut LuaState, Val),
) {
    let item = create_table(state);
    attach(state, item); // Root it in an already reachable result before allocating strings.
    let link = create_string(state, link);
    table_set(state, item, "hyperlink", link);
}

fn comparison_delta(state: &mut LuaState) -> LuaResult<u32> {
    let source = read_item(state, 1, Api::Delta)?;
    let equipped = read_item(state, 2, Api::Delta)?;
    let paired = if stack_val(state, 3).is_nil() {
        None
    } else {
        Some(read_item(state, 3, Api::Delta)?)
    };
    let add_paired = Api::Delta
        .readable(state, stack_val(state, 4), 4)?
        .is_truthy();
    let Some(lines) = model::delta(state, &source, &equipped, paired.as_ref(), add_paired)? else {
        return Ok(0);
    };
    crate::c_api::tooltip_data::push_string_lines(state, &lines);
    Ok(1)
}
