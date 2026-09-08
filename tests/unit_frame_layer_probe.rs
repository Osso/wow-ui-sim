use std::fs;

use wow_ui_sim::lua_api::WowLuaEnv;

const SOURCE: &str = "docs/addons/UnitFrameLayerProbe/UnitFrameLayerProbe.lua";

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
    let source = fs::read_to_string(SOURCE).unwrap();
    let addon = env.create_addon_table().unwrap();
    env.loader_env()
        .exec_with_varargs(&source, SOURCE, "UnitFrameLayerProbe", addon)
        .unwrap();
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
        assert(db.schemaVersion == 1 and db.probeVersion == '1.0.0')
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

#[test]
fn unit_layer_probe_preserves_serialized_captures_across_reload_and_at_cap() {
    let env = fixture();
    load_probe(&env);
    env.exec("SlashCmdList.UNITFRAMELAYERPROBE('')").unwrap();
    let saved: String = env
        .eval("return SerializeLayerValue(UnitFrameLayerProbeDB)")
        .unwrap();
    let first: String = env
        .eval("return SerializeLayerValue(UnitFrameLayerProbeDB.captures[1])")
        .unwrap();
    let reloaded = fixture();
    load_probe(&reloaded);
    reloaded
        .exec(&format!("UnitFrameLayerProbeDB = {saved}"))
        .unwrap();
    loaded_event(&reloaded, "UnitFrameLayerProbe");
    reloaded.fire_event("PLAYER_LOGIN").unwrap();
    reloaded
        .exec("FlushLayerTimers(); for i=1,35 do SlashCmdList.UNITFRAMELAYERPROBE('') end")
        .unwrap();
    assert_eq!(
        first,
        reloaded
            .eval::<String>("return SerializeLayerValue(UnitFrameLayerProbeDB.captures[1])")
            .unwrap()
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
    let capped: String = reloaded
        .eval("return SerializeLayerValue(UnitFrameLayerProbeDB)")
        .unwrap();
    let captures: String = reloaded
        .eval("return SerializeLayerValue(UnitFrameLayerProbeDB.captures)")
        .unwrap();
    let third = fixture();
    load_probe(&third);
    third
        .exec(&format!("UnitFrameLayerProbeDB = {capped}"))
        .unwrap();
    loaded_event(&third, "UnitFrameLayerProbe");
    third.fire_event("PLAYER_LOGIN").unwrap();
    third.exec("FlushLayerTimers()").unwrap();
    assert_eq!(
        captures,
        third
            .eval::<String>("return SerializeLayerValue(UnitFrameLayerProbeDB.captures)")
            .unwrap()
    );
    third.exec("assert(UnitFrameLayerProbeDB.nextSequence == 31 and UnitFrameLayerProbeDB.skippedCaptures == 8)").unwrap();
}
