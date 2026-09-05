use std::fs;
use std::path::Path;

use wow_ui_sim::lua_api::WowLuaEnv;

const ADDON_NAME: &str = "PixelRoundingProbe";
const ADDON_SOURCE: &str = "docs/addons/PixelRoundingProbe/PixelRoundingProbe.lua";

#[test]
fn pixel_rounding_probe_records_read_only_cases_and_capture_errors() {
    let env = WowLuaEnv::new().expect("create Lua environment");
    env.exec(
        r#"
        GetBuildInfo = function() return "12.1.5", "69594", "Sep 5 2026", 120105 end
        GetPhysicalScreenWidth = nil
        GetPhysicalScreenHeight = nil
        GetPhysicalScreenSize = function() return 3440, 1440 end
        InCombatLockdown = function() return false end
        C_Timer.After = function(_, callback) callback() end
        local originalCreateFrame = CreateFrame
        CreateFrame = function(...)
            local frame = originalCreateFrame(...)
            frame.SetRoundLayoutToNearestPixel = false
            return frame
        end
        "#,
    )
    .expect("install probe fixture");
    load_probe_source(&env);
    env.exec("SlashCmdList.PIXELROUNDINGPROBE('test')")
        .expect("capture probe");
    env.exec(
        r#"
        local samples = PixelRoundingProbeDB.samples
        assert(#samples == 2, "next-tick plus settled capture")
        local sample = samples[1]
        assert(sample.label == "test+next-tick")
        assert(sample.build.build == "69594" and sample.build.interface == 120105)
        assert(sample.build.matchesExpectedBuild == true)
        assert(sample.physicalScreen.width == 3440 and sample.physicalScreen.height == 1440)
        assert(#sample.cases == 12, "compact rounding case matrix")
        assert(sample.cases[1].label == "default")
        assert(sample.cases[2].label == "bottomleft-fractional")
        assert(sample.cases[3].label == "center-negative")
        assert(sample.cases[4].label == "stretch-two-anchors")
        assert(sample.cases[11].label == "parent-scale-and-reposition")
        assert(sample.cases[12].label == "regions-round-layout")
        assert(sample.cases[2].after.points[1].point == "BOTTOMLEFT")
        assert(sample.cases[2].after.points[1].x == 0.375)
        assert(sample.cases[2].after.points[1].y == -0.625)
        assert(sample.cases[8].afterFlag ~= nil)
        assert(sample.cases[9].beforeFlag ~= nil)
        assert(sample.cases[11].parentAfter ~= nil)
        assert(sample.cases[12].texture and sample.cases[12].fontString)
        assert(not (sample.cases[1].after.errors and sample.cases[1].after.errors.setRoundLayout))
        assert(sample.cases[8].errors.setRoundLayout, "unavailable native method is recorded")
        local function serializable(value)
            local kind = type(value)
            assert(kind == "nil" or kind == "string" or kind == "number" or kind == "boolean" or kind == "table")
            if kind == "table" then
                assert(getmetatable(value) == nil, "probe must not retain frame userdata")
                for _, child in pairs(value) do serializable(child) end
            end
        end
        serializable(PixelRoundingProbeDB)
        "#,
    )
    .expect("capture protocol assertions");
}

#[test]
fn pixel_rounding_probe_skips_combat_without_changing_frames() {
    let env = WowLuaEnv::new().expect("create Lua environment");
    env.exec(
        r#"
        GetBuildInfo = function() return "12.1.5", "69594", "Sep 5 2026", 120105 end
        InCombatLockdown = function() return true end
        C_Timer.After = function() error("combat path must not schedule") end
        "#,
    )
    .expect("install combat fixture");
    load_probe_source(&env);
    env.exec("SlashCmdList.PIXELROUNDINGPROBE('combat')")
        .expect("skip combat capture");
    env.exec(
        r#"
        assert(#PixelRoundingProbeDB.samples == 0)
        assert(PixelRoundingProbeDB.skipped.label == "combat")
        assert(PixelRoundingProbeDB.skipped.reason == "in combat")
        "#,
    )
    .expect("combat diagnostic assertions");
}

#[test]
fn pixel_rounding_probe_resamples_same_objects_without_leaking_roots() {
    let env = WowLuaEnv::new().expect("create Lua environment");
    env.exec(
        r#"
        probeQueue = {}
        C_Timer.After = function(_, callback) probeQueue[#probeQueue + 1] = callback end
        InCombatLockdown = function() return false end
        probeOriginalWidth, probeOriginalHeight = UIParent:GetSize()
        "#,
    )
    .unwrap();
    load_probe_source(&env);
    env.exec(
        r#"
        SlashCmdList.PIXELROUNDINGPROBE('deferred')
        probeQueue[1]()
        local childCount = UIParent:GetNumChildren()
        local children = {UIParent:GetChildren()}
        local target = select(1, children[#children]:GetChildren())
        assert(target, "probe's last case has its own hidden frame")
        target:SetWidth(177.25)
        local changedWidth = target:GetWidth()
        probeQueue[2]()
        assert(UIParent:GetNumChildren() == childCount, "settled capture must reuse the same objects")
        local first = PixelRoundingProbeDB.samples[1]
        local settled = PixelRoundingProbeDB.samples[2]
        assert(first.cases[12].after.width ~= changedWidth)
        assert(settled.cases[12].after.width == changedWidth, "capture observes intervening geometry change")
        assert(settled.cases[12].after.points[1].relativeTo == "regions-round-layout:parent")
        assert(settled.cases[12].parent and settled.cases[12].after.size)
        SlashCmdList.PIXELROUNDINGPROBE('repeat')
        probeQueue[3]()
        probeQueue[4]()
        assert(UIParent:GetNumChildren() == childCount, "repeat capture must not leak roots")
        local width, height = UIParent:GetSize()
        assert(width == probeOriginalWidth and height == probeOriginalHeight, "existing UI geometry unchanged")
        "#,
    ).expect("deferred capture protocol");
}

fn load_probe_source(env: &WowLuaEnv) {
    let source = fs::read_to_string(Path::new(ADDON_SOURCE))
        .unwrap_or_else(|error| panic!("read {ADDON_SOURCE}: {error}"));
    let addon_table = env.create_addon_table().expect("create addon table");
    env.loader_env()
        .exec_with_varargs(&source, ADDON_SOURCE, ADDON_NAME, addon_table)
        .unwrap_or_else(|error| panic!("load {ADDON_SOURCE}: {error}"));
}
