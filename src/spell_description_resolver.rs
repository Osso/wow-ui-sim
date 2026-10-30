//! Resolve Blizzard spell-description tokens into simulator tooltip text.

use crate::lua_api::game_data;
use crate::lua_api::globals::spellbook_data;
use crate::lua_api::state::SimState;

pub fn resolve_spell_description(sim: &SimState, spell_id: u32) -> String {
    let Some(raw) = crate::spell_descriptions::get_spell_description(spell_id) else {
        return "No description available.".to_string();
    };
    resolve_text(sim, spell_id, raw, 0)
}

pub fn resolve_spell_description_or_empty(sim: &SimState, spell_id: u32) -> String {
    let Some(raw) = crate::spell_descriptions::get_spell_description(spell_id) else {
        return String::new();
    };
    resolve_text(sim, spell_id, raw, 0)
}

fn resolve_text(sim: &SimState, spell_id: u32, text: &str, depth: usize) -> String {
    if depth > 4 {
        return cleanup_control_tokens(text);
    }

    let expanded = expand_named_references(sim, text, depth);
    let expanded = replace_expressions(sim, spell_id, &expanded);
    let expanded = replace_angle_tokens(sim, spell_id, &expanded);
    let expanded = replace_dollar_tokens(sim, spell_id, &expanded);
    let expanded = replace_spell_conditionals(&expanded);
    let expanded = replace_class_conditionals(sim, &expanded);
    cleanup_control_tokens(&expanded)
}

fn spell_variables(spell_id: u32) -> &'static [(&'static str, &'static str)] {
    match spell_id {
        // SimulationCraft SpellDataDump/allspells.txt:
        // Avenger's Shield: $dmg=${$378286s1*(1+($378285s2/100))}
        31935 => &[("dmg", "$378286s1*(1+($378285s2/100))")],
        // Eye Beam: inactive talent conditionals collapse to the base
        // SimulationCraft branch, $dmg=${$198030s1*10}.
        198013 => &[("dmg", "$198030s1*10")],
        // Consecration: SimulationCraft has
        // $dmg=${$81297s1*$ceil($d/$t1)}. Consecration lasts 12 sec and
        // ticks every 1 sec in our data model.
        26573 => &[("dmg", "$81297s1*12")],
        // Crusader Strike: $damage=${$s1*$<retribution>}; inactive
        // talent/PvP conditionals collapse to a 1.0 multiplier here.
        35395 => &[("damage", "$s1")],
        // Shield of Vengeance: $shield=${$s2/100*$MHP*(1+$@versadmg)}
        184662 => &[("shield", "$s2/100*$MHP*(1+$@versadmg)")],
        _ => &[],
    }
}

fn named_variable_value(sim: &SimState, spell_id: u32, name: &str) -> Option<f64> {
    spell_variables(spell_id)
        .iter()
        .find_map(|(candidate, expression)| (*candidate == name).then_some(*expression))
        .and_then(|expression| {
            let numeric = replace_expression_variables(sim, spell_id, expression);
            evaluate_number_expression(&numeric)
        })
}

fn expand_named_references(sim: &SimState, text: &str, depth: usize) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(index) = rest.find("$@") {
        out.push_str(&rest[..index]);
        let token = &rest[index + 2..];
        rest = append_named_reference(sim, &mut out, token);
    }
    out.push_str(rest);
    if depth == 0 {
        out
    } else {
        resolve_text(sim, 0, &out, depth + 1)
    }
}

fn append_named_reference<'a>(sim: &SimState, out: &mut String, token: &'a str) -> &'a str {
    if let Some(id_text) = token.strip_prefix("spellname") {
        append_spell_name_reference(out, id_text);
        return remaining_named_reference(token, "spellname", id_text);
    }
    if let Some(id_text) = token.strip_prefix("spelldesc") {
        append_spell_desc_reference(sim, out, id_text);
        return remaining_named_reference(token, "spelldesc", id_text);
    }
    out.push_str("$@");
    token
}

