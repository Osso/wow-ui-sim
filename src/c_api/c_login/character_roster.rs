//! Host roster slots and the R45 selection/candidate distinction.

use crate::lua_api::SimState;
use crate::lua_api::state_types::BnetGameAccount;

/// A slot may have no ID, or an ID whose character record is unavailable.
#[derive(Clone, Debug, Default)]
pub struct CharacterSlot {
    pub character_id: String,
    pub record: Option<CharacterRecord>,
}

/// Host character-list record. An absent season represents native construction 0.
#[derive(Clone, Debug)]
pub struct CharacterRecord {
    pub account: BnetGameAccount,
    pub timerunning_season_id: Option<u32>,
    /// Explicit host race ID, not inferred from the social fixture name.
    pub race_id: u32,
    /// Independent R52 decoded background value; native construction starts at 0.
    pub background_override: u32,
}

impl From<BnetGameAccount> for CharacterSlot {
    fn from(record: BnetGameAccount) -> Self {
        Self {
            character_id: record.wow_account_guid.clone(),
            record: Some(CharacterRecord {
                account: record,
                timerunning_season_id: None,
                race_id: 0,
                background_override: 0,
            }),
        }
    }
}

impl CharacterSlot {
    pub fn resolved_record(&self) -> Option<&CharacterRecord> {
        self.record.as_ref().filter(|record| {
            !self.character_id.is_empty() && record.account.wow_account_guid == self.character_id
        })
    }
}

/// Preserve holes and indexes. Direct-screen startup retains its explicit
/// historical fixture; the modern Login flow uses only the supplied host roster,
/// including before submission. Other profiles retain their existing fixtures.
pub(crate) fn slots_for_sim(sim: &SimState) -> Vec<CharacterSlot> {
    let host_login = matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::Ptr
            | crate::client_profile::ClientProfile::WowForever
    ) && sim.screen_kind == crate::screen::ScreenKind::Login;
    if sim.glue_login.submitted || host_login {
        return sim.glue_login.characters.clone();
    }
    sim.bnet_friends
        .iter()
        .flat_map(|friend| &friend.game_accounts)
        .filter(|account| account.client_program == "WoW")
        .cloned()
        .map(CharacterSlot::from)
        .collect()
}

/// R45 clamp/search rule. The host input/output are zero-based; neither is a
/// native saved setting or the selected character. Unset host input starts at
/// slot zero, without claiming a native initial candidate value.
pub fn resolve_saved_candidate(slots: &[CharacterSlot], saved: Option<i64>) -> Option<usize> {
    if slots.is_empty() {
        return None;
    }
    let start = saved
        .and_then(|index| usize::try_from(index).ok())
        .filter(|index| *index < slots.len())
        .unwrap_or(0);
    (start..slots.len())
        .chain(0..start)
        .find(|index| slots[*index].resolved_record().is_some())
        .or(Some(0))
}
