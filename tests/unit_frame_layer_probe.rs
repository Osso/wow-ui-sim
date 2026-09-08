use std::{fs, path::Path};

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::saved_variables::SavedVariablesManager;

const PROBE_DIR: &str = "docs/addons/UnitFrameLayerProbe";

fn fixture() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        layerTimers = {}
        C_Timer.After = function(delay, callback)
            assert(delay == 0)
            layerTimers[#layerTimers + 1] = callback
        end
        function FlushLayerTimers()
            local pending = layerTimers
            layerTimers = {}
            for _, callback in ipairs(pending) do callback() end
        end
        layerLoaded = false
        C_AddOns.IsAddOnLoaded = function(name)
            return name ~= 'Blizzard_PlayerSpells' or layerLoaded, true
        end
        C_AddOns.GetAddOnMetadata = function(_, key)
            assert(key == 'Version'); return 'fixture-1'
        end
        C_AddOns.LoadAddOn = function() error('probe must not load addons') end
        GetBuildInfo = function() return '12.1.5', '69594', 'fixture', 120105 end
        GetPhysicalScreenSize = function() return 3440, 1440 end
        GetServerTime = function() return 1700000000 end
        BetterBlizzFramesDB = { noPortraitModes = true, noPortraitPixelBorder = false,
            privateSetting = 'must not be captured' }
        PlayerFrame = CreateFrame('Frame', 'LayerPlayer', UIParent)
        PlayerFrame:SetSize(232, 100)
        PlayerFrame:SetPoint('BOTTOMLEFT', UIParent, 'BOTTOMLEFT', 142, 90)
        PlayerFrame:SetFrameStrata('LOW')
        PlayerFrame:SetFrameLevel(7)
        PlayerFrame.noPortraitMode = CreateFrame('Frame', nil, PlayerFrame)
        PlayerFrame.noPortraitMode:SetAllPoints(PlayerFrame)
        PlayerFrame.noPortraitMode:SetFrameStrata('HIGH')
        PlayerFrame.bbfName = PlayerFrame.noPortraitMode:CreateFontString(nil, 'OVERLAY')
        PlayerFrame.bbfName:SetText('Fixture')
        PlayerFrame.noPortraitMode.Texture = PlayerFrame.noPortraitMode:CreateTexture(nil, 'OVERLAY')
        function MakeLayerSpellBook()
            layerLoaded = true
            PlayerSpellsFrame = CreateFrame('Frame', 'LayerSpells', UIParent)
            PlayerSpellsFrame:SetSize(800, 600)
            PlayerSpellsFrame:SetPoint('BOTTOMLEFT', UIParent, 'BOTTOMLEFT', 60, 80)
            PlayerSpellsFrame:SetFrameStrata('MEDIUM')
            PlayerSpellsFrame.SpellBookFrame = CreateFrame('Frame', nil, PlayerSpellsFrame)
            PlayerSpellsFrame.SpellBookFrame:SetSize(300, 200)
            PlayerSpellsFrame.SpellBookFrame:SetPoint('BOTTOMLEFT', PlayerSpellsFrame, 'BOTTOMLEFT', 10, 10)
            layerExistingShows, layerExistingHides = 0, 0
            PlayerSpellsFrame.SpellBookFrame:SetScript('OnShow', function() layerExistingShows = layerExistingShows + 1 end)
            PlayerSpellsFrame.SpellBookFrame:SetScript('OnHide', function() layerExistingHides = layerExistingHides + 1 end)
            PlayerSpellsFrame.SpellBookFrame:Hide()
        end
        function SerializeLayerValue(value)
            local kind = type(value)
            if kind == 'string' then return string.format('%q', value) end
            if kind ~= 'table' then
                assert(kind == 'number' or kind == 'boolean' or kind == 'nil')
                return tostring(value)
            end
            assert(getmetatable(value) == nil, 'captured a live object')
            local fields = {}
            for key, child in pairs(value) do
                fields[#fields + 1] = '[' .. SerializeLayerValue(key) .. ']=' .. SerializeLayerValue(child)
            end
            table.sort(fields)
            return '{' .. table.concat(fields, ',') .. '}'
        end
        function LayerFrameState()
            local f = PlayerFrame
            local c = f.noPortraitMode
            local left, bottom, width, height = f:GetRect()
            local point, relative, relativePoint, x, y = f:GetPoint()
            return SerializeLayerValue({f:GetFrameStrata(), f:GetFrameLevel(), f:GetAlpha(),
                f:GetScale(), f:IsShown(), left, bottom, width, height, tostring(f:GetParent()),
                point, tostring(relative), relativePoint, x, y,
                c:GetFrameStrata(), c:GetFrameLevel(), c:GetAlpha(), c:GetScale(),
                c:IsShown(), tostring(c:GetParent()), PlayerFrame.bbfName:GetText()})
        end
        "#,
    )
    .unwrap();
    env
}

