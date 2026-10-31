use std::fmt::Write;

use rilua::Val;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;

/// Serialize one owned global without substituting or omitting unsupported data.
/// On failure, leave any previously serialized assignments intact.
pub(super) fn serialize_assignment(
    out: &mut String,
    state: &LuaState,
    name: &str,
    value: Val,
) -> crate::Result<()> {
    if !valid_global_name(name) {
        return Err(unsupported("invalid SavedVariables global name"));
    }
    let start = out.len();
    let _ = write!(out, "{name} = ");
    if let Err(error) = serialize_value(out, state, value, 0, &mut Vec::new()) {
        out.truncate(start);
        return Err(crate::Error::Other(format!(
            "SavedVariables {name}: {error}"
        )));
    }
    out.push('\n');
    Ok(())
}

fn valid_global_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    matches!(bytes.next(), Some(b'a'..=b'z' | b'A'..=b'Z' | b'_'))
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        && !matches!(
            name,
            "and"
                | "break"
                | "do"
                | "else"
                | "elseif"
                | "end"
                | "false"
                | "for"
                | "function"
                | "if"
                | "in"
                | "local"
                | "nil"
                | "not"
                | "or"
                | "repeat"
                | "return"
                | "then"
                | "true"
                | "until"
                | "while"
        )
}

fn unsupported(message: &str) -> crate::Error {
    crate::Error::Other(message.to_string())
}

fn serialize_value(
    out: &mut String,
    state: &LuaState,
    value: Val,
    depth: usize,
    seen: &mut Vec<GcRef<Table>>,
) -> crate::Result<()> {
    match value {
        Val::Nil => out.push_str("nil"),
        Val::Bool(value) => out.push_str(if value { "true" } else { "false" }),
        Val::Num(value) => write_number(out, value)?,
        Val::Str(value) => {
            let value = state
                .gc
                .string_arena
                .get(value)
                .ok_or_else(|| unsupported("invalid string reference"))?;
            write_bytes(out, value.data());
        }
        Val::Table(table) => serialize_table(out, state, table, depth, seen)?,
        _ => {
            return Err(unsupported(&format!(
                "unsupported value type: {}",
                value.type_name()
            )));
        }
    }
    Ok(())
}

fn write_number(out: &mut String, value: f64) -> crate::Result<()> {
    if !value.is_finite() {
        return Err(unsupported("non-finite number"));
    }
    // Rust's shortest decimal representation round-trips finite f64 values.
    // Avoid converting integral numbers to i64, which also loses signed zero.
    let _ = write!(out, "{value}");
    Ok(())
}

fn write_bytes(out: &mut String, value: &[u8]) {
    out.push('"');
    for &byte in value {
        match byte {
            b'"' => out.push_str("\\\""),
            b'\\' => out.push_str("\\\\"),
            b'\n' => out.push_str("\\n"),
            b'\r' => out.push_str("\\r"),
            b'\t' => out.push_str("\\t"),
            b' '..=b'~' => out.push(char::from(byte)),
            // Lua decimal escapes consume up to three digits. Always emit all
            // three so a following ASCII digit cannot become part of the byte.
            _ => {
                let _ = write!(out, "\\{byte:03}");
            }
        }
    }
    out.push('"');
}

fn serialize_table(
    out: &mut String,
    state: &LuaState,
    table_ref: GcRef<Table>,
    depth: usize,
    seen: &mut Vec<GcRef<Table>>,
) -> crate::Result<()> {
    if seen.contains(&table_ref) {
        return Err(unsupported("cyclic table"));
    }
    // Bound recursion before overflowing Rust's stack or Lua's parser limit.
    if depth >= 128 {
        return Err(unsupported("table nesting exceeds supported depth (128)"));
    }
    seen.push(table_ref);

    out.push_str("{\n");
    let indent = "\t".repeat(depth + 1);
    let table = state
        .gc
        .tables
        .get(table_ref)
        .ok_or_else(|| unsupported("invalid table reference"))?;
    let array_values = table.array_slice();
    let array_len = array_values
        .iter()
        .take_while(|value| !matches!(value, Val::Nil))
        .count();

    for (index, value) in array_values.iter().take(array_len).copied().enumerate() {
        let _ = write!(out, "{indent}");
        serialize_value(out, state, value, depth + 1, seen)?;
        let _ = writeln!(out, ", -- [{}]", index + 1);
    }

    for (key, value) in collect_remaining_entries(state, table, array_len)? {
        let _ = write!(out, "{indent}[");
        write_key(out, &key)?;
        out.push_str("] = ");
        serialize_value(out, state, value, depth + 1, seen)?;
        out.push_str(",\n");
    }

    let _ = write!(out, "{}}}", "\t".repeat(depth));
    seen.pop();
    Ok(())
}

