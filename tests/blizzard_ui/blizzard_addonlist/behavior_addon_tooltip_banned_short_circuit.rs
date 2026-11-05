//! Banned-addon tooltip short-circuit behavior for `Blizzard_AddOnList`.

use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;
use wow_ui_sim::lua_api::AddonInfo;

const ROOT: &str = "Blizzard_AddOnList";
const BANNED_ADDON: &str = "AddonTooltipBannedProbe";
const BANNED_DEP: &str = "AddonTooltipBannedDependency";

// Wrath has no AddonList vendor widget/handler input; mixed and modeled probes remain enabled.
post_wrath_ui_fixture! {
#[test]
fn addon_tooltip_update_banned_security_short_circuits_details() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        seed_banned_probe_addon(env);

        let (line_count, first_line_text, second_line_exists, matches_banned_text): (
            i64,
            String,
            bool,
            bool,
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
                local addonIndex = FindAddonIndex("AddonTooltipBannedProbe")

                local owner = CreateFrame("Button", "AddonTooltipBannedProbeOwner", UIParent)
                owner:SetID(addonIndex)
                -- AddonListEntryMixin OnEnter (AddonList.lua:1002) owns the tooltip first.
                AddonTooltip:SetOwner(owner, "ANCHOR_RIGHT", -270, 0)
                AddonTooltip_Update(owner)

                local firstLine = AddonTooltip:GetLeftLine(1)
                local secondLine = AddonTooltip:GetLeftLine(2)
                local firstText = firstLine:GetText()

                return AddonTooltip:NumLines(),
                       firstText,
                       secondLine ~= nil,
                       firstText == ADDON_BANNED_TOOLTIP
                "#,
            )
            .expect("AddonTooltip_Update banned-addon probe must run cleanly");

        assert_eq!(
            line_count, 1,
            "`AddonTooltip_Update` must stop after one line for banned addons"
        );
        assert!(
            matches_banned_text,
            "banned addon tooltip must use `ADDON_BANNED_TOOLTIP`; got {first_line_text:?}"
        );
        assert!(
            !second_line_exists,
            "banned addon tooltip must skip title/version/notes/dependencies"
        );
    });
}
}

fn seed_banned_probe_addon(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    state.addons.push(AddonInfo {
        folder_name: BANNED_DEP.into(),
        title: BANNED_DEP.into(),
        enabled: true,
        loaded: true,
        ..Default::default()
    });
    state.addons.push(AddonInfo {
        folder_name: BANNED_ADDON.into(),
        title: "Addon Tooltip Banned Probe".into(),
        notes: "Banned probe notes".into(),
        enabled: true,
        loaded: false,
        security: Some("BANNED".into()),
        dependencies: vec![BANNED_DEP.into()],
        ..Default::default()
    });
}