fn load_probe(env: &WowLuaEnv) {
    let toc_path = Path::new(PROBE_DIR).join("UnitFrameLayerProbe.toc");
    let toc = wow_ui_sim::toc::TocFile::parse(
        Path::new(PROBE_DIR),
        &fs::read_to_string(toc_path).unwrap(),
    );
    let addon = env.create_addon_table().unwrap();
    for file in toc.files {
        let path = Path::new(PROBE_DIR).join(file);
        let source = fs::read_to_string(&path).unwrap();
        env.loader_env()
            .exec_with_varargs(
                &source,
                &path.to_string_lossy(),
                "UnitFrameLayerProbe",
                addon,
            )
            .unwrap();
    }
}

fn loaded_event(env: &WowLuaEnv, name: &str) {
    env.fire_event_with_args("ADDON_LOADED", &[env.lua_string(name)])
        .unwrap();
}

#[test]
fn unit_layer_probe_manual_and_login_preserve_observed_frame_state() {
    let env = fixture();
    env.exec("layerBefore = LayerFrameState()").unwrap();
    load_probe(&env);
    loaded_event(&env, "UnitFrameLayerProbe");
    env.fire_event("PLAYER_LOGIN").unwrap();
    env.exec(
        r#"
        FlushLayerTimers()
        SlashCmdList.UNITFRAMELAYERPROBE('')
        local db = UnitFrameLayerProbeDB
        assert(db.schemaVersion == 1 and db.probeVersion == '1.1.0')
        assert(#db.captures == 2)
        assert(db.captures[1].reason == 'PLAYER_LOGIN')
        local c = db.captures[2]
        assert(c.reason == 'manual' and c.sequence == 2)
        assert(c.timestamp.values[1].value == 1700000000)
        assert(c.build.values[4].value == 120105)
        assert(c.metrics.GetPhysicalScreenSize.values[2].value == 1440)
        assert(c.frames['PlayerFrame.noPortraitMode'].methods.GetFrameStrata.values[1].value == 'HIGH')
        assert(c.frames['PlayerFrame.noPortraitMode'].parents.entries[1].methods.GetName.values[1].value == 'LayerPlayer')
        assert(c.frames['PlayerFrame.bbfName'].status == 'ok')
        assert(c.frames['PlayerFrame.noPortraitMode.Texture'].status == 'ok')
        assert(c.frames['PlayerSpellsFrame'].status == 'missing')
        assert(c.addons.BetterBlizzFrames.version.values[1].value == 'fixture-1')
        assert(c.noPortraitConfig.noPortraitModes.value == true)
        assert(c.noPortraitConfig.noPortraitPixelBorder.value == false)
        assert(c.noPortraitConfig.privateSetting == nil)
        assert(LayerFrameState() == layerBefore, 'probe changed observed frame properties')
        SerializeLayerValue(db)
        "#,
    )
    .unwrap();
}

#[test]
fn unit_layer_probe_hooks_late_spellbook_once_and_observes_settled_state() {
    let env = fixture();
    load_probe(&env);
    env.exec("MakeLayerSpellBook()").unwrap();
    loaded_event(&env, "Blizzard_PlayerSpells");
    loaded_event(&env, "Blizzard_PlayerSpells");
    env.exec(
        r#"
        local book = PlayerSpellsFrame.SpellBookFrame
        book:Show()
        assert(#layerTimers == 1, 'duplicate or non-deferred OnShow hook')
        assert(layerExistingShows == 1, 'probe replaced the existing OnShow handler')
        book:SetSize(450, 250)
        FlushLayerTimers()
        local c = UnitFrameLayerProbeDB.captures[1]
        assert(c.reason == 'SpellBook.OnShow')
        assert(c.frames['PlayerSpellsFrame.SpellBookFrame'].methods.GetRect.values[3].value == 450)
        book:Hide()
        assert(#layerTimers == 1, 'duplicate OnHide hook')
        assert(layerExistingHides == 2, 'probe replaced the existing OnHide handler')
        FlushLayerTimers()
        assert(#UnitFrameLayerProbeDB.captures == 2)
        assert(UnitFrameLayerProbeDB.captures[2].reason == 'SpellBook.OnHide')
        assert(UnitFrameLayerProbeDB.captures[2].frames['PlayerSpellsFrame.SpellBookFrame'].methods.IsShown.values[1].value == false)
        "#,
    )
    .unwrap();
}

#[test]
fn unit_layer_probe_hooks_already_loaded_spellbook_without_showing_it() {
    let env = fixture();
    env.exec("MakeLayerSpellBook()").unwrap();
    load_probe(&env);
    loaded_event(&env, "UnitFrameLayerProbe");
    loaded_event(&env, "Blizzard_PlayerSpells");
    env.exec(
        r#"
        assert(not PlayerSpellsFrame.SpellBookFrame:IsShown())
        assert(#layerTimers == 0, 'installation must not capture or show')
        PlayerSpellsFrame.SpellBookFrame:Show()
        assert(#layerTimers == 1)
        FlushLayerTimers()
        assert(#UnitFrameLayerProbeDB.captures == 1)
        "#,
    )
    .unwrap();
}

#[test]
fn unit_layer_probe_records_missing_errors_and_nil_returns_without_inference() {
    let env = fixture();
    env.exec(
        r#"
        PlayerFrame.GetRaisedFrameLevel = false
        PlayerFrame.GetAlpha = function() error('alpha getter refused') end
        PlayerFrame.GetFrameLevel = function() return nil, false, 7 end
        PlayerFrame.noPortraitMode.GetParent = function() error('parent getter refused') end
        GetPhysicalScreenSize = nil
        C_AddOns.GetAddOnMetadata = function() error('metadata refused') end
        "#,
    )
    .unwrap();
    load_probe(&env);
    env.exec(
        r#"
        SlashCmdList.UNITFRAMELAYERPROBE('')
        local c = UnitFrameLayerProbeDB.captures[1]
        local m = c.frames.PlayerFrame.methods
        assert(m.GetRaisedFrameLevel.status == 'missing')
        assert(m.GetAlpha.status == 'error' and m.GetAlpha.message:find('alpha getter refused', 1, true))
        assert(m.GetFrameLevel.status == 'ok' and m.GetFrameLevel.values.n == 3)
        assert(m.GetFrameLevel.values[1].kind == 'nil')
        assert(m.GetFrameLevel.values[2].value == false and m.GetFrameLevel.values[3].value == 7)
        assert(c.metrics.GetPhysicalScreenSize.status == 'missing')
        assert(c.frames['PlayerFrame.noPortraitMode'].parents.status == 'error')
        assert(c.addons.BetterBlizzFrames.version.status == 'error')
        SerializeLayerValue(UnitFrameLayerProbeDB)
        "#,
    )
    .unwrap();
}

fn save_global(env: &WowLuaEnv, name: &str) -> String {
    let dir = tempfile::tempdir().unwrap();
    let mut manager = SavedVariablesManager::with_storage_dir(dir.path().to_path_buf());
    env.loader_env()
        .with_state(|state| {
            manager.init_for_addon(state, "UnitFrameLayerProbe", &[name.to_string()], &[])?;
            manager.save_addon(state, "UnitFrameLayerProbe")
        })
        .unwrap();
    fs::read_to_string(dir.path().join("UnitFrameLayerProbe.lua")).unwrap()
}

fn save_capture_snapshot(env: &WowLuaEnv, expression: &str) -> String {
    env.exec(&format!("LayerCaptureSnapshot = {expression}"))
        .unwrap();
    save_global(env, "LayerCaptureSnapshot")
}

#[test]
fn unit_layer_probe_preserves_serialized_captures_across_reload_and_at_cap() {
    let env = fixture();
    load_probe(&env);
    env.exec("SlashCmdList.UNITFRAMELAYERPROBE('')").unwrap();
    let saved = save_global(&env, "UnitFrameLayerProbeDB");
    let first = save_capture_snapshot(&env, "UnitFrameLayerProbeDB.captures[1]");
    let reloaded = fixture();
    load_probe(&reloaded);
    reloaded.exec(&saved).unwrap();
    loaded_event(&reloaded, "UnitFrameLayerProbe");
    reloaded.fire_event("PLAYER_LOGIN").unwrap();
    reloaded
        .exec("FlushLayerTimers(); for i=1,35 do SlashCmdList.UNITFRAMELAYERPROBE('') end")
        .unwrap();
    assert_eq!(
        first,
        save_capture_snapshot(&reloaded, "UnitFrameLayerProbeDB.captures[1]")
    );
    reloaded
        .exec(
            r#"
        local db = UnitFrameLayerProbeDB
        assert(#db.captures == 30 and db.nextSequence == 31)
        assert(db.captures[2].reason == 'PLAYER_LOGIN')
        for i, c in ipairs(db.captures) do assert(c.sequence == i) end
        assert(db.skippedCaptures == 7)
        "#,
        )
        .unwrap();
    let capped = save_global(&reloaded, "UnitFrameLayerProbeDB");
    let captures = save_capture_snapshot(&reloaded, "UnitFrameLayerProbeDB.captures");
    let third = fixture();
    load_probe(&third);
    third.exec(&capped).unwrap();
    loaded_event(&third, "UnitFrameLayerProbe");
    third.fire_event("PLAYER_LOGIN").unwrap();
    third.exec("FlushLayerTimers()").unwrap();
    assert_eq!(
        captures,
        save_capture_snapshot(&third, "UnitFrameLayerProbeDB.captures")
    );
    third.exec("assert(UnitFrameLayerProbeDB.nextSequence == 31 and UnitFrameLayerProbeDB.skippedCaptures == 8)").unwrap();
}

fn control_fixture() -> WowLuaEnv {
    let env = fixture();
    env.exec(r#"
        controlNow, controlTimers, controlShots, controlFrames = 0, {}, {}, {}
        GetTime = function() return controlNow end
        C_Timer.After = function(delay, callback)
            controlTimers[#controlTimers + 1] = { at = controlNow + delay, callback = callback }
        end
        function AdvanceControls(seconds)
            local target = controlNow + seconds
            while true do
                local index
                for i, timer in ipairs(controlTimers) do
                    if timer.at <= target and (not index or timer.at < controlTimers[index].at) then index = i end
                end
                if not index then break end
                local timer = table.remove(controlTimers, index)
                controlNow = timer.at
                timer.callback()
            end
            controlNow = target
        end
        local originalCreate = CreateFrame
        CreateFrame = function(...)
            local frame = originalCreate(...)
            controlFrames[#controlFrames + 1] = frame
            return frame
        end
        Screenshot = function(...)
            assert(select('#', ...) == 0, 'Screenshot does not accept a filename')
            local runs = UnitFrameLayerProbeDB.controlRuns
            local run = runs[#runs]
            controlShots[#controlShots + 1] = { at = controlNow, phase = run.phases[#run.phases].name }
        end
        function AssertControlsClean()
            for _, frame in ipairs(controlFrames) do
                assert(not frame:IsShown(), 'owned fixture remains shown')
                assert(frame:GetParent() == nil, 'owned fixture remains parented')
                assert(frame:GetNumPoints() == 0, 'owned fixture keeps UI anchors')
            end
        end
    "#).unwrap();
    load_probe(&env);
    env.exec("controlFrames = {}; controlBefore = LayerFrameState(); SlashCmdList.UNITFRAMELAYERPROBE('')").unwrap();
    env
}

fn screenshot_event(env: &WowLuaEnv, name: &str) {
    env.fire_event(name).unwrap();
}

#[test]
fn unit_layer_controls_capture_three_phases_and_clean_only_owned_fixtures() {
    let env = control_fixture();
    env.exec(r#"
        local original = SerializeLayerValue(UnitFrameLayerProbeDB.captures)
        controlHistory = original
        SlashCmdList.UNITFRAMELAYERPROBE('test')
        AdvanceControls(0)
        assert(#controlShots == 1 and controlShots[1].phase == 'created')
        local run = UnitFrameLayerProbeDB.controlRuns[1]
        assert(run.status == 'running' and #run.phases == 1)
        local p = run.phases[1]
        assert(p.build.values[2].value == '69594')
        assert(p.frames['case1.parent'].methods.GetFrameStrata.values[1].value == 'LOW')
        assert(p.frames['case1.parent'].methods.IsToplevel.values[1].value == true)
        for i, strata in ipairs({'HIGH', 'HIGH', 'DIALOG', 'TOOLTIP'}) do
            local red = p.frames['case'..i..'.red']
            local blue = p.frames['case'..i..'.blue']
            assert(red.methods.GetFrameStrata.values[1].value == strata)
            assert(blue.methods.GetFrameStrata.values[1].value == 'MEDIUM')
            assert(blue.methods.IsToplevel.values[1].value == true)
            assert(red.methods.GetRaisedFrameLevel.status == 'ok')
            local a, b = red.methods.GetRect.values, blue.methods.GetRect.values
            assert(a[1].value < b[1].value and a[1].value + a[3].value > b[1].value)
            assert(a[2].value < b[2].value + b[4].value and a[2].value + a[4].value > b[2].value)
            local expectedParent = i == 1 and p.frames['case1.parent'].object.identity or tostring(UIParent)
            assert(red.methods.GetParent.values[1].identity == expectedParent)
            assert(blue.methods.GetParent.values[1].identity == tostring(UIParent))
            assert(p.frames['case'..i..'.label'].methods.GetParent.values[1].identity == tostring(UIParent))
        end
        assert(p.frames['case5.red'].methods.GetObjectType.values[1].value == 'GameTooltip')
        assert(p.tooltipOwner.values[1].identity == p.frames['case5.blue'].object.identity)
        assert(p.frames['case5.red'].methods.GetParent.values[1].identity == tostring(UIParent))
        assert(p.frames.header.methods.GetParent.values[1].identity == tostring(UIParent))
        assert(LayerFrameState() == controlBefore)
        AdvanceControls(2)
        assert(#controlShots == 1, 'advanced without screenshot completion')
    "#).unwrap();
    screenshot_event(&env, "SCREENSHOT_SUCCEEDED");
    env.exec("AdvanceControls(1); assert(#controlShots == 1); AdvanceControls(.11); assert(controlShots[2].phase == 'panel-hide-show')").unwrap();
    screenshot_event(&env, "SCREENSHOT_SUCCEEDED");
    env.exec("AdvanceControls(1.1); assert(controlShots[3].phase == 'panel-raise')")
        .unwrap();
    screenshot_event(&env, "SCREENSHOT_SUCCEEDED");
    env.exec(
        r#"
        local run = UnitFrameLayerProbeDB.controlRuns[1]
        assert(run.status == 'complete' and #run.phases == 3)
        for _, p in ipairs(run.phases) do
            assert(p.screenshot.status == 'succeeded')
            assert(p.frames['case5.red'].methods.IsShown.values[1].value == true)
            assert(p.tooltipOwner.values[1].identity == p.frames['case5.blue'].object.identity)
        end
        assert(controlShots[2].at - controlShots[1].at >= 1.1)
        assert(controlShots[3].at - controlShots[2].at >= 1.1)
        AssertControlsClean()
        assert(LayerFrameState() == controlBefore)
        assert(SerializeLayerValue(UnitFrameLayerProbeDB.captures) == controlHistory)
        AdvanceControls(30)
        assert(#controlShots == 3, 'stale timeout advanced a completed run')
    "#,
    )
    .unwrap();
    let saved = save_global(&env, "UnitFrameLayerProbeDB");
    let restored = fixture();
    load_probe(&restored);
    restored.exec(&saved).unwrap();
    restored.exec("assert(UnitFrameLayerProbeDB.controlRuns[1].status == 'complete'); assert(#UnitFrameLayerProbeDB.captures == 1)").unwrap();
}

#[test]
fn unit_layer_controls_cleanup_on_screenshot_failure_and_cancel() {
    let env = control_fixture();
    env.exec("SlashCmdList.UNITFRAMELAYERPROBE('test'); AdvanceControls(0)")
        .unwrap();
    screenshot_event(&env, "SCREENSHOT_FAILED");
    env.exec("assert(UnitFrameLayerProbeDB.controlRuns[1].status == 'screenshot_failed'); AssertControlsClean(); controlFrames = {}").unwrap();
    env.exec("SlashCmdList.UNITFRAMELAYERPROBE('test'); AdvanceControls(0); SlashCmdList.UNITFRAMELAYERPROBE('cancel'); AssertControlsClean(); assert(UnitFrameLayerProbeDB.controlRuns[2].status == 'cancelled')").unwrap();
    env.exec("SlashCmdList.UNITFRAMELAYERPROBE('test'); assert(#UnitFrameLayerProbeDB.controlRuns == 2, 'started while cancelled screenshot outstanding')").unwrap();
    screenshot_event(&env, "SCREENSHOT_SUCCEEDED");
    env.exec("AdvanceControls(30); assert(#controlShots == 2); assert(LayerFrameState() == controlBefore)").unwrap();
}

#[test]
fn unit_layer_controls_timeout_and_screenshot_call_error_cleanup() {
    let env = control_fixture();
    env.exec(
        r#"
        SlashCmdList.UNITFRAMELAYERPROBE('test'); AdvanceControls(0)
        AdvanceControls(11)
        assert(UnitFrameLayerProbeDB.controlRuns[1].status == 'timeout')
        AssertControlsClean()
        SlashCmdList.UNITFRAMELAYERPROBE('test')
        assert(#UnitFrameLayerProbeDB.controlRuns == 1, 'ambiguous screenshot still outstanding')
    "#,
    )
    .unwrap();
    screenshot_event(&env, "SCREENSHOT_SUCCEEDED");
    env.exec(
        r#"
        controlFrames = {}
        Screenshot = function() error('screenshot unavailable') end
        SlashCmdList.UNITFRAMELAYERPROBE('test'); AdvanceControls(0)
        assert(UnitFrameLayerProbeDB.controlRuns[2].status == 'error')
        assert(UnitFrameLayerProbeDB.controlRuns[2].error:find('screenshot unavailable', 1, true))
        AssertControlsClean()
        assert(LayerFrameState() == controlBefore)
    "#,
    )
    .unwrap();
}

#[test]
fn unit_layer_controls_bound_allocations_and_cancel_delayed_next_phase() {
    let env = control_fixture();
    for _ in 0..3 {
        env.exec("SlashCmdList.UNITFRAMELAYERPROBE('test'); AdvanceControls(0)")
            .unwrap();
        screenshot_event(&env, "SCREENSHOT_SUCCEEDED");
        env.exec("SlashCmdList.UNITFRAMELAYERPROBE('cancel'); AdvanceControls(20); AssertControlsClean()").unwrap();
    }
    env.exec(
        r#"
        local count = #controlFrames
        for i=1,10 do SlashCmdList.UNITFRAMELAYERPROBE('test') end
        assert(#controlFrames == count, 'unbounded native frame allocations')
        assert(#UnitFrameLayerProbeDB.controlRuns == 3 and #controlShots == 3)
        assert(#UnitFrameLayerProbeDB.captures == 1, 'control cap damaged original capture history')
        SlashCmdList.UNITFRAMELAYERPROBE('')
        assert(#UnitFrameLayerProbeDB.captures == 2, 'manual command no longer works')
    "#,
    )
    .unwrap();
}
