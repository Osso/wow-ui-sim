/// Host-supplied roster records read by Retail `GetGroupMemberCounts`.
#[derive(Debug, Clone, Default)]
pub struct GroupMemberCountsState {
    /// First container. Its native semantic name is not established.
    pub container_one: Option<Vec<Option<GroupMemberCountsMember>>>,
    /// Second container. Its native semantic name is not established.
    pub container_two: Option<Vec<Option<GroupMemberCountsMember>>>,
}

/// Member fields copied into the native role-count record.
#[derive(Debug, Clone, Default)]
pub struct GroupMemberCountsMember {
    /// Incoming flags; bit 16 excludes the member from both count groups.
    pub flags: u32,
    /// First native role mask. Its semantic name is not established.
    pub role_mask_a: u32,
    /// Second native role mask. Its semantic name is not established.
    pub role_mask_b: u32,
    /// Class from a resolved player object, when the host has resolved one.
    pub resolved_class_id: Option<i32>,
    /// Cached roster class used when no resolved class is supplied.
    pub cached_class_id: i32,
}
