//! Host roster slots and the R45 selection/candidate distinction.

use crate::lua_api::SimState;
use crate::lua_api::state_types::BnetGameAccount;

/// A slot may have no ID, or an ID whose character record is unavailable.
#[derive(Clone, Debug, Default)]
pub struct CharacterSlot {
    pub character_id: String,
    pub record: Option<BnetGameAccount>,
}

impl From<BnetGameAccount> for CharacterSlot {
    fn from(record: BnetGameAccount) -> Self {
        Self {
            character_id: record.wow_account_guid.clone(),
            record: Some(record),
        }
    }
}

impl CharacterSlot {
    pub fn resolved_record(&self) -> Option<&BnetGameAccount> {
        self.record.as_ref().filter(|record| {
            !self.character_id.is_empty() && record.wow_account_guid == self.character_id
        })
    }
}

/// Preserve holes and indexes. Direct-screen startup retains its explicit
/// historical fixture; submitted login uses only the supplied host roster.
pub(crate) fn slots_for_sim(sim: &SimState) -> Vec<CharacterSlot> {
    if sim.glue_login.submitted {
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