fn append_spell_name_reference(out: &mut String, id_text: &str) {
    let (id, _) = parse_number_prefix(id_text);
    if let Some(name) = id.and_then(|id| crate::spells::get_spell(id).map(|spell| spell.name)) {
        out.push_str(name);
    }
}

fn append_spell_desc_reference(sim: &SimState, out: &mut String, id_text: &str) {
    let (id, _) = parse_number_prefix(id_text);
    if let Some(id) = id {
        out.push_str(&resolve_spell_description(sim, id));
    }
}

fn remaining_named_reference<'a>(token: &'a str, prefix: &str, id_text: &str) -> &'a str {
    let (_, consumed) = parse_number_prefix(id_text);
    &token[prefix.len() + consumed..]
}

fn replace_expressions(sim: &SimState, spell_id: u32, text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("${") {
        out.push_str(&rest[..start]);
        let expression_start = start + 2;
        let Some(end) = rest[expression_start..].find('}') else {
            out.push_str(&rest[start..]);
            return out;
        };
        let expression = &rest[expression_start..expression_start + end];
        let numeric = replace_expression_variables(sim, spell_id, expression);
        match evaluate_number_expression(&numeric) {
            Some(value) => out.push_str(&format_tooltip_value(value)),
            None => out.push_str(&cleanup_control_tokens(expression)),
        }
        rest = &rest[expression_start + end + 1..];
    }
    out.push_str(rest);
    out
}

fn replace_expression_variables(sim: &SimState, spell_id: u32, expression: &str) -> String {
    let mut out = String::with_capacity(expression.len());
    let mut chars = expression.char_indices().peekable();
    while let Some((index, ch)) = chars.next() {
        if ch != '$' {
            out.push(ch);
            continue;
        }

        let tail = &expression[index + 1..];
        if let Some((value, consumed)) = parse_angle_value_token(sim, spell_id, tail) {
            out.push_str(&value.to_string());
            while chars.peek().is_some_and(|(i, _)| *i < index + 1 + consumed) {
                chars.next();
            }
        } else if let Some((value, consumed)) = parse_value_token(sim, spell_id, tail) {
            out.push_str(&value.to_string());
            while chars.peek().is_some_and(|(i, _)| *i < index + 1 + consumed) {
                chars.next();
            }
        } else {
            out.push('0');
        }
    }
    out
}

fn replace_angle_tokens(sim: &SimState, spell_id: u32, text: &str) -> String {
    let damage = named_variable_value(sim, spell_id, "damage")
        .unwrap_or_else(|| spell_amount(sim, spell_id, 1));
    let shield = named_variable_value(sim, spell_id, "shield")
        .unwrap_or_else(|| shield_amount(sim, spell_id) as f64);
    let mut out = text.replace("$<damage>", &format_tooltip_value(damage));
    out = out.replace(
        "$<damageValue>",
        &format_tooltip_value(spell_amount(sim, spell_id, 1)),
    );
    out = out.replace("$<shield>", &format_tooltip_value(shield));

    for (name, _expression) in spell_variables(spell_id) {
        let token = format!("$<{name}>");
        if let Some(value) = named_variable_value(sim, spell_id, name) {
            out = out.replace(&token, &format_tooltip_value(value));
        }
    }

    out
}

fn replace_dollar_tokens(sim: &SimState, spell_id: u32, text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.char_indices().peekable();
    while let Some((index, ch)) = chars.next() {
        if ch != '$' {
            out.push(ch);
            continue;
        }

        let tail = &text[index + 1..];
        if tail.starts_with('?') {
            out.push_str("$?");
            chars.next();
            continue;
        }
        if let Some((value, consumed)) = parse_value_token(sim, spell_id, tail) {
            out.push_str(&format_tooltip_value(value));
            while chars.peek().is_some_and(|(i, _)| *i < index + 1 + consumed) {
                chars.next();
            }
            continue;
        }

        out.push('$');
    }
    out
}

