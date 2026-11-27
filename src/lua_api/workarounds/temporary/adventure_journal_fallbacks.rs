//! Temporary adventure journal fallback helpers.
//!
//! The main `C_AdventureJournal` surface is state-backed in Rust. These Lua
//! fallbacks only preserve startup compatibility when that surface or
//! `AdventureGuideUtil` helpers are absent.

const ADVENTURE_JOURNAL_FALLBACKS_LUA: &str = r#"
if type(C_AdventureJournal) ~= "table" then
    C_AdventureJournal = __wow_namespace()
end
if rawget(C_AdventureJournal, "CanBeShown") == nil then
    function C_AdventureJournal.CanBeShown()
        return true
    end
end
if rawget(C_AdventureJournal, "UpdateSuggestions") == nil then
    function C_AdventureJournal.UpdateSuggestions(_forceUpdate)
    end
end
if rawget(C_AdventureJournal, "GetPrimaryOffset") == nil then
    function C_AdventureJournal.GetPrimaryOffset()
        return 0
    end
end
if rawget(C_AdventureJournal, "SetPrimaryOffset") == nil then
    function C_AdventureJournal.SetPrimaryOffset(_offset)
    end
end
if rawget(C_AdventureJournal, "GetNumAvailableSuggestions") == nil then
    function C_AdventureJournal.GetNumAvailableSuggestions()
        return 0
    end
end
if rawget(C_AdventureJournal, "GetSuggestions") == nil then
    function C_AdventureJournal.GetSuggestions(suggestions)
        if type(suggestions) == "table" then
            for index = #suggestions, 1, -1 do
                suggestions[index] = nil
            end
        end
    end
end
if rawget(C_AdventureJournal, "GetReward") == nil then
    function C_AdventureJournal.GetReward(_suggestionIndex)
        return nil
    end
end
if rawget(C_AdventureJournal, "ActivateEntry") == nil then
    function C_AdventureJournal.ActivateEntry(_suggestionIndex)
    end
end

if type(AdventureGuideUtil) ~= "table" then
    AdventureGuideUtil = {}
end
"#;

const ADVENTURE_GUIDE_HELPER_DEFAULTS_LUA: &str = r#"
if rawget(AdventureGuideUtil, "OpenJournalLink") == nil then
    function AdventureGuideUtil.OpenJournalLink(_journalType, _id, _difficultyID)
        if not EncounterJournal and type(EncounterJournal_LoadUI) == "function" then
            EncounterJournal_LoadUI()
        end
        if EncounterJournal then
            ShowUIPanel(EncounterJournal)
            return true
        end
        return false
    end
end
if rawget(AdventureGuideUtil, "OpenHyperLink") == nil then
    function AdventureGuideUtil.OpenHyperLink(_tag, journalType, id, difficultyID)
        if not AdventureGuideUtil.IsAvailable() then
            return false
        end
        return AdventureGuideUtil.OpenJournalLink(
            tonumber(journalType),
            tonumber(id),
            tonumber(difficultyID)
        )
    end
end
if rawget(AdventureGuideUtil, "GetCurrentJournalInstance") == nil then
    function AdventureGuideUtil.GetCurrentJournalInstance()
        return nil
    end
end
if rawget(AdventureGuideUtil, "IsInInstance") == nil then
    function AdventureGuideUtil.IsInInstance(_journalInstanceID)
        return false
    end
end
"#;

const AVAILABILITY_FALLBACK_LUA: &str = r#"
-- Retail original FrameXMLUtil owns availability before the MicroMenu consumer.
-- Other profiles retain bootstrap behavior until their ownership is proven.
if rawget(AdventureGuideUtil, "IsAvailable") == nil then
    function AdventureGuideUtil.IsAvailable()
        local kioskEnabled = Kiosk and Kiosk.IsEnabled and Kiosk.IsEnabled()
        return not kioskEnabled and C_AdventureJournal.CanBeShown()
    end
    __wow_temporary_adventureguide_availability = AdventureGuideUtil.IsAvailable
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(ADVENTURE_JOURNAL_FALLBACKS_LUA)?;
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Retail {
        lua.exec(ADVENTURE_GUIDE_HELPER_DEFAULTS_LUA)?;
    }
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Retail {
        lua.exec(AVAILABILITY_FALLBACK_LUA)?;
    }
    Ok(())
}

