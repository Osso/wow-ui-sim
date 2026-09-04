use std::fs;
use std::path::Path;
use std::time::Duration;

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::startup::process_pending_timers;

use crate::common;

const ADDON_NAME: &str = "FramePositionProbe";
const ADDON_SOURCE: &str = "docs/addons/FramePositionProbe/FramePositionProbe.lua";

#[test]
fn frame_position_probe_records_lifecycle_manual_and_missing_frame_state() {
    let env = probe_env();
    load_probe_source(&env);
    common::fire_addon_loaded(&env, ADDON_NAME);
    env.fire_event("PLAYER_LOGIN").expect("fire login");
    common::fire_player_entering_world(&env, true, false);
    process_pending_timers(&env);
    std::thread::sleep(Duration::from_millis(5200));
    process_pending_timers(&env);
    env.exec("SlashCmdList.FRAMEPOSITIONPROBE('test')")
        .expect("capture manual sample");
    env.exec(
        r#"
        local samples = FramePositionProbeDB.samples
        assert(#samples == 6)
        assert(samples[1].kind == "PLAYER_LOGIN")
        assert(samples[2].kind == "PLAYER_ENTERING_WORLD")
        assert(samples[3].label == "world+0")
        assert(samples[4].label == "world+2")
        assert(samples[5].label == "world+5")
        local sample = samples[6]
        assert(sample.kind == "manual" and sample.label == "test")
        assert(sample.build.version == "12.1.0" and sample.build.build == "69587")
        assert(sample.build.interface == 120100)
        local private = sample.frames.PrivateRaidBossEmoteFrameAnchor
        assert(private.present and private.parent == "UIParent", "parent name lost")
        assert(private.rect.left == 400 and private.rect.bottom == 938)
        assert(private.rect.width == 800 and private.rect.height == 80)
        assert(private.points[1].point == "TOP")
        assert(private.points[1].relativeTo == "RaidWarningFrame")
        assert(private.points[1].relativePoint == "TOP")
        assert(private.points[1].x == 0 and private.points[1].y == 0)
        assert(sample.frames.ObjectiveTrackerFrame.parent == "RightManagedFrameContainer")
        assert(sample.frames.UIParentRightManagedFrameContainer.present == false)
        assert(sample.frames.DeadlyDebuffFrame.shown == false)
        assert(sample.raidWarning.lowestMessagePresent == false)
        local left, bottom, width, height = PrivateRaidBossEmoteFrameAnchor:GetRect()
        assert(left == 400 and bottom == 938 and width == 800 and height == 80)
        assert(ObjectiveTrackerFrame:GetParent() == RightManagedFrameContainer)
        assert(not DeadlyDebuffFrame:IsShown())
        "#,
    )
    .expect("capture must retain exact state without changing geometry");
}

#[test]
fn frame_position_probe_marks_secrets_and_records_api_errors() {
    let env = probe_env();
    env.exec(
        r#"
        issecretvalue = function(value) return value == 2222 end
        PrivateRaidBossEmoteFrameAnchor.GetEffectiveScale = function() return 2222 end
        RaidWarningFrame.GetLowestMessage = function() error("fixture lowest message error") end
        C_EditMode.GetLayouts = function() return { activeLayout = 7 } end
        EditModeManagerFrame = {
            IsEditModeActive = function() return false end,
            GetActiveLayoutInfo = function()
                return { layoutName = "Fixture", layoutType = 1, systems = {
                    { system = Enum.EditModeSystem.RaidWarning, systemIndex = 0,
                      anchorInfo = { point = "TOP", relativeTo = "UIParent", relativePoint = "TOP",
                                     offsetX = 0, offsetY = -182 } },
                } }
            end,
        }
        "#,
    )
    .expect("install concrete inaccessible-value fixture");
    load_probe_source(&env);
    env.exec(
        r#"
        SlashCmdList.FRAMEPOSITIONPROBE("errors")
        local sample = FramePositionProbeDB.samples[1]
        local private = sample.frames.PrivateRaidBossEmoteFrameAnchor
        assert(private.effectiveScale == "<secret>", "secret must have an explicit marker")
        assert(private.errors.effectiveScale == "secret value")
        assert(sample.raidWarning.errors.getLowestMessage:find("fixture lowest message error", 1, true))
        assert(sample.editMode.activeLayout == 7, "active layout index missing")
        assert(sample.editMode.layoutName == "Fixture" and sample.editMode.layoutType == 1)
        assert(sample.editMode.systemAnchors[1].system == Enum.EditModeSystem.RaidWarning)
        assert(sample.editMode.systemAnchors[1].anchorInfo.offsetY == -182)
        local function checkSerializable(value)
            local kind = type(value)
            assert(kind == "table" or kind == "string" or kind == "number" or kind == "boolean")
            assert(value ~= 2222, "secret leaked")
            if kind == "table" then
                assert(getmetatable(value) == nil, "frame object leaked into capture")
                for key, item in pairs(value) do
                    assert(type(key) == "string" or type(key) == "number")
                    checkSerializable(item)
                end
            end
        end
        checkSerializable(FramePositionProbeDB)
        "#,
    )
    .expect("capture must distinguish errors/secrets and contain serializable values only");
}

fn probe_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create Lua environment");
    env.set_screen_size(1600.0, 1200.0);
    env.exec(
        r#"
        GetBuildInfo = function() return "12.1.0", "69587", "Sep 4 2026", 120100 end
        local function create(name, parent)
            local frame = CreateFrame("Frame", name, parent or UIParent)
            frame:SetSize(800, 80)
            return frame
        end
        RaidWarningFrame = create("RaidWarningFrame")
        RaidWarningFrame:SetPoint("TOP", UIParent, "TOP", 0, -182)
        RaidWarningFrame.GetLowestMessage = function() return nil end
        PrivateRaidBossEmoteFrameAnchor = create("PrivateRaidBossEmoteFrameAnchor")
        PrivateRaidBossEmoteFrameAnchor:SetPoint("TOP", RaidWarningFrame, "TOP", 0, 0)
        DeadlyDebuffFrame = create("DeadlyDebuffFrame")
        DeadlyDebuffFrame:Hide()
        RightManagedFrameContainer = create("RightManagedFrameContainer")
        RightManagedFrameContainer:SetPoint("TOPRIGHT", UIParent, "TOPRIGHT", -5, -260)
        ObjectiveTrackerFrame = create("ObjectiveTrackerFrame", RightManagedFrameContainer)
        ObjectiveTrackerFrame:SetPoint("TOPRIGHT", RightManagedFrameContainer, "TOPRIGHT", 0, 0)
        "#,
    )
    .expect("install concrete frame fixture");
    env
}

fn load_probe_source(env: &WowLuaEnv) {
    let source = fs::read_to_string(Path::new(ADDON_SOURCE))
        .unwrap_or_else(|error| panic!("read {ADDON_SOURCE}: {error}"));
    let addon_table = env.create_addon_table().expect("create addon table");
    env.loader_env()
        .exec_with_varargs(&source, ADDON_SOURCE, ADDON_NAME, addon_table)
        .unwrap_or_else(|error| panic!("load {ADDON_SOURCE}: {error}"));
}
