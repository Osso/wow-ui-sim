//! Retail CVar write protection introduced in patch 5.4.8.
//! This is C_CVar backing policy shared with the legacy SetCVar entry points.

/// Historical protected names; missing/retired defaults are not republished here.
pub(crate) fn is_combat_protected_cvar(name: &str) -> bool {
    cfg!(feature = "client-retail")
        && matches!(
            name.to_ascii_lowercase().as_str(),
            "alwaysshowactionbars"
                | "bloatnameplates"
                | "bloattest"
                | "bloatthreat"
                | "consolidatebuffs"
                | "fullsizefocusframe"
                | "maxalgoplates"
                | "nameplatemotion"
                | "nameplateoverlaph"
                | "nameplateoverlapv"
                | "nameplateshowenemies"
                | "nameplateshowenemyguardians"
                | "nameplateshowenemypets"
                | "nameplateshowenemytotems"
                | "nameplateshowfriendlyguardians"
                | "nameplateshowfriendlypets"
                | "nameplateshowfriendlytotems"
                | "nameplateshowfriends"
                | "repositionfrequency"
                | "showarenaenemyframes"
                | "showarenaenemypets"
                | "showpartypets"
                | "showtargetoftarget"
                | "targetoftargetmode"
                | "uiscale"
                | "usecompactpartyframes"
                | "useuiscale"
        )
}
