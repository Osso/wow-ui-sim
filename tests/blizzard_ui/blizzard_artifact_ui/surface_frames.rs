//! Frame-surface probes for `Blizzard_ArtifactUI`.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_ArtifactUI";
const ARTIFACT_FRAME_CHILDREN: &[&str] = &[
    "PerksTab",
    "AppearancesTab",
    "AppearancesTabButton",
    "ForgeBadgeFrame",
    "ForgeLevelFrame",
    "VisitForgeOverlay",
];

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn artifact_ui_exposes_frame_and_overlay_children_after_load_addon() {
    with_blizzard_addon_smoke_shape(&[], &[], |env, _loaded| {
        load_artifact_ui(env);

        let mismatches: Vec<String> = env
            .eval(&frame_surface_probe())
            .expect("ArtifactUI frame surface probe should run cleanly");

        assert!(
            mismatches.is_empty(),
            "`{ROOT}` must expose ArtifactFrame, expected children, and sibling underlay; \
             mismatches: {mismatches:?}"
        );
    });
}
}

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn artifact_ui_registers_uipanel_window_entry_after_load_addon() {
    with_blizzard_addon_smoke_shape(&[], &[], |env, _loaded| {
        load_artifact_ui(env);

        let mismatches: Vec<String> = env
            .eval(UIPANEL_WINDOW_SURFACE_PROBE)
            .expect("ArtifactFrame UIPanelWindows surface probe should run cleanly");

        assert!(
            mismatches.is_empty(),
            "`{ROOT}` must register the ArtifactFrame UIPanelWindows entry; \
             mismatches: {mismatches:?}"
        );
    });
}
}

// ArtifactUI vendor handlers/templates do not ship in Wrath; modeled API probes stay enabled.
post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn artifact_ui_registers_respec_static_popup_dialogs_after_load_addon() {
    with_blizzard_addon_smoke_shape(&[], &[], |env, _loaded| {
        load_artifact_ui(env);

        let mismatches: Vec<String> = env
            .eval(RESPEC_POPUP_SURFACE_PROBE)
            .expect("ArtifactUI respec popup surface probe should run cleanly");

        assert!(
            mismatches.is_empty(),
            "`{ROOT}` must register the artifact-respec static popup dialogs; \
             mismatches: {mismatches:?}"
        );
    });
}
}

