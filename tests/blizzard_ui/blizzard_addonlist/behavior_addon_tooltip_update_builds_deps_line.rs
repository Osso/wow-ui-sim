//! Addon tooltip dependency-line behavior for `Blizzard_AddOnList`.

use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;
use wow_ui_sim::lua_api::AddonInfo;

const ROOT: &str = "Blizzard_AddOnList";
const TOOLTIP_ADDON: &str = "AddonTooltipDepsProbe";
const TOOLTIP_DEP: &str = "AddonTooltipDependency";

// Wrath has no AddonList vendor widget/handler input; mixed and modeled probes remain enabled.
timerunning_util_panel_fixture! {
#[test]
fn addon_tooltip_update_builds_dependency_line() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        seed_tooltip_probe_addon(env);

        let (line_count, title_text, version_text, notes_text, deps_matches_prefix, deps_text): (
            i64,
            String,
            String,
            String,
            bool,
            String,
        ) = env
            .eval(
                r#"
                local function FindAddonIndex(addonName)
                    for index = 1, C_AddOns.GetNumAddOns() do
                        if C_AddOns.GetAddOnName(index) == addonName then
                            return index
                        end
                    end
                end

                local owner = CreateFrame("Button", "AddonTooltipDepsProbeOwner", UIParent)
                owner:SetID(FindAddonIndex("AddonTooltipDepsProbe"))
                -- AddonListEntryMixin OnEnter (AddonList.lua:1002) owns the tooltip first.
                AddonTooltip:SetOwner(owner, "ANCHOR_RIGHT", -270, 0)
                AddonTooltip_Update(owner)

                local titleLine = AddonTooltip:GetLeftLine(1)
                local versionLine = AddonTooltip:GetRightLine(1)
                local notesLine = AddonTooltip:GetLeftLine(2)
                local depsLine = AddonTooltip:GetLeftLine(3)
                local depsText = depsLine:GetText()

                return AddonTooltip:NumLines(),
                       titleLine:GetText(),
                       versionLine:GetText(),
                       notesLine:GetText(),
                       depsText:sub(1, #ADDON_DEPENDENCIES) == ADDON_DEPENDENCIES,
                       depsText
                "#,
            )
            .expect("AddonTooltip_Update dependency-line probe must run cleanly");

        assert!(
            line_count >= 4,
            "`AddonTooltip_Update` must populate title, version, notes, and dependency lines; got {line_count}"
        );
        assert_eq!(title_text, "Addon Tooltip Dependency Probe");
        assert_eq!(version_text, "@project-version@");
        assert_eq!(notes_text, "Probe addon notes");
        assert!(
            deps_matches_prefix,
            "dependency line must begin with `ADDON_DEPENDENCIES`; got {deps_text:?}"
        );
        assert!(
            deps_text.contains(TOOLTIP_DEP),
            "dependency line must include the declared dependency; got {deps_text:?}"
        );
    });
}
}

fn seed_tooltip_probe_addon(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    state.addons.push(AddonInfo {
        folder_name: TOOLTIP_DEP.into(),
        title: TOOLTIP_DEP.into(),
        enabled: true,
        loaded: true,
        ..Default::default()
    });
    state.addons.push(AddonInfo {
        folder_name: TOOLTIP_ADDON.into(),
        title: "Addon Tooltip Dependency Probe".into(),
        notes: "Probe addon notes".into(),
        enabled: true,
        loaded: true,
        dependencies: vec![TOOLTIP_DEP.into()],
        ..Default::default()
    });
}
