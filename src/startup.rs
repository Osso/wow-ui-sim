//! Startup event sequence for headless (dump-tree / screenshot) mode.
//!
//! Fires the WoW login event sequence, processes pending timers,
//! and runs one OnUpdate tick so OnUpdate-dependent state (e.g. buff
//! durations) is populated even without a GUI loop.

use crate::lua_api::WowLuaEnv;

/// Process any C_Timer callbacks that became ready during startup.
pub fn process_pending_timers(env: &WowLuaEnv) {
    for _ in 0..10 {
        match env.process_timers() {
            Ok(0) => break,
            Ok(_) => {}
            Err(e) => {
                eprintln!("[Timers] error: {e}");
                break;
            }
        }
    }
}

/// Sleep for the given number of milliseconds (if specified).
pub fn apply_delay(delay: Option<u64>) {
    if let Some(ms) = delay {
        eprintln!("[Startup] Delaying {}ms", ms);
        std::thread::sleep(std::time::Duration::from_millis(ms));
    }
}

/// Fire a single OnUpdate tick so OnUpdate-dependent state (e.g. buff
/// durations) is populated in headless modes where the GUI loop never runs.
pub fn fire_one_on_update_tick(env: &WowLuaEnv) {
    if let Err(e) = env.fire_on_update(0.016) {
        eprintln!("[OnUpdate tick] error: {e}");
    }
}

/// Fire startup events to simulate WoW login sequence.
pub fn fire_startup_events(env: &WowLuaEnv) {
    let fire = |name| {
        eprintln!("[Startup] Firing {}", name);
        if let Err(e) = env.fire_event(name) {
            eprintln!("Error firing {}: {}", name, e);
        }
    };

    eprintln!("[Startup] Firing ADDON_LOADED");
    if let Err(e) = env.fire_event_with_args(
        "ADDON_LOADED",
        &[mlua::Value::String(env.lua().create_string("WoWUISim").unwrap())],
    ) {
        eprintln!("Error firing ADDON_LOADED: {}", e);
    }

    fire("VARIABLES_LOADED");
    fire("PLAYER_LOGIN");

    eprintln!("[Startup] Firing EDIT_MODE_LAYOUTS_UPDATED");
    if let Err(e) = env.fire_edit_mode_layouts_updated() {
        eprintln!("  {}", e);
    }

    eprintln!("[Startup] Firing TIME_PLAYED_MSG via RequestTimePlayed");
    if let Err(e) = env.lua().globals().get::<mlua::Function>("RequestTimePlayed")
        .and_then(|f| f.call::<()>(()))
    {
        eprintln!("Error calling RequestTimePlayed: {}", e);
    }

    eprintln!("[Startup] Firing PLAYER_ENTERING_WORLD");
    if let Err(e) = env.fire_event_with_args(
        "PLAYER_ENTERING_WORLD",
        &[mlua::Value::Boolean(true), mlua::Value::Boolean(false)],
    ) {
        eprintln!("Error firing PLAYER_ENTERING_WORLD: {}", e);
    }

    call_unit_frame_set_unit(env);

    fire_unit_aura(env);
    diagnose_buff_pipeline(env);

    fire("BAG_UPDATE_DELAYED");
    fire("GROUP_ROSTER_UPDATE");
    force_show_party_member_frames(env);
    fire("UPDATE_BINDINGS");
    fire("DISPLAY_SIZE_CHANGED");
    fire("UI_SCALE_CHANGED");
    fire("UPDATE_CHAT_WINDOWS");
    fire("PLAYER_LEAVING_WORLD");
}

/// Call `UnitFrame_SetUnit` on the main unit frames after PLAYER_ENTERING_WORLD.
///
/// In real WoW, `PlayerFrame_ToPlayerArt` calls `UnitFrame_SetUnit` during
/// `PLAYER_ENTERING_WORLD`. `UnitFrame_Initialize` (called during `OnLoad`)
/// already sets `self.unit`, but `UnitFrame_SetUnit` also registers unit events
/// on health/mana bars, sets the `"unit"` attribute, and calls `UnitFrame_Update`.
/// If something in the event chain errors before reaching `UnitFrame_SetUnit`,
/// the unit binding is incomplete. This ensures the call happens for each frame.
pub fn call_unit_frame_set_unit(env: &WowLuaEnv) {
    if let Err(e) = env.exec(
        r#"
        if not UnitFrame_SetUnit then return end

        local frames = {
            {
                frame = PlayerFrame,
                unit = "player",
                healthbar = PlayerFrame_GetHealthBar and PlayerFrame_GetHealthBar(),
                manabar = PlayerFrame_GetManaBar and PlayerFrame_GetManaBar(),
            },
            {
                frame = PetFrame,
                unit = "pet",
                healthbar = PetFrameHealthBar,
                manabar = PetFrameManaBar,
            },
            {
                frame = TargetFrame,
                unit = "target",
                healthbar = TargetFrame and TargetFrame.healthbar,
                manabar = TargetFrame and TargetFrame.manabar,
            },
            {
                frame = FocusFrame,
                unit = "focus",
                healthbar = FocusFrame and FocusFrame.healthbar,
                manabar = FocusFrame and FocusFrame.manabar,
            },
        }

        for _, info in ipairs(frames) do
            if info.frame and info.healthbar then
                local ok, err = pcall(UnitFrame_SetUnit,
                    info.frame, info.unit, info.healthbar, info.manabar)
                if not ok then
                    print("[startup] UnitFrame_SetUnit("
                        .. (info.frame:GetName() or "?") .. ", "
                        .. info.unit .. ") failed: " .. tostring(err))
                end
            end
        end
    "#,
    ) {
        eprintln!("[startup] call_unit_frame_set_unit error: {e}");
    }
}