fn collect_remaining_entries(
    state: &LuaState,
    table: &Table,
    array_len: usize,
) -> crate::Result<Vec<(SavedVarKey, Val)>> {
    let mut entries = Vec::new();
    // The dense prefix ends at the first hole, but later occupied array slots
    // remain saved data and need explicit numeric keys.
    for (index, &value) in table.array_slice().iter().enumerate().skip(array_len) {
        if !matches!(value, Val::Nil) {
            entries.push((SavedVarKey::Number((index + 1) as f64), value));
        }
    }
    for (key, value) in table.hash_entries() {
        // Hash tombstones retain their key after deletion. They are not data.
        if !matches!(value, Val::Nil) {
            entries.push((classify_key(state, key)?, value));
        }
    }
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(entries)
}

fn classify_key(state: &LuaState, key: Val) -> crate::Result<SavedVarKey> {
    match key {
        Val::Str(key) => {
            let string = state
                .gc
                .string_arena
                .get(key)
                .ok_or_else(|| unsupported("invalid string key reference"))?;
            Ok(SavedVarKey::String(string.data().to_vec()))
        }
        Val::Num(number) if number.is_finite() => Ok(SavedVarKey::Number(number)),
        Val::Num(_) => Err(unsupported("non-finite numeric key")),
        Val::Bool(value) => Ok(SavedVarKey::Bool(value)),
        _ => Err(unsupported(&format!(
            "unsupported table key type: {}",
            key.type_name()
        ))),
    }
}