pub(crate) fn update_for_screen_mode(
    lua: &mut rilua::Lua,
    screen: crate::screen::ScreenKind,
) -> crate::Result<()> {
    if screen.is_glue() {
        lua.exec(AVAILABILITY_FALLBACK_LUA)?;
        return Ok(());
    }
    // Other profiles keep their bootstrap until their ownership is proven.
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Retail {
        return Ok(());
    }
    // Retail Game startup loads the original AdventureGuideUtil before consumers.
    lua.exec(ADVENTURE_GUIDE_HELPER_DEFAULTS_LUA)?;
    // Remove only our Glue fallback, never the original Game owner.
    lua.exec(
        r#"
        local fallback = rawget(_G, "__wow_temporary_adventureguide_availability")
        if fallback ~= nil and rawget(AdventureGuideUtil, "IsAvailable") == fallback then
            rawset(AdventureGuideUtil, "IsAvailable", nil)
        end
        rawset(_G, "__wow_temporary_adventureguide_availability", nil)
    "#,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;
    use crate::screen::ScreenKind;

    #[test]
    fn installs_adventure_journal_fallback_shapes_when_missing() {
        if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Retail {
            return;
        }
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            C_AdventureJournal = nil
            AdventureGuideUtil = nil
            "#,
        )
        .expect("fixture should clear adventure journal fallbacks");

        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("adventure journal fallbacks should apply");
        }

        let result: String = env
            .eval(
                r#"
                if C_AdventureJournal.CanBeShown() ~= true then return "shown" end
                C_AdventureJournal.UpdateSuggestions(true)
                if C_AdventureJournal.GetPrimaryOffset() ~= 0 then return "offset" end
                C_AdventureJournal.SetPrimaryOffset(7)
                if C_AdventureJournal.GetNumAvailableSuggestions() ~= 0 then return "count" end

                local suggestions = { "stale" }
                C_AdventureJournal.GetSuggestions(suggestions)
                if next(suggestions) ~= nil then return "suggestions" end
                if C_AdventureJournal.GetReward(1) ~= nil then return "reward" end
                C_AdventureJournal.ActivateEntry(1)

                if AdventureGuideUtil.GetCurrentJournalInstance() ~= nil then return "current" end
                if AdventureGuideUtil.IsInInstance(1) ~= false then return "instance" end
                return "ok"
                "#,
            )
            .expect("adventure journal fallback shape probe should run");

        assert_eq!(result, "ok");
        if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Retail {
            let missing: bool = env
                .eval("return rawget(AdventureGuideUtil, 'IsAvailable') == nil")
                .expect("Retail waits for the original Game helper");
            assert!(missing);
            env.exec(
                r#"
                local ok = pcall(AdventureGuideUtil.OpenHyperLink, "journal", "1", "2", "3")
                assert(not ok)
            "#,
            )
            .expect("pre-owner Game hyperlink call is explicitly unsupported");
        } else {
            env.exec(
                r#"
                assert(AdventureGuideUtil.IsAvailable() == true)
                assert(AdventureGuideUtil.OpenHyperLink("journal", "1", "2", "3") == false)
            "#,
            )
            .expect("non-Retail bootstrap behavior unchanged");
        }
    }

    // Blizzard_FrameXMLUtil (AllowLoad: Game) owns OpenHyperLink. Since
    // 7796e28aa, Retail installs helper defaults only at the Game boundary, so
    // Glue has none until a Game visit has left them behind.
    const GLUE_AVAILABILITY_PROBE: &str = r#"
        assert(AdventureGuideUtil.IsAvailable() == true)
        local open = rawget(AdventureGuideUtil, "OpenHyperLink")
        if open == nil then return "absent" end
        return tostring(open("journal", "1", "2", "3"))
    "#;

    fn expected_glue_open_result(game_visited: bool) -> &'static str {
        let is_retail =
            crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Retail;
        if is_retail && !game_visited {
            "absent"
        } else {
            "false"
        }
    }

    fn assert_glue_then_game_boundary(env: &WowLuaEnv, screen: ScreenKind, game_visited: bool) {
        env.set_screen_mode(screen);
        let open_result: String = env
            .eval(GLUE_AVAILABILITY_PROBE)
            .expect("Glue availability");
        assert_eq!(open_result, expected_glue_open_result(game_visited));
        env.set_screen_mode(ScreenKind::Game);
        let missing: bool = env
            .eval("return rawget(AdventureGuideUtil, 'IsAvailable') == nil")
            .expect("Game boundary");
        let is_retail =
            crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Retail;
        assert_eq!(missing, is_retail);
    }

    #[test]
    fn availability_fallback_follows_screen_without_removing_other_owners() {
        let env = WowLuaEnv::new().expect("lua env");
        let glue_screens = [
            ScreenKind::Login,
            ScreenKind::CharacterSelect,
            ScreenKind::CharacterCreate,
        ];
        for (index, screen) in glue_screens.into_iter().enumerate() {
            assert_glue_then_game_boundary(&env, screen, index > 0);
        }
        env.set_screen_mode(ScreenKind::Login);
        env.exec("AdventureGuideUtil.IsAvailable = function() return 'independent owner' end")
            .expect("boundary-only owner identity fixture");
        env.set_screen_mode(ScreenKind::Game);
        let owner: String = env
            .eval("return AdventureGuideUtil.IsAvailable()")
            .expect("unrelated owner survives transition");
        assert_eq!(owner, "independent owner");
    }

    #[test]
    fn preserves_state_backed_adventure_journal_members() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("adventure journal fallbacks should apply");
        }

        let suggestion_count: i32 = env
            .eval("return C_AdventureJournal.GetNumAvailableSuggestions()")
            .expect("state-backed adventure journal count should remain registered");

        assert!(suggestion_count >= 3);
    }
}
