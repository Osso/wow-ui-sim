//! R82 Q1/Q2: request parsing only. No guessed downstream field meanings.
use crate::c_api::item_descriptor::{ParsedItemLink, parse_item_link};

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
    let text = text.split('\0').next()?;
    let (marker, payload) = MARKERS.iter().find_map(|&(marker, tag)| {
        text.find(tag)
            .map(|offset| (marker, &text[offset + tag.len()..]))
    })?;
    let payload = payload.split('|').next()?;
    let mut fields = payload.split(':');
    let mut next = || fields.next().and_then(number);
    Some(match marker {
        Marker::UrlIndex
        | Marker::StoreCategory
        | Marker::PerksActivity
        | Marker::InitiativeTask => HyperlinkRequest::NoProducer(marker),
        Marker::Item => HyperlinkRequest::Item(ItemRequest {
            link: parse_item_link(&format!("item:{payload}"))?,
            optional_arg1: optional_arg1.unwrap_or(0),
            optional_arg2: optional_arg2.unwrap_or(previous_arg2),
            hide_vendor_price,
        }),
        Marker::Spell => HyperlinkRequest::Spell(SpellRequest {
            spell_id: next()?,
            difficulty: next().map_or(-1, |n| n as i32),
            mode: 0,
            is_link: true,
            resolve_overrides: false,
            optional_arg1: optional_arg1.unwrap_or(0),
            optional_arg2: optional_arg2.unwrap_or(0),
        }),
        Marker::Currency => HyperlinkRequest::Currency {
            id: next()?,
            quantity: next().unwrap_or(0) as i32,
        },
        Marker::ArtifactPower => HyperlinkRequest::ArtifactPower {
            id: next()?,
            base: next()? as u8,
            total: next()? as u8,
        },
        Marker::AzeriteEssence => HyperlinkRequest::AzeriteEssence {
            id: next()?,
            rank: next()?.max(1),
        },
        Marker::Conduit => HyperlinkRequest::Conduit {
            id: next()?,
            rank: (next()? as i32).max(1),
        },
        Marker::Trade => {
            let mut fields = payload.split(':');
            HyperlinkRequest::Trade {
                guid: fields.next()?.to_string(),
                first: number(fields.next()?)?,
                second: number(fields.next()?)?,
            }
        }
        Marker::Unit | Marker::Achievement | Marker::InstanceLock | Marker::Keystone => {
            HyperlinkRequest::Unmodelled {
                marker,
                payload: payload.to_string(),
            }
        }
        _ => HyperlinkRequest::Single {
            marker,
            id: next()?,
        },
    })
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
