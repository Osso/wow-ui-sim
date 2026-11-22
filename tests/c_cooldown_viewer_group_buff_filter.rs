#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

fn group_buff_consumer_env() -> WowLuaEnv {
    let ui = crate::common::blizzard_addon_harness::blizzard_ui_dir();
    let env = crate::common::blizzard_addon_harness::new_blizzard_addon_env(&ui);
    crate::common::load_profile_framexml(&env);
    let loaded = crate::common::blizzard_addon_harness::load_blizzard_addon_closure_into_env(
        &env,
        &ui,
        &["Blizzard_CooldownViewer"],
        &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_CooldownViewer"));
    env
}

const GROUP_BUFF_FILTER_FIXTURE: &str = r#"
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
"#;

#[test]
fn original_group_buff_filter_partitions_supplied_items_by_default_hidden_flag() {
    let env = group_buff_consumer_env();
    env.exec(GROUP_BUFF_FILTER_FIXTURE)
        .expect("original Blizzard GroupBuffFilter consumer must partition supplied records");
    #[cfg(feature = "client-retail")]
    assert!(
        env.eval::<bool>("return C_CooldownViewer.GetGroupBuffItems()[1].displaySpellID == nil")
            .unwrap()
    );
    #[cfg(feature = "client-wowforever")]
    assert_eq!(
        env.eval::<i32>("return C_CooldownViewer.GetGroupBuffItems()[1].displaySpellID")
            .unwrap(),
        1459
    );
}
