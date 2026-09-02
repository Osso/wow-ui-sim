//! Startup event sequence for headless (dump-tree / screenshot) mode.
//!
//! Fires the WoW login event sequence, processes pending timers,
//! and runs one OnUpdate tick so OnUpdate-dependent state (e.g. buff
//! durations) is populated even without a GUI loop.

use crate::lua_api::WowLuaEnv;
use crate::screen::ScreenKind;
use rilua::Val;

const UNIT_FRAME_SET_UNIT_LUA: &str = r#"
    if not UnitFrame_SetUnit then return end

    local frames = {
        {
            frame = PlayerFrame,
            unit = "player",
            healthbar = PlayerFrame and PlayerFrame_GetHealthBar and PlayerFrame_GetHealthBar(),
            manabar = PlayerFrame and PlayerFrame_GetManaBar and PlayerFrame_GetManaBar(),
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
"#;

const FORCE_SHOW_PARTY_MEMBER_FRAMES_LUA: &str = r#"
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
"#;

const GLUE_HIDE_CHAT: &str = r#"
    if GeneralDockManager then GeneralDockManager:Hide() end
    if ChatFrame1 then ChatFrame1:Hide() end
    if ChatFrame1Tab then ChatFrame1Tab:Hide() end
    if ChatFrame1EditBox then ChatFrame1EditBox:Hide() end
"#;

const GLUE_LOGIN_HIDE_CHAT: &str = r#"
    if AllowChatFramesToShow and ChatFrame1 and not AllowChatFramesToShow(ChatFrame1) then
        if GeneralDockManager then GeneralDockManager:Hide() end
        if ChatFrame1 then ChatFrame1:Hide() end
        if ChatFrame1Tab then ChatFrame1Tab:Hide() end
        if ChatFrame1EditBox then ChatFrame1EditBox:Hide() end
    end
    if CharCustomizeFrame then CharCustomizeFrame:Hide() end
"#;

const UNBLOCK_HIDDEN_SPLASH_ALERTS: &str = r#"
    if AlertFrame and SplashFrame and not SplashFrame:IsShown() then
        AlertFrame:SetAlertsEnabled(true, "splashFrame")
    end
"#;

fn log_with_timestamp(env: &WowLuaEnv, message: &str) {
    let start_time = env.state().borrow().start_time;
    eprintln!("{} {}", crate::logging::elapsed_prefix(start_time), message);
}

/// Process any C_Timer callbacks that became ready during startup.
pub fn process_pending_timers(env: &WowLuaEnv) {
    for _ in 0..10 {
        match env.process_timers() {
            Ok(0) => break,
            Ok(_) => {}
            Err(e) => {
                log_with_timestamp(env, &format!("[Timers] error: {e}"));
                break;
            }
        }
    }
}

/// Sleep for the given number of milliseconds (if specified).
pub fn apply_delay(delay: Option<u64>) {
    if let Some(ms) = delay {
        crate::logging::eprintln_elapsed(&format!("[delay] sleeping {ms}ms"));
        std::thread::sleep(std::time::Duration::from_millis(ms));
    }
}

/// `--delay` for the headless commands: sleep, then fire one OnUpdate tick
/// that reports the slept time as `elapsed` and run the timers that came due.
///
/// `GetTime` is wall-clock, so an addon that defers its initialisation with
/// `C_Timer.After(2)` (QuickRoute does) has its timer due after the sleep, and
/// an OnUpdate handler that throttles on accumulated `elapsed` (QuickRoute's
/// secure-overlay positioner waits for 0.1 s) sees the whole delay in one
/// tick. A bare sleep left both waiting: the three 16 ms ticks that follow
/// `--exec-lua` never reached the timer, and the handler saw 48 ms.
pub fn apply_delay_with_tick(env: &WowLuaEnv, delay: Option<u64>) {
    let Some(ms) = delay else {
        return;
    };
    apply_delay(Some(ms));
    env.state().borrow_mut().ensure_layout_rects();
    if let Err(e) = env.fire_on_update(ms as f64 / 1000.0) {
        log_with_timestamp(env, &format!("[delay tick] error: {e}"));
    }
    normalize_headless_frame_positions(env);
    process_pending_timers(env);
}

/// Demand-load Blizzard_PlayerSpells during game-screen startup and keep it hidden.
pub fn prewarm_player_spells_spellbook(env: &WowLuaEnv) -> bool {
    if env.state().borrow().screen_kind != ScreenKind::Game {
        return false;
    }

    env.eval::<bool>(
        r#"
        if not C_AddOns or type(C_AddOns.LoadAddOn) ~= "function" or type(C_AddOns.IsAddOnLoaded) ~= "function" then
            return false
        end

        if not C_AddOns.IsAddOnLoaded("Blizzard_PlayerSpells") then
            C_AddOns.LoadAddOn("Blizzard_PlayerSpells")
        end

        if PlayerSpellsFrame and PlayerSpellsFrame:IsShown() then
            PlayerSpellsFrame:Hide()
        end

        return C_AddOns.IsAddOnLoaded("Blizzard_PlayerSpells")
            and PlayerSpellsFrame ~= nil
            and not PlayerSpellsFrame:IsShown()
        "#,
    )
    .unwrap_or(false)
}

/// Fire a single OnUpdate tick so OnUpdate-dependent state (e.g. buff
/// durations) is populated in headless modes where the GUI loop never runs.
pub fn fire_one_on_update_tick(env: &WowLuaEnv) {
    if let Err(e) = env.fire_on_update(0.016) {
        log_with_timestamp(env, &format!("[OnUpdate tick] error: {e}"));
    }
    normalize_headless_frame_positions(env);
}

/// Fire one GUI startup OnUpdate tick without applying headless-only layout normalizers.
pub fn fire_gui_startup_on_update_tick(env: &WowLuaEnv) {
    if let Err(e) = env.fire_on_update(0.016) {
        log_with_timestamp(env, &format!("[GUI OnUpdate tick] error: {e}"));
    }
}

/// Scale UIParent before the canvas size is applied. `set_screen_size` fires
/// DISPLAY_SIZE_CHANGED / UI_SCALE_CHANGED and replays the edit-mode anchor
/// hooks, which is what re-runs `ManageFramePositions` and sizes the managed
/// containers for the scaled UIParent. Scaling from `--exec-lua` instead runs
/// after that pass and leaves the containers laid out for scale 1.
pub fn apply_ui_scale(env: &WowLuaEnv, ui_scale: f32) {
    if ui_scale == 1.0 {
        return;
    }
    if let Err(e) = env.exec(&format!("UIParent:SetScale({ui_scale})")) {
        eprintln!("[ui-scale] error: {e}");
    }
}

/// Fire extra OnUpdate ticks so deferred UI can process in headless commands.
pub fn run_extra_update_ticks(env: &WowLuaEnv, n: usize) {
    for _ in 0..n {
        env.state().borrow_mut().ensure_layout_rects();
        fire_one_on_update_tick(env);
        process_pending_timers(env);
    }
}

/// Finish startup animation groups that were kicked by login/update events
/// before the first rendered frame is captured.
pub fn settle_startup_animation_groups(env: &WowLuaEnv) {
    if let Err(e) =
        crate::lua_api::frame::methods::button_anchor_hierarchy::advance_animation_groups(env, 2.0)
    {
        log_with_timestamp(env, &format!("[Startup animations] error: {e}"));
    }
}

fn dismiss_headless_glue_overlays(env: &WowLuaEnv) {
    let screen = env.state().borrow().screen_kind;
    if screen == ScreenKind::Game {
        return;
    }

    let _ = env.exec(
        r#"
        if type(PhotosensitivityWarningFrame) == "table"
            and PhotosensitivityWarningFrame:IsShown()
            and type(PhotosensitivityWarningFrame.GetLockedByOtherWarning) == "function"
            and not PhotosensitivityWarningFrame:GetLockedByOtherWarning()
            and type(PhotosensitivityWarningFrame.ShowNextFrame) == "function"
        then
            PhotosensitivityWarningFrame:ShowNextFrame()
        end
        "#,
    );
}

fn unblock_hidden_splash_alerts(env: &WowLuaEnv) {
    let _ = env.exec(UNBLOCK_HIDDEN_SPLASH_ALERTS);
}

/// Fire startup events to simulate WoW login sequence.
pub fn fire_startup_events(env: &WowLuaEnv) {
    env.set_screen_mode(ScreenKind::Game);
    time_startup_step(env, "login sequence", || fire_login_sequence(env, false));
    time_startup_step(env, "world enter sequence", || {
        fire_world_enter_sequence(env)
    });
    time_startup_step(env, "post-login events", || fire_post_login_events(env));
    time_startup_step(env, "close startup special windows", || {
        crate::lua_api::workarounds::close_startup_special_windows_before_first_frame(env)
    });
    time_startup_step(env, "FIRST_FRAME_RENDERED", || {
        fire_simple_event(env, "FIRST_FRAME_RENDERED")
    });
    time_startup_step(env, "unblock hidden splash alerts", || {
        unblock_hidden_splash_alerts(env)
    });
    time_startup_step(env, "post-event workarounds", || {
        env.apply_post_event_workarounds()
    });
}

/// Fire startup events for a selected top-level screen.
pub fn fire_startup_events_for_screen(env: &WowLuaEnv, screen: ScreenKind) {
    match screen {
        ScreenKind::Game => fire_startup_events(env),
        ScreenKind::Login | ScreenKind::CharacterSelect | ScreenKind::CharacterCreate => {
            fire_glue_startup_events(env, screen)
        }
    }
}

/// Run startup events, workarounds, timers, and a few extra update ticks so
/// headless commands see the same settled UI state.
pub fn settle_headless_startup(env: &WowLuaEnv) {
    let screen = env.state().borrow().screen_kind;
    fire_startup_events_for_screen(env, screen);
    env.apply_post_event_workarounds();
    settle_startup_animation_groups(env);
    dismiss_headless_glue_overlays(env);
    {
        let mut state = env.state().borrow_mut();
        state.widgets.rebuild_anchor_index();
        state.initialize_render_state();
    }
    process_pending_timers(env);
    fire_one_on_update_tick(env);
    let _ = crate::lua_api::globals::global_frames::hide_runtime_hidden_frames(&*env.rilua());
    run_extra_update_ticks(env, 3);
    refresh_character_select_screen(env);
    run_extra_update_ticks(env, 2);
}

/// Fire startup events needed for `lua-errors` without doing render/layout
/// settling that only dump and screenshot commands need.
pub fn collect_lua_error_startup(env: &WowLuaEnv) {
    let screen = env.state().borrow().screen_kind;
    fire_startup_events_for_screen(env, screen);
    run_lua_error_update_ticks(env);
}

/// Run a small OnUpdate/timer slice for `lua-errors` without full headless
/// layout/render settling.
pub fn run_lua_error_update_ticks(env: &WowLuaEnv) {
    process_pending_timers(env);
    fire_one_on_update_tick(env);
    process_pending_timers(env);
    run_extra_update_ticks(env, 2);
}

/// Fire startup events for headless test mode (skips IsLoggedIn override).
pub fn fire_startup_events_headless(env: &WowLuaEnv) {
    env.set_screen_mode(ScreenKind::Game);
    time_startup_step(env, "login sequence", || fire_login_sequence(env, true));
    time_startup_step(env, "world enter sequence", || {
        fire_world_enter_sequence(env)
    });
    time_startup_step(env, "post-login events", || fire_post_login_events(env));
}

fn time_startup_step(env: &WowLuaEnv, label: &str, step: impl FnOnce()) {
    let start = std::time::Instant::now();
    log_with_timestamp(env, &format!("[Startup] begin {label}"));
    step();
    log_with_timestamp(
        env,
        &format!("[Startup] end {label} in {:.2?}", start.elapsed()),
    );
}

/// Fire ADDON_LOADED, VARIABLES_LOADED, PLAYER_LOGIN and optionally set IsLoggedIn.
fn fire_login_sequence(env: &WowLuaEnv, skip_is_logged_in: bool) {
    env.set_logged_in(false);
    let fire = |name| fire_simple_event(env, name);

    log_with_timestamp(env, "[Startup] Firing ADDON_LOADED");
    if let Err(e) = env.fire_event_with_args("ADDON_LOADED", &[env.lua_string("WoWUISim")]) {
        log_with_timestamp(env, &format!("Error firing ADDON_LOADED: {e}"));
    }

    fire("VARIABLES_LOADED");

    // Retail fires the display/scale pair before PLAYER_LOGIN (live probe:
    // docs/wiki/investigations/display-size-ui-scale-events.md). The first
    // pre-login pair comes from set_screen_size during GUI canvas startup;
    // this is the second, after variables load and the UI scale applies.
    fire("DISPLAY_SIZE_CHANGED");
    fire("UI_SCALE_CHANGED");

    // In WoW, IsLoggedIn() returns true once the player is logged in.
    // AceAddon-3.0 checks IsLoggedIn() before enabling addons from its queue.
    if !skip_is_logged_in {
        env.set_logged_in(true);
    }

    fire("PLAYER_LOGIN");
}

fn fire_glue_startup_events(env: &WowLuaEnv, screen: ScreenKind) {
    env.set_screen_mode(screen);
    env.set_logged_in(false);
    fire_simple_event(env, "FRAMES_LOADED");
    if let Some(screen_name) = screen.glue_screen_name()
        && let Err(e) = env.exec(&format!(
            "if GlueParent_SetScreen then GlueParent_SetScreen({screen_name:?}) end"
        ))
    {
        log_with_timestamp(
            env,
            &format!("Error switching glue screen to {screen_name}: {e}"),
        );
    }
    apply_glue_screen_visibility(env, screen);
    if screen == ScreenKind::CharacterSelect {
        prime_character_select_frame(env);
    }
    env.state().borrow_mut().screen_first_displayed = true;
    fire_simple_event(env, "SCREEN_FIRST_DISPLAYED");
    fire_simple_event(env, "LOGIN_STATE_CHANGED");
}

fn prime_character_select_frame(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if type(CharacterSelect) == "table"
            and type(CharacterSelect.OnLoad) == "function"
            and CharacterSelectCharacterFrame == nil
        then
            CharacterSelect:OnLoad()
        end
        "#,
    );
}

fn alias_character_select_globals(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if type(CharacterSelectUI) == "table"
            and type(CharacterSelectUI.VisibilityFramesContainer) == "table" then
            if CharacterSelectCharacterFrame == nil
                and type(CharacterSelectUI.VisibilityFramesContainer.CharacterList) == "table" then
                CharacterSelectCharacterFrame = CharacterSelectUI.VisibilityFramesContainer.CharacterList
            end
            if CharSelectCharacterName == nil
                and type(CharacterSelectUI.VisibilityFramesContainer.CharSelectCharacterName) == "table" then
                CharSelectCharacterName = CharacterSelectUI.VisibilityFramesContainer.CharSelectCharacterName
            end
        end
        "#,
    );
}

