//! Supplied-state container actions. No native stack/use-policy claim.
use wow_ui_sim::c_api::container_actions::ContainerUseRequest;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{CursorInfo, CursorItemOrigin};

const ITEM: u32 = 6948;
const CONTROL: u32 = 6256;

fn supplied_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        "A_Admin.ClearBags(); ClearCursor(); \
         A_Admin.AddBagItem(0, 3, 6948, 5); \
         A_Admin.AddBagItem(0, 4, 6256, 7)",
    )
    .unwrap();
    env
}

// Wrath public consumers: FrameXML/BankFrame.lua:15 and ChatFrame.lua:1005.
// Other cached profiles use C_Container, not these legacy dispatch functions.
fn split_actions() -> &'static [&'static str] {
    #[cfg(feature = "client-wrath")]
    {
        &["SplitContainerItem", "C_Container.SplitContainerItem"]
    }
    #[cfg(not(feature = "client-wrath"))]
    {
        &["C_Container.SplitContainerItem"]
    }
}

fn use_actions() -> &'static [&'static str] {
    #[cfg(feature = "client-wrath")]
    {
        &["UseContainerItem", "C_Container.UseContainerItem"]
    }
    #[cfg(not(feature = "client-wrath"))]
    {
        &["C_Container.UseContainerItem"]
    }
}

// Legacy first on Wrath: its RED must reach the same concrete state assertion.
fn attempt(env: &WowLuaEnv, action: &str, arguments: &str) {
    env.exec(&format!(
        "local action = {action}; assert(type(action) == 'function', 'missing {action}'); action({arguments})"
    ))
    .expect("required container entry point must be callable");
}

fn bag(env: &WowLuaEnv, slot: i32) -> Option<(u32, i32)> {
    env.state().borrow().get_bag_item(0, slot)
}

fn cursor(env: &WowLuaEnv) -> Option<(u32, i32, CursorItemOrigin)> {
    match env.state().borrow().cursor_item.clone() {
        Some(CursorInfo::Item {
            item_id,
            stack_count,
            origin,
            applied_transmog: None,
        }) => Some((item_id, stack_count, origin)),
        None => None,
        other => panic!("unexpected fixture cursor: {other:?}"),
    }
}

fn assert_split(env: &WowLuaEnv) {
    assert_eq!(bag(env, 3), Some((ITEM, 3)), "split reduces source");
    assert_eq!(bag(env, 4), Some((CONTROL, 7)));
    assert_eq!(
        cursor(env),
        Some((ITEM, 2, CursorItemOrigin::Bag { bag: 0, slot: 3 }))
    );
    let projection: (String, i64, i32, bool) = env
        .eval("local kind, id, count = GetCursorInfo(); return kind, id, count, CursorHasItem()")
        .unwrap();
    assert_eq!(projection, ("item".into(), i64::from(ITEM), 2, true));
    let remaining: i32 = env
        .eval("return C_Container.GetContainerItemInfo(0, 3).stackCount")
        .unwrap();
    assert_eq!(remaining + projection.2, 5, "conserve supplied stack");
}

#[test]
fn b1_split_moves_supplied_quantity_and_reports_cursor_count() {
    for action in split_actions() {
        let env = supplied_env();
        attempt(&env, action, "0, 3, 2");
        assert_split(&env);
        // Existing pickup can drop the split stack without duplicating it.
        env.exec("C_Container.PickupContainerItem(0, 6)").unwrap();
        assert_eq!(bag(&env, 6), Some((ITEM, 2)));
        assert_eq!(bag(&env, 3), Some((ITEM, 3)));
        assert_eq!(cursor(&env), None);
    }
}

#[test]
fn b1_invalid_amounts_are_inert_then_valid_split_succeeds() {
    for action in split_actions() {
        let env = supplied_env();
        for amount in ["0", "5", "6", "'two'", "'2'", "nil", "-1", "1.5"] {
            attempt(&env, action, &format!("0, 3, {amount}"));
            assert_eq!(bag(&env, 3), Some((ITEM, 5)), "amount {amount}");
            assert_eq!(bag(&env, 4), Some((CONTROL, 7)));
            assert_eq!(cursor(&env), None);
        }
        // Positive control prevents an all-no-op provider passing this case.
        attempt(&env, action, "0, 3, 2");
        assert_split(&env);
    }
}

#[test]
fn b1_occupied_cursor_and_missing_source_are_inert() {
    for action in split_actions() {
        let env = supplied_env();
        env.state().borrow_mut().cursor_item = Some(CursorInfo::Item {
            item_id: CONTROL,
            stack_count: 9,
            origin: CursorItemOrigin::Unknown,
            applied_transmog: None,
        });
        attempt(&env, action, "0, 3, 2");
        assert_eq!(bag(&env, 3), Some((ITEM, 5)));
        assert_eq!(cursor(&env), Some((CONTROL, 9, CursorItemOrigin::Unknown)));
        env.exec("ClearCursor()").unwrap();
        attempt(&env, action, "0, 16, 2");
        assert_eq!(bag(&env, 16), None);
        assert_eq!(cursor(&env), None);
        attempt(&env, action, "0, 3, 2");
        assert_split(&env);
    }
}

#[test]
fn b3_namespaced_and_legacy_use_record_the_same_supplied_request_model() {
    let env = supplied_env();
    env.state()
        .borrow_mut()
        .container_actions
        .usable_items
        .insert(ITEM);
    env.state().borrow_mut().cursor_item = Some(CursorInfo::Item {
        item_id: CONTROL,
        stack_count: 9,
        origin: CursorItemOrigin::Unknown,
        applied_transmog: None,
    });
    for (index, action) in use_actions().iter().enumerate() {
        attempt(&env, action, "0, 3");
        let requests = env.state().borrow().container_actions.use_requests.clone();
        assert_eq!(
            requests,
            vec![
                ContainerUseRequest {
                    bag: 0,
                    slot: 3,
                    item_id: ITEM
                };
                index + 1
            ],
            "each accepted use is an observable dispatch, not item consumption"
        );
        assert_eq!(bag(&env, 3), Some((ITEM, 5)));
        assert_eq!(bag(&env, 4), Some((CONTROL, 7)));
        assert_eq!(cursor(&env), Some((CONTROL, 9, CursorItemOrigin::Unknown)));
    }
    let other = supplied_env();
    assert!(
        other
            .state()
            .borrow()
            .container_actions
            .use_requests
            .is_empty()
    );
    assert!(
        other
            .state()
            .borrow()
            .container_actions
            .usable_items
            .is_empty()
    );
}

#[test]
fn b3_unsupplied_and_empty_use_are_inert_then_eligible_use_records() {
    for action in use_actions() {
        let env = supplied_env();
        attempt(&env, action, "0, 3");
        env.state()
            .borrow_mut()
            .container_actions
            .usable_items
            .insert(ITEM);
        attempt(&env, action, "0, 16");
        attempt(&env, action, "0, 4");
        assert!(
            env.state()
                .borrow()
                .container_actions
                .use_requests
                .is_empty()
        );
        assert_eq!(bag(&env, 3), Some((ITEM, 5)));
        assert_eq!(bag(&env, 4), Some((CONTROL, 7)));
        assert_eq!(cursor(&env), None);
        attempt(&env, action, "0, 3");
        assert_eq!(
            env.state().borrow().container_actions.use_requests,
            vec![ContainerUseRequest {
                bag: 0,
                slot: 3,
                item_id: ITEM
            }]
        );
        assert_eq!(bag(&env, 3), Some((ITEM, 5)));
        assert_eq!(cursor(&env), None);
    }
}
