//! `C_PingSecure` pending hit-test information (WM38).

/// WM38: the pending target GUID, position and transport GUID stored by hit-test
/// selection. The simulator has no world hit selection, so hosts supply it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PingPendingTarget {
    pub target_guid: String,
    pub position: [f32; 3],
    pub transport_guid: String,
}