fn refresh_character_select_roster(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if type(CharacterSelectListUtil) == "table"
            and type(CharacterSelectListUtil.BuildCharIndexToIDMapping) == "function" then
            pcall(function()
                CharacterSelectListUtil.BuildCharIndexToIDMapping()
            end)
        end
        if type(CharacterSelectUI) == "table"
            and type(CharacterSelectUI.RefreshConfig) == "function" then
            pcall(function()
                CharacterSelectUI:RefreshConfig()
            end)
        end
        if type(CharacterSelectListUtil) == "table"
            and type(CharacterSelectListUtil.GetCharacterListUpdate) == "function" then
            pcall(function()
                CharacterSelectListUtil.GetCharacterListUpdate()
            end)
        elseif type(GetCharacterListUpdate) == "function" then
            pcall(function()
                GetCharacterListUpdate()
            end)
        end
        "#,
    );
}

fn show_character_select_frame(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if type(CharacterSelect) == "table"
            and type(CharacterSelect.OnShow) == "function"
            and not rawget(_G, "__wow_character_select_frame_onshow_ran") then
            pcall(function()
                CharacterSelect:OnShow()
            end)
            rawset(_G, "__wow_character_select_frame_onshow_ran", true)
        end
        "#,
    );
}

fn refresh_character_select_screen(env: &WowLuaEnv) {
    alias_character_select_globals(env);
    prime_character_select_frame(env);
    show_character_select_frame(env);
    refresh_character_select_roster(env);
}

