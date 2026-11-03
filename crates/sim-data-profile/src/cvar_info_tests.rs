//! Captured metadata integrity and profile selection, without Lua runtime behavior.

use crate::cvar_info::CVAR_FLAGS;

#[test]
fn captured_keys_and_masks_match_the_catalog_contract() {
    for (&name, &mask) in CVAR_FLAGS.entries() {
        assert!(!name.is_empty());
        assert!(
            name.bytes().all(|byte| {
                byte.is_ascii_lowercase()
                    || byte.is_ascii_digit()
                    || matches!(byte, b'_' | b'.' | b'-')
            }),
            "invalid lowercase CVar key: {name}"
        );
        assert_eq!(mask & !0b1_1111, 0, "unknown flag bits for {name}");
    }
}

#[cfg(any(
    feature = "profile-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]
#[test]
fn captured_zero_flags_remain_distinct_from_missing_metadata() {
    assert_eq!(CVAR_FLAGS.get("useipv6"), Some(&0));
    assert_eq!(CVAR_FLAGS.get("__missing_cvar_info_probe"), None);
    assert_eq!(CVAR_FLAGS.get("praisethesun"), None);
    assert_eq!(CVAR_FLAGS.get("ttsusecharactersettings"), None);
    assert_eq!(CVAR_FLAGS.get("showtimestamps"), Some(&1));
    assert_eq!(CVAR_FLAGS.get("nameplatemaxdistance"), Some(&10));
    assert_eq!(CVAR_FLAGS.get("accountneedsturnstrafedialog"), Some(&5));
    assert_eq!(CVAR_FLAGS.get("actionbuttonusekeydown"), Some(&9));
    assert_eq!(CVAR_FLAGS.get("portal"), Some(&16));
}

#[cfg(feature = "profile-retail")]
#[test]
fn retail_keeps_its_captured_scopes_and_catalog_size() {
    assert_eq!(CVAR_FLAGS.len(), 1663);
    assert_eq!(CVAR_FLAGS.get("kioskcharactertemplateset"), Some(&24));
    assert_eq!(CVAR_FLAGS.get("kioskcansessionexpire"), Some(&24));
    assert_eq!(CVAR_FLAGS.get("nameplateshowclasscolor"), Some(&8));
    assert_eq!(CVAR_FLAGS.get("simplekeybinds"), None);
    assert_eq!(CVAR_FLAGS.get("bloomenable"), None);
}

#[cfg(feature = "client-ptr")]
#[test]
fn ptr_keeps_its_captured_scopes_and_catalog_size() {
    assert_eq!(CVAR_FLAGS.len(), 1780);
    assert_eq!(CVAR_FLAGS.get("simplekeybinds"), Some(&2));
    assert_eq!(
        CVAR_FLAGS.get("usekeyheldspellinitialrecastdelay"),
        Some(&5)
    );
    assert_eq!(CVAR_FLAGS.get("kioskcansessionexpire"), Some(&8));
    assert_eq!(CVAR_FLAGS.get("nameplateshowclasscolor"), Some(&8));
    assert_eq!(CVAR_FLAGS.get("kioskcharactertemplateset"), None);
    assert_eq!(CVAR_FLAGS.get("bloomenable"), None);
}

#[cfg(feature = "client-wowforever")]
#[test]
fn forever_keeps_its_captured_scopes_and_catalog_size() {
    assert_eq!(CVAR_FLAGS.len(), 1786);
    assert_eq!(CVAR_FLAGS.get("nameplateshowclasscolor"), Some(&1));
    assert_eq!(CVAR_FLAGS.get("kioskcansessionexpire"), Some(&8));
    assert_eq!(CVAR_FLAGS.get("bloomenable"), Some(&0));
    assert_eq!(CVAR_FLAGS.get("simplekeybinds"), None);
    assert_eq!(CVAR_FLAGS.get("kioskcharactertemplateset"), None);
}

#[cfg(not(any(
    feature = "profile-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
)))]
#[test]
fn unsupported_profiles_have_no_borrowed_native_metadata() {
    assert!(CVAR_FLAGS.is_empty());
    assert_eq!(CVAR_FLAGS.get("useipv6"), None);
}
