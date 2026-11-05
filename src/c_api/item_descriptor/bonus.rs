//! WM97/WM97b/WM97c bonus dispatch. Behavior evidence is Retail 70688; records are
//! the pinned 70391 export, so row equality to 70688 is not claimed.
//!
//! Implemented rows are those with a verified effect on fields this simulator
//! models: level delta, quality, requirement, stats and the level/curve family.
//! Rows whose verified destination is a scaling record are resolved from the
//! pinned tables. Unknown curve types and absent records raise named errors.
//! Types 4/5/6/7/10/12/16–23/28/29/31/33/34/35/37–47 affect fields this
//! tooltip layer does not model (sockets, effects, title suffixes, flags); they
//! are recorded here as gaps rather than silently reinterpreted.

use super::scaling;
use super::tables::{self, BonusRow};
use super::{ItemDescriptor, unknown};
use crate::items::ItemInfo;

const DEFAULT_PRIORITY: i32 = i32::MAX;

/// Curve family state selected by Types 11/13/48/49/51.
pub(super) struct Family {
    priority: i32,
    curve: u32,
    tuning: u32,
    offset_curve: u32,
    offset_input: i32,
    era: i32,
    config: u32,
}

/// Modes written by the configuration reader and curve-family winners.
struct Modes {
    scaling_input: bool,
    config_item_level: bool,
    requirement_copy: bool,
    disable_era: bool,
    tuning_clamp: bool,
    offset: i32,
}

struct Selected<T> {
    priority: i32,
    value: Option<T>,
}

impl<T> Selected<T> {
    fn new() -> Self {
        Self {
            priority: DEFAULT_PRIORITY,
            value: None,
        }
    }

    /// WM97b: a strictly smaller priority wins; finite ties keep the old payload.
    fn offer(&mut self, priority: i32, value: T) {
        if priority < self.priority {
            self.priority = priority;
            self.value = Some(value);
        }
    }
}

pub(super) struct Accumulator {
    level_delta: i16,
    quality: u8,
    quality_applied: bool,
    required_level: Option<u8>,
    base_required_level: u8,
    stats: [(i16, u16); 10],
    family: Family,
    base_level: Selected<i32>,
    pre_conversion: Selected<i32>,
    squish: Selected<[i32; 4]>,
    fixed_priority: i32,
    modes: Modes,
}

impl Accumulator {
    fn empty() -> Self {
        Self {
            level_delta: 0,
            quality: 0,
            quality_applied: false,
            required_level: None,
            base_required_level: 0,
            stats: [(0, 0); 10],
            family: Family {
                priority: DEFAULT_PRIORITY,
                curve: 0,
                tuning: 0,
                offset_curve: 0,
                offset_input: 0,
                era: -1,
                config: 0,
            },
            base_level: Selected::new(),
            pre_conversion: Selected::new(),
            squish: Selected::new(),
            fixed_priority: DEFAULT_PRIORITY,
            modes: Modes {
                scaling_input: false,
                config_item_level: false,
                requirement_copy: false,
                disable_era: false,
                tuning_clamp: false,
                offset: 0,
            },
        }
    }

    fn new(item: &ItemInfo) -> Self {
        let mut accumulator = Self::empty();
        for (slot, entry) in accumulator.stats.iter_mut().enumerate() {
            *entry = (
                item.stat_modifier_bonus_stat[slot],
                item.stat_percent_editor[slot],
            );
        }
        accumulator.quality = item.quality;
        accumulator.base_required_level = item.required_level.min(255) as u8;
        accumulator
    }

    pub(super) fn level_delta(&self) -> i16 {
        self.level_delta
    }

    pub(super) fn base_level(&self) -> Option<u16> {
        self.base_level.value.map(|level| level as u16)
    }

    pub(super) fn pre_conversion(&self) -> i32 {
        self.pre_conversion.value.unwrap_or(0)
    }

    pub(super) fn family_curve(&self) -> u32 {
        self.family.curve
    }

    pub(super) fn family_tuning(&self) -> u32 {
        self.family.tuning
    }

    pub(super) fn family_offset_curve(&self) -> u32 {
        self.family.offset_curve
    }

    pub(super) fn family_offset_input(&self) -> i32 {
        self.family.offset_input
    }

    pub(super) fn offset_offset(&self) -> i32 {
        self.modes.offset
    }

    pub(super) fn uses_scaling_input(&self) -> bool {
        self.modes.scaling_input
    }

    pub(super) fn tuning_clamps_offset(&self) -> bool {
        self.modes.tuning_clamp
    }

    pub(super) fn disables_era(&self) -> bool {
        self.modes.disable_era
    }

    pub(super) fn era(&self) -> i32 {
        self.family.era
    }

