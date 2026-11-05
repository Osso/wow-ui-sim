//! WM97b/WM97c numbered item-level pipeline. Behavior evidence is Retail 70688;
//! records are the pinned 70391 export (WM97c lists seven tables whose rows differ,
//! so row equality is not claimed).
//!
//! Named seams: the player context is the `player` unit with the supplied
//! effective-level field (WM97c A); the expansion index uses the documented
//! native fallback; the global squish switch is read from its zero-filled file
//! image default (WM97c A: its runtime writer is UNKNOWN). The alternate level,
//! unsquished companion and owned-instance level policies have no consumer here.

use super::bonus::Accumulator;
use super::tables::{self, ItemScaling};
use super::{ItemDescriptor, unknown};

/// Native expansion-level bases for indices zero through eleven.
const EXPANSION_LEVEL_BASE: [i32; 12] = [30, 30, 30, 35, 35, 40, 45, 50, 60, 70, 80, 90];
const CURRENT_ERA_PATCH: i32 = 120_105;
/// WM97b: the documented native fallback when no provider value is available.
const FALLBACK_EXPANSION: i32 = 11;
const MIN_LEVEL: i32 = 1;
const MAX_LEVEL: i32 = 1300;

pub(super) fn resolve_level(
    accumulator: &Accumulator,
    descriptor: &ItemDescriptor,
    scaling: &ItemScaling,
    catalog_level: u16,
    player_level: i32,
) -> Result<i32, String> {
    let expansion = FALLBACK_EXPANSION;
    let (curve_input, ceiling) = core_input(descriptor, player_level, expansion)?;
    // WM97b step 1: preparation adds the accumulated delta to the selected base
    // level with 16-bit wrap; the curve branches add it themselves instead.
    let base = accumulator.base_level().unwrap_or(catalog_level);
    let prepared_base = resolve_prepared_base(base, accumulator.level_delta());
    let mut level = curve_or_base(
        accumulator,
        scaling,
        prepared_base,
        curve_input,
        ceiling,
        expansion,
    )?;
    level = reset_level(level.wrapping_add(accumulator.pre_conversion()));
    level = convert_eras(accumulator, scaling, level)?;
    level = reset_level(level.wrapping_add(accumulator.squish_delta(scaling)));
    Ok(working_level(level))
}

/// WM97b step 2-3: modifier 9 supplies the ceiling; otherwise the player level is
/// bounded through the preliminary ContentTuning reader.
fn core_input(
    descriptor: &ItemDescriptor,
    player_level: i32,
    expansion: i32,
) -> Result<(i32, i32), String> {
    let supplied = descriptor.modifier(9) as i32;
    let ceiling = if supplied != 0 {
        supplied
    } else {
        preliminary_bound(player_level, descriptor.modifier(28), expansion)?
    };
    let input = if ceiling != 0 && player_level > ceiling {
        ceiling
    } else {
        player_level
    };
    Ok((input, ceiling))
}

fn curve_or_base(
    accumulator: &Accumulator,
    scaling: &ItemScaling,
    prepared_base: u16,
    curve_input: i32,
    ceiling: i32,
    expansion: i32,
) -> Result<i32, String> {
    let player_curve = match accumulator.family_curve() {
        0 => scaling.player_level_curve,
        curve => curve,
    };
    if player_curve != 0 {
        let family_tuning = accumulator.family_tuning();
        let tuning = if family_tuning != 0 {
            family_tuning
        } else {
            scaling.content_tuning
        };
        let input = core_range_input(curve_input, tuning, expansion)?;
        let input = if ceiling != 0 && input > ceiling {
            ceiling
        } else {
            input
        };
        let level = reset_level(evaluate(player_curve, input)?);
        return Ok(level.wrapping_add(i32::from(accumulator.level_delta())));
    }
    let (offset_curve_id, mut input, offset) = offset_selection(accumulator, scaling)?;
    if offset_curve_id != 0 {
        if accumulator.uses_scaling_input() {
            input = curve_input;
        }
        if accumulator.tuning_clamps_offset() {
            input = core_range_input(input, accumulator.family_tuning(), expansion)?;
        }
        return Ok(evaluate(offset_curve_id, input)?.wrapping_add(offset));
    }
    Ok(i32::from(prepared_base))
}

/// WM97b step 6: the selected family curve precedes the description curve.
fn offset_selection(
    accumulator: &Accumulator,
    scaling: &ItemScaling,
) -> Result<(u32, i32, i32), String> {
    let family_curve = accumulator.family_offset_curve();
    if family_curve != 0 {
        return Ok((
            family_curve,
            accumulator.family_offset_input(),
            accumulator.offset_offset(),
        ));
    }
    if scaling.offset_curve != 0 {
        let row = tables::offset_curve(scaling.offset_curve)?;
        return Ok((row.curve_id, scaling.offset_item_level, row.offset));
    }
    Ok((0, 0, 0))
}

/// WM97b step 4: ContentTuning bounds with expansion-level modes.
fn tuning_bounds(tuning_id: u32, expansion: i32) -> Result<Option<(i32, i32)>, String> {
    if tuning_id == 0 {
        return Ok(None);
    }
    let Ok(row) = tables::content_tuning(tuning_id) else {
        // A missing tuning record retains the supplied level instead of failing.
        return Ok(None);
    };
    let lower = expansion_level(row.min_level, row.min_mode, expansion);
    let upper = expansion_level(row.max_level, row.max_mode, expansion);
    Ok(Some((
        lower
            .wrapping_add(row.allowed_min_offset)
            .clamp(MIN_LEVEL, 123),
        upper,
    )))
}

