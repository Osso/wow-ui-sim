//! WM97 DQ7 link parsing. Fields stay distinct: they are not all descriptor content.
//!
//! INFERRED seam: a truncated link decodes missing numeric fields as empty (zero),
//! matching the native parser's empty-field policy. Exceptional overflow is UNKNOWN.

use super::{ItemBonusContext, ItemDescriptor, ItemModifier};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ParsedItemLink {
    pub descriptor: ItemDescriptor,
    pub enchant_id: u32,
    pub socket_item_ids: [u32; 3],
    pub fourth_gem_scalar: u32,
    pub link_level: u32,
    pub request_context: u32,
    pub upgrade_mask: u32,
    pub socket_bonus_groups: [Vec<u32>; 3],
    pub crafter_guid: String,
    pub trailing_scalar: u32,
}

/// Parse the first `item:` link and keep the descriptor's stored bonus/modifier order.
pub fn parse_item_link(text: &str) -> Option<ParsedItemLink> {
    let mut fields = text.get(text.find("item:")? + 5..)?.split(':');
    let mut field = || fields.next().unwrap_or("").to_owned();
    let item_id = parse_number(&field())?;
    if item_id == 0 {
        return None;
    }
    let link = ParsedItemLink {
        descriptor: ItemDescriptor::catalog(item_id),
        enchant_id: parse_number(&field())?,
        socket_item_ids: [
            parse_number(&field())?,
            parse_number(&field())?,
            parse_number(&field())?,
        ],
        fourth_gem_scalar: parse_number(&field())?,
        ..ParsedItemLink::default()
    };
    // Fields 7 (suffix) and 8 (unique ID) are consumed and discarded.
    field();
    field();
    let mut link = link;
    link.link_level = parse_number(&field())?;
    link.request_context = parse_number(&field())?;
    link.upgrade_mask = parse_number(&field())?;
    let creation_context = parse_number(&field())? as u8;
    let bonus_count = parse_number(&field())?;
    if bonus_count >= 16 {
        return None;
    }
    let mut bonus_list_ids = Vec::new();
    for _ in 0..bonus_count {
        bonus_list_ids.push(parse_number(&field())?);
    }
    if link.upgrade_mask == 0 {
        let pairs = parse_signed(&field());
        for _ in 0..pairs.max(0) {
            let key = parse_number(&field())? as u8;
            let value = parse_number(&field())?;
            apply_modifier(&mut link.descriptor.modifiers, key, value);
        }
    } else {
        // Legacy upgrade bits consume one discardable field each; they are not modifiers.
        for bit in 0..30 {
            if link.upgrade_mask >> bit & 1 == 1 {
                field();
            }
        }
    }
    for group in &mut link.socket_bonus_groups {
        let count = parse_signed(&field());
        for _ in 0..count.max(0) {
            group.push(parse_number(&field())?);
        }
    }
    link.crafter_guid = field();
    link.trailing_scalar = parse_number(&field())?;
    link.descriptor.bonus_context = Some(ItemBonusContext {
        creation_context,
        bonus_list_ids,
    });
    Some(link)
}

/// WM97 DQ1: nonzero updates the first matching entry or appends; zero removes the first.
fn apply_modifier(modifiers: &mut Vec<ItemModifier>, key: u8, value: u32) {
    match modifiers.iter().position(|entry| entry.key == key) {
        Some(index) if value == 0 => {
            modifiers.remove(index);
        }
        Some(index) => modifiers[index].value = value,
        None if value != 0 => modifiers.push(ItemModifier { key, value }),
        None => {}
    }
}

fn parse_number(text: &str) -> Option<u32> {
    if text.is_empty() {
        return Some(0);
    }
    let (radix, digits) = match text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        Some(digits) => (16, digits),
        None => (10, text),
    };
    u32::from_str_radix(digits, radix).ok()
}

fn parse_signed(text: &str) -> i32 {
    match text.strip_prefix('-') {
        Some(rest) => -parse_number(rest).map_or(0, |value| value as i32),
        None => parse_number(text).map_or(0, |value| value as i32),
    }
}