fn apply_glue_screen_visibility(env: &WowLuaEnv, screen: ScreenKind) {
    let screen_name = match screen {
        ScreenKind::Game => return,
        ScreenKind::CharacterSelect => "charselect",
        ScreenKind::CharacterCreate => "charcreate",
        ScreenKind::Login => "login",
    };
    let hide_chat = if screen == ScreenKind::Login {
        // Login screen only hides chat when AllowChatFramesToShow returns false
        GLUE_LOGIN_HIDE_CHAT
    } else {
        GLUE_HIDE_CHAT
    };
    let script = format!(
        "if GlueParent_GetCurrentScreen and GlueParent_GetCurrentScreen() == \"{screen_name}\" then\n\
         {hide_chat}\n\
         end"
    );
    if let Err(e) = env.exec(&script) {
        log_with_timestamp(
            env,
            &format!("[Startup] glue visibility normalization failed: {e}"),
        );
    }
}

/// Fire EDIT_MODE_LAYOUTS_UPDATED, TIME_PLAYED_MSG, and PLAYER_ENTERING_WORLD.
fn fire_world_enter_sequence(env: &WowLuaEnv) {
    log_with_timestamp(env, "[Startup] Skipping EDIT_MODE_LAYOUTS_UPDATED");

    log_with_timestamp(
        env,
        "[Startup] Firing TIME_PLAYED_MSG via RequestTimePlayed",
    );
    if let Err(e) = env.call_global("RequestTimePlayed", &[]) {
        log_with_timestamp(env, &format!("Error calling RequestTimePlayed: {e}"));
    }

    log_with_timestamp(env, "[Startup] Firing PLAYER_ENTERING_WORLD");
    if let Err(e) = env.fire_event_with_args(
        "PLAYER_ENTERING_WORLD",
        &[Val::Bool(true), Val::Bool(false)],
    ) {
        log_with_timestamp(env, &format!("Error firing PLAYER_ENTERING_WORLD: {e}"));
    }
}

