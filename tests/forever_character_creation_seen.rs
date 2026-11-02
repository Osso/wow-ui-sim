#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn saved_seen_choices_mark_the_offered_character_creation_catalog() {
    let env = WowLuaEnv::new().expect("create Forever environment");
    env.exec(
        r#"
        local offered = {}
        local offeredCount = 0
        for _, category in ipairs(C_CharacterCreation.GetAvailableCustomizations()) do
            for _, option in ipairs(category.options) do
                for _, choice in ipairs(option.choices) do
                    assert(choice.isNew ~= false)
                    offered[choice.id] = true
                    offeredCount = offeredCount + 1
                end
            end
        end
        assert(offeredCount > 0, "the existing offered catalog must be populated")
        assert(select('#', C_CharacterCreation.SaveSeenChoices()) == 0)

        local seenCount = 0
        for _, category in ipairs(C_CharacterCreation.GetAvailableCustomizations()) do
            for _, option in ipairs(category.options) do
                for _, choice in ipairs(option.choices) do
                    assert(offered[choice.id], "saving seen choices must preserve the catalog")
                    assert(choice.isNew == false, "offered choices must now be marked seen")
                    seenCount = seenCount + 1
                end
            end
        end
        assert(seenCount == offeredCount)
        "#,
    )
    .expect("Forever should share the existing offered-choice seen tracker");
}
