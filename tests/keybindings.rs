//! `SetBinding` / `GetBindingKey` / `GetBindingAction` / override
//! bindings round-trip through `SimState.keybindings`.
//!
//! Retail WoW populates its binding registry from `Bindings.xml` at
//! load; the sim only backs the *user-set* half (what `SetBinding`
//! writes). `GetBinding(index)` / `GetNumBindings()` iterate over
//! user-set bindings, not a fixed command registry.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("WowLuaEnv init")
}

#[test]
fn default_store_reports_empty_lookups() {
    let env = env();
    let (k1, k2, action, num): (Option<String>, Option<String>, String, i64) = env
        .eval(
            r#"
            local k1, k2 = GetBindingKey("JUMP")
            local action = GetBindingAction("SPACE")
            return k1, k2, action, GetNumBindings()
            "#,
        )
        .unwrap();
    assert_eq!(k1, None);
    assert_eq!(k2, None);
    assert_eq!(action, "");
    assert!(num > 0, "binding registry should expose default actions");
}

#[test]
fn set_binding_round_trips_key_and_action() {
    let env = env();
    let (k1, action): (Option<String>, String) = env
        .eval(
            r#"
            SetBinding("SPACE", "JUMP")
            local k, _ = GetBindingKey("JUMP")
            return k, GetBindingAction("SPACE")
            "#,
        )
        .unwrap();
    assert_eq!(k1.as_deref(), Some("SPACE"));
    assert_eq!(action, "JUMP");
}

#[test]
fn two_keys_per_action_both_returned() {
    let env = env();
    let (k1, k2): (Option<String>, Option<String>) = env
        .eval(
            r#"
            SetBinding("SPACE", "JUMP")
            SetBinding("X", "JUMP")
            return GetBindingKey("JUMP")
            "#,
        )
        .unwrap();
    assert_eq!(k1.as_deref(), Some("SPACE"));
    assert_eq!(k2.as_deref(), Some("X"));
}

#[test]
fn third_key_for_same_action_evicts_oldest() {
    let env = env();
    let (k1, k2): (Option<String>, Option<String>) = env
        .eval(
            r#"
            SetBinding("SPACE", "JUMP")
            SetBinding("X", "JUMP")
            SetBinding("Y", "JUMP")
            return GetBindingKey("JUMP")
            "#,
        )
        .unwrap();
    assert_eq!(k1.as_deref(), Some("X"));
    assert_eq!(k2.as_deref(), Some("Y"));
}

#[test]
fn set_binding_with_empty_action_unbinds_key() {
    let env = env();
    let action: String = env
        .eval(
            r#"
            SetBinding("SPACE", "JUMP")
            SetBinding("SPACE", "")
            return GetBindingAction("SPACE")
            "#,
        )
        .unwrap();
    assert_eq!(action, "");
}

#[test]
fn set_binding_same_key_replaces_action() {
    let env = env();
    let action: String = env
        .eval(
            r#"
            SetBinding("SPACE", "JUMP")
            SetBinding("SPACE", "TOGGLEAUTORUN")
            return GetBindingAction("SPACE")
            "#,
        )
        .unwrap();
    assert_eq!(action, "TOGGLEAUTORUN");
}

#[test]
fn get_binding_key_for_action_returns_first_match() {
    let env = env();
    let k: Option<String> = env
        .eval(
            r#"
            SetBinding("SPACE", "JUMP")
            SetBinding("X", "JUMP")
            return GetBindingKeyForAction("JUMP")
            "#,
        )
        .unwrap();
    assert_eq!(k.as_deref(), Some("SPACE"));
}

#[test]
fn get_num_bindings_counts_user_set_entries() {
    let env = env();
    let (base_count, after_count): (i64, i64) = env
        .eval(
            r#"
            local baseCount = GetNumBindings()
            SetBinding("A", "ACTIONA")
            SetBinding("B", "ACTIONB")
            SetBinding("A", "") -- unbind
            return baseCount, GetNumBindings()
            "#,
        )
        .unwrap();
    assert_eq!(after_count, base_count + 1);
}

