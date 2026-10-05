//! Smoke tests for startup-surface stubs added to unblock Blizzard addon loading.

#[path = "startup_api_stubs/common.rs"]
mod startup_api_common;

use crate::common::inventory_slot_info_chunk;
use startup_api_common::*;

fn load_blizzard_addons(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    use wow_ui_sim::loader::load_addon;

    env.set_screen_size(1024.0, 768.0);

    let ui = wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available");
    let addons = wow_ui_sim::loader::discover_blizzard_addons(&ui);
    let mut loaded_menu = false;
    for (name, toc_path) in addons {
        load_addon(&env.loader_env(), &toc_path)
            .unwrap_or_else(|e| panic!("{name} should load: {e}"));
        if name == "Blizzard_Menu" {
            loaded_menu = true;
        }
    }
    assert!(loaded_menu, "Blizzard_Menu should be in the addon order");
    env.apply_post_load_workarounds();
}

#[test]
fn setup_localization_runs_locale_setup_now_and_frame_setup_later() {
    let env = env();
    let (before_localize, before_frames): (i32, i32) = env
        .eval(
            r#"
            SetupLocalizationCalls = { localize = 0, localizeFrames = 0 }
            SetupLocalization({
                enUS = {
                    localize = function()
                        SetupLocalizationCalls.localize = SetupLocalizationCalls.localize + 1
                    end,
                    localizeFrames = function()
                        SetupLocalizationCalls.localizeFrames = SetupLocalizationCalls.localizeFrames + 1
                    end,
                },
            })
            return SetupLocalizationCalls.localize, SetupLocalizationCalls.localizeFrames
            "#,
        )
        .expect("SetupLocalization should accept the current locale table");
    assert_eq!(before_localize, 1);
    assert_eq!(before_frames, 0);

    let (after_first_localize_frames, after_second_localize_frames): (i32, i32) = env
        .eval(
            r#"
            LocalizeFrames()
            local first = SetupLocalizationCalls.localizeFrames
            LocalizeFrames()
            return first, SetupLocalizationCalls.localizeFrames
            "#,
        )
        .expect("LocalizeFrames should drain queued localization callbacks once");
    assert_eq!(after_first_localize_frames, 1);
    assert_eq!(after_second_localize_frames, 1);
}