/// Fire post-login events: unit frames, auras, bags, UI updates.
fn fire_post_login_events(env: &WowLuaEnv) {
    let fire = |name| fire_simple_event(env, name);

    call_unit_frame_set_unit(env);
    fire_unit_aura(env);

    fire("PET_UI_UPDATE");
    fire("BAG_UPDATE_DELAYED");
    fire("QUEST_LOG_UPDATE");
    resize_party_state(&mut env.state().borrow_mut(), 4);
    refresh_party_frames(env);
    fire("ACTIONBAR_UPDATE_STATE");
    fire("ACTIONBAR_UPDATE_COOLDOWN");
    fire("UPDATE_BONUS_ACTIONBAR");
    fire("PLAYER_CAN_GLIDE_CHANGED");
    fire("PLAYER_IS_GLIDING_CHANGED");
    fire("UPDATE_BINDINGS");
    // Retail does not fire DISPLAY_SIZE_CHANGED / UI_SCALE_CHANGED after
    // PLAYER_LOGIN at startup — both pairs fire pre-login (see
    // docs/wiki/investigations/display-size-ui-scale-events.md). The pair is
    // fired in fire_login_sequence and by set_screen_size instead.
    fire("UPDATE_CHAT_WINDOWS");
    // Drives LFDQueueFrame_SetType, which shows the Specific/Follower
    // sub-frame whose OnShow=LFDQueueFrame_Update populates the dungeon
    // list. Without this, opening the Dungeons & Raids panel leaves the
    // list empty until the user changes the Type dropdown.
    //
    // LFGLockList is initialized via a post-load workaround instead of
    // firing LFG_LOCK_INFO_RECEIVED, because that event also triggers
    // RaidFinder/ScenarioFinder availability checks that require many
    // additional unmodeled APIs (GetNumRFDungeons, GetNumRandomScenarios,
    // etc.). Direct assignment is enough to satisfy LFDQueueFrame.
    fire("LFG_UPDATE_RANDOM_INFO");
    seed_buff_durations(env);
}

