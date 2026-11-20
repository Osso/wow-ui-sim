#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

fn open_book() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.state().borrow_mut().addon_base_paths = vec![crate::common::blizzard_ui_dir()];
    crate::common::supply_empty_forever_spellbook_outfits(&env);
    crate::common::load_game_startup_inventory(&env);
    env.apply_post_load_workarounds();
    wow_ui_sim::startup::fire_startup_events_for_screen(&env, wow_ui_sim::screen::ScreenKind::Game);
    env.exec("PlayerSpellsUtil.ToggleSpellBookFrame()")
        .expect("original spellbook demand-load");
    env
}

fn select_original_items(env: &WowLuaEnv) -> (i32, i32) {
    env.eval(
        r#"
        local book = PlayerSpellsFrame.SpellBookFrame
        assert(book:IsVisible(), 'original spellbook is not visible')
        local items, ids = {}, {}
        for _, item in book.PagedSpellsFrame:EnumerateFrames() do
            if item:IsVisible() and item.HasValidData and item:HasValidData()
                and not item.isOffSpec and item.spellBank == Enum.SpellBookSpellBank.Player then
                local id = item.spellBookItemInfo.spellID
                if type(id) == 'number' and id > 0 and id ~= ids[1] then
                    items[#items + 1], ids[#ids + 1] = item, id
                    if #items == 2 then break end
                end
            end
        end
        assert(#items == 2, 'two distinct visible original spell items are required')
        LevelLinkTestItems = items
        return ids[1], ids[2]
        "#,
    )
    .expect("two original spellbook items")
}

fn supply_lock(env: &WowLuaEnv, spell_id: i32) {
    env.exec(&format!(
        "C_LevelLink._state.lockedSpells = {{ [{spell_id}] = true }}"
    ))
    .expect("supply legacy spell lock input");
}

fn assert_overlays(env: &WowLuaEnv, locked_index: i32) {
    env.exec(&format!(
        r#"
        for index, item in ipairs(LevelLinkTestItems) do
            item:UpdateVisuals()
            local locked = index == {locked_index}
            assert(item.Button.LevelLinkLock:IsShown() == locked,
                'original spellbook lock must follow the supplied spell ID')
            assert(item.Button.LevelLinkIconCover:IsShown() == locked,
                'original spellbook icon cover must follow the supplied spell ID')
        end
        "#
    ))
    .expect("original spellbook lock indicators");
}

#[test]
fn original_spellbook_lock_visuals_follow_two_supplied_spell_ids() {
    let env = open_book();
    let (first, second) = select_original_items(&env);
    supply_lock(&env, first);
    assert_overlays(&env, 1);
    supply_lock(&env, second);
    assert_overlays(&env, 2);
}