/// Fire UNIT_AURA("player", {isFullUpdate=true}) to populate buff frames.
fn fire_unit_aura(env: &WowLuaEnv) {
    eprintln!("[Startup] Firing UNIT_AURA");
    let lua = env.lua();
    if let (Ok(unit), Ok(info)) = (lua.create_string("player"), lua.create_table()) {
        let _ = info.set("isFullUpdate", true);
        if let Err(e) = env.fire_event_with_args(
            "UNIT_AURA",
            &[mlua::Value::String(unit), mlua::Value::Table(info)],
        ) {
            eprintln!("Error firing UNIT_AURA: {}", e);
        }
    }
}

/// Force-show party member frames after GROUP_ROSTER_UPDATE.
///
/// UpdateRaidAndPartyFrames() hides all party frames first, then calls
/// CompactRaidFrameManager_UpdateShown() which errors on missing dividerVerticalPool,
/// preventing PartyFrame:UpdatePartyFrames() from re-showing them.
/// This safety net shows each member frame individually with pcall wrappers.
fn force_show_party_member_frames(env: &WowLuaEnv) {
    if let Err(e) = env.exec(r#"
        if not PartyFrame or not PartyFrame.PartyMemberFramePool then return end
        local pool = PartyFrame.PartyMemberFramePool
        local i = 0
        for mf in pool:EnumerateActive() do
            i = i + 1
            if not mf.layoutIndex then mf.layoutIndex = i end
            if not mf.unitToken then
                mf.unitToken = "party" .. mf.layoutIndex
            end
            pcall(function() mf:Setup() end)
        end
        for mf in pool:EnumerateActive() do
            if PartyFrame:ShouldShow() and UnitExists(mf.unitToken) then
                mf:Show()
                pcall(function() UnitFrame_Update(mf, true) end)
                pcall(function() mf:UpdatePet() end)
                pcall(function() mf:UpdateAuras() end)
                pcall(function() mf:UpdateOnlineStatus() end)
                pcall(function() mf:UpdateArt() end)
            end
        end
        PartyFrame:Layout()
    "#) {
        eprintln!("[startup] party frame safety-net error: {e}");
    }
}

/// Trace the buff pipeline state after UNIT_AURA fires (temporary diagnostic).
pub fn diagnose_buff_pipeline(env: &WowLuaEnv) {
    if let Err(e) = env.exec(r#"
        print("[DIAG] PlayerFrame.unit =", tostring(PlayerFrame and PlayerFrame.unit))
        print("[DIAG] AuraUtil.ForEachAura type =", type(AuraUtil and AuraUtil.ForEachAura))
        if AuraUtil and AuraUtil.ForEachAura and PlayerFrame and PlayerFrame.unit then
            local count = 0
            AuraUtil.ForEachAura(PlayerFrame.unit, "HELPFUL", 32, function(auraData)
                count = count + 1
                local n = type(auraData) == "table" and auraData.name or tostring(auraData)
                print("[DIAG] ForEachAura cb #" .. count .. ": " .. tostring(n))
            end, true)
            print("[DIAG] ForEachAura returned " .. count .. " buffs")
        else
            print("[DIAG] ForEachAura or PlayerFrame.unit missing")
        end
        if BuffFrame then
            print("[DIAG] BuffFrame vis=" .. tostring(BuffFrame:IsVisible()))
            local ai = BuffFrame.auraInfo
            if ai then
                local n = 0; for _ in pairs(ai) do n = n + 1 end
                print("[DIAG] BuffFrame.auraInfo has " .. n .. " entries")
            else
                print("[DIAG] BuffFrame.auraInfo is nil")
            end
            local af = BuffFrame.auraFrames
            if af and af[1] then
                print("[DIAG] auraFrames[1] vis=" .. tostring(af[1]:IsVisible()))
            else
                print("[DIAG] auraFrames missing or empty")
            end
        else
            print("[DIAG] BuffFrame missing")
        end
    "#) {
        eprintln!("[DIAG] error: {e}");
    }
}
