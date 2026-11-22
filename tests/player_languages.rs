#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

const CHAT_LANGUAGE_MENU_OBSERVATION: &str = r#"
local function inspectLanguageMenu(race)
    A_Admin.SetPlayerRace(race)
    ChatFrame1:ConfigEventHandler("PLAYER_ENTERING_WORLD", false, false)
    ChatFrame1.editBox.languageID = -1
    ChatFrameMenuButton:ValidateSelectedLanguage()
    ChatFrameMenuButton:GenerateMenu()

    local languageMenu
    MenuUtil.TraverseMenu(ChatFrameMenuButton:GetMenuDescription(), function(description)
        if MenuUtil.GetElementText(description) == LANGUAGE then
            languageMenu = description
            return true
        end
    end)

    local languages = {}
    for _, description in languageMenu:EnumerateElementDescriptions() do
        table.insert(languages, MenuUtil.GetElementText(description))
    end
    return table.concat(languages, ","), ChatFrame1.defaultLanguage,
        ChatFrame1.editBox.language
end

local allianceLanguages, allianceDefault, allianceShown = inspectLanguageMenu(0)
local hordeLanguages, hordeDefault, hordeShown = inspectLanguageMenu(7)
return allianceLanguages, allianceDefault, allianceShown,
    hordeLanguages .. "|" .. hordeDefault .. "|" .. hordeShown
"#;

prefork_full_ui_case! {
fn chat_language_menu_uses_the_player_race(env: &WowLuaEnv) {
    let observed: (String, String, String, String) = env
        .eval(CHAT_LANGUAGE_MENU_OBSERVATION)
        .expect("the Blizzard chat language menu should read the player languages");

    assert_eq!(observed.0, "Common");
    assert_eq!(observed.1, "Common");
    assert_eq!(observed.2, "Common");
    assert_eq!(observed.3, "Orcish,Troll|Orcish|Orcish");
}
}
