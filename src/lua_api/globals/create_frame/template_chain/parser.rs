use super::{FastHandlerRef, FastLiteralValue};

#[path = "parser_args.rs"]
mod parser_args;
#[path = "parser_function_family.rs"]
mod parser_function_family;
#[path = "parser_global_family.rs"]
mod parser_global_family;
#[path = "parser_inline_sequence.rs"]
mod parser_inline_sequence;
#[path = "parser_method_family.rs"]
mod parser_method_family;

use self::parser_args::split_top_level_args;
use self::parser_function_family::parse_function_family;
use self::parser_global_family::{
    parse_global_family, parse_global_tooltip_set_owner_then_set_text,
};
use self::parser_inline_sequence::split_inline_sequence_parts;
use self::parser_method_family::parse_method_family;

pub(super) fn parse_inline_fast_handler<'a>(
    _handler_name: &'static str,
    body: &'a str,
) -> Option<FastHandlerRef<'a>> {
    let trimmed = strip_leading_comment_lines(body.trim());
    if trimmed.is_empty() {
        return Some(FastHandlerRef::NoOp);
    }
    let stmt = trimmed.strip_suffix(';').unwrap_or(trimmed).trim();
    if stmt.is_empty() {
        return Some(FastHandlerRef::NoOp);
    }
    if let Some(handler) = parse_pre_split_special_handler(stmt) {
        return Some(handler);
    }
    if let Some(sequence) = parse_inline_sequence(stmt) {
        return Some(sequence);
    }
    parse_inline_single_fast_handler(stmt)
}

fn parse_pre_split_special_handler<'a>(stmt: &'a str) -> Option<FastHandlerRef<'a>> {
    parse_play_sound_then_copy_club_ticket(stmt)
        .or_else(|| parse_prefix_conditional_suffix_sequence(stmt))
        .or_else(|| parse_global_tooltip_set_owner_then_function_text(stmt))
        .or_else(|| parse_global_tooltip_set_owner_then_parent_assign(stmt))
        .or_else(|| parse_parent_field_local_click_if_enabled(stmt))
}

fn parse_play_sound_then_copy_club_ticket<'a>(stmt: &'a str) -> Option<FastHandlerRef<'a>> {
    let _ = stmt;
    None
}

fn parse_parent_field_local_click_if_enabled<'a>(stmt: &'a str) -> Option<FastHandlerRef<'a>> {
    let stmt = stmt.trim();
    let remainder = stmt.strip_prefix("local ")?;
    let (local_name, remainder) = remainder.split_once('=')?;
    let local_name = local_name.trim();
    if !is_fast_identifier(local_name) {
        return None;
    }
    let (target_expr, remainder) = remainder.split_once(';')?;
    let field = target_expr.trim().strip_prefix("self:GetParent().")?.trim();
    if !is_fast_identifier(field) {
        return None;
    }
    let remainder = remainder.trim();
    let prefix = format!("if {local_name}:IsEnabled() then");
    let remainder = remainder.strip_prefix(&prefix)?.trim_start();
    let body = remainder.strip_suffix("end")?.trim();
    let expected = format!("{local_name}:GetScript(\"OnClick\")({local_name});");
    (body == expected || body == expected.trim_end_matches(';'))
        .then_some(FastHandlerRef::ParentFieldLocalClickIfEnabled { field })
}

fn parse_prefix_conditional_suffix_sequence<'a>(stmt: &'a str) -> Option<FastHandlerRef<'a>> {
    let (first_stmt, rest) = stmt.split_once(';')?;
    let first_stmt = first_stmt.trim();
    let rest = rest.trim();
    if !(rest.starts_with("if ") || rest.starts_with("if(")) {
        return None;
    }

    let end_idx = rest.rfind("end")?;
    let conditional_stmt = rest[..end_idx + 3].trim();
    let tail_stmt = rest[end_idx + 3..].trim();
    if tail_stmt.is_empty() {
        return None;
    }

    let first_ref = parse_inline_single_fast_handler(first_stmt)?;
    let conditional_ref = parse_inline_single_fast_handler(conditional_stmt)?;
    let tail_ref = parse_inline_single_fast_handler(tail_stmt)?;
    Some(FastHandlerRef::Sequence3(Box::new((
        first_ref,
        conditional_ref,
        tail_ref,
    ))))
}

