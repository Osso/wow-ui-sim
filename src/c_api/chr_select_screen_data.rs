//! Profile-selected class and race select-screen FileDataIDs.

use crate::client_profile::{ACTIVE, ClientProfile};

mod retail {
    include!("../../data/chr-select-screen/retail-12.1.5.70783/tables.rs");
}

mod forever {
    include!("../../data/chr-select-screen/forever-1.60.1.70888/tables.rs");
}

pub fn class_select_screen_fdid(class_id: i32) -> Option<i64> {
    let rows = match ACTIVE {
        ClientProfile::Retail => retail::CLASS_ROWS,
        ClientProfile::WowForever => forever::CLASS_ROWS,
        _ => return None,
    };
    let index = rows.binary_search_by_key(&class_id, |(id, _)| *id).ok()?;
    Some(rows[index].1)
}

pub fn race_select_screen_fdids(race_id: i32) -> Option<(i64, i64)> {
    let rows = match ACTIVE {
        ClientProfile::Retail => retail::RACE_ROWS,
        ClientProfile::WowForever => forever::RACE_ROWS,
        _ => return None,
    };
    let index = rows.binary_search_by_key(&race_id, |(id, _, _)| *id).ok()?;
    Some((rows[index].1, rows[index].2))
}

#[cfg(test)]
mod tests {
    use super::{class_select_screen_fdid, race_select_screen_fdids};
    use crate::client_profile::{ACTIVE, ClientProfile};

    #[test]
    fn returns_exact_rows_from_the_profile_export() {
        match ACTIVE {
            ClientProfile::Retail => {
                assert_eq!(class_select_screen_fdid(1), Some(0));
                assert_eq!(class_select_screen_fdid(6), Some(236_082));
                assert_eq!(class_select_screen_fdid(12), Some(1_305_036));
                assert_eq!(race_select_screen_fdids(1), Some((131_948, 651_555)));
                assert_eq!(race_select_screen_fdids(12), Some((0, 0)));
                assert_eq!(race_select_screen_fdids(84), Some((5_797_525, 651_558)));
            }
            ClientProfile::WowForever => {
                assert_eq!(class_select_screen_fdid(1), Some(0));
                assert_eq!(class_select_screen_fdid(7), Some(0));
                assert_eq!(race_select_screen_fdids(1), Some((131_948, 0)));
                assert_eq!(race_select_screen_fdids(12), Some((0, 0)));
                assert_eq!(race_select_screen_fdids(95), Some((8_035_354, 8_035_354)));
            }
            _ => {}
        }
    }

    #[test]
    fn absent_rows_return_none() {
        assert_eq!(class_select_screen_fdid(999), None);
        assert_eq!(race_select_screen_fdids(999), None);
        if ACTIVE == ClientProfile::WowForever {
            assert_eq!(class_select_screen_fdid(6), None);
        }
    }

    #[test]
    fn profiles_without_select_screen_exports_return_none() {
        if !matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever) {
            assert_eq!(class_select_screen_fdid(1), None);
            assert_eq!(race_select_screen_fdids(1), None);
        }
    }
}