pub(crate) fn resize_party_state(state: &mut crate::lua_api::SimState, size: usize) {
    let clamped_size = size.min(4);
    let defaults = crate::lua_api::game_data::default_party();
    while state.party_members.len() < clamped_size {
        let next_idx = state.party_members.len();
        let Some(member) = defaults.get(next_idx).cloned() else {
            break;
        };
        state.party_members.push(member);
    }
    state.party_members.truncate(clamped_size);
    state.party_group_active = clamped_size > 0;
    state.party_leader_index = None;
}

fn normalize_headless_frame_positions(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if ChatFrame1EditBox then
            ChatFrame1EditBox:SetWidth(447)
        end

        if CompactPartyFrame then
            CompactPartyFrame:SetSize(98, 234)
        end

        if PlayerCastingBarFrame then
            PlayerCastingBarFrame:SetAlpha(1)
        end
    "#,
    );
}

/// Fire a simple event with no arguments, logging to stderr.
fn fire_simple_event(env: &WowLuaEnv, name: &str) {
    log_with_timestamp(env, &format!("[Startup] Firing {name}"));
    if let Err(e) = env.fire_event(name) {
        log_with_timestamp(env, &format!("Error firing {name}: {e}"));
    }
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
    if let Err(e) = env.exec(UNIT_FRAME_SET_UNIT_LUA) {
        log_with_timestamp(
            env,
            &format!("[startup] call_unit_frame_set_unit error: {e}"),
        );
    }
}