#[test]
fn get_binding_returns_1_indexed_pair() {
    let env = env();
    let (a1, k1, a2, k2): (String, String, String, String) = env
        .eval(
            r#"
            SetBinding("A", "ACTIONA")
            SetBinding("B", "ACTIONB")
            local act1, key1 = GetBinding(1)
            local act2, key2 = GetBinding(2)
            return act1, key1, act2, key2
            "#,
        )
        .unwrap();
    assert_eq!((a1.as_str(), k1.as_str()), ("ACTIONA", "A"));
    assert_eq!((a2.as_str(), k2.as_str()), ("ACTIONB", "B"));
}

#[test]
fn override_shadows_base_for_get_binding_action() {
    let env = env();
    let (base_then, override_then): (String, String) = env
        .eval(
            r#"
            SetBinding("SPACE", "JUMP")
            local before = GetBindingAction("SPACE")
            SetOverrideBinding(nil, false, "SPACE", "TOGGLEAUTORUN")
            local after = GetBindingAction("SPACE")
            return before, after
            "#,
        )
        .unwrap();
    assert_eq!(base_then, "JUMP");
    assert_eq!(override_then, "TOGGLEAUTORUN");
}

#[test]
fn clear_overrides_restores_base_bindings() {
    let env = env();
    let action: String = env
        .eval(
            r#"
            SetBinding("SPACE", "JUMP")
            SetOverrideBinding(nil, false, "SPACE", "TOGGLEAUTORUN")
            ClearOverrideBindings()
            return GetBindingAction("SPACE")
            "#,
        )
        .unwrap();
    assert_eq!(action, "JUMP");
}

#[test]
fn override_appears_in_get_binding_key() {
    let env = env();
    let (k1, k2): (Option<String>, Option<String>) = env
        .eval(
            r#"
            SetBinding("SPACE", "JUMP")
            SetOverrideBinding(nil, false, "X", "JUMP")
            return GetBindingKey("JUMP")
            "#,
        )
        .unwrap();
    // Overrides take precedence, so X should come first.
    assert_eq!(k1.as_deref(), Some("X"));
    assert_eq!(k2.as_deref(), Some("SPACE"));
}

#[test]
fn set_binding_click_encodes_click_action() {
    let env = env();
    let action: String = env
        .eval(
            r#"
            SetBindingClick("F1", "MyButton")
            return GetBindingAction("F1")
            "#,
        )
        .unwrap();
    assert_eq!(action, "CLICK MyButton:LeftButton");
}

#[test]
fn set_binding_spell_encodes_spell_action() {
    let env = env();
    let action: String = env
        .eval(
            r#"
            SetBindingSpell("F2", "Fireball")
            return GetBindingAction("F2")
            "#,
        )
        .unwrap();
    assert_eq!(action, "SPELL Fireball");
}

