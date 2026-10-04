//! INFERRED explicit host URL texture notifications, not a downloader.

use std::collections::VecDeque;

#[derive(Default)]
pub struct UrlTextureInputs {
    pub pending: VecDeque<UrlTextureNotification>,
}

pub struct UrlTextureNotification {
    pub texture_id: u64,
    pub result: UrlTextureResult,
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum UrlTextureResult {
    Found = 1,
    NotFound = 2,
    Requested = 3,
    NotAllowed = 4,
}
