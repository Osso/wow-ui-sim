use crate::items;
use crate::lua_api::methods::{borrow_state, create_string, val_to_string};
use crate::lua_bridge::{FromStack, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

#[derive(Default)]
struct Socket {
    id: u32,
    bonuses: Vec<u32>,
}

fn parse_number(field: &str) -> u32 {
    let field = field.trim();
    if let Some(hex) = field
        .strip_prefix("0x")
        .or_else(|| field.strip_prefix("0X"))
    {
        u32::from_str_radix(hex, 16).unwrap_or(0)
    } else {
        field.parse().unwrap_or(0)
    }
}

fn parse_socket(link: &str, socket: usize) -> Socket {
    let Some((_, payload)) = link.split_once("item:") else {
        return Socket::default();
    };
    let payload = payload.split('|').next().unwrap_or(payload);
    let fields: Vec<&str> = payload.split(':').collect();
    let id = fields
        .get(2 + socket)
        .map_or(0, |field| parse_number(field));
    let main_count = fields.get(12).map_or(0, |field| parse_number(field)) as usize;
    let Some(mut position) = 13usize.checked_add(main_count) else {
        return Socket {
            id,
            bonuses: Vec::new(),
        };
    };
    let modifier_count = fields.get(position).map_or(0, |field| parse_number(field)) as usize;
    let Some(next) = modifier_count
        .checked_mul(2)
        .and_then(|n| position.checked_add(1 + n))
    else {
        return Socket {
            id,
            bonuses: Vec::new(),
        };
    };
    position = next;
    for index in 0..3 {
        let count = fields.get(position).map_or(0, |field| parse_number(field)) as usize;
        position += 1;
        let Some(end) = position.checked_add(count) else {
            break;
        };
        if end > fields.len() {
            break;
        }
        if index == socket {
            return Socket {
                id,
                bonuses: fields[position..end]
                    .iter()
                    .map(|field| parse_number(field))
                    .collect(),
            };
        }
        position = end;
    }
    Socket {
        id,
        bonuses: Vec::new(),
    }
}

fn find_owned_socket(state: &LuaState, query: &str, socket: usize) -> LuaResult<Socket> {
    let sim = borrow_state(state)?;
    let query = query.to_lowercase();
    let mut equipment: Vec<_> = sim.player.equipped_items.iter().collect();
    equipment.sort_unstable_by_key(|(slot, _)| *slot);
    for (_, item) in equipment {
        if items::get_item(item.item_id)
            .is_some_and(|info| info.name.to_lowercase().starts_with(&query))
        {
            return Ok(Socket {
                id: item.gem_ids[socket],
                bonuses: Vec::new(),
            });
        }
    }
    let mut bags: Vec<_> = sim.bag_items.iter().collect();
    bags.sort_unstable_by_key(|(slot, _)| *slot);
    for (_, item) in bags {
        if items::get_item(item.item_id)
            .is_some_and(|info| info.name.to_lowercase().starts_with(&query))
        {
            return Ok(item
                .hyperlink
                .as_deref()
                .map_or_else(Socket::default, |link| parse_socket(link, socket)));
        }
    }
    Ok(Socket::default())
}

fn lookup_gem(id: u32) -> Option<(&'static str, u8)> {
    let gems = sim_data::gem_items::GEMS;
    if let Ok(index) = gems.binary_search_by_key(&id, |gem| gem.0) {
        let (_, name, quality) = gems[index];
        return Some((name, quality));
    }
    items::get_item(id).map(|item| (item.name, item.quality))
}

fn gem_link(id: u32, name: &str, quality: u8, bonuses: &[u32], level: i32, spec: u32) -> String {
    let mut fields = vec![id.to_string()];
    fields.extend((0..7).map(|_| String::new()));
    fields.push(if level == 0 {
        String::new()
    } else {
        level.to_string()
    });
    fields.push(if spec == 0 {
        String::new()
    } else {
        spec.to_string()
    });
    fields.extend([String::new(), String::new()]);
    fields.push(if bonuses.is_empty() {
        String::new()
    } else {
        bonuses.len().to_string()
    });
    fields.extend(bonuses.iter().map(u32::to_string));
    fields.extend((0..6).map(|_| String::new()));
    format!("|cnIQ{quality}:|Hitem:{}|h[{name}]|h|r", fields.join(":"))
}

pub(super) fn get_item_gem(state: &mut LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let query = match value {
        Val::Str(_) => {
            val_to_string(state, value).ok_or_else(|| runtime_error("invalid item string"))?
        }
        Val::Num(number) => number.to_string(),
        _ => return Err(runtime_error("C_Item.GetItemGem: expected item string")),
    };
    let index = f64::from_stack(state, 2)?;
    if !index.is_finite() || !(0.0..=u32::MAX as f64).contains(&index) {
        return Err(runtime_error(
            "C_Item.GetItemGem: socket index out of range",
        ));
    }
    if index == 0.0 || index >= 4.0 {
        return Ok(0);
    }
    let index = ((index - 1.0).trunc() as usize).min(2);
    let socket = if query.contains("item:") {
        parse_socket(&query, index)
    } else {
        find_owned_socket(state, &query, index)?
    };
    let Some((name, quality)) = lookup_gem(socket.id) else {
        return Ok(0);
    };
    let (level, spec) = {
        let sim = borrow_state(state)?;
        let player = &sim.player;
        let spec = usize::try_from(player.active_spec_index - 1)
            .ok()
            .and_then(|index| {
                crate::specializations::specs_for_class(player.class_index as u32).nth(index)
            })
            .map_or(0, |spec| spec.id);
        (player.level, spec)
    };
    let link = gem_link(socket.id, name, quality, &socket.bonuses, level, spec);
    let name = create_string(state, name);
    let link = create_string(state, &link);
    state.push(name);
    state.push(link);
    Ok(2)
}