fn load_artifact_ui(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let (loaded, error): (bool, Option<String>) = env
        .eval(r#"return C_AddOns.LoadAddOn("Blizzard_ArtifactUI")"#)
        .expect("C_AddOns.LoadAddOn probe should run cleanly");
    assert!(
        loaded,
        "`{ROOT}` must load before frame surface probes; error={error:?}"
    );
}

const UIPANEL_WINDOW_SURFACE_PROBE: &str = r#"
local mismatches = {}

local function expect(condition, message)
    if not condition then
        table.insert(mismatches, message)
    end
end

local entry = UIPanelWindows and UIPanelWindows["ArtifactFrame"]
expect(type(entry) == "table", "UIPanelWindows.ArtifactFrame:" .. type(entry))

if type(entry) == "table" then
    expect(entry.area == "doublewide", "area:" .. tostring(entry.area))
    expect(entry.pushable == 0, "pushable:" .. tostring(entry.pushable))
    expect(entry.xoffset == 35, "xoffset:" .. tostring(entry.xoffset))
    expect(entry.yoffset == -9, "yoffset:" .. tostring(entry.yoffset))
    expect(
        entry.bottomClampOverride == 100,
        "bottomClampOverride:" .. tostring(entry.bottomClampOverride)
    )
    expect(
        type(C_ArtifactUI.Clear) == "function",
        "C_ArtifactUI.Clear:" .. type(C_ArtifactUI.Clear)
    )
    expect(
        entry.showFailedFunc == C_ArtifactUI.Clear,
        "showFailedFunc direct reference"
    )
end

return mismatches
"#;

const RESPEC_POPUP_SURFACE_PROBE: &str = r#"
local mismatches = {}

local function expect(condition, message)
    if not condition then
        table.insert(mismatches, message)
    end
end

local confirm = StaticPopupDialogs and StaticPopupDialogs["CONFIRM_ARTIFACT_RESPEC"]
local notEnough = StaticPopupDialogs and StaticPopupDialogs["NOT_ENOUGH_POWER_ARTIFACT_RESPEC"]
expect(type(confirm) == "table", "CONFIRM_ARTIFACT_RESPEC:" .. type(confirm))
expect(type(notEnough) == "table", "NOT_ENOUGH_POWER_ARTIFACT_RESPEC:" .. type(notEnough))

if type(confirm) == "table" then
    expect(confirm.text == ARTIFACT_RESPEC, "confirm text")
    expect(confirm.button1 == YES, "confirm button1")
    expect(confirm.button2 == NO, "confirm button2")
    expect(type(confirm.OnAccept) == "function", "confirm OnAccept")
    expect(type(confirm.OnUpdate) == "function", "confirm OnUpdate")
end

if type(notEnough) == "table" then
    expect(notEnough.text == ARTIFACT_RESPEC_NOT_ENOUGH_POWER, "not-enough text")
    expect(notEnough.button1 == OKAY, "not-enough button1")
    expect(type(notEnough.OnUpdate) == "function", "not-enough OnUpdate")
end

local originalConfirmRespec = C_ArtifactUI.ConfirmRespec
local originalCheckRespecNPC = C_ArtifactUI.CheckRespecNPC
local originalHideUIPanel = HideUIPanel
local originalStaticPopupHide = StaticPopup_Hide
local calls = {}

C_ArtifactUI.ConfirmRespec = function()
    table.insert(calls, "confirm")
end
C_ArtifactUI.CheckRespecNPC = function()
    table.insert(calls, "check")
    return false
end
HideUIPanel = function(frame)
    table.insert(calls, frame == ArtifactFrame and "hide-panel" or "hide-other")
end
StaticPopup_Hide = function(which)
    table.insert(calls, "hide-popup:" .. tostring(which))
end

local ok, errorMessage = pcall(function()
    if type(confirm) == "table" and type(confirm.OnAccept) == "function" then
        confirm.OnAccept({}, nil)
    end
    if type(confirm) == "table" and type(confirm.OnUpdate) == "function" then
        confirm.OnUpdate({}, 0)
    end
    if type(notEnough) == "table" and type(notEnough.OnUpdate) == "function" then
        notEnough.OnUpdate({}, 0)
    end
end)

C_ArtifactUI.ConfirmRespec = originalConfirmRespec
C_ArtifactUI.CheckRespecNPC = originalCheckRespecNPC
HideUIPanel = originalHideUIPanel
StaticPopup_Hide = originalStaticPopupHide

expect(ok, "popup callbacks error:" .. tostring(errorMessage))
expect(calls[1] == "confirm", "confirm OnAccept first call:" .. tostring(calls[1]))
expect(calls[2] == "hide-panel", "confirm OnAccept second call:" .. tostring(calls[2]))
expect(calls[3] == "check", "confirm OnUpdate CheckRespecNPC:" .. tostring(calls[3]))
expect(
    calls[4] == "hide-popup:CONFIRM_ARTIFACT_RESPEC",
    "confirm OnUpdate StaticPopup_Hide:" .. tostring(calls[4])
)
expect(calls[5] == "hide-panel", "confirm OnUpdate HideUIPanel:" .. tostring(calls[5]))
expect(calls[6] == "check", "not-enough OnUpdate CheckRespecNPC:" .. tostring(calls[6]))
expect(
    calls[7] == "hide-popup:NOT_ENOUGH_POWER_ARTIFACT_RESPEC",
    "not-enough OnUpdate StaticPopup_Hide:" .. tostring(calls[7])
)
expect(calls[8] == "hide-panel", "not-enough OnUpdate HideUIPanel:" .. tostring(calls[8]))

return mismatches
"#;

fn frame_surface_probe() -> String {
    let child_list = lua_array_literal(ARTIFACT_FRAME_CHILDREN);

    format!(
        r#"
        local children = {{{child_list}}}
        local mismatches = {{}}

        local function expect(condition, message)
            if not condition then
                table.insert(mismatches, message)
            end
        end

        local frame = ArtifactFrame
        local underlay = ArtifactFrameUnderlay
        expect(type(frame) == "table", "ArtifactFrame:" .. type(frame))
        expect(frame and frame:GetObjectType() == "Frame", "ArtifactFrame object type")
        expect(frame and frame:GetParent() == UIParent, "ArtifactFrame parent")

        for _, childName in ipairs(children) do
            local child = frame and frame[childName]
            expect(type(child) == "table", "ArtifactFrame." .. childName .. ":" .. type(child))
            expect(child and child:GetParent() == frame, "ArtifactFrame." .. childName .. " parent")
        end

        expect(type(underlay) == "table", "ArtifactFrameUnderlay:" .. type(underlay))
        expect(underlay and underlay:GetObjectType() == "Frame", "ArtifactFrameUnderlay object type")
        expect(underlay and underlay:GetParent() == UIParent, "ArtifactFrameUnderlay parent")
        expect(underlay and underlay ~= frame, "ArtifactFrameUnderlay is distinct from ArtifactFrame")
        expect(
            underlay and frame and underlay:GetParent() == frame:GetParent(),
            "ArtifactFrameUnderlay sibling parent"
        )

        return mismatches
        "#
    )
}

fn lua_array_literal(values: &[&str]) -> String {
    values
        .iter()
        .map(|value| format!("{value:?}"))
        .collect::<Vec<_>>()
        .join(", ")
}