fn expansion_level(base: i32, mode: u8, expansion: i32) -> i32 {
    match mode {
        1 => base.wrapping_add(1),
        2 => base.wrapping_add(expansion_base(expansion)),
        3 => base.wrapping_add(expansion_base(expansion.saturating_sub(1))),
        _ => base,
    }
}

fn expansion_base(expansion: i32) -> i32 {
    EXPANSION_LEVEL_BASE[expansion.clamp(0, 11) as usize]
}

fn range_enabled(tuning_id: u32) -> bool {
    match tables::content_tuning(tuning_id) {
        Ok(row) => {
            row.flags & 4 == 0
                && (row.min_level != 0
                    || row.max_level != 0
                    || row.min_mode != 0
                    || row.max_mode != 0)
        }
        Err(_) => false,
    }
}

/// WM97b step 5: bounds apply only when the found record passes the range-enabled test.
fn core_range_input(input: i32, tuning_id: u32, expansion: i32) -> Result<i32, String> {
    if !range_enabled(tuning_id) {
        return Ok(input);
    }
    Ok(apply_bounds(input, tuning_bounds(tuning_id, expansion)?))
}

/// WM97b step 2: the preliminary reader bounds without the range-enabled test.
fn preliminary_bound(input: i32, tuning_id: u32, expansion: i32) -> Result<i32, String> {
    Ok(apply_bounds(input, tuning_bounds(tuning_id, expansion)?))
}

fn apply_bounds(input: i32, bounds: Option<(i32, i32)>) -> i32 {
    let Some((lower, upper)) = bounds else {
        return input;
    };
    if input < lower {
        lower
    } else if input > upper {
        upper
    } else {
        input
    }
}

/// WM97b: convert, evaluate, round halfway away from zero, then truncate.
fn evaluate(curve_id: u32, input: i32) -> Result<i32, String> {
    let curve = tables::curve(curve_id)?;
    let value = curve_value(curve, input as f32)?;
    Ok(round_half_away(value) as i64 as i32)
}

fn curve_value(curve: &tables::Curve, input: f32) -> Result<f32, String> {
    match curve.kind {
        0 => Ok(linear(curve, input)),
        5 => Ok(previous_point(curve, input)),
        kind => Err(unknown(format!(
            "curve type {kind} is not implemented; WM97b curve types are partially modelled"
        ))),
    }
}

/// WM97b: left value plus the difference times the normalized input difference.
fn linear(curve: &tables::Curve, input: f32) -> f32 {
    let points = &curve.points;
    let (Some(first), Some(last)) = (points.first(), points.last()) else {
        return 0.0;
    };
    if input <= first.0 {
        return first.1;
    }
    if input >= last.0 {
        return last.1;
    }
    let index = points.partition_point(|point| point.0 <= input) - 1;
    let (left_x, left_y) = points[index];
    let (right_x, right_y) = points[index + 1];
    let width = right_x - left_x;
    if width <= 0.0 {
        return right_y;
    }
    left_y + (right_y - left_y) * ((input - left_x) / width)
}

/// WM97b curve type 5: the interval selects its previous point.
fn previous_point(curve: &tables::Curve, input: f32) -> f32 {
    let points = &curve.points;
    if points.is_empty() {
        return 0.0;
    }
    let index = points.partition_point(|point| point.0 <= input);
    points[index.saturating_sub(1).min(points.len() - 1)].1
}

fn round_half_away(value: f32) -> f32 {
    if value < 0.0 {
        (value - 0.5).trunc()
    } else {
        (value + 0.5).trunc()
    }
}

fn reset_level(level: i32) -> i32 {
    if (MIN_LEVEL..=MAX_LEVEL).contains(&level) {
        level
    } else {
        MIN_LEVEL
    }
}

/// WM97b step 9: apply each following era curve through and including the target.
fn convert_eras(
    accumulator: &Accumulator,
    scaling: &ItemScaling,
    level: i32,
) -> Result<i32, String> {
    if accumulator.disables_era() || !global_squish_enabled() {
        return Ok(level);
    }
    let eras = tables::squish_eras();
    let Some(target) = eras.iter().rposition(|row| row.patch <= CURRENT_ERA_PATCH) else {
        return Ok(level);
    };
    let explicit_era = accumulator.era();
    let source_id = if explicit_era > 0 {
        explicit_era as u32
    } else {
        scaling.squish_era.max(1)
    };
    let Some(source) = tables::squish_era_index(source_id) else {
        return Ok(level);
    };
    if source >= target {
        return Ok(level);
    }
    let mut level = level;
    for row in &eras[source + 1..=target] {
        level = evaluate(row.curve_id, level)?;
    }
    Ok(level)
}

/// WM97c A: the global lies in a zero-filled image section, so the image default is
/// zero (conversion disabled). Its runtime writer is UNKNOWN, so this simulator does
/// not assume a nonzero live value.
fn global_squish_enabled() -> bool {
    false
}

fn resolve_prepared_base(base: u16, delta: i16) -> u16 {
    let level = i32::from(base).wrapping_add(i32::from(delta));
    if (1..=1300).contains(&level) {
        level as u16
    } else {
        1
    }
}

fn working_level(level: i32) -> i32 {
    // WM97b step 11: wrapping subtraction, not a wider clamp.
    let shifted = level.wrapping_sub(1);
    if (shifted as u32) <= 1299 {
        level
    } else if shifted < 0 {
        1
    } else {
        1300
    }
}