fn write_key(out: &mut String, key: &SavedVarKey) -> crate::Result<()> {
    match key {
        SavedVarKey::String(key) => write_bytes(out, key),
        SavedVarKey::Number(number) => write_number(out, *number)?,
        SavedVarKey::Bool(value) => out.push_str(if *value { "true" } else { "false" }),
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
enum SavedVarKey {
    String(Vec<u8>),
    Number(f64),
    Bool(bool),
}

impl SavedVarKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering::{Greater, Less};
        match (self, other) {
            (Self::Number(left), Self::Number(right)) => left.total_cmp(right),
            (Self::Bool(left), Self::Bool(right)) => left.cmp(right),
            (Self::String(left), Self::String(right)) => left.cmp(right),
            (Self::Number(_), _) | (Self::Bool(_), Self::String(_)) => Less,
            _ => Greater,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use rilua::LuaApi;
    use rilua::LuaApiMut;
    use tempfile::tempdir;

    use crate::lua_api::WowLuaEnv;
    use crate::saved_variables::{SavedVariablesManager, WtfConfig};

    fn new_env() -> WowLuaEnv {
        WowLuaEnv::new().unwrap()
    }

    fn with_state<T>(env: &WowLuaEnv, f: impl FnOnce(&mut LuaState) -> T) -> T {
        let mut lua = env.rilua_mut();
        f(lua.state_mut())
    }

    fn ensure_saved_var_table(env: &WowLuaEnv, var_name: &str) {
        env.exec(&format!("{var_name} = {var_name} or {{}}"))
            .unwrap();
    }

    #[test]
    fn test_init_missing_variables_stay_nil_until_initialized_by_addon() {
        let env = new_env();
        let dir = tempdir().unwrap();
        let mut mgr = SavedVariablesManager::with_storage_dir(dir.path().to_path_buf());

        with_state(&env, |state| {
            mgr.init_for_addon(state, "TestAddon", &["TestDB".to_string()], &[])
        })
        .unwrap();

        let value_type: String = env.eval("return type(TestDB)").unwrap();
        assert_eq!(value_type, "nil");
    }

    #[test]
    fn test_save_and_load() {
        let dir = tempdir().unwrap();

        {
            let env = new_env();
            let mut mgr = SavedVariablesManager::with_storage_dir(dir.path().to_path_buf());

            with_state(&env, |state| {
                mgr.init_for_addon(state, "TestAddon", &["TestDB".to_string()], &[])
            })
            .unwrap();

            ensure_saved_var_table(&env, "TestDB");
            env.exec(r#"TestDB.setting1 = "hello"; TestDB.setting2 = 42"#)
                .unwrap();

            with_state(&env, |state| mgr.save_addon(state, "TestAddon")).unwrap();
        }

        {
            let env = new_env();
            let mut mgr = SavedVariablesManager::with_storage_dir(dir.path().to_path_buf());

            with_state(&env, |state| {
                mgr.init_for_addon(state, "TestAddon", &["TestDB".to_string()], &[])
            })
            .unwrap();

            let val1: String = env.eval("return TestDB.setting1").unwrap();
            let val2: i64 = env.eval("return TestDB.setting2").unwrap();

            assert_eq!(val1, "hello");
            assert_eq!(val2, 42);
        }
    }

    #[test]
    fn test_save_produces_lua_format() {
        let dir = tempdir().unwrap();
        let env = new_env();
        let mut mgr = SavedVariablesManager::with_storage_dir(dir.path().to_path_buf());

        with_state(&env, |state| {
            mgr.init_for_addon(state, "TestAddon", &["TestDB".to_string()], &[])
        })
        .unwrap();

        ensure_saved_var_table(&env, "TestDB");
        env.exec(r#"TestDB.name = "Haky"; TestDB.level = 70; TestDB.active = true"#)
            .unwrap();

        with_state(&env, |state| mgr.save_addon(state, "TestAddon")).unwrap();

        let content = fs::read_to_string(dir.path().join("TestAddon.lua")).unwrap();
        assert!(content.contains("TestDB = {"), "should have Lua assignment");
        assert!(
            content.contains("[\"name\"] = \"Haky\""),
            "should have string value"
        );
        assert!(
            content.contains("[\"level\"] = 70"),
            "should have integer value"
        );
        assert!(
            content.contains("[\"active\"] = true"),
            "should have boolean value"
        );
    }

    #[test]
    fn test_save_nested_tables() {
        let dir = tempdir().unwrap();
        let env = new_env();
        let mut mgr = SavedVariablesManager::with_storage_dir(dir.path().to_path_buf());

        with_state(&env, |state| {
            mgr.init_for_addon(state, "TestAddon", &["TestDB".to_string()], &[])
        })
        .unwrap();

        ensure_saved_var_table(&env, "TestDB");
        env.exec(
            r#"
            TestDB.nested = { a = 1, b = { c = "deep" } }
            TestDB.list = { 10, 20, 30 }
        "#,
        )
        .unwrap();

        with_state(&env, |state| mgr.save_addon(state, "TestAddon")).unwrap();

        let env2 = new_env();
        let mut mgr2 = SavedVariablesManager::with_storage_dir(dir.path().to_path_buf());
        with_state(&env2, |state| {
            mgr2.init_for_addon(state, "TestAddon", &["TestDB".to_string()], &[])
        })
        .unwrap();

        let deep: String = env2.eval("return TestDB.nested.b.c").unwrap();
        assert_eq!(deep, "deep");

        let second: i64 = env2.eval("return TestDB.list[2]").unwrap();
        assert_eq!(second, 20);

        let len: i64 = env2.eval("return #TestDB.list").unwrap();
        assert_eq!(len, 3);
    }

    #[test]
    fn test_save_string_escaping() {
        let dir = tempdir().unwrap();
        let env = new_env();
        let mut mgr = SavedVariablesManager::with_storage_dir(dir.path().to_path_buf());

        with_state(&env, |state| {
            mgr.init_for_addon(state, "TestAddon", &["TestDB".to_string()], &[])
        })
        .unwrap();

        ensure_saved_var_table(&env, "TestDB");
        env.exec(r#"TestDB.msg = "line1\nline2"; TestDB.path = "C:\\Users\\test""#)
            .unwrap();

        with_state(&env, |state| mgr.save_addon(state, "TestAddon")).unwrap();

        let env2 = new_env();
        let mut mgr2 = SavedVariablesManager::with_storage_dir(dir.path().to_path_buf());
        with_state(&env2, |state| {
            mgr2.init_for_addon(state, "TestAddon", &["TestDB".to_string()], &[])
        })
        .unwrap();

        let msg: String = env2.eval("return TestDB.msg").unwrap();
        assert_eq!(msg, "line1\nline2");

        let path: String = env2.eval("return TestDB.path").unwrap();
        assert_eq!(path, "C:\\Users\\test");
    }

    #[test]
    fn test_per_character_variables() {
        let dir = tempdir().unwrap();

        {
            let env = new_env();
            let mut mgr = SavedVariablesManager::with_storage_dir(dir.path().to_path_buf());
            mgr.set_character("Thrall", "Hyjal");

            with_state(&env, |state| {
                mgr.init_for_addon(state, "TestAddon", &[], &["CharDB".to_string()])
            })
            .unwrap();

            ensure_saved_var_table(&env, "CharDB");
            env.exec("CharDB.level = 70").unwrap();
            with_state(&env, |state| mgr.save_addon(state, "TestAddon")).unwrap();
        }

        {
            let env = new_env();
            let mut mgr = SavedVariablesManager::with_storage_dir(dir.path().to_path_buf());
            mgr.set_character("Thrall", "Hyjal");

            with_state(&env, |state| {
                mgr.init_for_addon(state, "TestAddon", &[], &["CharDB".to_string()])
            })
            .unwrap();

            let level: i64 = env.eval("return CharDB.level").unwrap();
            assert_eq!(level, 70);
        }

        {
            let env = new_env();
            let mut mgr = SavedVariablesManager::with_storage_dir(dir.path().to_path_buf());
            mgr.set_character("Jaina", "Hyjal");

            with_state(&env, |state| {
                mgr.init_for_addon(state, "TestAddon", &[], &["CharDB".to_string()])
            })
            .unwrap();

            let value_type: String = env.eval("return type(CharDB)").unwrap();
            assert_eq!(value_type, "nil");
        }
    }

    #[test]
    fn test_multiple_variables_per_addon() {
        let dir = tempdir().unwrap();

        {
            let env = new_env();
            let mut mgr = SavedVariablesManager::with_storage_dir(dir.path().to_path_buf());

            with_state(&env, |state| {
                mgr.init_for_addon(
                    state,
                    "Angleur",
                    &[
                        "AngleurConfig".to_string(),
                        "AngleurMinimapButton".to_string(),
                    ],
                    &["AngleurCharacter".to_string()],
                )
            })
            .unwrap();

            ensure_saved_var_table(&env, "AngleurConfig");
            ensure_saved_var_table(&env, "AngleurMinimapButton");
            ensure_saved_var_table(&env, "AngleurCharacter");
            env.exec(
                r#"
                AngleurConfig.method = "oneKey"
                AngleurMinimapButton.hide = true
                AngleurCharacter.sleeping = false
            "#,
            )
            .unwrap();

            with_state(&env, |state| mgr.save_addon(state, "Angleur")).unwrap();
        }

        {
            let env = new_env();
            let mut mgr = SavedVariablesManager::with_storage_dir(dir.path().to_path_buf());

            with_state(&env, |state| {
                mgr.init_for_addon(
                    state,
                    "Angleur",
                    &[
                        "AngleurConfig".to_string(),
                        "AngleurMinimapButton".to_string(),
                    ],
                    &["AngleurCharacter".to_string()],
                )
            })
            .unwrap();

            let method: String = env.eval("return AngleurConfig.method").unwrap();
            assert_eq!(method, "oneKey");

            let hide: bool = env.eval("return AngleurMinimapButton.hide").unwrap();
            assert!(hide);

            let sleeping: bool = env.eval("return AngleurCharacter.sleeping").unwrap();
            assert!(!sleeping);
        }
    }

    #[test]
    fn test_wtf_import_source_takes_precedence_over_local_storage() {
        let dir = tempdir().unwrap();
        let wtf_root = dir.path().join("WTF");
        let local_root = dir.path().join("LocalSavedVariables");
        let wtf_path = wtf_root
            .join("Account")
            .join("TestAccount")
            .join("SavedVariables");
        fs::create_dir_all(&wtf_path).unwrap();
        fs::create_dir_all(&local_root).unwrap();
        fs::write(
            wtf_path.join("TestAddon.lua"),
            "\nTestDB = { [\"source\"] = \"wtf\" }\n",
        )
        .unwrap();
        fs::write(
            local_root.join("TestAddon.lua"),
            "\nTestDB = { [\"source\"] = \"local\" }\n",
        )
        .unwrap();

        let env = new_env();
        let mut mgr = SavedVariablesManager::with_storage_dir(local_root);
        mgr.set_wtf_config(WtfConfig::new(
            &wtf_root,
            "TestAccount",
            "Realm",
            "Character",
        ));

        let loaded = with_state(&env, |state| mgr.load_wtf_for_addon(state, "TestAddon")).unwrap();
        assert_eq!(loaded, 1);
        with_state(&env, |state| {
            mgr.init_for_addon(state, "TestAddon", &["TestDB".to_string()], &[])
        })
        .unwrap();

        let source: String = env.eval("return TestDB.source").unwrap();
        assert_eq!(source, "wtf");
    }

    #[test]
    fn test_wtf_source_is_read_only_when_saving() {
        let dir = tempdir().unwrap();
        let wtf_root = dir.path().join("WTF");
        let local_root = dir.path().join("LocalSavedVariables");
        let wtf_path = wtf_root
            .join("Account")
            .join("TestAccount")
            .join("SavedVariables");
        fs::create_dir_all(&wtf_path).unwrap();
        fs::create_dir_all(&local_root).unwrap();
        let wtf_file = wtf_path.join("TestAddon.lua");
        fs::write(
            &wtf_file,
            "\nTestDB = { [\"source\"] = \"wtf\", [\"unchanged\"] = true }\n",
        )
        .unwrap();

        let env = new_env();
        let mut mgr = SavedVariablesManager::with_storage_dir(local_root.clone());
        mgr.set_wtf_config(WtfConfig::new(
            &wtf_root,
            "TestAccount",
            "Realm",
            "Character",
        ));

        let loaded = with_state(&env, |state| mgr.load_wtf_for_addon(state, "TestAddon")).unwrap();
        assert_eq!(loaded, 1);
        with_state(&env, |state| {
            mgr.init_for_addon(state, "TestAddon", &["TestDB".to_string()], &[])
        })
        .unwrap();

        env.exec(r#"TestDB.source = "sim"; TestDB.newValue = 42"#)
            .unwrap();
        with_state(&env, |state| mgr.save_addon(state, "TestAddon")).unwrap();

        let wtf_content = fs::read_to_string(&wtf_file).unwrap();
        assert!(
            wtf_content.contains("[\"source\"] = \"wtf\""),
            "live WTF file should remain unchanged"
        );
        assert!(
            !wtf_content.contains("newValue"),
            "live WTF file should not receive simulator writes"
        );

        let local_content = fs::read_to_string(local_root.join("TestAddon.lua")).unwrap();
        assert!(local_content.contains("[\"source\"] = \"sim\""));
        assert!(local_content.contains("[\"newValue\"] = 42"));
    }

    #[test]
    fn test_invalid_local_saved_variables_report_failure_without_seeding_defaults() {
        let dir = tempdir().unwrap();
        let local_root = dir.path().join("LocalSavedVariables");
        fs::create_dir_all(&local_root).unwrap();
        fs::write(
            local_root.join("TestAddon.lua"),
            "\nTestDB TestMinimapDB = nil\n",
        )
        .unwrap();

        let env = new_env();
        let mut mgr = SavedVariablesManager::with_storage_dir(local_root);
        let error = with_state(&env, |state| {
            mgr.init_for_addon(
                state,
                "TestAddon",
                &["TestDB".to_string(), "TestMinimapDB".to_string()],
                &[],
            )
        })
        .unwrap_err();
        assert!(error.to_string().contains("TestAddon.lua"), "{error}");

        let types: (String, String) = env
            .eval("return type(TestDB), type(TestMinimapDB)")
            .unwrap();
        assert_eq!(types, ("nil".to_string(), "nil".to_string()));
    }

    #[test]
    fn test_invalid_local_saved_variables_fall_back_to_wtf_source() {
        let dir = tempdir().unwrap();
        let wtf_root = dir.path().join("WTF");
        let local_root = dir.path().join("LocalSavedVariables");
        let wtf_path = wtf_root
            .join("Account")
            .join("TestAccount")
            .join("SavedVariables");
        fs::create_dir_all(&wtf_path).unwrap();
        fs::create_dir_all(&local_root).unwrap();
        fs::write(
            local_root.join("TestAddon.lua"),
            "\nTestDB TestMinimapDB = nil\n",
        )
        .unwrap();
        fs::write(
            wtf_path.join("TestAddon.lua"),
            "\nTestDB = { [\"source\"] = \"wtf\" }\nTestMinimapDB = { [\"hide\"] = true }\n",
        )
        .unwrap();

        let env = new_env();
        let mut mgr = SavedVariablesManager::with_storage_dir(local_root);
        mgr.set_wtf_config(WtfConfig::new(
            &wtf_root,
            "TestAccount",
            "Realm",
            "Character",
        ));
        with_state(&env, |state| {
            mgr.init_for_addon(
                state,
                "TestAddon",
                &["TestDB".to_string(), "TestMinimapDB".to_string()],
                &[],
            )
        })
        .unwrap();

        let values: (String, bool) = env
            .eval("return TestDB.source, TestMinimapDB.hide")
            .unwrap();
        assert_eq!(values, ("wtf".to_string(), true));
    }

    #[test]
    fn test_serialize_format_matches_wow() {
        let env = new_env();
        env.exec(
            r#"
            TestVar = {
                ["setting"] = "hello",
                ["items"] = { 10, 20, 30 },
            }
        "#,
        )
        .unwrap();

        let value: Val = env.eval("return TestVar").unwrap();
        let lua = env.rilua();
        let mut output = String::new();
        serialize_assignment(&mut output, lua.state(), "TestVar", value).unwrap();

        assert!(output.starts_with("TestVar = {"));
        assert!(output.contains("-- [1]"));
        assert!(output.contains("-- [2]"));
        assert!(output.contains("[\"setting\"] = \"hello\""));
    }

    fn serialize_globals(env: &WowLuaEnv, names: &[&str]) -> String {
        let mut output = String::new();
        for &name in names {
            let value: Val = env.eval(&format!("return {name}")).unwrap();
            let lua = env.rilua();
            serialize_assignment(&mut output, lua.state(), name, value).unwrap();
        }
        output
    }

    #[test]
    fn binary_strings_and_keys_round_trip_every_byte() {
        let env = new_env();
        env.exec(
            r#"
            local bytes = {}
            for i = 0, 255 do bytes[#bytes + 1] = string.char(i) end
            Binary = table.concat(bytes)
            BinaryKeys = {}
            for i = 0, 255 do
                -- A following digit exercises decimal escape boundaries.
                BinaryKeys[string.char(i) .. "123"] = string.char(i) .. "987"
            end
            "#,
        )
        .unwrap();

        let output = serialize_globals(&env, &["Binary", "BinaryKeys"]);
        assert!(output.is_ascii());
        assert!(output.contains("\\000123"));
        assert!(output.contains("\\255987"));
        let fresh = new_env();
        fresh.exec(&output).unwrap();
        fresh
            .exec(
                r#"
            assert(#Binary == 256)
            for i = 0, 255 do
                assert(string.byte(Binary, i + 1) == i)
                assert(BinaryKeys[string.char(i) .. "123"] == string.char(i) .. "987")
            end
            local count = 0
            for _ in pairs(BinaryKeys) do count = count + 1 end
            assert(count == 256)
            "#,
            )
            .unwrap();
    }

    #[test]
    fn sparse_tables_changes_scalar_globals_and_raw_data_round_trip() {
        let env = new_env();
        env.exec(
            r#"
            local shared = { nested = "kept" }
            Data = { 10, 20, 30, 40, 50, 60, 70, 80 }
            Data[2] = nil
            Data[7] = nil
            Data[-2] = "negative"
            Data[0.25] = "fractional"
            Data[true] = false
            Data[false] = true
            Data.left = shared
            Data.right = shared
            Data.deleted = "old"
            Data.deleted = nil
            Data.added = { false, "new" }
            -- Raw saved fields do not invoke lookup or traversal metamethods.
            setmetatable(Data, {
                __index = function() error("lookup") end,
                __pairs = function() error("pairs") end,
            })
            Count = 4.5
            Enabled = false
            Removed = nil
            "#,
        )
        .unwrap();
        let output = serialize_globals(&env, &["Data", "Count", "Enabled", "Removed"]);
        assert!(output.contains("Removed = nil\n"));
        let fresh = new_env();
        fresh.exec("Removed = 'old'").unwrap();
        fresh.exec(&output).unwrap();
        fresh
            .exec(
                r#"
            assert(Data[1] == 10 and Data[2] == nil and Data[3] == 30)
            assert(Data[6] == 60 and Data[7] == nil and Data[8] == 80)
            assert(Data[-2] == "negative" and Data[0.25] == "fractional")
            assert(Data[true] == false and Data[false] == true)
            assert(Data.left.nested == "kept" and Data.right.nested == "kept")
            assert(Data.left ~= Data.right)
            assert(Data.deleted == nil and Data.added[1] == false and Data.added[2] == "new")
            assert(getmetatable(Data) == nil)
            assert(type(Count) == "number" and Count == 4.5)
            assert(type(Enabled) == "boolean" and Enabled == false and Removed == nil)
            local count = 0
            for _ in pairs(Data) do count = count + 1 end
            assert(count == 13)
            "#,
            )
            .unwrap();
    }

    #[test]
    fn finite_numbers_round_trip_exact_bits_including_signed_zero() {
        let env = new_env();
        let fresh = new_env();
        for number in [
            0.0,
            -0.0,
            0.1,
            -0.1,
            1.2345678901234567,
            f64::from_bits(1),
            f64::MIN_POSITIVE,
            f64::MAX,
            f64::MIN,
            9_007_199_254_740_991.0,
            9_223_372_036_854_775_808.0,
            1e-200,
            -1e200,
        ] {
            let mut output = String::new();
            {
                let lua = env.rilua();
                serialize_assignment(&mut output, lua.state(), "Number", Val::Num(number)).unwrap();
            }
            fresh.exec(&output).unwrap();
            let Val::Num(actual) = fresh.eval::<Val>("return Number").unwrap() else {
                panic!("number became another type");
            };
            assert_eq!(actual.to_bits(), number.to_bits(), "{output}");
        }
    }

    #[test]
    fn unrepresentable_data_fails_without_partial_assignment() {
        let env = new_env();
        let cases = [
            ("Data = function() end", "unsupported value type: function"),
            (
                "Data = coroutine.create(function() end)",
                "unsupported value type: thread",
            ),
            ("Data = newproxy()", "unsupported value type: userdata"),
            (
                "Data = { nested = function() end }",
                "unsupported value type: function",
            ),
            ("Data = {}; Data.self = Data", "cyclic table"),
            ("Data = { [{}] = 1 }", "unsupported table key type: table"),
            (
                "Data = { [function() end] = 1 }",
                "unsupported table key type: function",
            ),
            (
                "Data = { [newproxy()] = 1 }",
                "unsupported table key type: userdata",
            ),
            ("Data = 0 / 0", "non-finite number"),
            ("Data = math.huge", "non-finite number"),
            ("Data = -math.huge", "non-finite number"),
            ("Data = { [math.huge] = 1 }", "non-finite numeric key"),
            (
                "Data = {}; local t = Data; for i = 1, 128 do t.child = {}; t = t.child end",
                "table nesting exceeds supported depth",
            ),
        ];
        for (setup, message) in cases {
            env.exec(setup).unwrap();
            let value: Val = env.eval("return Data").unwrap();
            let mut output = "Previous = true\n".to_string();
            let error = {
                let lua = env.rilua();
                serialize_assignment(&mut output, lua.state(), "Data", value).unwrap_err()
            };
            assert!(error.to_string().contains(message), "{setup}: {error}");
            assert_eq!(output, "Previous = true\n", "{setup}");
        }
    }

    #[test]
    fn invalid_global_names_fail_and_deleted_unsupported_keys_are_absent() {
        let env = new_env();
        for name in [
            "",
            "two words",
            "end",
            "nil",
            "bad.name",
            "bad\nname",
            "x;evil()",
        ] {
            let mut output = "Previous = true\n".to_string();
            let lua = env.rilua();
            assert!(serialize_assignment(&mut output, lua.state(), name, Val::Nil).is_err());
            assert_eq!(output, "Previous = true\n");
        }
        env.exec("local key = {}; Data = { [key] = 1 }; Data[key] = nil")
            .unwrap();
        let output = serialize_globals(&env, &["Data"]);
        let fresh = new_env();
        fresh.exec(&output).unwrap();
        assert!(fresh.eval::<bool>("return next(Data) == nil").unwrap());
    }
}
