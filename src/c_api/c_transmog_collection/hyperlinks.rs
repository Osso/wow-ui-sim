//! Reversible custom-set list serialization, without inventing a shared native codec.

use crate::lua_api::methods::create_string;
use crate::lua_bridge::FromStack;
use rilua::LuaResult;
use rilua::vm::state::LuaState;

use super::ItemTransmogInfo;

pub(super) fn encode(state: &mut LuaState) -> LuaResult<u32> {
    let items = super::read_items(state, 1)?;
    if items.is_empty() {
        return Ok(0);
    }
    // INFERRED wire layout: count followed by ordered appearance/secondary/
    // illusion triples. The API docs do not specify the native hyperlink codec;
    // this roundtrip contract does not claim interoperability with native links.
    let mut payload = format!("transmogset:{}", items.len());
    for item in items {
        payload.push_str(&format!(
            ":{}:{}:{}",
            item.appearance_id, item.secondary_appearance_id, item.illusion_id
        ));
    }
    let link = create_string(state, &format!("|H{payload}|h[Custom Set]|h"));
    state.push(link);
    Ok(1)
}

pub(super) fn decode(state: &mut LuaState) -> LuaResult<u32> {
    let link = String::from_stack(state, 1)?;
    let Some(items) = parse_link(&link) else {
        return Ok(0);
    };
    let array = super::build_items(state, &items);
    state.push(array);
    Ok(1)
}

fn parse_link(link: &str) -> Option<Vec<ItemTransmogInfo>> {
    let (_, payload) = link.split_once("|Htransmogset:")?;
    let (payload, _) = payload.split_once("|h")?;
    let mut fields = payload.split(':');
    let count = fields.next()?.parse::<usize>().ok()?;
    let numbers: Vec<i32> = fields
        .map(str::parse::<i32>)
        .collect::<Result<_, _>>()
        .ok()?;
    if count == 0
        || numbers.len() / 3 != count
        || numbers.len() % 3 != 0
        || numbers.iter().any(|id| *id < 0)
    {
        return None;
    }
    Some(
        numbers
            .chunks_exact(3)
            .map(|row| ItemTransmogInfo {
                appearance_id: row[0],
                secondary_appearance_id: row[1],
                illusion_id: row[2],
            })
            .collect(),
    )
}
