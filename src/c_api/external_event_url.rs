//! Modeled external-event promotion state.
//!
//! Pinned `Blizzard_APIDocumentationGenerated/ExternalEventURLDocumentation.lua:5`
//! declares the `C_ExternalEventURL` namespace on Retail, PTR, Mists, Era and
//! Anniversary. Wrath publishes no such documentation file. WowForever publishes
//! its own modeled namespace from `c_api::forever_startup` and is not registered
//! here.

/// Host-supplied promotion state. The default describes no promoted event in the
/// simulator scenario; it is not evidence of a native default.
#[derive(Debug, Clone, Default)]
pub struct ExternalEventUrlState {
    /// Promotion URL the host made available, if any.
    pub url: Option<String>,
    /// Host-supplied newness for the supplied promotion.
    pub is_new: bool,
    /// Host-supplied visibility for the PTR `CanShow` consumer.
    pub can_show: bool,
}
