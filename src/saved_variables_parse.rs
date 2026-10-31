use super::set_global;
use crate::lua_api::methods::{create_string, table_set_num};
use rilua::Val;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::string::LuaString;
use rilua::vm::table::Table;
use rustc_hash::FxHashMap;

const MAX_CACHED_TABLE_PATH_BYTES: usize = 8192;

pub(super) fn parse_saved_variables_file_with_cache(
    state: &mut LuaState,
    source: &str,
    table_size_cache: Option<&mut SavedVariablesTableSizeCache>,
) -> Result<(), String> {
    Parser::new(state, source, table_size_cache).parse_file()
}

/// Read one simulator-owned table without publishing globals or executing Lua.
pub(super) fn parse_model_value(state: &mut LuaState, source: &str) -> Result<Val, String> {
    let mut parser = Parser::new(state, source, None);
    parser.skip_ws();
    if parser.parse_identifier()? != "preferences" {
        return Err("expected preferences assignment".to_string());
    }
    parser.skip_ws();
    parser.expect_byte(b'=')?;
    let value = parser.parse_value(None)?;
    parser.skip_field_separator();
    parser.skip_ws();
    if !parser.is_eof() || !matches!(value, Val::Table(_)) {
        return Err("expected exactly one preferences table".to_string());
    }
    Ok(value)
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct SavedVariablesTableSize {
    pub array_count: usize,
    pub hash_count: usize,
}

#[derive(Debug, Default)]
pub(super) struct SavedVariablesTableSizeCache {
    sizes: FxHashMap<String, SavedVariablesTableSize>,
}

impl SavedVariablesTableSizeCache {
    pub(super) fn get(&self, path: &str) -> Option<SavedVariablesTableSize> {
        self.sizes.get(path).copied()
    }

    pub(super) fn insert_if_changed(&mut self, path: &str, size: SavedVariablesTableSize) -> bool {
        if self.get(path) == Some(size) {
            return false;
        }

        self.insert(path, size);
        true
    }

    pub(super) fn insert(&mut self, path: impl Into<String>, size: SavedVariablesTableSize) {
        self.sizes.insert(path.into(), size);
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = (&str, SavedVariablesTableSize)> {
        self.sizes.iter().map(|(path, size)| (path.as_str(), *size))
    }
}

struct Parser<'a, 's> {
    state: &'a mut LuaState,
    bytes: &'s [u8],
    pos: usize,
    key_cache: FxHashMap<String, GcRef<LuaString>>,
    table_size_cache: Option<&'a mut SavedVariablesTableSizeCache>,
}

impl<'a, 's> Parser<'a, 's> {
    fn new(
        state: &'a mut LuaState,
        source: &'s str,
        table_size_cache: Option<&'a mut SavedVariablesTableSizeCache>,
    ) -> Self {
        Self {
            state,
            bytes: source.as_bytes(),
            pos: 0,
            key_cache: FxHashMap::default(),
            table_size_cache,
        }
    }

    fn parse_file(&mut self) -> Result<(), String> {
        self.skip_ws();
        while !self.is_eof() {
            let name = self.parse_identifier()?;
            self.skip_ws();
            self.expect_byte(b'=')?;
            let value = self.parse_value(Some(&name))?;
            set_global(self.state, &name, value);
            self.skip_field_separator();
            self.skip_ws();
        }
        Ok(())
    }

    fn parse_table(&mut self, table_path: Option<&str>) -> Result<Val, String> {
        self.expect_byte(b'{')?;
        let (table, table_ref) = self.allocate_table(table_path);
        let mut next_array_index = 1.0;
        let mut array_count = 0usize;
        let mut hash_count = 0usize;

        loop {
            self.skip_ws();
            if self.consume_byte(b'}') {
                break;
            }

            if self.consume_byte(b'[') {
                self.parse_bracket_table_field(table, table_ref, table_path)?;
                hash_count += 1;
            } else if self.next_field_is_named_assignment() {
                self.parse_named_table_field(table, table_path)?;
                hash_count += 1;
            } else {
                self.parse_array_table_field(table_ref, table_path, next_array_index)?;
                next_array_index += 1.0;
                array_count += 1;
            }

            self.skip_field_separator();
        }

        self.record_table_size(table_path, array_count, hash_count);
        Ok(table)
    }

    fn allocate_table(
        &mut self,
        table_path: Option<&str>,
    ) -> (Val, rilua::vm::gc::arena::GcRef<Table>) {
        let size_hint = self.size_hint_for(table_path);
        let table = Val::Table(self.state.gc.alloc_table(Table::with_sizes(
            size_hint.array_count,
            size_hint.hash_count,
        )));
        let Val::Table(table_ref) = table else {
            unreachable!("fresh table is always a table")
        };
        (table, table_ref)
    }

    fn record_table_size(
        &mut self,
        table_path: Option<&str>,
        array_count: usize,
        hash_count: usize,
    ) {
        if let (Some(cache), Some(path)) = (&mut self.table_size_cache, table_path) {
            cache.insert_if_changed(
                path,
                SavedVariablesTableSize {
                    array_count,
                    hash_count,
                },
            );
        }
    }

    fn parse_bracket_table_field(
        &mut self,
        table: Val,
        table_ref: rilua::vm::gc::arena::GcRef<Table>,
        table_path: Option<&str>,
    ) -> Result<(), String> {
        let key = self.parse_key()?;
        self.skip_ws();
        self.expect_byte(b']')?;
        self.skip_ws();
        self.expect_byte(b'=')?;
        let child_path = table_path.and_then(|path| key.child_path(path));
        let value = self.parse_value(child_path.as_deref())?;
        self.set_table_key(table, table_ref, key, value);
        Ok(())
    }

    fn parse_named_table_field(
        &mut self,
        table: Val,
        table_path: Option<&str>,
    ) -> Result<(), String> {
        let key = self.parse_identifier()?;
        self.skip_ws();
        self.expect_byte(b'=')?;
        let child_path = table_path.and_then(|path| extend_cached_table_path(path, &key));
        let value = self.parse_value(child_path.as_deref())?;
        self.set_table_string_key(table, &key, value);
        Ok(())
    }

    fn parse_array_table_field(
        &mut self,
        table_ref: rilua::vm::gc::arena::GcRef<Table>,
        table_path: Option<&str>,
        next_array_index: f64,
    ) -> Result<(), String> {
        let child_path = table_path.and_then(|path| {
            let index = format_lua_number_for_path(next_array_index);
            bracket_cached_table_path(path, &index)
        });
        let value = self.parse_value(child_path.as_deref())?;
        table_set_num(self.state, table_ref, next_array_index, value);
        Ok(())
    }

    fn size_hint_for(&self, table_path: Option<&str>) -> SavedVariablesTableSize {
        table_path
            .and_then(|path| self.table_size_cache.as_deref()?.get(path))
            .map(SavedVariablesTableSize::capped)
            .unwrap_or_default()
    }

    fn parse_key(&mut self) -> Result<SavedVarKey, String> {
        self.skip_ws();
        match self.peek_byte() {
            Some(b'"') | Some(b'\'') => self.parse_string().map(SavedVarKey::String),
            Some(b'-' | b'0'..=b'9') => self.parse_number().map(SavedVarKey::Number),
            _ => Err(self.error("unsupported table key")),
        }
    }

    fn set_table_key(
        &mut self,
        table: Val,
        table_ref: rilua::vm::gc::arena::GcRef<Table>,
        key: SavedVarKey,
        value: Val,
    ) {
        match key {
            SavedVarKey::String(key) => self.set_table_string_key(table, &key, value),
            SavedVarKey::Number(key) => table_set_num(self.state, table_ref, key, value),
        }
    }

    fn set_table_string_key(&mut self, table: Val, key: &str, value: Val) {
        let Val::Table(table_ref) = table else { return };
        let stack_slot = self.state.top;
        self.state.ensure_stack(stack_slot + 1);
        self.state.stack_set(stack_slot, value);
        self.state.top = stack_slot + 1;

        let key_ref = self.intern_saved_var_key(key);
        if let Some(table) = self.state.gc.tables.get_mut(table_ref) {
            let _ = table.raw_set(Val::Str(key_ref), value, &self.state.gc.string_arena);
        }
        self.state.gc.barrier_back(table_ref);
        self.state.top = stack_slot;
    }

    fn intern_saved_var_key(&mut self, key: &str) -> GcRef<LuaString> {
        if let Some(&key_ref) = self.key_cache.get(key) {
            return key_ref;
        }
        let key_ref = self.state.gc.intern_string(key.as_bytes());
        self.key_cache.insert(key.to_string(), key_ref);
        key_ref
    }

    fn next_field_is_named_assignment(&self) -> bool {
        let mut pos = self.pos;
        if !is_ident_start(self.bytes.get(pos).copied()) {
            return false;
        }
        pos += 1;
        while is_ident_continue(self.bytes.get(pos).copied()) {
            pos += 1;
        }
        while matches!(self.bytes.get(pos), Some(b' ' | b'\t' | b'\r' | b'\n')) {
            pos += 1;
        }
        self.bytes.get(pos) == Some(&b'=')
    }

    fn parse_identifier(&mut self) -> Result<String, String> {
        let start = self.pos;
        if !is_ident_start(self.peek_byte()) {
            return Err(self.error("expected identifier"));
        }
        self.pos += 1;
        while is_ident_continue(self.peek_byte()) {
            self.pos += 1;
        }
        std::str::from_utf8(&self.bytes[start..self.pos])
            .map(|s| s.to_string())
            .map_err(|_| self.error("identifier is not UTF-8"))
    }

    fn parse_string(&mut self) -> Result<String, String> {
        let quote = self
            .next_byte()
            .ok_or_else(|| self.error("expected string"))?;
        let start = self.pos;
        let mut out = Vec::new();
        while let Some(byte) = self.next_byte() {
            if byte == quote {
                if out.is_empty() {
                    return std::str::from_utf8(&self.bytes[start..self.pos - 1])
                        .map(|value| value.to_string())
                        .map_err(|_| self.error("string is not UTF-8"));
                }
                return String::from_utf8(out).map_err(|_| self.error("string is not UTF-8"));
            }
            if byte != b'\\' {
                if !out.is_empty() {
                    out.push(byte);
                }
                continue;
            }
            if out.is_empty() {
                out.extend_from_slice(&self.bytes[start..self.pos - 1]);
            }
            self.parse_escape(&mut out)?;
        }
        Err(self.error("unterminated string"))
    }

    fn parse_escape(&mut self, out: &mut Vec<u8>) -> Result<(), String> {
        let byte = self
            .next_byte()
            .ok_or_else(|| self.error("unterminated escape"))?;
        match byte {
            b'\\' => out.push(b'\\'),
            b'"' => out.push(b'"'),
            b'\'' => out.push(b'\''),
            b'a' => out.push(7),
            b'b' => out.push(8),
            b'f' => out.push(12),
            b'n' => out.push(b'\n'),
            b'r' => out.push(b'\r'),
            b't' => out.push(b'\t'),
            b'v' => out.push(11),
            b'\r' => {
                self.consume_byte(b'\n');
                out.push(b'\n');
            }
            b'\n' => {
                self.consume_byte(b'\r');
                out.push(b'\n');
            }
            b'0'..=b'9' => self.parse_decimal_escape(out, byte)?,
            other => out.push(other),
        }
        Ok(())
    }

    fn parse_decimal_escape(&mut self, out: &mut Vec<u8>, first: u8) -> Result<(), String> {
        let mut value = first - b'0';
        for _ in 0..2 {
            let Some(next) = self.peek_byte() else {
                break;
            };
            if !next.is_ascii_digit() {
                break;
            }
            value = value
                .checked_mul(10)
                .and_then(|value| value.checked_add(next - b'0'))
                .ok_or_else(|| self.error("invalid decimal escape"))?;
            self.pos += 1;
        }
        out.push(value);
        Ok(())
    }

    fn parse_number(&mut self) -> Result<f64, String> {
        let start = self.pos;
        self.consume_byte(b'-');
        self.consume_digits();
        if self.consume_byte(b'.') {
            self.consume_digits();
        }
        if self.consume_byte(b'e') || self.consume_byte(b'E') {
            let _ = self.consume_byte(b'-') || self.consume_byte(b'+');
            self.consume_digits();
        }
        let text = std::str::from_utf8(&self.bytes[start..self.pos])
            .map_err(|_| self.error("number is not UTF-8"))?;
        text.parse::<f64>()
            .map_err(|_| self.error("invalid number"))
    }

    fn consume_digits(&mut self) {
        while matches!(self.peek_byte(), Some(b'0'..=b'9')) {
            self.pos += 1;
        }
    }

    fn expect_keyword(&mut self, keyword: &str) -> Result<(), String> {
        if self.bytes[self.pos..].starts_with(keyword.as_bytes()) {
            self.pos += keyword.len();
            return Ok(());
        }
        Err(self.error(&format!("expected {keyword}")))
    }

    fn skip_ws(&mut self) {
        loop {
            while matches!(self.peek_byte(), Some(b' ' | b'\t' | b'\r' | b'\n')) {
                self.pos += 1;
            }
            if self.bytes[self.pos..].starts_with(b"--") {
                while !matches!(self.peek_byte(), None | Some(b'\n')) {
                    self.pos += 1;
                }
                continue;
            }
            break;
        }
    }

    fn skip_field_separator(&mut self) {
        self.skip_ws();
        let _ = self.consume_byte(b',') || self.consume_byte(b';');
    }

    fn expect_byte(&mut self, expected: u8) -> Result<(), String> {
        if self.consume_byte(expected) {
            return Ok(());
        }
        Err(self.error(&format!("expected '{}'", expected as char)))
    }

    fn consume_byte(&mut self, expected: u8) -> bool {
        if self.peek_byte() == Some(expected) {
            self.pos += 1;
            return true;
        }
        false
    }

    fn next_byte(&mut self) -> Option<u8> {
        let byte = self.peek_byte()?;
        self.pos += 1;
        Some(byte)
    }

    fn peek_byte(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.bytes.len()
    }

    fn error(&self, message: &str) -> String {
        format!("{message} at byte {}", self.pos)
    }

    fn parse_string_value(&mut self) -> Result<Val, String> {
        let value = self.parse_string()?;
        Ok(create_string(self.state, &value))
    }

    fn parse_keyword_value(&mut self, byte: u8) -> Result<Val, String> {
        if byte == b't' {
            self.expect_keyword("true")?;
            return Ok(Val::Bool(true));
        }
        if byte == b'f' {
            self.expect_keyword("false")?;
            return Ok(Val::Bool(false));
        }
        if byte == b'n' {
            self.expect_keyword("nil")?;
            return Ok(Val::Nil);
        }
        Err(self.error("unsupported SavedVariables value"))
    }

    fn parse_value(&mut self, table_path: Option<&str>) -> Result<Val, String> {
        self.skip_ws();
        let byte = if let Some(byte) = self.peek_byte() {
            byte
        } else {
            return Err(self.error("unexpected end of file"));
        };
        if byte == b'{' {
            return self.parse_table(table_path);
        }
        if is_quote_byte(byte) {
            return self.parse_string_value();
        }
        if byte == b'-' || byte.is_ascii_digit() {
            return self.parse_number().map(Val::Num);
        }
        self.parse_keyword_value(byte)
    }
}

enum SavedVarKey {
    String(String),
    Number(f64),
}

impl SavedVarKey {
    fn child_path(&self, parent: &str) -> Option<String> {
        match self {
            SavedVarKey::String(key) if is_identifier(key) => extend_cached_table_path(parent, key),
            SavedVarKey::String(key) => {
                bracket_cached_table_path(parent, &format!("\"{}\"", escape_path_string(key)))
            }
            SavedVarKey::Number(key) => {
                bracket_cached_table_path(parent, &format_lua_number_for_path(*key))
            }
        }
    }
}

impl SavedVariablesTableSize {
    const MAX_CACHED_ARRAY_COUNT: usize = 1_000_000;
    const MAX_CACHED_HASH_COUNT: usize = 1_000_000;

    fn capped(self) -> Self {
        Self {
            array_count: self.array_count.min(Self::MAX_CACHED_ARRAY_COUNT),
            hash_count: self.hash_count.min(Self::MAX_CACHED_HASH_COUNT),
        }
    }
}

fn is_ident_start(byte: Option<u8>) -> bool {
    matches!(byte, Some(b'a'..=b'z' | b'A'..=b'Z' | b'_'))
}

fn is_ident_continue(byte: Option<u8>) -> bool {
    matches!(byte, Some(b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_'))
}

fn is_quote_byte(byte: u8) -> bool {
    byte == b'"' || byte == b'\''
}

fn is_identifier(value: &str) -> bool {
    let mut bytes = value.bytes();
    if !is_ident_start(bytes.next()) {
        return false;
    }
    bytes.all(|byte| is_ident_continue(Some(byte)))
}

fn escape_path_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn extend_cached_table_path(parent: &str, child: &str) -> Option<String> {
    cached_table_path_from_parts(parent, ".", child, "")
}

fn bracket_cached_table_path(parent: &str, child: &str) -> Option<String> {
    cached_table_path_from_parts(parent, "[", child, "]")
}

fn cached_table_path_from_parts(
    parent: &str,
    prefix: &str,
    child: &str,
    suffix: &str,
) -> Option<String> {
    let total_len = parent.len() + prefix.len() + child.len() + suffix.len();
    if total_len > MAX_CACHED_TABLE_PATH_BYTES {
        return None;
    }

    let mut path = String::with_capacity(total_len);
    path.push_str(parent);
    path.push_str(prefix);
    path.push_str(child);
    path.push_str(suffix);
    Some(path)
}

fn format_lua_number_for_path(value: f64) -> String {
    if value.fract() == 0.0 && value >= i64::MIN as f64 && value <= i64::MAX as f64 {
        return (value as i64).to_string();
    }
    value.to_string()
}

#[cfg(test)]
mod tests {
    use super::super::get_global;
    use super::*;
    use crate::lua_api::WowLuaEnv;
    use rilua::LuaApiMut;

    #[test]
    fn parses_saved_variable_assignments_without_lua_compilation() {
        let env = WowLuaEnv::new().unwrap();
        let mut lua = env.rilua_mut();
        let state = lua.state_mut();

        parse_saved_variables_file_with_cache(
            state,
            r#"
                TEST_SV = {
                    ["name"] = "hello\nworld",
                    count = 3,
                    true,
                    [42] = false,
                }
                "#,
            None,
        )
        .unwrap();

        assert!(matches!(get_global(state, "TEST_SV"), Val::Table(_)));
        drop(lua);

        let (name, count): (String, i32) = env.eval("return TEST_SV.name, TEST_SV.count").unwrap();
        assert_eq!(name, "hello\nworld");
        assert_eq!(count, 3);
    }

    #[test]
    fn preserves_utf8_strings() {
        let env = WowLuaEnv::new().unwrap();
        {
            let mut lua = env.rilua_mut();
            parse_saved_variables_file_with_cache(
                lua.state_mut(),
                r#"
                    TEST_SV = {
                        ["localized"] = "Café",
                    }
                    "#,
                None,
            )
            .unwrap();
        }

        let localized: String = env.eval("return TEST_SV.localized").unwrap();
        assert_eq!(localized, "Café");
    }

    #[test]
    fn short_string_escapes_match_lua_vm_bytes() {
        let cases: &[(&str, &[u8])] = &[
            (
                r#"ESC = "\a\b\f\n\r\t\v\\\"\'""#,
                &[7, 8, 12, 10, 13, 9, 11, b'\\', b'"', b'\''],
            ),
            ("ESC = \"left\\\nright\"", b"left\nright"),
            ("ESC = \"left\\\rright\"", b"left\nright"),
            ("ESC = \"left\\\r\nright\"", b"left\nright"),
            ("ESC = \"left\\\n\rright\"", b"left\nright"),
        ];
        let mut parsed = rilua::Lua::new().unwrap();
        let mut compiled = rilua::Lua::new().unwrap();
        for &(source, expected) in cases {
            parse_saved_variables_file_with_cache(parsed.state_mut(), source, None).unwrap();
            compiled.exec(source).unwrap();
            for lua in [&mut parsed, &mut compiled] {
                let state = lua.state_mut();
                let Val::Str(value) = get_global(state, "ESC") else {
                    panic!("ESC should be a string: {source:?}");
                };
                assert_eq!(
                    state.gc.string_arena.get(value).unwrap().data(),
                    expected,
                    "{source:?}"
                );
            }
        }
    }

    #[test]
    fn records_table_sizes_by_stable_path() {
        let env = WowLuaEnv::new().unwrap();
        let mut lua = env.rilua_mut();
        let mut cache = SavedVariablesTableSizeCache::default();

        parse_saved_variables_file_with_cache(
            lua.state_mut(),
            r#"
                TEST_SV = {
                    account = {
                        enabled = true,
                        count = 3,
                    },
                    list = {
                        "one",
                        "two",
                    },
                }
                "#,
            Some(&mut cache),
        )
        .unwrap();

        assert_eq!(
            cache.get("TEST_SV.account"),
            Some(SavedVariablesTableSize {
                array_count: 0,
                hash_count: 2,
            })
        );
        assert_eq!(
            cache.get("TEST_SV.list"),
            Some(SavedVariablesTableSize {
                array_count: 2,
                hash_count: 0,
            })
        );
    }

    #[test]
    fn uses_recorded_table_size_when_allocating_table() {
        let env = WowLuaEnv::new().unwrap();
        let mut lua = env.rilua_mut();
        let mut cache = SavedVariablesTableSizeCache::default();
        cache.insert(
            "TEST_SV",
            SavedVariablesTableSize {
                array_count: 8,
                hash_count: 16,
            },
        );

        parse_saved_variables_file_with_cache(
            lua.state_mut(),
            r#"
                TEST_SV = {
                    first = true,
                }
                "#,
            Some(&mut cache),
        )
        .unwrap();

        let Val::Table(table_ref) = get_global(lua.state_mut(), "TEST_SV") else {
            panic!("TEST_SV should be a table");
        };
        let table = lua.state_mut().gc.tables.get(table_ref).unwrap();
        assert_eq!(table.array_len(), 8);
        assert_eq!(table.hash_size(), 16);
    }

    #[test]
    fn drops_table_size_paths_after_oversized_key() {
        let env = WowLuaEnv::new().unwrap();
        let mut lua = env.rilua_mut();
        let mut cache = SavedVariablesTableSizeCache::default();
        let long_key = "x".repeat(MAX_CACHED_TABLE_PATH_BYTES);
        let source = format!(
            r#"
                TEST_SV = {{
                    ["{long_key}"] = {{
                        child = {{}},
                    }},
                }}
                "#
        );

        parse_saved_variables_file_with_cache(lua.state_mut(), &source, Some(&mut cache)).unwrap();

        let longest_path = cache.iter().map(|(path, _)| path.len()).max().unwrap_or(0);
        assert!(longest_path <= MAX_CACHED_TABLE_PATH_BYTES);
        assert!(cache.get("TEST_SV").is_some());
    }

    #[test]
    fn formats_integer_table_paths_without_float_formatting() {
        assert_eq!(format_lua_number_for_path(42.0), "42");
        assert_eq!(format_lua_number_for_path(-7.0), "-7");
        assert_eq!(format_lua_number_for_path(3.5), "3.5");
    }

    #[test]
    fn skips_unchanged_table_size_cache_writes() {
        let mut cache = SavedVariablesTableSizeCache::default();
        let size = SavedVariablesTableSize {
            array_count: 3,
            hash_count: 5,
        };

        assert!(cache.insert_if_changed("AddonDB.profile", size));
        assert!(!cache.insert_if_changed("AddonDB.profile", size));

        let changed_size = SavedVariablesTableSize {
            array_count: 3,
            hash_count: 6,
        };
        assert!(cache.insert_if_changed("AddonDB.profile", changed_size));
        assert_eq!(cache.get("AddonDB.profile"), Some(changed_size));
    }
}
