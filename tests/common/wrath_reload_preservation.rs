//! Geometry-derived placement uses new frame load flags, never old anchor identities.
use wow_ui_sim::lua_api::state::MirrorTimer;
use wow_ui_sim::saved_variables::SavedVariablesManager;

const CURRENT_FRAMES: &str = r#"
    UIParent:SetScale(1)
    WorldMapFrame:SetMovable(true)
    WorldMapFrame:SetResizable(true)
    WorldMapFrame:ClearAllPoints()
    WorldMapFrame:SetPoint('BOTTOMLEFT', UIParent, 'BOTTOMLEFT', 10, 20)
    WorldMapFrame:SetSize(90, 70)
    for _, name in ipairs({'ReloadResizable', 'ReloadMovableOnly', 'ReloadNotPlaced', 'ReloadImmovable', 'ReloadDontSave', 'ReloadLoadDontSave', 'ReloadUnresolved'}) do
        local f = CreateFrame('Frame', name, UIParent)
        f:SetSize(120, 80)
        f:SetPoint('BOTTOMLEFT', UIParent, 'BOTTOMLEFT', 10, 20)
        f:SetFrameLevel(7)
        if name == 'ReloadResizable' then f:SetResizable(true)
        elseif name ~= 'ReloadImmovable' then f:SetMovable(true) end
        if name == 'ReloadLoadDontSave' then f:SetDontSavePosition(true) end
    end
"#;