fn parse_global_tooltip_set_owner_then_parent_assign<'a>(
    stmt: &'a str,
) -> Option<FastHandlerRef<'a>> {
    let (tooltip_stmt, assign_stmt) = stmt.rsplit_once(';')?;
    let tooltip_stmt = tooltip_stmt.trim();
    let assign_stmt = assign_stmt.trim();
    let (target_path, anchor, text_path, red_path, green_path, blue_path, wrap) =
        parse_global_tooltip_set_owner_then_set_text(tooltip_stmt)?;
    let FastHandlerRef::AssignParentField { field, value } =
        parse_inline_parent_assignment(assign_stmt)?
    else {
        return None;
    };
    Some(FastHandlerRef::Sequence2(Box::new((
        FastHandlerRef::GlobalTooltipSetOwnerThenSetText {
            target_path,
            anchor,
            text_path,
            red_path,
            green_path,
            blue_path,
            wrap,
        },
        FastHandlerRef::AssignParentField { field, value },
    ))))
}

fn parse_global_tooltip_set_owner_then_function_text<'a>(
    stmt: &'a str,
) -> Option<FastHandlerRef<'a>> {
    let (first_stmt, second_stmt) = stmt.split_once(';')?;
    let (target_path, method_name, arg) = parse_set_owner_self_string_arg(first_stmt.trim())?;
    let second_ref = parse_matching_function_result_set_text(second_stmt, target_path)?;
    Some(FastHandlerRef::Sequence2(Box::new((
        FastHandlerRef::GlobalMethodWithSelfStringArg {
            target_path,
            method_name,
            arg,
        },
        second_ref,
    ))))
}

fn parse_set_owner_self_string_arg<'a>(stmt: &'a str) -> Option<(&'a str, &'a str, &'a str)> {
    let FastHandlerRef::GlobalMethodWithSelfStringArg {
        target_path,
        method_name,
        arg,
    } = parse_inline_single_fast_handler(stmt)?
    else {
        return None;
    };
    (method_name == "SetOwner").then_some((target_path, method_name, arg))
}

fn parse_matching_function_result_set_text<'a>(
    stmt: &'a str,
    target_path: &'a str,
) -> Option<FastHandlerRef<'a>> {
    let stmt = stmt.trim().trim_end_matches(';').trim();
    match parse_inline_single_fast_handler(stmt)? {
        FastHandlerRef::GlobalMethodWithStringStringFunctionResultAndThreeNumberArgs { .. } => {
            accept_string_string_function_result_set_text(stmt, target_path)
        }
        FastHandlerRef::GlobalMethodWithGlobalStringFunctionResultAndThreeNumberArgs { .. } => {
            accept_global_string_function_result_set_text(stmt, target_path)
        }
        _ => None,
    }
}

fn accept_string_string_function_result_set_text<'a>(
    stmt: &'a str,
    target_path: &'a str,
) -> Option<FastHandlerRef<'a>> {
    let FastHandlerRef::GlobalMethodWithStringStringFunctionResultAndThreeNumberArgs {
        target_path: text_target_path,
        method_name: text_method_name,
        function_name,
        first,
        second,
        third,
        fourth,
        fifth,
    } = parse_inline_single_fast_handler(stmt)?
    else {
        return None;
    };
    is_matching_set_text(text_target_path, text_method_name, target_path).then_some(
        FastHandlerRef::GlobalMethodWithStringStringFunctionResultAndThreeNumberArgs {
            target_path: text_target_path,
            method_name: text_method_name,
            function_name,
            first,
            second,
            third,
            fourth,
            fifth,
        },
    )
}

