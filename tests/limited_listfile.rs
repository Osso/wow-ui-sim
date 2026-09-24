#[test]
fn bundled_limited_listfile_resolves_common_assets_case_insensitively() {
    assert_eq!(
        wow_ui_sim::limited_listfile::lookup_path("Fonts/frizqt__.ttf"),
        Some(615960)
    );
    assert_eq!(
        wow_ui_sim::limited_listfile::lookup_path("INTERFACE/BUTTONS/UI-PANEL-BUTTON-UP.BLP"),
        Some(130828)
    );
    assert_eq!(
        wow_ui_sim::limited_listfile::lookup_path("Interface/Icons/Trade_Engineering.blp"),
        Some(136243)
    );
}

#[test]
fn bundled_limited_listfile_resolves_forever_character_tab_icons() {
    // Verified against the active Forever 1.60.1.69977 CASC root name hashes.
    for (name, fdid) in [
        ("Currency", 8197078),
        ("Honor_Alliance", 8197097),
        ("Reputation2", 8197103),
        ("Stats", 8197104),
    ] {
        let path = format!("Interface\\Icons\\INV_SideTab_{name}_c60");
        assert_eq!(
            wow_ui_sim::limited_listfile::lookup_texture_path(&path),
            Some(fdid),
            "{path}"
        );
    }
}

#[test]
fn bundled_limited_listfile_preserves_canonical_override_path_casing() {
    let expected_entries = [
        ("fonts/frizqt__.ttf", 615960, "Fonts/FRIZQT__.TTF"),
        (
            "interface/buttons/ui-panel-button-up.blp",
            130828,
            "Interface/Buttons/UI-Panel-Button-Up.blp",
        ),
        (
            "interface/icons/trade_engineering.blp",
            136243,
            "Interface/Icons/Trade_Engineering.blp",
        ),
    ];

    for (lookup_path, expected_fdid, expected_path) in expected_entries {
        let entry = wow_ui_sim::limited_listfile::lookup_entry(lookup_path)
            .unwrap_or_else(|| panic!("missing canonical listfile entry for {lookup_path}"));

        assert_eq!(entry.fdid, expected_fdid);
        assert_eq!(entry.path, expected_path);
    }
}