pub fn assert_placed_frames() {
    let mut env = super::original::load_original_wrath_startup();
    env.exec(CURRENT_FRAMES).unwrap();
    env.set_reload_loader(|env, _| {
        super::original::load_original_wrath_ui(env);
        env.exec(CURRENT_FRAMES)
    });
    env.exec(r#"
        local relative = CreateFrame('Frame', nil, UIParent)
        relative:SetSize(100, 100)
        relative:SetPoint('BOTTOMLEFT', UIParent, 'BOTTOMLEFT', 40, 50)
        WorldMapFrame:SetUserPlaced(true)
        WorldMapFrame:SetDontSavePosition(false)
        WorldMapFrame:ClearAllPoints()
        WorldMapFrame:SetPoint('TOPLEFT', relative, 'BOTTOMLEFT', 41, 534)
        WorldMapFrame:SetSize(713, 491)
        WorldMapFrame:SetFrameLevel(42)
        ReloadDontSave:SetDontSavePosition(true)
        ReloadLoadDontSave:SetDontSavePosition(false)
        for _, f in ipairs({ReloadResizable, ReloadMovableOnly, ReloadDontSave, ReloadImmovable, ReloadLoadDontSave, ReloadUnresolved}) do
            f:SetUserPlaced(true)
        end
        for _, f in ipairs({ReloadResizable, ReloadMovableOnly, ReloadNotPlaced, ReloadDontSave, ReloadImmovable, ReloadLoadDontSave, ReloadUnresolved}) do
            f:ClearAllPoints()
            f:SetPoint('BOTTOMLEFT', UIParent, 'BOTTOMLEFT', 177, 188)
            f:SetSize(321, 219)
            f:SetFrameLevel(31)
        end
        ReloadUnresolved:ClearAllPoints()
        assert(select('#', ReloadUnresolved:GetRect()) == 0, 'fixture has no queryable rect')
        ReloadUnnamed = CreateFrame('Frame', nil, UIParent)
        ReloadUnnamed:SetMovable(true)
        ReloadUnnamed:SetUserPlaced(true)
        ReloadUnnamed:SetSize(100, 50)
        ReloadUnnamed:SetPoint('CENTER', UIParent, 'CENTER')
        ReloadMissing = CreateFrame('Frame', 'ReloadMissing', UIParent)
        ReloadMissing:SetMovable(true)
        ReloadMissing:SetUserPlaced(true)
        ReloadMissing:SetSize(100, 50)
        ReloadMissing:SetPoint('CENTER', UIParent, 'CENTER')
        OldReloadSentinel = {}
        C_Timer.After(0, function() OldReloadTimer = true end)
        ReloadUI()
    "#).unwrap();
    let storage = tempfile::tempdir().unwrap();
    let mut manager = Some(SavedVariablesManager::with_storage_dir(
        storage.path().to_path_buf(),
    ));
    assert!(env.service_reload(&mut manager).unwrap());
    env.exec(r#"
        assert(OldReloadSentinel == nil and ReloadUnnamed == nil and ReloadMissing == nil, 'no old globals/frames')
        local function placement(f, width, height, level, x, y)
            assert(f:GetWidth() == width and f:GetHeight() == height, 'placed size for '..tostring(f:GetName()))
            assert(f:GetFrameLevel() == level, 'placed frame level')
            local point, relative, relativePoint, actualX, actualY = f:GetPoint()
            assert(point == 'BOTTOMLEFT' and relative == UIParent and relativePoint == 'BOTTOMLEFT', 'geometry-derived root anchor, not the original TOPLEFT/unnamed target')
            assert(actualX == x and actualY == y, 'placed offsets')
        end
        placement(WorldMapFrame, 713, 491, 42, 81, 93)
        placement(ReloadResizable, 321, 219, 31, 10, 20)
        placement(ReloadMovableOnly, 120, 80, 31, 177, 188)
        placement(ReloadLoadDontSave, 120, 80, 31, 177, 188)
        assert(ReloadLoadDontSave:IsUserPlaced(), 'load does not recheck DontSavePosition')
        for _, f in ipairs({ReloadNotPlaced, ReloadDontSave, ReloadImmovable, ReloadUnresolved}) do placement(f, 120, 80, 7, 10, 20) end
    "#).unwrap();
    assert!(
        storage
            .path()
            .join("SimRealm/SimPlayer/layout-local.txt")
            .is_file(),
        "per-character placement snapshot is separate from Lua SV"
    );
    wow_ui_sim::startup::run_lua_error_update_ticks(&env);
    assert!(env.eval::<bool>("return OldReloadTimer == nil").unwrap());
}

pub fn assert_anchor_tie() {
    let mut env = super::original::load_original_wrath_startup();
    env.exec("UIParent:SetScale(1); WorldMapFrame:SetMovable(true); WorldMapFrame:SetUserPlaced(true); WorldMapFrame:SetDontSavePosition(false); WorldMapFrame:ClearAllPoints(); WorldMapFrame:SetAllPoints(UIParent)").unwrap();
    env.set_reload_loader(|env, _| {
        super::original::load_original_wrath_ui(env);
        env.exec("UIParent:SetScale(1); WorldMapFrame:SetMovable(true)")
    });
    let storage = tempfile::tempdir().unwrap();
    let mut manager = Some(SavedVariablesManager::with_storage_dir(
        storage.path().to_path_buf(),
    ));
    env.exec("ReloadUI()").unwrap();
    env.service_reload(&mut manager).unwrap();
    env.exec("local point, root, relativePoint, x, y = WorldMapFrame:GetPoint(); assert(WorldMapFrame:GetNumPoints() == 1 and point == 'TOPLEFT' and relativePoint == 'TOPLEFT' and root == UIParent and x == 0 and y == 0, 'first corresponding point wins geometry ties')").unwrap();
}

pub fn assert_tolerant_reader() {
    const FRAMES: &str = "UIParent:SetScale(1); for _, name in ipairs({'ReloadReader', 'ReloadMissingField'}) do local f = CreateFrame('Frame', name, UIParent); f:SetMovable(true); f:SetResizable(true); f:SetSize(99, 88); f:SetPoint('BOTTOMLEFT', UIParent, 'BOTTOMLEFT', 9, 8); f:SetFrameLevel(7) end";
    let mut env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.set_screen_size(1600.0, 1200.0);
    env.exec(FRAMES).unwrap();
    let storage = tempfile::tempdir().unwrap();
    let path = storage.path().join("SimRealm/SimPlayer/layout-local.txt");
    env.set_reload_loader(move |env, _| {
        env.exec(FRAMES)?;
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "vErSiOn: 999\r\nUnknown: ignored\r\nfRaMe: ReloadReader\r\nx: 12junk\r\ny: -7rest\r\nw: 41\r\nh: 52\r\nFrameLevel: 777\r\nAnchor: BOTTOMRIGHT\r\nAnchor: invalid\rFrame: ReloadMissingField\rX: 5").unwrap();
        Ok(())
    });
    let mut manager = Some(SavedVariablesManager::with_storage_dir(
        storage.path().to_path_buf(),
    ));
    env.exec("ReloadUI()").unwrap();
    env.service_reload(&mut manager).unwrap();
    env.exec(r#"
        local point, root, relativePoint, x, y = ReloadReader:GetPoint()
        assert(point == 'BOTTOMRIGHT' and relativePoint == point and root == UIParent and x == 12 and y == -7)
        assert(ReloadReader:GetWidth() == 41 and ReloadReader:GetHeight() == 52 and ReloadReader:GetFrameLevel() == 135, 'level increases cap at 128')
        local p, _, rp, dx, dy = ReloadMissingField:GetPoint()
        assert(p == 'TOPLEFT' and rp == p and dx == 5 and dy == 0, 'per-frame defaults')
        assert(ReloadMissingField:GetWidth() == 0 and ReloadMissingField:GetHeight() == 0, 'reset H rather than reconstruction carry-over')
        assert(ReloadMissingField:GetFrameLevel() == 7, 'missing level does not assign')
    "#).unwrap();
}

pub fn assert_selective_timers() {
    let mut env = super::original::load_original_wrath_startup();
    {
        let mut state = env.state().borrow_mut();
        state.in_world = true;
        state.player.class_index = 7;
        state.world.mirror_timers = vec![MirrorTimer {
            name: "BREATH".into(),
            start_value: 30000.0,
            max_value: 60000.0,
            scale: -1.0,
            label: "Breath".into(),
            ..Default::default()
        }];
        state.totem_slots = vec![
            Some(wow_ui_sim::lua_api::globals::real::totems::Totem {
                name: "Reload fire totem".into(),
                start_time: 0.0,
                duration: 600.0,
                icon: 135817,
                mod_rate: 1.0,
                spell_id: 0,
            }),
            None,
            None,
            None,
        ];
    }
    let guid: String = env.eval("return UnitGUID('player')").unwrap();
    env.fire_event("PLAYER_ENTERING_WORLD").unwrap();
    env.exec("assert(GetMirrorTimerInfo(1) == 'BREATH'); assert(TotemFrame.activeTotems == 1, 'original totem consumer')").unwrap();
    env.set_reload_loader(|env, _| {
        super::original::load_original_wrath_ui(env);
        Ok(())
    });
    env.exec("ReloadUI()").unwrap();
    assert!(env.service_reload(&mut None).unwrap());
    wow_ui_sim::startup::fire_startup_events(&env);
    assert_eq!(
        env.eval::<String>("return GetMirrorTimerInfo(1)").unwrap(),
        "UNKNOWN",
        "12340 LeaveWorld cancels mirror timers"
    );
    assert_eq!(
        env.eval::<String>("return UnitGUID('player')").unwrap(),
        guid
    );
    env.exec(r#"
        assert(GetTotemInfo(1) == true and TotemFrame.activeTotems == 1 and TotemFrame:IsShown(), 'totem survives and original fresh PEW consumer renders it')
        for i = 1, MIRRORTIMER_NUMTIMERS do assert(_G['MirrorTimer'..i].timer == nil, 'fresh original mirror bars do not restart') end
    "#).unwrap();
}
