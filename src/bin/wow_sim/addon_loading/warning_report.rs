use wow_ui_sim::loader::LoadResult;

const VERBOSE_WARNING_ADDONS: &[&str] = &[
    "BetterWardrobe",
    "Plumber",
    "BetterBlizzFrames",
    "Baganator",
    "Angleur",
    "ExtraQuestButton",
    "WaypointUI",
    "TomTom",
    "WorldQuestTracker",
    "SavedInstances",
    "Rarity",
    "SimpleItemLevel",
    "TalentLoadoutManager",
    "Simulationcraft",
    "TomCats",
    "RaiderIO",
    "!BugGrabber",
    "CraftSim",
    "AdvancedInterfaceOptions",
    "BlizzMove_Debug",
    "ClickableRaidBuffs",
    "Dejunk",
    "Cell",
    "AngryKeystones",
    "AutoPotion",
    "BigWigs_Plugins",
    "BugSack",
    "Clicked",
    "DeathNote",
    "DeModal",
    "ElvUI_OptionsUI",
    "DragonRaceTimes",
    "DynamicCam",
    "DialogueUI",
    "Chattynator",
    "AstralKeys",
    "Leatrix_Plus",
    "CooldownToGo_Options",
    "HousingItemTracker",
    "idTip",
    "Macroriffic",
    "NameplateSCT",
    "Krowi_ExtendedVendorUI",
    "OmniCD",
    "Auctionator",
    "EditModeExpanded",
    "GlobalIgnoreList",
    "AllTheThings",
    "BigWigs_KhazAlgar",
    "LegionRemixHelper",
    "Collectionator",
    "Syndicator",
    "BigWigs",
    "!KalielsTracker",
    "KRaidSkipTracker",
    "MacroToolkit",
    "MinimapButtonButton",
    "OribosExchange",
];

pub(super) fn print_addon_warnings(name: &str, result: &LoadResult) {
    emit_addon_warnings(
        name,
        result,
        std::env::var("WOW_SIM_DEBUG_NIL_GLOBALS").is_ok(),
        |line| println!("{line}"),
    );
}

fn emit_addon_warnings(
    name: &str,
    result: &LoadResult,
    debug_enabled: bool,
    mut emit: impl FnMut(String),
) {
    if !debug_enabled {
        return;
    }

    for warning in &result.warnings {
        emit(format!("  [failure] {warning}"));
    }
    if !VERBOSE_WARNING_ADDONS.contains(&name) {
        return;
    }
    for observation in &result.nil_symbol_observations {
        emit(format!("  [nil-observation] {observation}"));
    }
    for requirement in &result.missing_requirements {
        emit(format!("  [missing-requirement] {requirement}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wow_ui_sim::loader::{
        LoadDiagnosticAttribution, LoadTiming, MissingRequirement, MissingRequirementKind,
        NilSymbolEnvironment, NilSymbolObservation, NilSymbolObservationKind,
    };

    fn load_result(name: &str, warnings: &[&str]) -> LoadResult {
        let attribution = LoadDiagnosticAttribution {
            addon_name: name.to_string(),
            environment: NilSymbolEnvironment::Public,
            source: Some(format!("Interface/AddOns/{name}/Probe.lua")),
            line: Some(12),
        };
        LoadResult {
            name: name.to_string(),
            lua_files: 1,
            xml_files: 1,
            timing: LoadTiming::default(),
            warnings: warnings.iter().map(|warning| warning.to_string()).collect(),
            nil_symbol_observations: vec![NilSymbolObservation {
                kind: NilSymbolObservationKind::Global {
                    name: "OptionalProbe".to_string(),
                },
                attribution: attribution.clone(),
            }],
            missing_requirements: vec![MissingRequirement {
                kind: MissingRequirementKind::CMethod {
                    namespace: "C_Probe".to_string(),
                    method: "Read".to_string(),
                },
                attribution,
            }],
        }
    }

    fn results() -> [LoadResult; 2] {
        [
            load_result("TomTom", &["TomTom: synthetic Lua failure"]),
            load_result(
                "DiagnosticProbe",
                &[
                    "DiagnosticProbe: synthetic XML failure",
                    "DiagnosticProbe: synthetic Lua failure",
                    "DiagnosticProbe: synthetic missing file",
                    "DiagnosticProbe: synthetic runtime failure",
                    "DiagnosticProbe: synthetic template failure",
                    "DiagnosticProbe: synthetic script failure",
                    "DiagnosticProbe: synthetic nested load failure",
                ],
            ),
        ]
    }

    #[test]
    fn debug_enabled_emits_all_eight_warnings_and_keeps_diagnostic_allowlist() {
        let mut lines = Vec::new();
        for result in results() {
            emit_addon_warnings(&result.name, &result, true, |line| lines.push(line));
        }
        assert_eq!(
            lines,
            [
                "  [failure] TomTom: synthetic Lua failure",
                "  [nil-observation] TomTom observed nil global OptionalProbe in public environment (accessed at Interface/AddOns/TomTom/Probe.lua:12)",
                "  [missing-requirement] TomTom needs C_Probe.Read in public environment (accessed at Interface/AddOns/TomTom/Probe.lua:12)",
                "  [failure] DiagnosticProbe: synthetic XML failure",
                "  [failure] DiagnosticProbe: synthetic Lua failure",
                "  [failure] DiagnosticProbe: synthetic missing file",
                "  [failure] DiagnosticProbe: synthetic runtime failure",
                "  [failure] DiagnosticProbe: synthetic template failure",
                "  [failure] DiagnosticProbe: synthetic script failure",
                "  [failure] DiagnosticProbe: synthetic nested load failure",
            ]
        );
    }

    #[test]
    fn debug_disabled_emits_no_warning_or_diagnostic_lines() {
        let mut lines = Vec::new();
        for result in results() {
            emit_addon_warnings(&result.name, &result, false, |line| lines.push(line));
        }
        assert!(lines.is_empty(), "unexpected flag-off output: {lines:?}");
    }
}