/// Fire UNIT_AURA("player", {isFullUpdate=true}) to populate buff frames.
fn fire_unit_aura(env: &WowLuaEnv) {
    log_with_timestamp(env, "[Startup] Firing UNIT_AURA");
    let unit = env.lua_string("player");
    if let Ok(info) = env.eval::<Val>("return { isFullUpdate = true }")
        && let Err(e) = env.fire_event_with_args("UNIT_AURA", &[unit, info])
    {
        log_with_timestamp(env, &format!("Error firing UNIT_AURA: {e}"));
    }
}

/// Force-show party member frames after GROUP_ROSTER_UPDATE.
///
/// UpdateRaidAndPartyFrames() hides all party frames first, then calls
/// CompactRaidFrameManager_UpdateShown() which errors on missing dividerVerticalPool,
/// preventing PartyFrame:UpdatePartyFrames() from re-showing them.
/// This safety net shows each member frame individually with pcall wrappers.
pub(crate) fn refresh_party_frames(env: &WowLuaEnv) {
    fire_simple_event(env, "GROUP_ROSTER_UPDATE");
    force_show_party_member_frames(env);
}

fn force_show_party_member_frames(env: &WowLuaEnv) {
    if let Err(e) = env.exec(FORCE_SHOW_PARTY_MEMBER_FRAMES_LUA) {
        log_with_timestamp(env, &format!("[startup] party frame safety-net error: {e}"));
    }
}

/// Seed buff duration text so it's visible immediately without waiting
/// for the first OnUpdate tick. OnUpdate handlers maintain it afterwards.
///
/// A permanent aura carries `duration == 0` and `expirationTime == 0`, so
/// `expirationTime - GetTime()` is negative and the icon renders a countdown
/// running backwards from zero. Blizzard gates the same computation on
/// `info.duration > 0 and info.expirationTime > 0` (BuffFrame.lua:812, again at
/// :1026); that guard is applied here, so an aura with no duration gets no
/// timer text at all.
const SEED_BUFF_DURATIONS_LUA: &str = r#"
        if not BuffFrame or not BuffFrame.auraFrames then return end
        for _, b in ipairs(BuffFrame.auraFrames) do
            if b:IsVisible() and b.UpdateDuration then
                local timeLeft = b.timeLeft
                local info = b.buttonInfo
                if not timeLeft and info
                        and info.duration and info.duration > 0
                        and info.expirationTime and info.expirationTime > 0 then
                    timeLeft = info.expirationTime - GetTime()
                    if info.timeMod and info.timeMod > 0 then
                        timeLeft = timeLeft / info.timeMod
                    end
                end
                if timeLeft and timeLeft > 0 then
                    pcall(b.UpdateDuration, b, timeLeft)
                end
            end
        end
    "#;