fn accept_global_string_function_result_set_text<'a>(
    stmt: &'a str,
    target_path: &'a str,
) -> Option<FastHandlerRef<'a>> {
    let FastHandlerRef::GlobalMethodWithGlobalStringFunctionResultAndThreeNumberArgs {
        target_path: text_target_path,
        method_name: text_method_name,
        function_name,
        first_arg_path,
        second,
        third,
        fourth,
        fifth,
    } = parse_inline_single_fast_handler(stmt)?
    else {
        return None;
    };
    is_matching_set_text(text_target_path, text_method_name, target_path).then_some(
        FastHandlerRef::GlobalMethodWithGlobalStringFunctionResultAndThreeNumberArgs {
            target_path: text_target_path,
            method_name: text_method_name,
            function_name,
            first_arg_path,
            second,
            third,
            fourth,
            fifth,
        },
    )
}

fn is_matching_set_text(text_target_path: &str, text_method_name: &str, target_path: &str) -> bool {
    text_target_path == target_path && text_method_name == "SetText"
}

fn parse_inline_single_fast_handler<'a>(stmt: &'a str) -> Option<FastHandlerRef<'a>> {
    parse_method_family(stmt)
        .or_else(|| parse_global_family(stmt))
        .or_else(|| parse_registration_family(stmt))
        .or_else(|| parse_assignment_family(stmt))
        .or_else(|| parse_function_family(stmt))
}

fn parse_registration_family<'a>(stmt: &'a str) -> Option<FastHandlerRef<'a>> {
    if let Some((first, second, third)) = parse_inline_register_for_clicks(stmt) {
        return Some(FastHandlerRef::RegisterForClicks {
            first,
            second,
            third,
        });
    }
    if let Some(button) = parse_inline_register_for_drag(stmt) {
        return Some(FastHandlerRef::RegisterForDrag(button));
    }
    if let Some(alpha) = parse_inline_set_alpha(stmt) {
        return Some(FastHandlerRef::SetAlpha(alpha));
    }
    parse_inline_set_frame_level_from_parent(stmt).map(FastHandlerRef::SetFrameLevelFromParent)
}

fn parse_assignment_family<'a>(stmt: &'a str) -> Option<FastHandlerRef<'a>> {
    parse_inline_ancestor_assignment(stmt)
        .or_else(|| parse_inline_global_assignment(stmt))
        .or_else(|| parse_inline_assignment(stmt))
        .or_else(|| parse_inline_nested_global_pair_table_assignment(stmt))
        .or_else(|| parse_inline_nested_assignment(stmt))
        .or_else(|| parse_inline_parent_assignment(stmt))
}

fn parse_inline_sequence(stmt: &str) -> Option<FastHandlerRef<'_>> {
    let parts = split_inline_sequence_parts(stmt);
    match parts.as_slice() {
        [first, second] => Some(FastHandlerRef::Sequence2(Box::new((
            parse_inline_single_fast_handler(first)?,
            parse_inline_single_fast_handler(second)?,
        )))),
        [first, second, third] => Some(FastHandlerRef::Sequence3(Box::new((
            parse_inline_single_fast_handler(first)?,
            parse_inline_single_fast_handler(second)?,
            parse_inline_single_fast_handler(third)?,
        )))),
        [first, second, third, fourth] => Some(FastHandlerRef::Sequence4(Box::new((
            parse_inline_single_fast_handler(first)?,
            parse_inline_single_fast_handler(second)?,
            parse_inline_single_fast_handler(third)?,
            parse_inline_single_fast_handler(fourth)?,
        )))),
        _ => None,
    }
}

fn parse_inline_register_for_clicks(stmt: &str) -> Option<(&str, Option<&str>, Option<&str>)> {
    let args = stmt
        .strip_prefix("self:RegisterForClicks(")?
        .strip_suffix(')')?
        .trim();
    let args = parse_string_literal_args(args)?;
    match args.as_slice() {
        [first] => Some((first, None, None)),
        [first, second] => Some((first, Some(second), None)),
        [first, second, third] => Some((first, Some(second), Some(third))),
        _ => None,
    }
}

