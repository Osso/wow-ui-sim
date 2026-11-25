//! `C_LevelLink.IsActionLocked` (state-backed via Rust) and
//! `C_LevelLink.IsSpellLocked` (Lua bootstrap fallback).

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn level_link_locks_default_to_unlocked() {
    let env = env();
    let (action_locked, spell_locked): (bool, bool) = env
        .eval(
            r#"
            return C_LevelLink.IsActionLocked(1),
                   C_LevelLink.IsSpellLocked(133)
            "#,
        )
        .unwrap();

    assert!(
        !action_locked,
        "actions should default to unlocked when no lock state is configured"
    );
    assert!(
        !spell_locked,
        "spells should default to unlocked when no lock state is configured"
    );
}

#[test]
fn level_link_action_locks_read_state_locked_action_slots() {
    let env = env();
    {
        let mut state = env.state().borrow_mut();
        state.locked_action_slots.insert(2);
        state.locked_action_slots.insert(7);
    }
    let listed_first: bool = env.eval("return C_LevelLink.IsActionLocked(2)").unwrap();
    let listed_second: bool = env.eval("return C_LevelLink.IsActionLocked(7)").unwrap();
    let unlisted: bool = env.eval("return C_LevelLink.IsActionLocked(99)").unwrap();
    assert!(listed_first);
    assert!(listed_second);
    assert!(!unlisted);
}

#[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
#[test]
fn level_link_spell_locks_use_state_backed_table() {
    let env = env();
    let (spell_lock_direct, spell_lock_struct, spell_unlock_missing): (bool, bool, bool) = env
        .eval(
            r#"
            C_LevelLink._state.lockedSpells = {
                [111] = true,
                [222] = { locked = true },
            }

            return C_LevelLink.IsSpellLocked(111),
                   C_LevelLink.IsSpellLocked(222),
                   not C_LevelLink.IsSpellLocked(333)
            "#,
        )
        .unwrap();

    assert!(spell_lock_direct, "boolean spell lock entries should lock");
    assert!(spell_lock_struct, "table spell lock entries should lock");
    assert!(
        spell_unlock_missing,
        "missing spell lock entry should not lock"
    );
}

#[test]
fn level_link_action_locks_reject_non_numeric_input() {
    let env = env();
    env.state().borrow_mut().locked_action_slots.insert(17);
    let from_string: bool = env
        .eval(r#"return C_LevelLink.IsActionLocked("17")"#)
        .unwrap();
    let from_invalid: bool = env
        .eval(r#"return C_LevelLink.IsActionLocked("bad-action")"#)
        .unwrap();
    let from_table: bool = env.eval("return C_LevelLink.IsActionLocked({})").unwrap();
    assert!(
        !from_string,
        "string-shaped action IDs should not match the integer slot set"
    );
    assert!(!from_invalid, "non-numeric action IDs should be unlocked");
    assert!(!from_table, "table action IDs should be unlocked");
}

#[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
#[test]
fn level_link_spell_locks_normalize_and_track_last_query() {
    let env = env();
    let (spell_from_string, invalid_spell_unlocked, last_spell_query_ok): (bool, bool, bool) = env
        .eval(
            r#"
            C_LevelLink._state.lockedSpells = { [204] = true }
            local spellFromString = C_LevelLink.IsSpellLocked("204")
            local invalidSpellUnlocked = not C_LevelLink.IsSpellLocked({})
            return spellFromString,
                   invalidSpellUnlocked,
                   C_LevelLink._state.lastSpellQuery == nil
            "#,
        )
        .unwrap();

    assert!(
        spell_from_string,
        "numeric-string spell IDs should normalize and lock"
    );
    assert!(
        invalid_spell_unlocked,
        "invalid spell IDs should be unlocked"
    );
    assert!(
        last_spell_query_ok,
        "lastSpellQuery should be nil after invalid spell lookup"
    );
}
