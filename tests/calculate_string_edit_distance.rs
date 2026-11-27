use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn edit_distance_uses_both_string_inputs() {
    let env = WowLuaEnv::new().expect("lua env should initialize");
    let distances: (i64, i64, i64) = env
        .eval(
            r#"
            return CalculateStringEditDistance("kitten", "sitting"),
                   CalculateStringEditDistance("", "wow"),
                   CalculateStringEditDistance("same", "same")
            "#,
        )
        .expect("edit distance inputs should be evaluated");
    assert_eq!(distances, (3, 3, 0));
}
