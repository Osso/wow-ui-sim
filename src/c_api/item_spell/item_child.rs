//! Supplied ItemChildEquipment rows; field meanings are not yet established.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemChildEquipment {
    /// Zero-based slot selector from the native row.
    pub slot_index: i32,
    /// The two numeric fields emitted by the native row serializer, in order.
    pub values: [i32; 2],
}
