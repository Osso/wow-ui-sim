//! R52 field cascade over explicit host rows (tracker policy 83).
use super::character_roster::CharacterRecord;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Default)]
pub struct RaceBackgroundRow {
    pub select_screen_file_data_id: u32,
    pub low_res_screen_file_data_id: u32,
}

/// These maps are not inferred from the identity-only class/race catalogs.
/// Empty is the approved simulator input, not a claim about shipped DB2 rows.
#[derive(Clone, Debug, Default)]
pub struct BackgroundRows {
    pub classes: HashMap<i32, u32>,
    pub races: HashMap<u32, RaceBackgroundRow>,
}

impl BackgroundRows {
    pub(crate) fn resolve(&self, record: Option<&CharacterRecord>) -> Option<u32> {
        let race = if let Some(record) = record {
            if record.background_override != 0 {
                return Some(record.background_override);
            }
            if let Some(&class) = self.classes.get(&record.account.class_id)
                && class != 0
            {
                return Some(class);
            }
            if let Some(race) = self.races.get(&record.race_id)
                && race.select_screen_file_data_id != 0
            {
                return Some(race.select_screen_file_data_id);
            }
            record.race_id
        } else {
            // Invalid/empty/unresolved slots do not take the class or primary race path.
            0
        };
        let file_id = self
            .races
            .get(&race)
            .map_or(651555, |row| row.low_res_screen_file_data_id);
        (file_id != 0).then_some(file_id)
    }
}
