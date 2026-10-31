/// Scenario-local mount journal filter choices; native defaults are unknown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MountFilters {
    collected: [bool; 3],
    sources: [bool; 12],
    types: [bool; 5],
}

impl Default for MountFilters {
    fn default() -> Self {
        Self {
            collected: [true; 3],
            sources: [true; 12],
            types: [true; 5],
        }
    }
}

fn value(values: &[bool], index: i32) -> bool {
    index
        .checked_sub(1)
        .and_then(|i| usize::try_from(i).ok())
        .and_then(|i| values.get(i))
        .copied()
        .unwrap_or(false)
}

fn set(values: &mut [bool], index: i32, checked: bool) {
    if let Some(slot) = index
        .checked_sub(1)
        .and_then(|i| usize::try_from(i).ok())
        .and_then(|i| values.get_mut(i))
    {
        *slot = checked;
    }
}

impl MountFilters {
    pub fn collected(&self, index: i32) -> bool {
        value(&self.collected, index)
    }
    pub fn source(&self, index: i32) -> bool {
        value(&self.sources, index)
    }
    pub fn mount_type(&self, index: i32) -> bool {
        value(&self.types, index)
    }
    pub fn set_collected(&mut self, index: i32, checked: bool) {
        set(&mut self.collected, index, checked);
    }
    pub fn set_source(&mut self, index: i32, checked: bool) {
        set(&mut self.sources, index, checked);
    }
    pub fn set_type(&mut self, index: i32, checked: bool) {
        set(&mut self.types, index, checked);
    }
    pub fn all_sources(&mut self, checked: bool) {
        self.sources.fill(checked);
    }
    pub fn all_types(&mut self, checked: bool) {
        self.types.fill(checked);
    }
}
