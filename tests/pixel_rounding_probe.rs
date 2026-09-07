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
        InClickBindingMode = function() return false end
        C_AddOns.IsAddOnLoaded = function(name)
            return name == "Blizzard_Collections", true
        end
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
        assert(sample.bootstrap.clickBindingModeType == "function")
        assert(sample.bootstrap.Blizzard_ClickBindingUI.loaded == false)
        assert(sample.bootstrap.Blizzard_ClickBindingUI.finished == true)
        assert(sample.bootstrap.Blizzard_Collections.loaded == true)
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

#[cfg(feature = "retail-12-1-5")]
#[test]
fn ptr_native_round_layout_matches_captured_fractional_geometry() {
    let env = ptr_pixel_rounding_environment();
    env.exec(
        r#"
        local tolerance = 0.0002
        local function close(actual, expected, label)
            assert(math.abs(actual - expected) <= tolerance,
                string.format("%s: expected %.9f, got %.9f", label, expected, actual))
        end
        local function rect(frame)
            local left, bottom, width, height = frame:GetRect()
            return left, bottom, width, height
        end
        local parent = CreateFrame("Frame", nil, UIParent)
        parent:Hide()
        parent:SetSize(301.25, 179.75)
        parent:SetPoint("BOTTOMLEFT", UIParent, "BOTTOMLEFT", 123.375, 87.625)

        local frame = CreateFrame("Frame", nil, parent)
        frame:Hide()
        assert(frame:GetRoundLayoutToNearestPixel() == false, "default round-layout flag")
        frame:SetSize(101.375, 40.625)
        frame:SetPoint("BOTTOMLEFT", parent, "BOTTOMLEFT", 0.375, -0.625)
        local left, bottom, width, height = rect(frame)
        close(left, 123.75, "unrounded left")
        close(bottom, 87, "unrounded bottom")
        close(width, 101.375, "unrounded width")
        close(height, 40.625, "unrounded height")

        frame:SetRoundLayoutToNearestPixel(true)
        assert(frame:GetRoundLayoutToNearestPixel() == true, "round-layout flag")
        left, bottom, width, height = rect(frame)
        close(left, 123.375, "rounded left")
        close(bottom, 86.825, "rounded bottom")
        close(width, 101.6, "rounded width")
        close(height, 40.8, "rounded height")
        local _, _, _, x, y = frame:GetPoint(1)
        close(x, 0.375, "raw anchor x")
        close(y, -0.625, "raw anchor y")

        frame:SetRoundLayoutToNearestPixel(false)
        left, bottom, width, height = rect(frame)
        close(left, 123.75, "toggle-off left")
        close(bottom, 87, "toggle-off bottom")
        close(width, 101.375, "toggle-off width")
        close(height, 40.625, "toggle-off height")

        local cases = {
            { scale = 0.8, left = 154.21875, bottom = 108.53125, width = 101, height = 41 },
            { scale = 1, left = 123.375, bottom = 86.825, width = 101.6, height = 40.8 },
            { scale = 1.25, left = 99.34, bottom = 69.46, width = 101.12, height = 40.32 },
        }
        for _, case in ipairs(cases) do
            frame:SetScale(case.scale)
            frame:SetRoundLayoutToNearestPixel(true)
            left, bottom, width, height = rect(frame)
            close(left, case.left, "scale left " .. case.scale)
            close(bottom, case.bottom, "scale bottom " .. case.scale)
            close(width, case.width, "scale width " .. case.scale)
            close(height, case.height, "scale height " .. case.scale)
        end

        frame:SetScale(1)
        frame:ClearAllPoints()
        frame:SetPoint("CENTER", parent, "CENTER", -0.375, 0.625)
        left, bottom, width, height = rect(frame)
        close(left, 223.2, "center left")
        close(bottom, 157.9, "center bottom")
        close(width, 101.6, "center width")
        close(height, 40.8, "center height")

        frame:ClearAllPoints()
        frame:SetPoint("BOTTOMLEFT", parent, "BOTTOMLEFT", 0.375, -0.625)
        frame:SetPoint("TOPRIGHT", parent, "TOPRIGHT", -0.875, 0.125)
        left, bottom, width, height = rect(frame)
        close(left, 123.375, "stretch left")
        close(bottom, 86.825, "stretch bottom")
        close(width, 300.45, "stretch width")
        close(height, 180.55, "stretch height")

        frame:ClearAllPoints()
        frame:SetScale(1)
        frame:SetSize(101.375, 40.625)
        frame:SetPoint("BOTTOMLEFT", parent, "BOTTOMLEFT", 0.375, -0.625)
        parent:SetScale(1.25)
        parent:ClearAllPoints()
        parent:SetPoint("BOTTOMLEFT", UIParent, "BOTTOMLEFT", 123.875, 87.125)
        left, bottom, width, height = rect(frame)
        close(left, 124.515, "parent scale left")
        close(bottom, 86.485, "parent scale bottom")
        close(width, 101.12, "parent scale width")
        close(height, 40.32, "parent scale height")

        parent:SetScale(1)
        parent:ClearAllPoints()
        parent:SetPoint("BOTTOMLEFT", UIParent, "BOTTOMLEFT", 123.375, 87.625)
        frame:SetRoundLayoutToNearestPixel(false)
        frame:ClearAllPoints()
        frame:SetSize(101.375, 40.625)
        frame:SetPoint("BOTTOMLEFT", parent, "BOTTOMLEFT", 0.375, -0.625)

        local texture = frame:CreateTexture(nil, "ARTWORK")
        texture:SetAllPoints(frame)
        texture:SetRoundLayoutToNearestPixel(true)
        assert(texture:GetRoundLayoutToNearestPixel() == true, "texture flag")
        left, bottom, width, height = rect(texture)
        close(left, 123.75, "texture left")
        close(bottom, 87, "texture bottom")
        close(width, 101.375, "texture width")
        close(height, 40.625, "texture height")
        local textureWidth, textureHeight = texture:GetSize()
        close(textureWidth, 101.375, "texture GetSize width")
        close(textureHeight, 40.625, "texture GetSize height")

        local font = frame:CreateFontString(nil, "OVERLAY", "GameFontNormal")
        font:SetPoint("CENTER", frame, "CENTER", -0.375, 0.625)
        font:SetSize(33.375, 14.625)
        font:SetText("Probe")
        font:SetRoundLayoutToNearestPixel(true)
        assert(font:GetRoundLayoutToNearestPixel() == true, "font flag")
        local fontLeft, fontBottom, fontWidth, fontHeight = rect(font)
        close(fontLeft, 157.6375, "font left")
        close(fontBottom, 100.9125, "font bottom")
        close(fontWidth, 33.6, "font width")
        close(fontHeight, 14.4, "font height")
        close(font:GetWidth(), 33.6, "font GetWidth")
        local fontSizeWidth, fontSizeHeight = font:GetSize()
        close(fontSizeWidth, 33.6, "font GetSize width")
        close(fontSizeHeight, 14.4, "font GetSize height")

        local timed = CreateFrame("Frame", nil, parent)
        timed:Hide()
        timed:SetRoundLayoutToNearestPixel(true)
        timed:SetSize(101.375, 40.625)
        timed:SetPoint("BOTTOMLEFT", parent, "BOTTOMLEFT", 0.375, -0.625)
        local timedLeft, timedBottom, timedWidth, timedHeight = rect(timed)
        C_Timer.After(0, function()
            local afterLeft, afterBottom, afterWidth, afterHeight = rect(timed)
            close(afterLeft, timedLeft, "next-tick left")
            close(afterBottom, timedBottom, "next-tick bottom")
            close(afterWidth, timedWidth, "next-tick width")
            close(afterHeight, timedHeight, "next-tick height")
            pixelRoundingNextTickObserved = true
        end)
        "#,
    )
    .expect("source-derived PTR round-layout capture assertions");
    assert_eq!(env.process_timers().expect("process next-tick query"), 1);
    assert!(env
        .eval::<bool>("return pixelRoundingNextTickObserved")
        .expect("read next-tick result"));
}

