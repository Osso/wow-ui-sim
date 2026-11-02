//! API-level coverage: no pinned retail Blizzard consumer calls GetItemUniqueness.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn item_uniqueness_distinguishes_missing_and_inferred_success_pairs() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        assert(select('#', C_Item.GetItemUniqueness(25)) == 0)
        assert(select('#', C_Item.GetItemUniqueness(999999)) == 0)
        local count, a, b = (function(...) return select('#', ...), ... end)(C_Item.GetItemUniqueness(5509))
        assert(count == 2 and a == 0 and b == 0)
        local flaggedCount, x, y = (function(...) return select('#', ...), ... end)(C_Item.GetItemUniqueness(776))
        assert(flaggedCount == 2 and x == 0 and y == 0)
        assert(not pcall(C_Item.GetItemUniqueness))
    "#).unwrap();
}
