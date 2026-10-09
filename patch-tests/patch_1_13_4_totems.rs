//! Existing current Era host-slot lifecycle; not native 1.13.4 signatures/defaults.
use wow_ui_sim::lua_api::{WowLuaEnv, globals::real::totems::Totem};

#[test]
fn existing_era_totem_info_tracks_slot_replacement_expiry_and_removal() {
    let env = WowLuaEnv::new().expect("headless Era environment");
    let now = env.state().borrow().start_time.elapsed().as_secs_f64();
    env.state().borrow_mut().totem_slots[0] = Some(Totem {
        name: "Healing Stream".into(),
        start_time: now,
        duration: 3600.0,
        icon: 135127,
    });
    env.state().borrow_mut().totem_slots[1] = Some(Totem {
        name: "Expired".into(),
        start_time: -20.0,
        duration: 1.0,
        icon: 135128,
    });
    let first: (bool, String, f64, f64, i32) = env
        .eval("return GetTotemInfo(1)")
        .expect("read active host slot");
    assert_eq!(first, (true, "Healing Stream".into(), now, 3600.0, 135127));
    env.exec(
        r#"
        local active, name, start, duration, icon = GetTotemInfo(2)
        assert(active == false and name == nil and start == 0 and duration == 0 and icon == nil)
        assert(GetTotemInfo(0) == false)
        assert(GetTotemInfo(99) == false)
        "#,
    )
    .expect("expired and out-of-range slots are inactive");
    env.state().borrow_mut().totem_slots[0]
        .as_mut()
        .unwrap()
        .name = "Replacement".into();
    let replacement: String = env
        .eval("local active, name = GetTotemInfo(1); assert(active); return name")
        .expect("read updated host state");
    assert_eq!(replacement, "Replacement");
    env.state().borrow_mut().totem_slots[0] = None;
    let active: bool = env.eval("return GetTotemInfo(1)").expect("read removal");
    assert!(!active);
}
