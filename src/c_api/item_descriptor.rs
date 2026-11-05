//! WM97: selected descriptors are distinct from parsed link/request fields.
//! Native evidence: 70688; catalog and bonus records: pinned 70391.

mod bonus;
mod link;
mod scaling;
mod tables;

pub use link::{ParsedItemLink, parse_item_link};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ItemDescriptor {
    pub item_id: u32,
    pub bonus_context: Option<ItemBonusContext>,
    /// Stored order and duplicate keys are significant. This is not a map.
    pub modifiers: Vec<ItemModifier>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ItemBonusContext {
    pub creation_context: u8,
    pub bonus_list_ids: Vec<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ItemModifier {
    pub key: u8,
    pub value: u32,
}

impl ItemDescriptor {
    pub fn catalog(item_id: u32) -> Self {
        Self {
            item_id,
            ..Self::default()
        }
    }

    pub fn modifier(&self, key: u8) -> u32 {
        self.modifiers
            .iter()
            .find(|entry| entry.key == key)
            .map_or(0, |entry| entry.value)
    }

    /// Resolve into a copy, never mutate the catalog or the selected descriptor.
    /// `player_level` is the inferred local-player context of WM97b step 3.
    ///
    /// The pinned bonus, scaling and era records are Retail 70391, so only Retail
    /// and PTR apply descriptor content. Other profiles publish the same getters
    /// but resolve the catalog item; their own record sets are not modelled.
    pub fn resolve(&self, player_level: i32) -> Result<Option<crate::items::ItemInfo>, String> {
        let Some(base) = crate::items::get_item(self.item_id) else {
            return Ok(None);
        };
        let mut item = base.clone();
        if matches!(
            crate::client_profile::ACTIVE,
            crate::client_profile::ClientProfile::Retail
                | crate::client_profile::ClientProfile::Ptr
        ) {
            bonus::apply(self, &mut item, player_level)?;
        }
        Ok(Some(item))
    }
}

/// UNKNOWN: unsupported native resolution is explicit, never a bare-item fallback.
fn unknown(detail: impl std::fmt::Display) -> String {
    format!("item-descriptor UNKNOWN: {detail}")
}
