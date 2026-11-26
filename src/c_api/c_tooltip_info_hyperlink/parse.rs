//! R82 Q1/Q2: request parsing only. No guessed downstream field meanings.
use crate::c_api::item_descriptor::{ParsedItemLink, parse_item_link};

/// R82 Q1: keystone links carry eight numeric scalars.
const KEYSTONE_SCALAR_COUNT: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Marker {
    Item,
    Currency,
    Enchant,
    Spell,
    Curio,
    Unit,
    Quest,
    Talent,
    PvpTalent,
    ArtifactPower,
    Trade,
    Achievement,
    InstanceLock,
    Keystone,
    UrlIndex,
    StoreCategory,
    AzeriteEssence,
    Conduit,
    MawPower,
    Mount,
    PerksActivity,
    InitiativeTask,
}

/// R82 Q1: search priority is independent of each marker's position in the text.
pub const MARKERS: &[(Marker, &str)] = &[
    (Marker::Item, "item:"),
    (Marker::Currency, "currency:"),
    (Marker::Enchant, "enchant:"),
    (Marker::Spell, "spell:"),
    (Marker::Curio, "curio:"),
    (Marker::Unit, "unit:"),
    (Marker::Quest, "quest:"),
    (Marker::Talent, "talent:"),
    (Marker::PvpTalent, "pvptal:"),
    (Marker::ArtifactPower, "apower:"),
    (Marker::Trade, "trade:"),
    (Marker::Achievement, "achievement:"),
    (Marker::InstanceLock, "instancelock:"),
    (Marker::Keystone, "keystone:"),
    (Marker::UrlIndex, "urlIndex:"),
    (Marker::StoreCategory, "storecategory:"),
    (Marker::AzeriteEssence, "azessence:"),
    (Marker::Conduit, "conduit:"),
    (Marker::MawPower, "mawpower:"),
    (Marker::Mount, "mount:"),
    (Marker::PerksActivity, "perksactivity:"),
    (Marker::InitiativeTask, "initiativetask:"),
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemRequest {
    pub link: ParsedItemLink,
    pub optional_arg1: i32,
    pub optional_arg2: i32,
    pub hide_vendor_price: bool,
}

struct RequestOptions {
    optional_arg1: Option<i32>,
    optional_arg2: Option<i32>,
    hide_vendor_price: bool,
    previous_arg2: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpellRequest {
    pub spell_id: u32,
    pub difficulty: i32,
    pub mode: u8,
    pub is_link: bool,
    pub resolve_overrides: bool,
    pub optional_arg1: i32,
    pub optional_arg2: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HyperlinkRequest {
    Item(ItemRequest),
    Spell(SpellRequest),
    Currency {
        id: u32,
        quantity: i32,
    },
    ArtifactPower {
        id: u32,
        base: u8,
        total: u8,
    },
    AzeriteEssence {
        id: u32,
        rank: u32,
    },
    Conduit {
        id: u32,
        rank: i32,
    },
    Single {
        marker: Marker,
        id: u32,
    },
    Trade {
        guid: String,
        first: u32,
        second: u32,
    },
    Keystone {
        /// INFERRED (R82/R88aw): the first of eight keystone scalars selects the item.
        /// The remaining scalar-to-request mapping is UNKNOWN.
        item_id: u32,
    },
    /// These routes have no proven payload-to-producer mapping. Preserve their
    /// text without guessing GUID validation or serialized tooltip rows.
    Unmodelled {
        marker: Marker,
        payload: String,
    },
    NoProducer(Marker),
}

/// R82 Q2: wrapper integers occupy distinct fields. `previous_arg2` is the
/// request's existing value, not an inferred class/spec meaning.
pub fn parse(
    text: &str,
    optional_arg1: Option<i32>,
    optional_arg2: Option<i32>,
    hide_vendor_price: bool,
    previous_arg2: i32,
) -> Option<HyperlinkRequest> {
    let (marker, payload) = find_marker_payload(text)?;
    let payload = payload.split('|').next()?;
    let options = RequestOptions {
        optional_arg1,
        optional_arg2,
        hide_vendor_price,
        previous_arg2,
    };
    parse_payload(marker, payload, &options)
}

fn find_marker_payload(text: &str) -> Option<(Marker, &str)> {
    let text = text.split('\0').next()?;
    MARKERS.iter().find_map(|&(marker, tag)| {
        text.find(tag)
            .map(|offset| (marker, &text[offset + tag.len()..]))
    })
}

fn parse_payload(
    marker: Marker,
    payload: &str,
    options: &RequestOptions,
) -> Option<HyperlinkRequest> {
    let mut fields = payload.split(':');
    Some(match marker {
        Marker::UrlIndex
        | Marker::StoreCategory
        | Marker::PerksActivity
        | Marker::InitiativeTask => HyperlinkRequest::NoProducer(marker),
        Marker::Item => parse_item_request(payload, options)?,
        Marker::Spell => parse_spell_request(&mut fields, options)?,
        Marker::Trade => parse_trade_request(payload)?,
        Marker::Keystone => parse_keystone_request(&mut fields)?,
        Marker::Unit | Marker::Achievement | Marker::InstanceLock => HyperlinkRequest::Unmodelled {
            marker,
            payload: payload.to_string(),
        },
        _ => parse_single_numeric_request(marker, &mut fields)?,
    })
}

fn parse_item_request(payload: &str, options: &RequestOptions) -> Option<HyperlinkRequest> {
    Some(HyperlinkRequest::Item(ItemRequest {
        link: parse_item_link(&format!("item:{payload}"))?,
        optional_arg1: options.optional_arg1.unwrap_or(0),
        optional_arg2: options.optional_arg2.unwrap_or(options.previous_arg2),
        hide_vendor_price: options.hide_vendor_price,
    }))
}

fn parse_spell_request(
    fields: &mut std::str::Split<'_, char>,
    options: &RequestOptions,
) -> Option<HyperlinkRequest> {
    Some(HyperlinkRequest::Spell(SpellRequest {
        spell_id: next_number(fields)?,
        difficulty: next_number(fields).map_or(-1, |n| n as i32),
        mode: 0,
        is_link: true,
        resolve_overrides: false,
        optional_arg1: options.optional_arg1.unwrap_or(0),
        optional_arg2: options.optional_arg2.unwrap_or(0),
    }))
}

fn parse_trade_request(payload: &str) -> Option<HyperlinkRequest> {
    let mut fields = payload.split(':');
    Some(HyperlinkRequest::Trade {
        guid: fields.next()?.to_string(),
        first: number(fields.next()?)?,
        second: number(fields.next()?)?,
    })
}

fn parse_keystone_request(fields: &mut std::str::Split<'_, char>) -> Option<HyperlinkRequest> {
    let item_id = next_number(fields)?;
    for _ in 1..KEYSTONE_SCALAR_COUNT {
        next_number(fields)?;
    }
    Some(HyperlinkRequest::Keystone { item_id })
}

fn parse_single_numeric_request(
    marker: Marker,
    fields: &mut std::str::Split<'_, char>,
) -> Option<HyperlinkRequest> {
    Some(match marker {
        Marker::Currency => HyperlinkRequest::Currency {
            id: next_number(fields)?,
            quantity: next_number(fields).unwrap_or(0) as i32,
        },
        Marker::ArtifactPower => HyperlinkRequest::ArtifactPower {
            id: next_number(fields)?,
            base: next_number(fields)? as u8,
            total: next_number(fields)? as u8,
        },
        Marker::AzeriteEssence => HyperlinkRequest::AzeriteEssence {
            id: next_number(fields)?,
            rank: next_number(fields)?.max(1),
        },
        Marker::Conduit => HyperlinkRequest::Conduit {
            id: next_number(fields)?,
            rank: (next_number(fields)? as i32).max(1),
        },
        _ => HyperlinkRequest::Single {
            marker,
            id: next_number(fields)?,
        },
    })
}

fn next_number(fields: &mut std::str::Split<'_, char>) -> Option<u32> {
    fields.next().and_then(number)
}

fn number(text: &str) -> Option<u32> {
    if let Some(rest) = text.strip_prefix('-') {
        return number(rest).map(u32::wrapping_neg);
    }
    let (digits, radix) = text
        .strip_prefix("0x")
        .or_else(|| text.strip_prefix("0X"))
        .map_or((text, 10), |digits| (digits, 16));
    u32::from_str_radix(digits, radix).ok()
}

/// R82/R82b: display total and lookup rank are distinct when total is zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArtifactRanks {
    pub displayed_total: u32,
    pub lookup_rank: u8,
    pub displayed_maximum: u32,
    pub adjusted_bonus: u32,
}

pub fn artifact_ranks(
    flags: u32,
    base: u8,
    total: u8,
    display: &super::inputs::ArtifactDisplay,
) -> ArtifactRanks {
    let total = if flags & 64 != 0 { 1 } else { total };
    let bonus = u32::from(total).wrapping_sub(u32::from(base));
    let mut maximum = display.normal_maximum.wrapping_add(bonus);
    if (maximum as i32) > (display.rank_limit as i32) {
        maximum = display.rank_limit;
    }
    ArtifactRanks {
        displayed_total: u32::from(total),
        lookup_rank: total.max(1),
        displayed_maximum: if display.use_normal_maximum {
            display.normal_maximum
        } else {
            maximum
        },
        adjusted_bonus: if flags & 16 != 0 && (bonus as i32) > 0 {
            bonus.wrapping_sub(1)
        } else {
            bonus
        },
    }
}