fn parse_value_token(sim: &SimState, current_spell_id: u32, token: &str) -> Option<(f64, usize)> {
    if let Some(value) = parse_special_value_token(sim, token) {
        return Some(value);
    }

    let (referenced_id, digits) = parse_number_prefix(token);
    let spell_id = referenced_id.unwrap_or(current_spell_id);
    let suffix = &token[digits..];
    let kind = suffix.chars().next()?;
    if !matches!(kind, 's' | 'm' | 'x' | 'd' | 't' | 'A' | 'u' | 'h') {
        return None;
    }

    let (effect_index, effect_digits) = parse_number_prefix(&suffix[kind.len_utf8()..]);
    let effect_index = effect_index.unwrap_or(1);
    let consumed = digits + kind.len_utf8() + effect_digits;
    let value = match kind {
        'd' => spell_duration(spell_id),
        'A' => spell_radius(spell_id, effect_index),
        'x' => spell_count(spell_id, effect_index),
        't' => spell_tick_time(spell_id, effect_index),
        'u' => spell_stack_count(spell_id),
        'h' => spell_proc_chance(spell_id),
        _ => spell_amount(sim, spell_id, effect_index),
    };
    Some((value, consumed))
}

fn parse_special_value_token(sim: &SimState, token: &str) -> Option<(f64, usize)> {
    match token {
        value if value.starts_with("@versadmg") => {
            Some((sim.player.stats.versatility_pct() / 100.0, 9))
        }
        value if value.starts_with("MHP") => Some((sim.player.health_max as f64, 3)),
        value if value.starts_with("STR") => Some((sim.player.stats.strength, 3)),
        value if value.starts_with("INT") => Some((sim.player.stats.intellect, 3)),
        value if value.starts_with("AP") => Some((player_attack_power(sim), 2)),
        value if value.starts_with("pl") => Some((sim.player.level as f64, 2)),
        _ => None,
    }
}

fn parse_angle_value_token(sim: &SimState, spell_id: u32, token: &str) -> Option<(f64, usize)> {
    let variable = token.strip_prefix('<')?;
    let end = variable.find('>')?;
    let name = &variable[..end];
    named_variable_value(sim, spell_id, name).map(|value| (value, end + 2))
}

fn parse_number_prefix(text: &str) -> (Option<u32>, usize) {
    let count = text
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .map(char::len_utf8)
        .sum();
    if count == 0 {
        return (None, 0);
    }
    (text[..count].parse().ok(), count)
}

fn spell_amount(sim: &SimState, spell_id: u32, effect_index: u32) -> f64 {
    match (spell_id, effect_index) {
        (184662, 2) => 30.0,
        (633, 2) => 100.0,
        (31935, 1) => player_attack_power(sim) * 1.55,
        (35395, 1) => player_attack_power(sim) * 1.4,
        (53600, 1) => player_attack_power(sim) * 0.95,
        (378286, 1) => player_attack_power(sim) * 0.12,
        (198030, 1) => player_attack_power(sim) * 0.4026,
        (81297, 1) => player_attack_power(sim) * 0.05,
        (378285, 2) => 0.0,
        (209389, 1) => 60.0,
        (209389, 2) => 50.0,
        (198013, 5) => 5.0,
        (26573, 2) => 1.0,
        (19750, _) | (85673, _) | (130551, _) => 20_000.0,
        (82326, _) => 35_000.0,
        (25912, _) | (25914, _) | (20473, _) => 10_000.0,
        (132403, 1 | 2) => 160.0,
        _ => game_data::spell_effect_amount(spell_id) as f64,
    }
}

fn shield_amount(sim: &SimState, spell_id: u32) -> i32 {
    match spell_id {
        184662 => named_variable_value(sim, spell_id, "shield")
            .unwrap_or(sim.player.health_max as f64 * 0.30)
            .round() as i32,
        _ => game_data::spell_effect_amount(spell_id),
    }
}