#[cfg(feature = "retail-12-1-5")]
#[test]
fn ptr_native_round_layout_reacts_to_display_resize() {
    let env = ptr_pixel_rounding_environment();
    env.exec(
        r#"
        local tolerance = 0.0002
        local function close(actual, expected, label)
            assert(math.abs(actual - expected) <= tolerance,
                string.format("%s: expected %.9f, got %.9f", label, expected, actual))
        end
        local frame = CreateFrame("Frame", nil, UIParent)
        frame:Hide()
        frame:SetSize(101.375, 40.625)
        frame:SetPoint("BOTTOMLEFT", UIParent, "BOTTOMLEFT", 0.375, -0.625)
        frame:SetRoundLayoutToNearestPixel(true)
        pixelRoundingResizeFrame = frame
        local _, _, width, height = frame:GetRect()
        close(width, 101.6, "1440px display width")
        close(height, 40.8, "1440px display height")
        close(frame:GetWidth(), 101.6, "1440px display GetWidth")
        local sizeWidth, sizeHeight = frame:GetSize()
        close(sizeWidth, 101.6, "1440px display GetSize width")
        close(sizeHeight, 40.8, "1440px display GetSize height")
        "#,
    )
    .expect("initial source-derived pixel conversion assertions");
    env.set_display_size(3440.0, 768.0)
        .expect("resize physical display");
    env.exec(
        r#"
        -- Source-derived: PixelUtil.GetNearestPixelSize uses 768 / physicalHeight.
        local tolerance = 0.0002
        local frame = pixelRoundingResizeFrame
        local _, _, width, height = frame:GetRect()
        assert(math.abs(width - 102) <= tolerance,
            string.format("768px display width: expected %.9f, got %.9f", 102, width))
        assert(math.abs(height - 40.5) <= tolerance,
            string.format("768px display height: expected %.9f, got %.9f", 40.5, height))
        assert(math.abs(frame:GetWidth() - 102) <= tolerance,
            string.format("768px display GetWidth: expected %.9f, got %.9f", 102, frame:GetWidth()))
        local sizeWidth, sizeHeight = frame:GetSize()
        assert(math.abs(sizeWidth - 102) <= tolerance and math.abs(sizeHeight - 40.5) <= tolerance,
            string.format("768px display GetSize: expected %.9f x %.9f, got %.9f x %.9f", 102, 40.5, sizeWidth, sizeHeight))
        "#,
    )
    .expect("display resize invalidates rounded layout");
}

