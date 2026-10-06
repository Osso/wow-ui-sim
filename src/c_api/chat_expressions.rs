//! Host-owned expression vocabulary; no secret plaintext reaches Lua callbacks.
//! Group replacements are explicit host snapshots, not fabricated raid membership.

use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::table_security::{is_secret_value, transform_host_secret_string};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct ChatExpressionInputs {
    /// Lowercase brace contents (without braces) to exact replacement bytes.
    pub icons: BTreeMap<Vec<u8>, Vec<u8>>,
    pub groups: BTreeMap<Vec<u8>, Vec<u8>>,
}

impl Default for ChatExpressionInputs {
    fn default() -> Self {
        let mut icons = BTreeMap::new();
        for (index, name) in [
            "star", "circle", "diamond", "triangle", "moon", "square", "cross", "skull",
        ]
        .iter()
        .enumerate()
        {
            let number = index + 1;
            let output = format!("|TInterface\\TargetingFrame\\UI-RaidTargetingIcon_{number}:0|t")
                .into_bytes();
            icons.insert(name.as_bytes().to_vec(), output.clone());
            icons.insert(format!("rt{number}").into_bytes(), output);
        }
        Self {
            icons,
            groups: BTreeMap::new(),
        }
    }
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_ChatInfo")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "ReplaceIconAndGroupExpressions",
        replace_expressions,
    )
}

fn replace_expressions(state: &mut LuaState) -> LuaResult<u32> {
    let flags = [stack_val(state, 2), stack_val(state, 3)];
    // Check both original identities before defaults, text decoding or state lookup.
    for (index, flag) in flags.iter().enumerate() {
        if is_secret_value(state, *flag) {
            return Err(runtime_error(format!(
                "ReplaceIconAndGroupExpressions argument #{} must not be secret",
                index + 2
            )));
        }
    }
    let no_icons = read_flag(flags[0])?;
    let no_groups = read_flag(flags[1])?;
    let vocabulary = borrow_state(state)?.chat_expression_inputs.clone();
    let input = stack_val(state, 1);
    let transform = |bytes: &[u8]| expand_expressions(bytes, &vocabulary, no_icons, no_groups);
    let output = if is_secret_value(state, input) {
        transform_host_secret_string(state, input, |bytes| Ok(transform(bytes)))?
    } else {
        let Val::Str(string) = input else {
            return Err(runtime_error(
                "ReplaceIconAndGroupExpressions input must be a string",
            ));
        };
        let bytes = state
            .gc
            .string_arena
            .get(string)
            .expect("rooted chat input")
            .data();
        let bytes = transform(bytes);
        Val::Str(state.gc.intern_string(&bytes))
    };
    state.push(output);
    Ok(1)
}

fn read_flag(value: Val) -> LuaResult<bool> {
    match value {
        Val::Nil => Ok(false),
        Val::Bool(value) => Ok(value),
        _ => Err(runtime_error(
            "expression replacement flag must be a boolean or nil",
        )),
    }
}

fn expand_expressions(
    bytes: &[u8],
    vocabulary: &ChatExpressionInputs,
    no_icons: bool,
    no_groups: bool,
) -> Vec<u8> {
    let mut output = Vec::with_capacity(bytes.len());
    let mut cursor = 0;
    while let Some(start) = bytes[cursor..].iter().position(|byte| *byte == b'{') {
        let start = cursor + start;
        output.extend_from_slice(&bytes[cursor..start]);
        let Some(end) = bytes[start + 1..].iter().position(|byte| *byte == b'}') else {
            cursor = start;
            break;
        };
        let end = start + 1 + end;
        let token = bytes[start + 1..end].to_ascii_lowercase();
        let replacement = find_replacement(vocabulary, &token, no_icons, no_groups);
        output.extend_from_slice(replacement.unwrap_or(&bytes[start..=end]));
        cursor = end + 1;
    }
    output.extend_from_slice(&bytes[cursor..]);
    output
}

fn find_replacement<'a>(
    vocabulary: &'a ChatExpressionInputs,
    token: &[u8],
    no_icons: bool,
    no_groups: bool,
) -> Option<&'a [u8]> {
    let icon = if no_icons {
        None
    } else {
        vocabulary.icons.get(token)
    };
    let group = if no_groups {
        None
    } else {
        vocabulary.groups.get(token)
    };
    icon.or(group).map(Vec::as_slice)
}