fn player_attack_power(sim: &SimState) -> f64 {
    (sim.player.stats.strength + sim.player.stats.agility + sim.player.level as f64 * 10.0).max(0.0)
        as i32 as f64
}

fn spell_duration(spell_id: u32) -> f64 {
    match spell_id {
        642 => 8.0,
        853 => 6.0,
        62124 => 6.0,
        31935 => 3.0,
        198013 => 2.0,
        26573 => 12.0,
        31850 => 8.0,
        132403 => 4.5,
        209388 => 8.0,
        184662 => 15.0,
        86659 => 8.0,
        _ => 0.0,
    }
}

fn spell_radius(spell_id: u32, effect_index: u32) -> f64 {
    match (spell_id, effect_index) {
        (378286, 1) => 5.0,
        _ => 0.0,
    }
}

fn spell_count(spell_id: u32, effect_index: u32) -> f64 {
    match (spell_id, effect_index) {
        (31935, 1) => 3.0,
        _ => spell_amount_for_count(spell_id),
    }
}

fn spell_amount_for_count(_spell_id: u32) -> f64 {
    1.0
}

fn spell_tick_time(_spell_id: u32, _effect_index: u32) -> f64 {
    1.0
}

fn spell_stack_count(_spell_id: u32) -> f64 {
    1.0
}

fn spell_proc_chance(_spell_id: u32) -> f64 {
    0.0
}

fn evaluate_number_expression(expression: &str) -> Option<f64> {
    let tokens = tokenize_expression(expression)?;
    let mut parser = NumberExpressionParser { tokens, index: 0 };
    let value = parser.parse_sum()?;
    (parser.index == parser.tokens.len()).then_some(value)
}

#[derive(Clone, Copy)]
enum NumberToken {
    Number(f64),
    Plus,
    Minus,
    Star,
    Slash,
    LeftParen,
    RightParen,
}

fn tokenize_expression(expression: &str) -> Option<Vec<NumberToken>> {
    let mut tokens = Vec::new();
    let mut index = 0;
    let bytes = expression.as_bytes();
    while index < bytes.len() {
        let ch = bytes[index] as char;
        if let Some(token) = operator_token(ch) {
            tokens.push(token);
            index += 1;
        } else if ch == ' ' {
            index += 1;
        } else {
            let number = parse_number_token(expression, &mut index)?;
            tokens.push(NumberToken::Number(number));
        }
    }
    Some(tokens)
}

fn operator_token(ch: char) -> Option<NumberToken> {
    match ch {
        '+' => Some(NumberToken::Plus),
        '-' => Some(NumberToken::Minus),
        '*' => Some(NumberToken::Star),
        '/' => Some(NumberToken::Slash),
        '(' => Some(NumberToken::LeftParen),
        ')' => Some(NumberToken::RightParen),
        _ => None,
    }
}

fn parse_number_token(expression: &str, index: &mut usize) -> Option<f64> {
    let bytes = expression.as_bytes();
    let start = *index;
    match bytes.get(*index).copied().map(char::from)? {
        '0'..='9' | '.' => {
            *index += 1;
            while *index < bytes.len()
                && ((bytes[*index] as char).is_ascii_digit() || bytes[*index] == b'.')
            {
                *index += 1;
            }
            expression[start..*index].parse().ok()
        }
        _ => None,
    }
}

fn replace_class_conditionals(sim: &SimState, text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("$?c") {
        out.push_str(&rest[..start]);
        let token = &rest[start + 3..];
        let (class_id, digits) = parse_number_prefix(token);
        let Some(class_id) = class_id else {
            out.push_str("$?c");
            rest = token;
            continue;
        };
        let Some((true_branch, false_branch, consumed)) =
            parse_two_branch_conditional(&token[digits..])
        else {
            out.push_str("$?c");
            out.push_str(&token[..digits]);
            rest = &token[digits..];
            continue;
        };
        if sim.player.class_index as u32 == class_id {
            out.push_str(true_branch);
        } else {
            out.push_str(false_branch.unwrap_or(""));
        }
        rest = &token[digits + consumed..];
    }
    out.push_str(rest);
    out
}