fn parse_inline_register_for_drag(stmt: &str) -> Option<&str> {
    let args = stmt
        .strip_prefix("self:RegisterForDrag(")?
        .strip_suffix(')')?
        .trim();
    let args = parse_string_literal_args(args)?;
    match args.as_slice() {
        [button] => Some(button),
        _ => None,
    }
}

fn parse_inline_set_alpha(stmt: &str) -> Option<f64> {
    stmt.strip_prefix("self:SetAlpha(")?
        .strip_suffix(')')?
        .trim()
        .parse::<f64>()
        .ok()
}

fn parse_inline_set_frame_level_from_parent(stmt: &str) -> Option<i32> {
    let remainder = stmt
        .strip_prefix("self:SetFrameLevel(self:GetParent():GetFrameLevel()")?
        .trim();
    if let Some(remainder) = remainder.strip_suffix(')') {
        let remainder = remainder.trim();
        if remainder.is_empty() {
            return Some(0);
        }
        if let Some(delta) = remainder.strip_prefix('+') {
            return delta.trim().parse::<i32>().ok();
        }
        if let Some(delta) = remainder.strip_prefix('-') {
            return delta.trim().parse::<i32>().ok().map(|delta| -delta);
        }
    }
    None
}

fn parse_inline_ancestor_assignment(stmt: &str) -> Option<FastHandlerRef<'_>> {
    let (field, depth) =
        if let Some((field, _)) = stmt.strip_prefix("self.")?.split_once("= self:GetParent()") {
            let field = field.trim();
            if let Some(suffix) = stmt.trim().strip_prefix(&format!("self.{field} = ")) {
                let depth = if suffix.trim() == "self:GetParent()" {
                    1
                } else if suffix.trim() == "self:GetParent():GetParent()" {
                    2
                } else {
                    return None;
                };
                (field, depth)
            } else {
                return None;
            }
        } else {
            return None;
        };
    if !is_fast_identifier(field) {
        return None;
    }
    Some(FastHandlerRef::AssignAncestorRef { field, depth })
}

fn parse_inline_assignment(stmt: &str) -> Option<FastHandlerRef<'_>> {
    let (field, raw_value) = stmt.strip_prefix("self.")?.split_once('=')?;
    let field = field.trim();
    let raw_value = raw_value.trim();
    if !is_fast_identifier(field) {
        return None;
    }
    let value = parse_fast_literal_value(raw_value)?;
    Some(FastHandlerRef::AssignLiteral { field, value })
}

fn parse_inline_global_assignment(stmt: &str) -> Option<FastHandlerRef<'_>> {
    let (lhs, raw_value) = stmt.split_once('=')?;
    let lhs = lhs.trim();
    let raw_value = raw_value.trim();
    if lhs.starts_with("self.") || lhs.starts_with("self:GetParent().") {
        return None;
    }
    let (target_path, field) = lhs.rsplit_once('.')?;
    let target_path = target_path.trim();
    let field = field.trim();
    if !(is_fast_handler_path(target_path) && is_fast_identifier(field)) {
        return None;
    }
    let value = parse_fast_literal_value(raw_value)?;
    Some(FastHandlerRef::AssignGlobalFieldLiteral {
        target_path,
        field,
        value,
    })
}

fn parse_inline_parent_assignment(stmt: &str) -> Option<FastHandlerRef<'_>> {
    let (field, raw_value) = stmt.strip_prefix("self:GetParent().")?.split_once('=')?;
    let field = field.trim();
    let raw_value = raw_value.trim();
    if !is_fast_identifier(field) {
        return None;
    }
    let value = parse_fast_literal_value(raw_value)?;
    Some(FastHandlerRef::AssignParentField { field, value })
}

