//! Character customization choices seen during this simulator session.

use crate::client_profile::{ACTIVE, ClientProfile};

pub(crate) fn install(lua: &mut rilua::Lua) -> crate::Result<()> {
    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    super::c_character_creation_customizations::install(lua)?;
    if matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    ) {
        // User-approved, not native evidence: record all currently offered choices.
        lua.exec(
            r#"
            local seen = {}
            local getAvailable = C_CharacterCreation.GetAvailableCustomizations
            function C_CharacterCreation.GetAvailableCustomizations()
                local categories = getAvailable()
                for _, category in ipairs(categories) do
                    for _, option in ipairs(category.options) do
                        for _, choice in ipairs(option.choices) do
                            if seen[choice.id] then choice.isNew = false end
                        end
                    end
                end
                return categories
            end
            function C_CharacterCreation.SaveSeenChoices()
                for _, category in ipairs(getAvailable()) do
                    for _, option in ipairs(category.options) do
                        for _, choice in ipairs(option.choices) do
                            seen[choice.id] = true
                        end
                    end
                end
            end
        "#,
        )?;
    }
    Ok(())
}
