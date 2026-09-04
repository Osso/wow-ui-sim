use std::fs;
use std::path::Path;

use wow_ui_sim::lua_api::WowLuaEnv;

use crate::common;

const ADDON_NAME: &str = "FramePositionProbe";
const ADDON_SOURCE: &str = "docs/addons/FramePositionProbe/FramePositionProbe.lua";

#[test]
fn frame_position_probe_records_lifecycle_manual_and_missing_frame_state() {
    let env = WowLuaEnv::new().expect("create Lua environment");
    install_named_probe_frames(&env);
    load_probe_source(&env);

    common::fire_addon_loaded(&env, ADDON_NAME);
    env.fire_event("PLAYER_LOGIN")
        .expect("fire PLAYER_LOGIN");
    install_timer_recorder(&env);
    common::fire_player_entering_world(&env, true, false);
    run_recorded_timer_callbacks(&env);
    env.exec("SlashCmdList.FRAMEPOSITIONPROBE('test')")
        .expect("run manual probe command");

    let result: (bool, bool, bool, bool, bool, bool, bool, bool) = env
        .eval(
            r#"
            local db = FramePositionProbeDB
            local samples = db and db.samples or {}
            local login = samples[1]
            local world = samples[2]
            local manual = samples[#samples]
            local frames = manual and manual.frames or {}
            local private = frames.PrivateRaidBossEmoteFrameAnchor
            local missing = frames.UIParentRightManagedFrameContainer
            return type(db) == "table",
                   login and login.kind == "PLAYER_LOGIN",
                   world and world.kind == "PLAYER_ENTERING_WORLD",
                   manual and manual.kind == "manual" and manual.label == "test",
                   private and private.present and type(private.rect) == "table"
                       and type(private.points) == "table",
                   missing and missing.present == false,
                   type(manual.errors) == "table",
                   samples[3] and samples[3].kind == "delayed" and samples[3].label == "world+0"
                       and samples[4] and samples[4].kind == "delayed" and samples[4].label == "world+2"
                       and samples[5] and samples[5].kind == "delayed" and samples[5].label == "world+5"
            "#,
        )
        .expect("read probe capture");

    assert!(result.0, "probe must reset its SavedVariables table at load");
    assert!(result.1, "probe must capture PLAYER_LOGIN");
    assert!(result.2, "probe must capture PLAYER_ENTERING_WORLD");
    assert!(result.3, "manual /fpprobe capture must append a labeled sample");
    assert!(result.4, "named present frames must record raw rects and points");
    assert!(result.5, "missing frames must be explicit rather than omitted");
    assert!(result.6, "each sample must retain explicit API errors");
    assert!(result.7, "world entry must schedule 0, 2, and 5 second samples");
}

fn install_timer_recorder(env: &WowLuaEnv) {
    env.exec(
        r#"
        FramePositionProbeTimerCallbacks = {}
        C_Timer.After = function(delay, callback)
            local info = debug.getinfo(callback, "S")
            FramePositionProbeTimerCallbacks[#FramePositionProbeTimerCallbacks + 1] = {
                delay = delay,
                callback = callback,
                source = info and info.source,
            }
        end
        "#,
    )
    .expect("record delayed probe callbacks");
}

fn run_recorded_timer_callbacks(env: &WowLuaEnv) {
    env.exec(
        r#"
        local probeTimers = {}
        for _, timer in ipairs(FramePositionProbeTimerCallbacks) do
            if timer.source:match("FramePositionProbe%.lua$") then
                probeTimers[#probeTimers + 1] = timer
            end
        end
        assert(#probeTimers == 3, "probe timers=" .. #probeTimers)
        assert(probeTimers[1].delay == 0)
        assert(probeTimers[2].delay == 2)
        assert(probeTimers[3].delay == 5)
        for _, timer in ipairs(probeTimers) do
            timer.callback()
        end
        "#,
    )
    .expect("run delayed probe callbacks");
}

fn install_named_probe_frames(env: &WowLuaEnv) {
    env.exec(
        r#"
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
    .expect("install concrete named frame fixture");
}

fn load_probe_source(env: &WowLuaEnv) {
    let source = fs::read_to_string(Path::new(ADDON_SOURCE))
        .unwrap_or_else(|error| panic!("read {ADDON_SOURCE}: {error}"));
    let addon_table = env
        .create_addon_table()
        .expect("create addon table for probe");

    env.loader_env()
        .exec_with_varargs(&source, ADDON_SOURCE, ADDON_NAME, addon_table)
        .unwrap_or_else(|error| panic!("load {ADDON_SOURCE}: {error}"));
}
