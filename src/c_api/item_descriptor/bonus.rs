//! WM97 DQ3 bonus-list application over pinned 70391 ItemBonus records.
//!
//! Implemented Types are those with a directly verified tooltip-visible effect on
//! the fields this simulator models. Types whose verified destination is a scaling
//! curve family have no pinned records here, so they return a named error instead
//! of inventing arithmetic. Remaining Types are recorded as no modelled output.

use super::{ItemDescriptor, unknown};
use crate::items::ItemInfo;

const BONUS_ROWS: &str = include_str!("../../../data/item-descriptor/70391/bonus.tsv");
const SCALING_SELECTORS: &str =
    include_str!("../../../data/item-descriptor/70391/scaling-selectors.tsv");

/// Verified curve-family selectors from WM97 DQ3. Their records are not pinned locally.
const SCALING_TYPES: [u8; 12] = [9, 10, 11, 13, 27, 30, 48, 49, 51, 52, 53, 28];

#[derive(Clone, Copy)]
struct BonusRow {
    list_id: u32,
    row_id: u32,
    kind: u8,
    values: [i32; 4],
}

struct Records(Vec<BonusRow>);

static RECORDS: std::sync::OnceLock<Records> = std::sync::OnceLock::new();

fn records() -> &'static Records {
    RECORDS.get_or_init(|| {
        let mut rows: Vec<BonusRow> = BONUS_ROWS
            .lines()
            .map(|line| {
                let mut fields = line.split('\t');
                let list_id = fields.next().unwrap().parse().unwrap();
                let row_id = fields.next().unwrap().parse().unwrap();
                let kind = fields.next().unwrap().parse().unwrap();
                fields.next();
                let mut values = [0i32; 4];
                for value in &mut values {
                    *value = fields.next().unwrap().parse().unwrap();
                }
                BonusRow {
                    list_id,
                    row_id,
                    kind,
                    values,
                }
            })
            .collect();
        // WM97: rows are visited in native row order after ascending OrderIndex.
        // The TSV is already written in that order; a stable list sort keeps it.
        rows.sort_by_key(|row| row.list_id);
        Records(rows)
    })
}

fn rows_for(list_id: u32) -> &'static [BonusRow] {
    let rows = &records().0;
    let start = rows.partition_point(|row| row.list_id < list_id);
    let end = start + rows[start..].partition_point(|row| row.list_id == list_id);
    &rows[start..end]
}

pub(super) fn apply(descriptor: &ItemDescriptor, item: &mut ItemInfo) -> Result<(), String> {
    reject_unpinned_scaling(descriptor.item_id)?;
    let Some(context) = &descriptor.bonus_context else {
        return Ok(());
    };
    let mut quality_applied = false;
    let mut level_delta: i16 = 0;
    let mut required_level: Option<u8> = None;
    for list_id in &context.bonus_list_ids {
        for row in rows_for(*list_id) {
            match row.kind {
                1 => level_delta = level_delta.wrapping_add((row.values[0] & 0xffff) as u16 as i16),
                3 => {
                    let quality = (row.values[0] & 0xff).min(8) as u8;
                    item.quality = if quality_applied {
                        item.quality.max(quality)
                    } else {
                        quality
                    };
                    quality_applied = true;
                }
                8 => {
                    let base = required_level.unwrap_or(item.required_level.min(255) as u8);
                    required_level = Some(base.wrapping_add(row.values[0] as u8));
                }
                18 => {
                    let value = row.values[0] as u8;
                    required_level = Some(required_level.map_or(value, |old| old.max(value)));
                }
                2 => apply_stat(row, item),
                42 | 43 => {
                    return Err(unknown(format!(
                        "item {} bonus list {list_id} row {} Type {} selects a base/alternate level with unpinned priority rules",
                        descriptor.item_id, row.row_id, row.kind
                    )));
                }
                kind if SCALING_TYPES.contains(&kind) => {
                    return Err(unknown(format!(
                        "bonus list {list_id} row {} Type {kind} selects a scaling curve family with no pinned records",
                        row.row_id
                    )));
                }
                _ => {}
            }
        }
    }
    item.item_level = resolve_level(item.item_level, level_delta);
    if let Some(level) = required_level {
        item.required_level = u16::from(level);
    }
    Ok(())
}

/// WM97 Type 2: stat ID limited to 0..=141, percentage accumulates with 32-bit wrap.
fn apply_stat(row: &BonusRow, item: &mut ItemInfo) {
    let stat_id = row.values[0];
    if !(0..=141).contains(&stat_id) {
        return;
    }
    let slot = item
        .stat_modifier_bonus_stat
        .iter()
        .position(|entry| i32::from(*entry) == stat_id)
        .or_else(|| {
            item.stat_modifier_bonus_stat
                .iter()
                .position(|entry| *entry < 0)
        });
    let Some(slot) = slot else { return };
    let accumulated = i32::from(item.stat_percent_editor[slot]).wrapping_add(row.values[1]);
    item.stat_percent_editor[slot] = accumulated.clamp(0, i32::from(u16::MAX)) as u16;
    item.stat_modifier_bonus_stat[slot] = stat_id as i16;
}

/// WM97: the core resets an out-of-range result to 1, not to the nearest bound.
fn resolve_level(base: u16, delta: i16) -> u16 {
    let level = i32::from(base).wrapping_add(i32::from(delta));
    if (1..=1300).contains(&level) {
        level as u16
    } else {
        1
    }
}

/// Named data-absence seam. Pinned ItemSparse selectors without pinned records must
/// error instead of silently rendering the unsquished catalog level.
fn reject_unpinned_scaling(item_id: u32) -> Result<(), String> {
    let mut found = false;
    for line in SCALING_SELECTORS.lines() {
        let mut fields = line.split('\t');
        if fields.next().and_then(|id| id.parse::<u32>().ok()) == Some(item_id) {
            found = true;
            break;
        }
    }
    if found {
        return Err(unknown(format!(
            "item {item_id} has ItemSparse scaling selectors; pinned scaling records are absent"
        )));
    }
    Ok(())
}
