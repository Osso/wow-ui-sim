/// Explicit host-supplied illusion row; no native catalog or category mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct IllusionInfo {
    /// TransmogCollectionType selector input; not a returned record field.
    pub category: u32,
    /// Documented nonnullable TransmogIllusionInfo.visualID (number).
    pub visual_id: u32,
    /// Documented nonnullable TransmogIllusionInfo.sourceID (number).
    pub source_id: u32,
    /// Documented nonnullable TransmogIllusionInfo.icon (fileID).
    pub icon: u32,
    /// Documented nonnullable TransmogIllusionInfo.isCollected (bool).
    pub is_collected: bool,
    /// Documented nonnullable TransmogIllusionInfo.isUsable (bool).
    pub is_usable: bool,
    /// Documented nonnullable TransmogIllusionInfo.isHideVisual (bool).
    pub is_hide_visual: bool,
}