#[cfg(feature = "retail-12-1-5")]
fn ptr_pixel_rounding_environment() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create Lua environment");
    env.set_display_size(3440.0, 1440.0)
        .expect("set captured physical display");
    env.exec("UIParent:SetScale(0.6666666865348816)")
        .expect("set captured UIParent scale");
    env
}

#[test]
fn bootstrap_probe_preserves_session_order_across_saved_variable_restore() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        GetBuildInfo = function() return "12.1.5", "69594", "Aug 28 2026", 120105 end
        C_AddOns.IsAddOnLoaded = function() return false, false end
        InClickBindingMode = function() return false end
        C_AddOns.LoadAddOn = function(name)
            assert(name == "BootstrapOrderProbe_B", "must not load Blizzard addons")
            return true
        end
    "#).unwrap();
    load_bootstrap_file(&env, "B", "Bootstrap.lua");
    load_bootstrap_file(&env, "A", "A.lua");
    env.exec("BootstrapOrderProbeDB = { events = { 'stale' } }").unwrap();
    env.fire_event_with_args("ADDON_LOADED", &[env.lua_string("BootstrapOrderProbe_A")]).unwrap();
    env.exec("assert(BootstrapOrderProbeDB.events[1].tag == 'B:bootstrap', 'restored data replaced current session')").unwrap();
    load_bootstrap_file(&env, "C", "C.lua");
    load_bootstrap_file(&env, "D", "Before.lua");
    load_bootstrap_file(&env, "D", "Bootstrap.lua");
    load_bootstrap_file(&env, "D", "Normal.lua");
    env.exec(r#"
        local db = BootstrapOrderProbeDB
        assert(db.events[1].tag == "B:bootstrap", "retain bootstrap before A")
        assert(db.events[2].tag == "A:eager", "discard restored stale records")
        assert(db.events[1].build.matchesExpectedBuild)
        assert(db.events[1].addons.Blizzard_ClickBindingUI.loaded == false)
        assert(db.events[1].helpers.clickBinding == "function")
        assert(db.events[4].tag == "D:before")
        assert(db.events[5].tag == "D:bootstrap")
        assert(db.events[6].tag == "D:after")
        SlashCmdList.BOOTSTRAPORDERPROBE('load')
        SlashCmdList.BOOTSTRAPORDERPROBE('load')
        assert(db.counts['B:bootstrap'] == 1)
        assert(db.counts['load:before'] == 2 and db.counts['load:after'] == 2)
        assert(db.events[#db.events].loadResult.ok == true)
    "#).unwrap();
}

#[test]
fn bootstrap_probe_records_explicit_load_and_repeat_without_loading_blizzard() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        GetBuildInfo = function() return "12.1.5", "99999", "unknown", 120105 end
        probeLoaded = false
        C_AddOns.IsAddOnLoaded = function(name) return name == "BootstrapOrderProbe_B" and probeLoaded, probeLoaded end
        C_AddOns.LoadAddOn = function(name)
            assert(name == "BootstrapOrderProbe_B")
            if not probeLoaded then
                probeBefore(); probeBootstrap(); probeAfter()
                probeLoaded = true
            end
            return true
        end
    "#).unwrap();
    for (function, file) in [("probeBefore", "Before.lua"), ("probeBootstrap", "Bootstrap.lua"), ("probeAfter", "Normal.lua")] {
        let source = fs::read_to_string(format!("docs/addons/BootstrapOrderProbe_B/{file}")).unwrap();
        env.exec(&format!("function {function}()\n{source}\nend")).unwrap();
    }
    load_bootstrap_file(&env, "A", "A.lua");
    env.fire_event_with_args("ADDON_LOADED", &[env.lua_string("BootstrapOrderProbe_A")]).unwrap();
    load_bootstrap_file(&env, "C", "C.lua");
    env.exec(r#"
        SlashCmdList.BOOTSTRAPORDERPROBE('load')
        SlashCmdList.BOOTSTRAPORDERPROBE('load')
        local db = BootstrapOrderProbeDB
        assert(not db.events[1].build.matchesExpectedBuild)
        assert(db.events[3].tag == 'load:before')
        assert(not db.events[3].addons.BootstrapOrderProbe_B.loaded)
        assert(db.events[4].tag == 'B:before' and db.events[5].tag == 'B:bootstrap' and db.events[6].tag == 'B:after')
        assert(db.events[7].addons.BootstrapOrderProbe_B.loaded)
        assert(db.events[9].loadResult.success and db.events[9].loadResult.ok)
        assert(db.counts['B:before'] == 1 and db.counts['B:bootstrap'] == 1 and db.counts['B:after'] == 1)
        C_AddOns.LoadAddOn = function() error('load failed') end
        SlashCmdList.BOOTSTRAPORDERPROBE('load')
        assert(not db.events[#db.events].loadResult.success)
        assert(string.find(db.events[#db.events].loadResult.error, 'load failed', 1, true))
    "#).unwrap();
}

fn load_bootstrap_file(env: &WowLuaEnv, addon: &str, file: &str) {
    let path = format!("docs/addons/BootstrapOrderProbe_{addon}/{file}");
    let source = fs::read_to_string(&path).unwrap();
    env.exec(&source).unwrap();
}

fn load_probe_source(env: &WowLuaEnv) {
    let source = fs::read_to_string(Path::new(ADDON_SOURCE))
        .unwrap_or_else(|error| panic!("read {ADDON_SOURCE}: {error}"));
    let addon_table = env.create_addon_table().expect("create addon table");
    env.loader_env()
        .exec_with_varargs(&source, ADDON_SOURCE, ADDON_NAME, addon_table)
        .unwrap_or_else(|error| panic!("load {ADDON_SOURCE}: {error}"));
}