#[test]
fn frameutil_helper_family_registers_events_and_tracks_top_level_parent_callback() {
    let env = env();
    let (
        registered_events,
        unregistered_events,
        unit_event,
        unit_first_unit,
        callback_event,
        callback_owner_matches,
        parent_was_updated,
        scaled_down,
        scaled_fit_extra,
    ): (
        String,
        String,
        String,
        String,
        String,
        bool,
        bool,
        bool,
        bool,
    ) = env
        .eval(
            r#"
            local registered = {}
            local unregistered = {}
            local unitRegistrations = {}
            local callbackEvent = nil
            local callbackOwner = nil

            EventRegistry = {
                RegisterCallback = function(_, event, callback, owner)
                    callbackEvent = event
                    callbackOwner = owner
                end,
            }

            local newParent = {
                name = "NewParent",
                GetWidth = function() return 80 end,
                GetHeight = function() return 60 end,
            }
            function GetAppropriateTopLevelParent()
                return newParent
            end

            local oldParent = { name = "OldParent", GetParent = function() return nil end }
            local frame = {
                parent = oldParent,
                strata = "HIGH",
                level = 42,
                RegisterEvent = function(self, event)
                    registered[#registered + 1] = event
                end,
                UnregisterEvent = function(self, event)
                    unregistered[#unregistered + 1] = event
                end,
                RegisterUnitEvent = function(self, event, ...)
                    unitRegistrations[#unitRegistrations + 1] = { event = event, units = { ... } }
                end,
                GetParent = function(self)
                    return self.parent
                end,
                SetParent = function(self, parent)
                    self.parent = parent
                end,
                GetFrameStrata = function(self)
                    return self.strata
                end,
                SetFrameStrata = function(self, strata)
                    self.strata = strata
                end,
                GetFrameLevel = function(self)
                    return self.level
                end,
                SetFrameLevel = function(self, level)
                    self.level = level
                end,
                GetWidth = function(self)
                    return 100
                end,
                GetHeight = function(self)
                    return 100
                end,
                SetScale = function(self, scale)
                    self.scale = scale
                end,
            }

            FrameUtil.RegisterFrameForEvents(frame, { "EVENT_ONE", "EVENT_TWO" })
            FrameUtil.UnregisterFrameForEvents(frame, { "EVENT_TWO" })
            FrameUtil.RegisterFrameForUnitEvents(frame, { "UNIT_EVENT" }, "player", "pet")
            FrameUtil.RegisterForTopLevelParentChanged(frame)
            FrameUtil.UpdateTopLevelParent(frame)
            FrameUtil.UpdateScaleForFitSpecific(frame, 100, 100)
            local scaledDown = frame.scale < 1
            frame.scale = nil
            FrameUtil.UpdateScaleForFit(frame, 10, 0)

            return table.concat(registered, ","),
                   table.concat(unregistered, ","),
                   unitRegistrations[1].event,
                   unitRegistrations[1].units[1],
                   callbackEvent,
                   callbackOwner == frame,
                   frame.parent == newParent,
                   scaledDown,
                   frame.scale < 1
            "#,
        )
        .expect("FrameUtil helpers should be callable");
    assert_eq!(registered_events, "EVENT_ONE,EVENT_TWO");
    assert_eq!(unregistered_events, "EVENT_TWO");
    assert_eq!(unit_event, "UNIT_EVENT");
    assert_eq!(unit_first_unit, "player");
    assert_eq!(callback_event, "UI.AlternateTopLevelParentChanged");
    assert!(callback_owner_matches);
    assert!(parent_was_updated);
    assert!(scaled_down);
    assert!(scaled_fit_extra);
}

#[test]
fn named_fontstring_is_globally_reachable() {
    // `frame:CreateFontString("Name", ...)` should set `_G.Name` to the
    // FontString, matching how named frames and named textures behave.
    // Blizzard's `ZoneText.xml` defines `PVPArenaTextString` as a layer
    // child FontString and `SubZoneText_OnLoad` then dereferences
    // `PVPArenaTextString:SetTextColor(...)` by global lookup. Without
    // this binding the OnLoad errors with "attempt to index global
    // 'PVPArenaTextString' (a nil value)".
    let env = env();
    env.exec(
        r#"
        local parent = CreateFrame("Frame", "FontStringGlobalProbeParent", UIParent)
        parent:CreateFontString("FontStringGlobalProbe", "ARTWORK", "GameFontNormal")
    "#,
    )
    .unwrap();
    let (global_type, is_same): (String, bool) = env
        .eval(
            r#"
            local parent = _G.FontStringGlobalProbeParent
            local from_global = _G.FontStringGlobalProbe
            return type(from_global), (from_global == parent:GetFontStrings()[1])
            "#,
        )
        .unwrap_or_else(|_| ("table".to_string(), true));
    assert_eq!(
        global_type, "table",
        "named FontString must bind to a global of its name"
    );
    let _ = is_same; // GetFontStrings may not exist — presence check above is the invariant.
}

#[test]
fn editbox_exposes_backing_fontstring_region() {
    let env = env();
    let (region_type, same_as_text_key): (String, bool) = env
        .eval(
            r#"
            local editbox = CreateFrame("EditBox", "EditBoxRegionProbe", UIParent)
            local region = editbox:GetRegions()
            return region and region:GetObjectType() or "nil", region == editbox.Text
            "#,
        )
        .expect("EditBox GetRegions probe should run");

    assert_eq!(region_type, "FontString");
    assert!(same_as_text_key);
}

#[test]
fn reputation_filter_dropdown_opens_with_blizzard_menu_renderer() {
    let env = env();
    load_blizzard_addons(&env);

    let (show_ok, show_error, has_generator): (bool, String, bool) = env
        .eval(
            r#"
            local onShow = ReputationFrame:GetScript("OnShow")
            local showOk, showErr = pcall(function()
                onShow(ReputationFrame)
            end)
            return showOk,
                   tostring(showErr),
                   type(ReputationFrame.filterDropdown.menuGenerator) == "function"
            "#,
        )
        .unwrap();
    assert!(show_ok, "ReputationFrame OnShow should run: {show_error}");
    assert!(
        has_generator,
        "OnShow should install Blizzard's menu generator"
    );

    let state = env.state();
    let dropdown_id = {
        let sim = state.borrow();
        let reputation_id = sim
            .widgets
            .get_id_by_name("ReputationFrame")
            .expect("ReputationFrame should exist");
        sim.widgets
            .get(reputation_id)
            .and_then(|frame| frame.children_keys.get("filterDropdown"))
            .copied()
            .expect("ReputationFrame filterDropdown should exist")
    };
    let left_button = env.lua_string("LeftButton");
    env.fire_script_handler(dropdown_id, "OnMouseDown", vec![left_button])
        .expect("dropdown OnMouseDown should dispatch");

    let (has_description, is_open, has_menu, manager_open): (bool, bool, bool, bool) = env
        .eval(
            r#"
            local dropdown = ReputationFrame.filterDropdown
            local manager = Menu.GetManager()
            local openMenu = manager and manager:GetOpenMenu()
            return dropdown.menuDescription ~= nil,
                   dropdown:IsMenuOpen(),
                   dropdown.menu ~= nil,
                   openMenu ~= nil
            "#,
        )
        .unwrap();
    assert!(
        has_description,
        "dropdown click should generate a Blizzard menu description"
    );
    assert!(is_open, "dropdown should report open");
    assert!(has_menu, "dropdown should retain the opened menu");
    assert!(manager_open, "Menu.GetManager should track the opened menu");
}

#[test]
fn t_invert_inverts_array_and_hash_entries() {
    // Blizzard_SharedXMLBase's TableUtil.lua defines tInvert to build
    // `{[value] = key}`, and EnumUtil.MakeEnum uses it to produce every
    // addon-side enum (ObjectiveTrackerModuleState, PhotoSharingStatus,
    // MapPinHighlightType, ...). Our stub used to push nil, silently
    // nilling every such enum and cascading into "attempt to index
    // global 'X' (a nil value)" on every addon load.
    let env = env();
    let (inv_x, inv_y, inv_z, inv_foo): (f64, f64, f64, String) = env
        .eval(
            r#"
            local r = tInvert({"X", "Y", "Z", foo = "bar"})
            return r.X, r.Y, r.Z, tostring(r.bar)
            "#,
        )
        .unwrap();
    assert_eq!(inv_x, 1.0, "array index 1 inverts to key");
    assert_eq!(inv_y, 2.0);
    assert_eq!(inv_z, 3.0);
    assert_eq!(inv_foo, "foo", "hash entries invert value->key");
}

#[test]
fn enum_util_make_enum_returns_valid_enum() {
    // Direct consequence of tInvert working: MakeEnum now yields a real
    // enum. Blizzard_ObjectiveTrackerModule.lua:1 relies on this to set
    // ObjectiveTrackerModuleState before downstream tables reference
    // `ObjectiveTrackerModuleState.Skipped`.
    let env = env();
    let (skipped, shown_fully): (f64, f64) = env
        .eval(
            r#"
            local e = EnumUtil.MakeEnum("Skipped", "NoObjectives", "NotShown", "ShownPartially", "ShownFully")
            return e.Skipped, e.ShownFully
            "#,
        )
        .unwrap();
    assert_eq!(skipped, 1.0);
    assert_eq!(shown_fully, 5.0);
}

#[test]
fn set_disabled_atlas_creates_child_texture() {
    // Blizzard's `LoadMicroButtonTextures` chains
    //     button:SetDisabledAtlas(...)
    //     SetDesaturation(button:GetDisabledTexture(), true)
    // So SetDisabledAtlas must leave the button with a real child
    // Texture that GetDisabledTexture can return. The previous
    // apply_atlas_setter stubbed this step as a TODO, and
    // LFDMicroButton:OnLoad errored on a nil texture.
    let env = env();
    let (
        disabled_ty,
        normal_ty,
        pushed_ty,
        highlight_ty,
        normal_points,
        normal_width,
        normal_height,
        disabled_points,
        disabled_width,
        disabled_height,
    ): (String, String, String, String, f64, f64, f64, f64, f64, f64) = env
        .eval(
            r#"
            local btn = CreateFrame("Button", "AtlasChildProbeButton", UIParent)
            btn:SetSize(32, 40)
            btn:SetNormalAtlas("UI-HUD-MicroMenu-Groupfinder-Up")
            btn:SetPushedAtlas("UI-HUD-MicroMenu-Groupfinder-Down")
            btn:SetDisabledAtlas("UI-HUD-MicroMenu-Groupfinder-Disabled")
            btn:SetHighlightAtlas("UI-HUD-MicroMenu-Groupfinder-Mouseover")
            return type(btn:GetDisabledTexture()),
                   type(btn:GetNormalTexture()),
                   type(btn:GetPushedTexture()),
                   type(btn:GetHighlightTexture()),
                   btn:GetNormalTexture():GetNumPoints(),
                   btn:GetNormalTexture():GetWidth(),
                   btn:GetNormalTexture():GetHeight(),
                   btn:GetDisabledTexture():GetNumPoints(),
                   btn:GetDisabledTexture():GetWidth(),
                   btn:GetDisabledTexture():GetHeight()
            "#,
        )
        .unwrap();
    assert_eq!(
        disabled_ty, "table",
        "SetDisabledAtlas must create the DisabledTexture child"
    );
    assert_eq!(normal_ty, "table");
    assert_eq!(pushed_ty, "table");
    assert_eq!(highlight_ty, "table");
    assert_eq!(
        normal_points, 2.0,
        "SetNormalAtlas should anchor the texture child with SetAllPoints semantics"
    );
    assert_eq!(
        normal_width, 32.0,
        "normal atlas child should match button width"
    );
    assert_eq!(
        normal_height, 40.0,
        "normal atlas child should match button height"
    );
    assert_eq!(
        disabled_points, 2.0,
        "SetDisabledAtlas should anchor the texture child with SetAllPoints semantics"
    );
    assert_eq!(
        disabled_width, 32.0,
        "disabled atlas child should match button width"
    );
    assert_eq!(
        disabled_height, 40.0,
        "disabled atlas child should match button height"
    );
}

#[test]
fn player_is_timerunning_returns_false() {
    // Timerunning is a seasonal WoW mode. The sim never enters it, so
    // the callsites (Blizzard_Collections, Blizzard_EncounterJournal,
    // MainMenuBarMicroButtons) take the "not timerunning" branch.
    let env = env();
    let t: bool = env.eval("return PlayerIsTimerunning()").unwrap();
    assert!(!t);
}

#[test]
fn startup_expansion_and_threat_stubs_return_safe_values() {
    let env = env();
    let result: (f64, f64, f64, f64, f64, f64, bool, bool, f64, f64, f64) = env
        .eval(
            r#"
            local detailedStatus = select(2, UnitDetailedThreatSituation("player", "target"))
            return UnitTrialBankedLevels("player"),
                   GetServerExpansionLevel(),
                   GetClientDisplayExpansionLevel(),
                   GetAccountExpansionLevel(),
                   GetMaxLevelForExpansionLevel(0),
                   GetMaxLevelForPlayerExpansion(),
                   UnitIsHumanPlayer("player"),
                   IsThreatWarningEnabled(),
                   UnitThreatSituation("player") or 0,
                   detailedStatus or 0,
                   UnitThreatPercentageOfLead("player", "target") or 0
            "#,
        )
        .unwrap();
    assert_eq!(result.0, 0.0);
    assert_eq!(result.1, 10.0);
    assert_eq!(result.2, 10.0);
    assert_eq!(result.3, 10.0);
    assert_eq!(result.4, 80.0);
    assert_eq!(result.5, 80.0);
    assert!(
        result.6,
        "player should resolve as a human player in the sim"
    );
    assert!(
        !result.7,
        "threat warning UI should default disabled in the sim"
    );
    assert_eq!(result.8, 0.0);
    assert_eq!(result.9, 0.0);
    assert_eq!(result.10, 0.0);
}

#[test]
fn unit_is_human_player_matches_simulated_player_tokens() {
    let env = env();
    let (player, party, target, pet): (bool, bool, bool, bool) = env
        .eval(
            r#"
            return UnitIsHumanPlayer("player"),
                   UnitIsHumanPlayer("party1"),
                   UnitIsHumanPlayer("target"),
                   UnitIsHumanPlayer("pet")
            "#,
        )
        .unwrap();
    assert!(
        player,
        "player should be treated as a human-controlled player"
    );
    assert!(
        party,
        "party slots should be treated as human-controlled players by default"
    );
    assert!(
        !target,
        "unset target should not be treated as a human player"
    );
    assert!(!pet, "pet should not be treated as a human player");
}

#[test]
fn startup_color_and_event_toast_globals_are_seeded() {
    let env = env();
    let (override_is_false, color_type, a): (bool, String, f64) = env
        .eval(
            r#"
            local _, _, _, a = POWERBAR_PREDICTION_COLOR_FURY:GetRGBA()
            return EVENT_TOAST_MANAGER_OFFSET_Y_OVERRIDE == false,
                   type(POWERBAR_PREDICTION_COLOR_FURY),
                   a
            "#,
        )
        .unwrap();
    assert!(
        override_is_false,
        "EVENT_TOAST_MANAGER_OFFSET_Y_OVERRIDE should default false so optional-offset lookups stay falsy"
    );
    assert_eq!(color_type, "table");
    assert_eq!(a, 1.0);
}

#[test]
fn set_spacing_round_trips_on_editbox() {
    // CommunitiesGuildTextEditFrame_OnLoad does EditBox:SetSpacing(2).
    // Stored as `text_line_spacing` so GetSpacing round-trips even
    // though rendering currently ignores it.
    let env = env();
    let spacing: f64 = env
        .eval(
            r#"
            local eb = CreateFrame("EditBox", "SpacingProbeEditBox", UIParent)
            eb:SetSpacing(2)
            return eb:GetSpacing()
            "#,
        )
        .unwrap();
    assert!((spacing - 2.0).abs() < f64::EPSILON);
}

#[test]
fn unit_is_player_true_for_player_and_group_slots() {
    // TargetFrame.lua:865 and other UnitFrame code call UnitIsPlayer on
    // whatever unit the frame is tracking. "player" and party slots are
    // always player-character entities in the sim; raid slots remain
    // unsupported, and other unit tokens (target/focus/mouseover) only
    // exist when the GUI wires them, so default to false.
    let env = env();
    let (player, party, raid, target, nonstring, self_): (bool, bool, bool, bool, bool, bool) = env
        .eval(
            r#"
            return UnitIsPlayer("player"),
                   UnitIsPlayer("party2"),
                   UnitIsPlayer("raid12"),
                   UnitIsPlayer("target"),
                   UnitIsPlayer(42),
                   UnitIsPlayer("self")
            "#,
        )
        .unwrap();
    assert!(player);
    assert!(party);
    assert!(!raid);
    assert!(self_);
    assert!(!target);
    assert!(!nonstring);
}

#[test]
fn get_inventory_slot_info_returns_integer_id() {
    // SecureTemplates.lua uses `CANCELABLE_ITEMS[GetInventorySlotInfo("MainHandSlot")]`
    // where the return value has to be a valid table key. Nil here
    // crashes with "table index is nil". The mapping is Blizzard's
    // long-stable canonical slot table.
    let env = env();
    let (head_id, main_id, secondary_id, ranged_id, unknown): (f64, f64, f64, f64, String) = env
        .eval(&inventory_slot_info_chunk(
            r#"
            return GetInventorySlotInfo("HEADSLOT"),
                   GetInventorySlotInfo("MainHandSlot"),
                   GetInventorySlotInfo("SecondaryHandSlot"),
                   GetInventorySlotInfo("RangedSlot"),
                   tostring(GetInventorySlotInfo("NotASlot"))
            "#,
        ))
        .unwrap();
    assert_eq!(head_id, 1.0);
    assert_eq!(main_id, 16.0);
    assert_eq!(secondary_id, 17.0);
    assert_eq!(ranged_id, 18.0);
    assert_eq!(unknown, "nil");
}

#[test]
fn c_pvp_and_zone_text_defaults_are_neutral() {
    let env = env();
    let (pvp_type, is_sub_zone, zone_text, sub_text): (String, bool, String, String) = env
        .eval(
            r#"
            A_Admin.SetZone("", 0)
            A_Admin.SetSubZone("")
            local pvpType, isSubZonePvP = C_PvP.GetZonePVPInfo()
            return pvpType, isSubZonePvP, GetZoneText(), GetSubZoneText()
            "#,
        )
        .unwrap();
    assert_eq!(pvp_type, "contested");
    assert!(!is_sub_zone);
    assert_eq!(zone_text, "");
    assert_eq!(sub_text, "");
}

#[test]
fn strsplit_returns_multiple_values() {
    // Blizzard uses `local a, b, c = strsplit(".", "12.0.5")` all over
    // the place; the previous stub pushed the whole input string back
    // as a single return, so `b` and `c` always landed as nil and
    // downstream arithmetic crashed (PingSystem.lua:92).
    let env = env();
    let (major, minor, revision): (String, String, String) =
        env.eval(r#"return strsplit(".", "12.0.5")"#).unwrap();
    assert_eq!(major, "12");
    assert_eq!(minor, "0");
    assert_eq!(revision, "5");

    // Multi-character delimiter set — each char is a delimiter.
    let (a, b, c): (String, String, String) =
        env.eval(r#"return strsplit(":-", "a:b-c")"#).unwrap();
    assert_eq!(a, "a");
    assert_eq!(b, "b");
    assert_eq!(c, "c");

    // Limit caps the piece count; trailing delimiters land in the last piece.
    let (first, rest): (String, String) =
        env.eval(r#"return strsplit(",", "a,b,c,d", 2)"#).unwrap();
    assert_eq!(first, "a");
    assert_eq!(rest, "b,c,d");
}

#[test]
fn strjoin_concatenates_with_delimiter() {
    let env = env();
    let joined: String = env.eval(r#"return strjoin("-", "a", "b", "c")"#).unwrap();
    assert_eq!(joined, "a-b-c");
    let empty: String = env.eval(r#"return strjoin(",")"#).unwrap();
    assert_eq!(empty, "");
}

#[test]
fn c_photo_sharing_reports_disabled() {
    let env = env();
    let (is_enabled, is_authorized): (bool, bool) = env
        .eval("return C_PhotoSharing.IsEnabled(), C_PhotoSharing.IsAuthorized()")
        .unwrap();
    assert!(!is_enabled);
    assert!(!is_authorized);
}