pub fn seed_buff_durations(env: &WowLuaEnv) {
    let _ = env.exec(SEED_BUFF_DURATIONS_LUA);
}

#[cfg(test)]
mod seed_buff_duration_tests {
    use super::*;

    /// A permanent aura reports `duration == 0` and `expirationTime == 0`.
    /// Subtracting `GetTime()` from that yields a negative number, which the
    /// icon rendered as a countdown running backwards ("-2 s"). Blizzard gates
    /// the same computation on both fields being positive.
    #[test]
    fn permanent_auras_get_no_duration_text() {
        let env = WowLuaEnv::new().unwrap();
        let (timed, permanent, no_info, zero_duration, stale_negative): (
            String,
            String,
            String,
            String,
            String,
        ) = env
            .eval(&format!(
                r#"
                local seen = {{}}
                local function frame(name, duration, expirationTime, presetTimeLeft)
                    return {{
                        name = name,
                        timeLeft = presetTimeLeft,
                        buttonInfo = duration and
                            {{ duration = duration, expirationTime = expirationTime }} or nil,
                        IsVisible = function() return true end,
                        UpdateDuration = function(self, timeLeft)
                            seen[self.name] = timeLeft
                        end,
                    }}
                end

                BuffFrame = {{ auraFrames = {{
                    frame("timed", 3600, GetTime() + 1800),
                    frame("permanent", 0, 0),
                    frame("no_info", nil, nil),
                    -- Discriminates the duration guard from the timeLeft>0 guard:
                    -- expiry is in the future, so a positive timeLeft would pass
                    -- the second check, but Blizzard requires BOTH fields positive.
                    frame("zero_duration", 0, GetTime() + 600),
                    -- The frame already carries a stale negative timeLeft, so the
                    -- duration guard never runs for it. Only the timeLeft>0 check
                    -- keeps this one from rendering a backwards countdown.
                    frame("stale_negative", 3600, GetTime() + 1800, -5),
                }} }}

                {snippet}

                local function report(key)
                    local v = seen[key]
                    if v == nil then return "none" end
                    return v > 0 and "positive" or "NONPOSITIVE"
                end
                return report("timed"), report("permanent"), report("no_info"),
                    report("zero_duration"), report("stale_negative")
                "#,
                snippet = SEED_BUFF_DURATIONS_LUA
            ))
            .unwrap();

        assert_eq!(
            timed, "positive",
            "an aura with a real duration keeps its timer"
        );
        assert_eq!(
            permanent, "none",
            "a permanent aura must get no timer text; a negative countdown is the defect"
        );
        assert_eq!(
            no_info, "none",
            "an aura with no buttonInfo must be skipped"
        );
        assert_eq!(
            zero_duration, "none",
            "duration 0 means permanent even when expirationTime is in the future; \
             this is the case the duration guard catches and the timeLeft>0 guard does not"
        );
        assert_eq!(
            stale_negative, "none",
            "a frame carrying a stale negative timeLeft must not render it; \
             this is the case only the timeLeft>0 guard catches"
        );
    }

    /// `--delay` has to reach a timer that came due while sleeping and to
    /// report the slept time to OnUpdate in one tick; a bare sleep did neither
    /// (QuickRoute's `C_Timer.After(2)` init and its 0.1 s OnUpdate throttle).
    #[test]
    fn apply_delay_with_tick_fires_due_timers_and_reports_the_delay_as_elapsed() {
        let env = WowLuaEnv::new().unwrap();
        env.exec(
            r#"
            DELAY_PROBE = { fired = false, elapsed = 0 }
            C_Timer.After(0.05, function() DELAY_PROBE.fired = true end)
            local f = CreateFrame("Frame")
            f:SetScript("OnUpdate", function(_, elapsed) DELAY_PROBE.elapsed = elapsed end)
            "#,
        )
        .unwrap();

        apply_delay_with_tick(&env, Some(80));

        let (fired, elapsed): (bool, f64) = env
            .eval("return DELAY_PROBE.fired, DELAY_PROBE.elapsed")
            .unwrap();
        assert!(fired, "a timer due after the sleep must have fired");
        assert!(
            elapsed >= 0.08,
            "OnUpdate must see the slept time, got {elapsed}"
        );
    }
}
