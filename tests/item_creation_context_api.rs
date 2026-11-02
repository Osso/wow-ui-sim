//! API-level test: no pinned retail Blizzard consumer calls this member.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn item_creation_context_returns_id_and_empty_for_unrecognized_contexts() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local function link(id, context)
            return 'item:' .. table.concat({id, 0, 0, 0, 0, 0, 0, 0, 80, 70, 0, context}, ':')
        end
        local id1, context1 = C_Item.GetItemCreationContext(link(6948, 0))
        assert(id1 == 6948 and context1 == '')
        local id2, context2 = C_Item.GetItemCreationContext(link(211993, 999))
        assert(id2 == 211993 and context2 == '')
        assert(select('#', C_Item.GetItemCreationContext(link(6948, 0))) == 2)
        local _, normal = C_Item.GetItemCreationContext(link(6948, 1))
        local _, bonus = C_Item.GetItemCreationContext(link(6948, 191))
        local _, empty = C_Item.GetItemCreationContext(link(6948, 192))
        assert(normal == 'dungeon-normal')
        assert(bonus == 'bonus-roll-5')
        assert(empty == '')
    "#,
    )
    .unwrap();
}
