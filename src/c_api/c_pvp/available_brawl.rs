//! Supplied regular-brawl cache; the native refresh response is unknown.

use std::time::Duration;

use super::PvpBrawlInfo;

#[derive(Debug, Clone)]
pub struct AvailableBrawlCache {
    pub record: PvpBrawlInfo,
    pub received_at: Duration,
    pub duration: Option<Duration>,
}