#[test]
fn set_binding_returns_false_on_empty_key() {
    let env = env();
    let ok: bool = env.eval(r#"return SetBinding("", "JUMP")"#).unwrap();
    assert!(!ok);
}

#[test]
fn save_load_bindings_are_noops_but_do_not_error() {
    let env = env();
    let ok: bool = env
        .eval(
            r#"
            SaveBindings(1)
            LoadBindings(1)
            return true
            "#,
        )
        .unwrap();
    assert!(ok);
}

#[test]
fn explicit_unbind_blocks_default_key_dispatch() {
    let env = env();
    env.exec(r#"SetBinding("F1", "")"#).unwrap();

    env.send_key_press("F1", None).unwrap();
    let has_target: bool = env.eval("return UnitExists('target')").unwrap();

    assert!(!has_target, "explicit unbind should shadow the F1 default");
}

#[test]
fn profession_book_binding_dispatches_registered_toggle() {
    let env = env();
    env.exec(
        r#"
        __profession_toggle_count = 0
        function ToggleProfessionsBook()
            __profession_toggle_count = __profession_toggle_count + 1
        end
        SetBinding("ALT-P", "TOGGLEPROFESSIONBOOK")
        "#,
    )
    .unwrap();

    env.send_key_press("ALT-P", None).unwrap();
    let count: i32 = env.eval("return __profession_toggle_count").unwrap();

    assert_eq!(
        count, 1,
        "TOGGLEPROFESSIONBOOK keybind should dispatch ToggleProfessionsBook()"
    );
}

#[test]
fn social_binding_dispatches_friends_panel_toggle() {
    let env = env();
    env.exec(
        r#"
        CreateFrame("Frame", "FriendsFrame")
        FriendsFrame:Hide()
        SetBinding("S", "TOGGLESOCIAL")
        "#,
    )
    .unwrap();

    env.send_key_press("S", None).unwrap();
    let shown: bool = env.eval("return FriendsFrame:IsShown()").unwrap();

    assert!(shown, "TOGGLESOCIAL keybind should show FriendsFrame");
}

#[test]
fn c_keybindings_index_resolves_registry_rows() {
    let env = env();
    let (has_index, action, category, key, custom_type_is_nil, tags_empty): (
        bool,
        String,
        String,
        Option<String>,
        bool,
        bool,
    ) = env
        .eval(
            r#"
            local index = C_KeyBindings.GetBindingIndex("INTERACTTARGET")
            local action, category, key = GetBinding(index)
            local tags = C_KeyBindings.GetSearchTagsForAction(action)
            return type(index) == "number",
                   action,
                   category,
                   key,
                   C_KeyBindings.GetCustomBindingType(index) == nil,
                   type(tags) == "table" and #tags == 0
            "#,
        )
        .unwrap();

    assert!(
        has_index,
        "C_KeyBindings should return concrete registry indexes"
    );
    assert_eq!(action, "INTERACTTARGET");
    assert_eq!(category, "BINDING_HEADER_OTHER");
    assert_eq!(key, None);
    assert!(custom_type_is_nil);
    assert!(tags_empty);
}

#[test]
fn modified_clicks_round_trip_through_runtime_surface() {
    // loader/addon.rs:537 loads selected Bindings XML defaults; Forever
    // Bindings_Camelot.xml:1533 supplies SELFCAST=ALT, not the bare environment.
    #[cfg(any(
        feature = "client-retail",
        feature = "client-ptr",
        feature = "client-wowforever"
    ))]
    let env = {
        let (env, loaded) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
            &crate::common::blizzard_addon_harness::blizzard_ui_dir(),
            &["Blizzard_FrameXML"],
            &[],
        );
        assert!(loaded.iter().any(|name| name == "Blizzard_FrameXML"));
        env
    };
    #[cfg(not(any(
        feature = "client-retail",
        feature = "client-ptr",
        feature = "client-wowforever"
    )))]
    let env = env();
    let (
        default_self_cast,
        default_focus_cast,
        default_auto_loot,
        updated_self_cast,
        unknown_default,
    ): (String, String, String, String, String) = env
        .eval(
            r#"
            local before = GetModifiedClick("SELFCAST")
            local focus = GetModifiedClick("FOCUSCAST")
            local autoLoot = GetModifiedClick("AUTOLOOTTOGGLE")
            SetModifiedClick("SELFCAST", "CTRL")
            return before, focus, autoLoot, GetModifiedClick("SELFCAST"),
                   GetModifiedClick("UNKNOWN_ACTION")
            "#,
        )
        .unwrap();

    assert_eq!(default_self_cast, "ALT");
    assert_eq!(default_focus_cast, "NONE");
    assert_eq!(default_auto_loot, "SHIFT");
    assert_eq!(updated_self_cast, "CTRL");
    assert_eq!(unknown_default, "NONE");
}