    /// WM97b Type 52 chooser: intervening eras with the squish-disable flag clear
    /// select `Value_2`; otherwise `Value_0`. The selected delta is narrowed to i16.
    pub(super) fn squish_delta(&self, scaling: &tables::ItemScaling) -> i32 {
        let Some(values) = self.squish.value else {
            return 0;
        };
        let eras = tables::squish_eras();
        let Some(target) = eras.iter().rposition(|row| row.patch <= 120_105) else {
            return 0;
        };
        let explicit_era = self.family.era;
        let source_id = if explicit_era > 0 {
            explicit_era as u32
        } else {
            scaling.squish_era.max(1)
        };
        let Some(source) = tables::squish_era_index(source_id) else {
            return 0;
        };
        let chosen = if source >= target {
            values[0]
        } else if eras[source + 1..=target]
            .iter()
            .any(|row| row.flags & 1 == 0)
        {
            values[2]
        } else {
            values[0]
        };
        i32::from(chosen as i16)
    }
}

/// WM104: one fresh accumulator per relic contribution call; the three effect
/// slots apply their lists in order and repeated lists apply again.
pub(super) fn contribution_accumulator() -> Accumulator {
    Accumulator::empty()
}

/// WM104: the returned number is the absolute-level slot written by Type 42.
pub(super) fn contribution_selected_level(accumulator: &Accumulator) -> u16 {
    accumulator.base_level().unwrap_or(0)
}

pub(super) fn apply_contribution_list(
    list_id: u32,
    accumulator: &mut Accumulator,
) -> Result<(), String> {
    apply_list(list_id, accumulator, 0)
}

pub(super) fn apply(
    descriptor: &ItemDescriptor,
    item: &mut ItemInfo,
    player_level: i32,
) -> Result<(), String> {
    let mut accumulator = Accumulator::new(item);
    if let Some(context) = &descriptor.bonus_context {
        for list_id in &context.bonus_list_ids {
            apply_list(*list_id, &mut accumulator, 0)?;
        }
    }
    if accumulator.quality_applied {
        item.quality = accumulator.quality;
    }
    for (slot, (stat_id, percent)) in accumulator.stats.iter().enumerate() {
        item.stat_modifier_bonus_stat[slot] = *stat_id;
        item.stat_percent_editor[slot] = *percent;
    }
    if let Some(level) = accumulator.required_level {
        item.required_level = u16::from(level);
    }
    let scaling = tables::item_scaling(descriptor.item_id);
    let level = scaling::resolve_level(
        &accumulator,
        descriptor,
        &scaling,
        item.item_level,
        player_level,
    )?;
    item.item_level = level.clamp(1, 1300) as u16;
    Ok(())
}

fn apply_list(list_id: u32, accumulator: &mut Accumulator, depth: u8) -> Result<(), String> {
    let rows = tables::bonus_rows(list_id);
    // WM97c: the 70391 base export has no repeated OrderIndex within a parent, and
    // the native sort is not stable for larger ranges. A duplicate is therefore a
    // named gap rather than an assumed CSV order.
    if tables::has_duplicate_order_index(list_id) {
        return Err(unknown(format!(
            "bonus list {list_id} repeats OrderIndex; native order is not stable (WM97c)"
        )));
    }
    for row in rows {
        apply_row(row, accumulator, depth)?;
    }
    Ok(())
}

fn apply_row(row: &BonusRow, accumulator: &mut Accumulator, depth: u8) -> Result<(), String> {
    match row.kind {
        1 => {
            accumulator.level_delta = accumulator
                .level_delta
                .wrapping_add((row.values[0] & 0xffff) as u16 as i16)
        }
        2 => apply_stat(row, accumulator)?,
        3 => {
            let quality = (row.values[0] & 0xff).min(8) as u8;
            accumulator.quality = if accumulator.quality_applied {
                accumulator.quality.max(quality)
            } else {
                quality
            };
            accumulator.quality_applied = true;
        }
        8 => {
            let base = accumulator
                .required_level
                .unwrap_or(accumulator.base_required_level);
            accumulator.required_level = Some(base.wrapping_add(row.values[0] as u8));
        }
        18 => {
            let value = row.values[0] as u8;
            accumulator.required_level = Some(match accumulator.required_level {
                Some(old) => old.max(value),
                None => value,
            });
        }
        11 | 13 => offer_player_curve(row, accumulator),
        27 => offer_fixed(row, accumulator),
        42 => accumulator
            .base_level
            .offer(row.values[1], row.values[0] & 0xffff),
        48 => offer_offset_family(row, accumulator),
        49 | 51 => {
            accumulator
                .family
                .offer_config(row.values[1], row.values[0] as u32);
            if row.kind == 51 {
                accumulator.modes.scaling_input = true;
            } else {
                offer_fixed(row, accumulator);
                accumulator.modes.config_item_level = true;
                accumulator.modes.requirement_copy = true;
            }
            apply_config(accumulator)?;
        }
        50 => {
            if depth > 0 {
                return Err(unknown(format!(
                    "bonus list {list_id} nests Type 50 deeper than one level (native fatal status 16)",
                    list_id = row.list_id
                )));
            }
            apply_list(row.values[0] as u32, accumulator, depth + 1)?;
        }
        52 => accumulator.squish.offer(row.values[3], row.values),
        53 => accumulator
            .pre_conversion
            .offer(row.values[1], row.values[0]),
        _ => {}
    }
    Ok(())
}