fn replace_spell_conditionals(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("$?s") {
        out.push_str(&rest[..start]);
        let token = &rest[start + 3..];
        let (spell_id, digits) = parse_number_prefix(token);
        let Some(spell_id) = spell_id else {
            out.push_str("$?s");
            rest = token;
            continue;
        };
        let Some((true_branch, false_branch, consumed)) =
            parse_two_branch_conditional(&token[digits..])
        else {
            out.push_str("$?s");
            out.push_str(&token[..digits]);
            rest = &token[digits..];
            continue;
        };

        if spellbook_data::is_spell_known(spell_id) {
            out.push_str(true_branch);
        } else {
            out.push_str(false_branch.unwrap_or(""));
        }
        rest = &token[digits + consumed..];
    }
    out.push_str(rest);
    out
}

fn parse_two_branch_conditional(text: &str) -> Option<(&str, Option<&str>, usize)> {
    let (true_branch, true_consumed) = parse_bracketed(text)?;
    let remaining = &text[true_consumed..];
    let Some((false_branch, false_consumed)) = parse_bracketed(remaining) else {
        return Some((true_branch, None, true_consumed));
    };
    Some((
        true_branch,
        Some(false_branch),
        true_consumed + false_consumed,
    ))
}

fn parse_bracketed(text: &str) -> Option<(&str, usize)> {
    let body = text.strip_prefix('[')?;
    let end = body.find(']')?;
    Some((&body[..end], end + 2))
}

struct NumberExpressionParser {
    tokens: Vec<NumberToken>,
    index: usize,
}

impl NumberExpressionParser {
    fn parse_sum(&mut self) -> Option<f64> {
        let mut value = self.parse_product()?;
        while let Some(token) = self.current() {
            match token {
                NumberToken::Plus => {
                    self.index += 1;
                    value += self.parse_product()?;
                }
                NumberToken::Minus => {
                    self.index += 1;
                    value -= self.parse_product()?;
                }
                _ => break,
            }
        }
        Some(value)
    }

    fn parse_product(&mut self) -> Option<f64> {
        let mut value = self.parse_factor()?;
        while let Some(token) = self.current() {
            match token {
                NumberToken::Star => {
                    self.index += 1;
                    value *= self.parse_factor()?;
                }
                NumberToken::Slash => {
                    self.index += 1;
                    let denominator = self.parse_factor()?;
                    if denominator == 0.0 {
                        return None;
                    }
                    value /= denominator;
                }
                _ => break,
            }
        }
        Some(value)
    }

    fn parse_factor(&mut self) -> Option<f64> {
        match self.current()? {
            NumberToken::Number(value) => {
                self.index += 1;
                Some(value)
            }
            NumberToken::Minus => {
                self.index += 1;
                self.parse_factor().map(|value| -value)
            }
            NumberToken::LeftParen => {
                self.index += 1;
                let value = self.parse_sum()?;
                if matches!(self.current(), Some(NumberToken::RightParen)) {
                    self.index += 1;
                    Some(value)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn current(&self) -> Option<NumberToken> {
        self.tokens.get(self.index).copied()
    }
}

fn cleanup_control_tokens(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '$' && chars.peek().is_some_and(|next| *next == '?') {
            chars.next();
            while chars.peek().is_some_and(|next| *next != '[') {
                chars.next();
            }
            if chars.peek().is_some_and(|next| *next == '[') {
                chars.next();
            }
            continue;
        }
        if ch == '$' {
            continue;
        }
        if ch != '[' && ch != ']' {
            out.push(ch);
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn format_number(value: f64) -> String {
    if value.abs() >= 100.0 || (value.fract()).abs() < 0.001 {
        (value.round() as i64).to_string()
    } else {
        format!("{value:.1}")
    }
}

fn format_tooltip_value(value: f64) -> String {
    format_number(value)
}