fn parse_string_literal_args(args: &str) -> Option<Vec<&str>> {
    if args.is_empty() {
        return Some(Vec::new());
    }
    let mut values = Vec::new();
    for part in args.split(',') {
        values.push(parse_single_string_literal(part.trim())?);
    }
    Some(values)
}

fn parse_single_string_literal(arg: &str) -> Option<&str> {
    let value = arg.strip_prefix('"')?.strip_suffix('"')?;
    // A quote inside the stripped value means `arg` was not one string
    // literal (e.g. the two-arg list `"LeftButtonUp", "RightButtonUp"`).
    // Fusing it would pass garbage to the method (mount-list rows ended up
    // registered for a click edge that can never match). Reject so the
    // multi-arg parsers or the compiled-Lua slow path handle the statement.
    (!value.contains('"')).then_some(value)
}

fn parse_single_bool_literal(arg: &str) -> Option<bool> {
    match arg {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn strip_leading_comment_lines(mut stmt: &str) -> &str {
    loop {
        let trimmed = stmt.trim_start();
        let Some(comment) = trimmed.strip_prefix("--") else {
            return trimmed;
        };
        let Some((_, rest)) = comment.split_once('\n') else {
            return "";
        };
        stmt = rest;
    }
}

fn is_fast_handler_path(path: &str) -> bool {
    path.split('.').all(is_fast_identifier)
}

fn is_fast_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    match chars.next() {
        Some(ch) if ch == '_' || ch.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn is_fast_passthrough_args(args: &str) -> bool {
    args.split(',')
        .map(str::trim)
        .all(|arg| arg.is_empty() || arg == "..." || is_fast_identifier(arg))
}

fn is_fast_global_path(path: &str) -> bool {
    let root = path.split('.').next();
    root != Some("self") && is_fast_handler_path(path)
}

fn parse_fast_literal_value(raw_value: &str) -> Option<FastLiteralValue<'_>> {
    if raw_value.eq("nil") {
        Some(FastLiteralValue::Nil)
    } else if raw_value.eq("true") {
        Some(FastLiteralValue::Bool(true))
    } else if raw_value.eq("false") {
        Some(FastLiteralValue::Bool(false))
    } else if let Ok(number) = raw_value.parse::<f64>() {
        Some(FastLiteralValue::Number(number))
    } else if is_fast_global_path(raw_value) {
        Some(FastLiteralValue::Global(raw_value))
    } else {
        None
    }
}

fn parse_inline_nested_assignment(stmt: &str) -> Option<FastHandlerRef<'_>> {
    let (lhs, rhs) = stmt.split_once('=')?;
    let lhs = lhs.trim();
    let rhs = rhs.trim();
    let lhs = lhs.strip_prefix("self.")?;
    let (parent_field, field) = lhs.split_once('.')?;
    let value = parse_fast_literal_value(rhs)?;
    (is_fast_identifier(parent_field) && is_fast_identifier(field)).then_some(
        FastHandlerRef::AssignNestedLiteral {
            parent_field,
            field,
            value,
        },
    )
}

fn parse_inline_nested_global_pair_table_assignment(stmt: &str) -> Option<FastHandlerRef<'_>> {
    let (lhs, rhs) = stmt.split_once('=')?;
    let lhs = lhs.trim().strip_prefix("self.")?;
    let (parent_field, field) = lhs.split_once('.')?;
    if !(is_fast_identifier(parent_field) && is_fast_identifier(field)) {
        return None;
    }

    let rhs = rhs.trim().strip_prefix('{')?.strip_suffix('}')?.trim();
    let mut parts = rhs.split(',').map(str::trim);
    let first_path = parts.next()?;
    let second_path = parts.next()?;
    if parts.next().is_some() {
        return None;
    }

    (is_fast_handler_path(first_path) && is_fast_handler_path(second_path)).then_some(
        FastHandlerRef::AssignNestedGlobalPairTable {
            parent_field,
            field,
            first_path,
            second_path,
        },
    )
}