impl Family {
    fn offer_config(&mut self, priority: i32, config: u32) {
        // WM97b: an equal priority does not compare or replace the config ID.
        if priority < self.priority {
            self.priority = priority;
            self.config = config;
        }
    }
}

/// WM97b Type 2: stat ID 0..=141, percentage accumulates with 32-bit wrap.
fn apply_stat(row: &BonusRow, accumulator: &mut Accumulator) -> Result<(), String> {
    let stat_id = row.values[0];
    if !(0..=141).contains(&stat_id) {
        return Ok(());
    }
    let slot = accumulator
        .stats
        .iter()
        .position(|(id, _)| i32::from(*id) == stat_id)
        .or_else(|| accumulator.stats.iter().position(|(id, _)| *id < 0))
        .ok_or_else(|| {
            unknown(format!(
                "bonus row {} Type 2 finds no free stat slot (native fatal status)",
                row.row_id
            ))
        })?;
    let accumulated = i32::from(accumulator.stats[slot].1).wrapping_add(row.values[1]);
    accumulator.stats[slot] = (
        stat_id as i16,
        accumulated.clamp(0, i32::from(u16::MAX)) as u16,
    );
    Ok(())
}

/// WM97b Types 11/13: Value_1 priority, Value_2 tuning, Value_3 player curve.
fn offer_player_curve(row: &BonusRow, accumulator: &mut Accumulator) {
    let family = &mut accumulator.family;
    if row.values[1] >= family.priority {
        return;
    }
    family.priority = row.values[1];
    family.curve = row.values[3] as u32;
    family.tuning = row.values[2] as u32;
    family.offset_curve = 0;
    family.offset_input = 0;
    family.era = -1;
    family.config = 0;
    accumulator.modes.scaling_input = false;
    accumulator.modes.config_item_level = false;
    accumulator.modes.disable_era = false;
    accumulator.modes.tuning_clamp = false;
}

/// WM97b Type 48: Value_0 offset curve, Value_1 input, Value_2 era, Value_3 priority.
fn offer_offset_family(row: &BonusRow, accumulator: &mut Accumulator) {
    let family = &mut accumulator.family;
    if row.values[3] >= family.priority {
        return;
    }
    family.priority = row.values[3];
    family.curve = 0;
    family.tuning = 0;
    family.offset_curve = row.values[0] as u32;
    family.offset_input = row.values[1];
    family.era = row.values[2];
    family.config = 0;
    accumulator.modes.scaling_input = false;
    accumulator.modes.config_item_level = false;
    accumulator.modes.disable_era = false;
    accumulator.modes.tuning_clamp = false;
}

/// WM97b Type 27/49 fixed-control priority competition.
fn offer_fixed(row: &BonusRow, accumulator: &mut Accumulator) {
    let priority = if row.kind == 27 {
        row.values[2]
    } else {
        row.values[1]
    };
    if priority >= accumulator.fixed_priority {
        return;
    }
    accumulator.fixed_priority = priority;
    accumulator.modes.config_item_level = false;
    if row.kind == 27 && row.values[1] != 0 {
        accumulator.family.tuning = row.values[1] as u32;
    }
}

/// WM97b configuration secondary reader: every Type 49/51 row resolves the
/// currently selected config, even when that row lost primary selection.
fn apply_config(accumulator: &mut Accumulator) -> Result<(), String> {
    let config_id = accumulator.family.config;
    if config_id == 0 {
        return Ok(());
    }
    let config = tables::scaling_config(config_id)?;
    let offset = tables::offset_curve(config.offset_curve_id)?;
    accumulator.modes.disable_era = config.flags & 1 != 0;
    accumulator.modes.tuning_clamp = config.flags & 2 != 0;
    accumulator.family.offset_curve = offset.curve_id;
    accumulator.modes.offset = offset.offset;
    accumulator.family.offset_input = if accumulator.modes.config_item_level {
        config.item_level
    } else {
        0
    };
    if accumulator.modes.requirement_copy {
        accumulator.required_level = Some(config.required_level);
    }
    accumulator.family.era = config.squish_era_id;
    Ok(())
}
