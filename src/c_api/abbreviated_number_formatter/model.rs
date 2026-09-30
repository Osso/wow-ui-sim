//! Breakpoint and rendering policies are inferred from localization structure docs.
use rilua::{LuaResult, runtime_error};

#[derive(Clone, Debug)]
pub(super) struct Breakpoint {
    pub threshold: f64,
    pub abbreviation: String,
    pub significand: f64,
    pub fraction: f64,
    pub global: bool,
    pub secret: bool,
}

pub(super) fn validate_number(number: f64) -> LuaResult<f64> {
    if number.is_finite() && number > 0.0 && (number == 1.0 || number % 10.0 == 0.0) {
        Ok(number)
    } else {
        Err(runtime_error(
            "abbreviation breakpoint/divisors must be positive finite multiples of ten (or one)",
        ))
    }
}

pub(super) fn sort_breakpoints(mut rows: Vec<Breakpoint>) -> LuaResult<Vec<Breakpoint>> {
    rows.sort_by(|left, right| left.threshold.total_cmp(&right.threshold));
    if rows
        .windows(2)
        .any(|pair| pair[0].threshold == pair[1].threshold)
    {
        return Err(runtime_error(
            "duplicate abbreviation breakpoints are unsupported",
        ));
    }
    Ok(rows)
}

pub(super) fn english_defaults() -> Vec<Breakpoint> {
    [(1000.0, "k"), (1_000_000.0, "m"), (1_000_000_000.0, "b")]
        .into_iter()
        .flat_map(|(magnitude, suffix)| {
            [
                (magnitude, magnitude / 10.0, 10.0),
                (magnitude * 10.0, magnitude, 1.0),
            ]
            .map(|(threshold, significand, fraction)| Breakpoint {
                threshold,
                abbreviation: suffix.to_owned(),
                significand,
                fraction,
                global: false,
                secret: false,
            })
        })
        .collect()
}

pub(super) fn render_number(number: f64, row: Option<&Breakpoint>) -> LuaResult<String> {
    if !number.is_finite() {
        return Err(runtime_error(
            "abbreviated formatter requires a finite number",
        ));
    }
    let Some(row) = row else {
        return Ok(number.to_string());
    };
    // Structure docs' 1234 -> 1.2k and 12345 -> 12k imply truncation.
    let units = (number.abs() / row.significand).floor();
    let rounded = units / row.fraction;
    let signed = if number < 0.0 { -rounded } else { rounded };
    Ok(signed.to_string())
}
