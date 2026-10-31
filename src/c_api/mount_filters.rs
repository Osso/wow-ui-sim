/// Session-local mount filter masks. The fixture publishes Drop as its only source.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MountFilters {
    collected: u32,
    sources: u32,
    types: u32,
}

fn bit(index: i32) -> u32 {
    u32::try_from(index)
        .ok()
        .and_then(|i| 1u32.checked_shl(i))
        .unwrap_or(0)
}

impl MountFilters {
    const VALID_SOURCES: u32 = 1; // The current mount fixture publishes source Drop (0).
    const DEFAULT_TYPES: u32 = 0x1f;

    pub fn valid_source(index: i32) -> bool {
        Self::VALID_SOURCES & bit(index) != 0
    }
    pub fn valid_type(index: i32) -> bool {
        bit(index) != 0 && index != 3
    }
    pub fn is_default(&self) -> bool {
        self.collected == 3
            && self.sources == Self::VALID_SOURCES & 0xfff
            && self.types == Self::DEFAULT_TYPES
    }
    pub fn reset(&mut self) {
        self.collected = 3;
        self.sources = Self::VALID_SOURCES & 0xfff;
        self.types = Self::DEFAULT_TYPES;
    }
    pub fn collected(&self, index: i32) -> bool {
        (0..4).contains(&index) && self.collected & bit(index) != 0
    }
    pub fn source(&self, index: i32) -> bool {
        Self::valid_source(index) && self.sources & bit(index) != 0
    }
    pub fn mount_type(&self, index: i32) -> bool {
        self.types & bit(index) != 0
    }
    pub fn set_collected(&mut self, index: i32, checked: bool) {
        update(&mut self.collected, bit(index), checked);
    }
    pub fn set_source(&mut self, index: i32, checked: bool) {
        if Self::valid_source(index) {
            update(&mut self.sources, bit(index), checked);
        }
    }
    pub fn set_type(&mut self, index: i32, checked: bool) {
        update(&mut self.types, bit(index), checked);
    }
    pub fn all_sources(&mut self, checked: bool) {
        self.sources = if checked {
            Self::VALID_SOURCES & 0xfff
        } else {
            0
        };
    }
    pub fn all_types(&mut self, checked: bool) {
        self.types = if checked { Self::DEFAULT_TYPES } else { 0 };
    }
}

fn update(mask: &mut u32, bit: u32, checked: bool) {
    if checked {
        *mask |= bit;
    } else {
        *mask &= !bit;
    }
}
