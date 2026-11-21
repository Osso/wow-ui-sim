#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

fn group_buff_consumer_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create Lua environment");
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
    crate::common::load_profile_framexml(&env);
    let addons = wow_ui_sim::loader::discover_blizzard_addons(&ui);
    assert!(
        addons
            .iter()
            .any(|(name, _)| name == "Blizzard_CooldownViewer")
    );
    for (name, toc) in addons {
        wow_ui_sim::loader::load_addon(&env.loader_env(), &toc)
            .unwrap_or_else(|error| panic!("{name}: {error}"));
    }
    env
}

#[test]
fn original_group_buff_filter_partitions_supplied_items_by_default_hidden_flag() {
    let env = group_buff_consumer_env();
    env.exec(
        r#"
        local source = debug.getinfo(GroupBuffFilterMixin.Refresh, 'S').source:gsub('\\\\', '/')
        assert(source:sub(-#'Blizzard_CooldownViewer/GroupBuffFilter.lua') == 'Blizzard_CooldownViewer/GroupBuffFilter.lua',
            'must execute the cached Blizzard group-buff filter')
        A_Admin.SetGroupBuffItems({
            { spellID = 1459, displaySpellID = 1459, name = 'Arcane Intellect', iconID = 135932, flags = 0, isKnown = true },
            { spellID = 21562, displaySpellID = 21562, name = 'Power Word: Fortitude', iconID = 135987, flags = Enum.GroupBuffItemFlags.HideByDefault, isKnown = true },
        })
        local filter = CreateFrame('Frame', nil, UIParent, 'GroupBuffFilterTemplate')
        filter:Refresh()
        local function ids(section)
            local result = {}
            for item in section.itemPool:EnumerateActive() do
                table.insert(result, item:GetGroupBuffItem().spellID)
            end
            return result
        end
        local shown, hidden = ids(filter.shownSection), ids(filter.hiddenSection)
        assert(#shown == 1 and shown[1] == 1459, 'shown section must contain Arcane Intellect')
        assert(#hidden == 1 and hidden[1] == 21562, 'hidden section must contain Power Word: Fortitude')
        "#,
    )
    .expect("original Blizzard GroupBuffFilter consumer must partition supplied records");
}
