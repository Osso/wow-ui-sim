//! Read-only descriptor tables from the pinned 70391 exports.

use std::collections::HashMap;
use std::sync::OnceLock;

/// Curves listed here are absent from the export; lookups must name the gap.
const ABSENT_CURVES: [u32; 2] = [121405, 121406];

const BONUS_ROWS: &str = include_str!("../../../data/item-descriptor/70391/bonus.tsv");
const SELECTORS: &str = include_str!("../../../data/item-descriptor/70391/scaling-selectors.tsv");
const CURVES: &str = include_str!("../../../data/item-descriptor/70391/curves.tsv");
const CURVE_POINTS: &str = include_str!("../../../data/item-descriptor/70391/curve-points.tsv");
const OFFSET_CURVES: &str = include_str!("../../../data/item-descriptor/70391/offset-curves.tsv");
const SCALING_CONFIGS: &str =
    include_str!("../../../data/item-descriptor/70391/scaling-configs.tsv");
const SQUISH_ERAS: &str = include_str!("../../../data/item-descriptor/70391/squish-eras.tsv");
const CONTENT_TUNING: &str = include_str!("../../../data/item-descriptor/70391/content-tuning.tsv");

fn columns(line: &str) -> impl Iterator<Item = &str> {
    line.split('\t')
}

fn number(field: Option<&str>) -> i32 {
    field.unwrap_or("0").parse().unwrap_or(0)
}

fn unsigned(field: Option<&str>) -> u32 {
    number(field) as u32
}

pub(super) struct BonusRow {
    pub list_id: u32,
    pub row_id: u32,
    pub order_index: u8,
    pub kind: u8,
    pub values: [i32; 4],
}

struct BonusTable(Vec<BonusRow>);

static BONUS: OnceLock<BonusTable> = OnceLock::new();

/// WM97: rows are visited by ascending OrderIndex within their own list. The TSV
/// preserves native order for that list; equal-OrderIndex stability is INFERRED.
pub(super) fn bonus_rows(list_id: u32) -> &'static [BonusRow] {
    let table = BONUS.get_or_init(|| {
        let mut rows: Vec<BonusRow> = BONUS_ROWS
            .lines()
            .skip(1)
            .map(|line| {
                let mut fields = columns(line);
                BonusRow {
                    list_id: unsigned(fields.next()),
                    row_id: unsigned(fields.next()),
                    order_index: number(fields.next()) as u8,
                    kind: number(fields.next()) as u8,
                    values: [
                        number(fields.next()),
                        number(fields.next()),
                        number(fields.next()),
                        number(fields.next()),
                    ],
                }
            })
            .collect();
        rows.sort_by_key(|row| row.list_id);
        BonusTable(rows)
    });
    let rows = &table.0;
    let start = rows.partition_point(|row| row.list_id < list_id);
    let end = start + rows[start..].partition_point(|row| row.list_id == list_id);
    &rows[start..end]
}

/// WM97c: duplicates make the native non-stable sort order a named gap.
pub(super) fn has_duplicate_order_index(list_id: u32) -> bool {
    let rows = bonus_rows(list_id);
    rows.windows(2)
        .any(|pair| pair[0].order_index == pair[1].order_index)
}

pub(super) struct ItemScaling {
    pub player_level_curve: u32,
    pub offset_curve: u32,
    pub content_tuning: u32,
    pub squish_era: u32,
    pub offset_item_level: i32,
}

static SELECTOR_TABLE: OnceLock<HashMap<u32, ItemScaling>> = OnceLock::new();

pub(super) fn item_scaling(item_id: u32) -> ItemScaling {
    let table = SELECTOR_TABLE.get_or_init(|| {
        SELECTORS
            .lines()
            .skip(1)
            .map(|line| {
                let mut fields = columns(line);
                let id = unsigned(fields.next());
                let scaling = ItemScaling {
                    player_level_curve: unsigned(fields.next()),
                    offset_curve: unsigned(fields.next()),
                    content_tuning: unsigned(fields.next()),
                    squish_era: unsigned(fields.next()),
                    offset_item_level: number(fields.next()),
                };
                (id, scaling)
            })
            .collect()
    });
    table.get(&item_id).map_or(
        ItemScaling {
            player_level_curve: 0,
            offset_curve: 0,
            content_tuning: 0,
            squish_era: 0,
            offset_item_level: 0,
        },
        |scaling| ItemScaling {
            player_level_curve: scaling.player_level_curve,
            offset_curve: scaling.offset_curve,
            content_tuning: scaling.content_tuning,
            squish_era: scaling.squish_era,
            offset_item_level: scaling.offset_item_level,
        },
    )
}

pub(super) struct Curve {
    pub kind: u8,
    pub points: Vec<(f32, f32)>,
}

struct CurveTable(HashMap<u32, Curve>);

static CURVE_TABLE: OnceLock<CurveTable> = OnceLock::new();

pub(super) fn curve(id: u32) -> Result<&'static Curve, String> {
    if ABSENT_CURVES.contains(&id) {
        return Err(super::unknown(format!(
            "curve {id} is in the encrypted set absent from the 70391 export"
        )));
    }
    let table = CURVE_TABLE.get_or_init(|| {
        let mut kinds: Vec<(u32, u8)> = CURVES
            .lines()
            .skip(1)
            .map(|line| {
                let mut fields = columns(line);
                (unsigned(fields.next()), number(fields.next()) as u8)
            })
            .collect();
        kinds.sort_by_key(|(id, _)| *id);
        let mut curves: HashMap<u32, Curve> = kinds
            .into_iter()
            .map(|(id, kind)| {
                (
                    id,
                    Curve {
                        kind,
                        points: Vec::new(),
                    },
                )
            })
            .collect();
        // Rows arrive in CurveID then OrderIndex order, which is native point order.
        for line in CURVE_POINTS.lines().skip(1) {
            let mut fields = columns(line);
            let curve_id = unsigned(fields.next());
            let _order_index = fields.next();
            let pos = pos_value(fields.next(), 0.0);
            let value = pos_value(fields.next(), 0.0);
            if let Some(entry) = curves.get_mut(&curve_id) {
                entry.points.push((pos, value));
            }
        }
        CurveTable(curves)
    });
    table.0.get(&id).ok_or_else(|| {
        super::unknown(format!(
            "curve {id} is not a pinned reachable curve (unreferenced curves are not exported)"
        ))
    })
}

fn pos_value(field: Option<&str>, default: f32) -> f32 {
    field.and_then(|text| text.parse().ok()).unwrap_or(default)
}

pub(super) struct OffsetCurve {
    pub curve_id: u32,
    pub offset: i32,
}

struct OffsetTable(HashMap<u32, OffsetCurve>);

static OFFSET_TABLE: OnceLock<OffsetTable> = OnceLock::new();

pub(super) fn offset_curve(id: u32) -> Result<&'static OffsetCurve, String> {
    let table = OFFSET_TABLE.get_or_init(|| {
        OffsetTable(
            OFFSET_CURVES
                .lines()
                .skip(1)
                .map(|line| {
                    let mut fields = columns(line);
                    let id = unsigned(fields.next());
                    let entry = OffsetCurve {
                        curve_id: unsigned(fields.next()),
                        offset: number(fields.next()),
                    };
                    (id, entry)
                })
                .collect(),
        )
    });
    table.0.get(&id).ok_or_else(|| {
        super::unknown(format!(
            "ItemOffsetCurve {id} is absent from the pinned export"
        ))
    })
}

pub(super) struct ScalingConfig {
    pub offset_curve_id: u32,
    pub item_level: i32,
    pub required_level: u8,
    pub squish_era_id: i32,
    pub flags: u8,
}

struct ConfigTable(HashMap<u32, ScalingConfig>);

static CONFIG_TABLE: OnceLock<ConfigTable> = OnceLock::new();

pub(super) fn scaling_config(id: u32) -> Result<&'static ScalingConfig, String> {
    let table = CONFIG_TABLE.get_or_init(|| {
        ConfigTable(
            SCALING_CONFIGS
                .lines()
                .skip(1)
                .map(|line| {
                    let mut fields = columns(line);
                    let id = unsigned(fields.next());
                    let entry = ScalingConfig {
                        offset_curve_id: unsigned(fields.next()),
                        item_level: number(fields.next()),
                        required_level: number(fields.next()) as u8,
                        squish_era_id: number(fields.next()),
                        flags: number(fields.next()) as u8,
                    };
                    (id, entry)
                })
                .collect(),
        )
    });
    table.0.get(&id).ok_or_else(|| {
        super::unknown(format!(
            "ItemScalingConfig {id} is absent; native fatal status 13"
        ))
    })
}

pub(super) struct SquishEra {
    pub id: u32,
    pub patch: i32,
    pub curve_id: u32,
    pub flags: u8,
}

struct EraTable(Vec<SquishEra>);

static ERA_TABLE: OnceLock<EraTable> = OnceLock::new();

fn eras() -> &'static EraTable {
    ERA_TABLE.get_or_init(|| {
        let mut rows: Vec<SquishEra> = SQUISH_ERAS
            .lines()
            .skip(1)
            .map(|line| {
                let mut fields = columns(line);
                SquishEra {
                    id: unsigned(fields.next()),
                    patch: number(fields.next()),
                    curve_id: unsigned(fields.next()),
                    flags: number(fields.next()) as u8,
                }
            })
            .collect();
        rows.sort_by_key(|row| row.patch);
        EraTable(rows)
    })
}

pub(super) fn squish_era_index(id: u32) -> Option<usize> {
    eras().0.iter().position(|row| row.id == id)
}

pub(super) fn squish_eras() -> &'static [SquishEra] {
    &eras().0
}

pub(super) struct ContentTuning {
    pub flags: u8,
    pub min_level: i32,
    pub max_level: i32,
    pub min_mode: u8,
    pub max_mode: u8,
    pub allowed_min_offset: i32,
}

struct TuningTable(HashMap<u32, ContentTuning>);

static TUNING_TABLE: OnceLock<TuningTable> = OnceLock::new();

pub(super) fn content_tuning(id: u32) -> Result<&'static ContentTuning, String> {
    let table = TUNING_TABLE.get_or_init(|| {
        TuningTable(
            CONTENT_TUNING
                .lines()
                .skip(1)
                .map(|line| {
                    let mut fields = columns(line);
                    let id = unsigned(fields.next());
                    let entry = ContentTuning {
                        flags: number(fields.next()) as u8,
                        min_level: number(fields.next()),
                        max_level: number(fields.next()),
                        min_mode: number(fields.next()) as u8,
                        max_mode: number(fields.next()) as u8,
                        allowed_min_offset: number(fields.next()),
                    };
                    (id, entry)
                })
                .collect(),
        )
    });
    table.0.get(&id).ok_or_else(|| {
        super::unknown(format!(
            "ContentTuning {id} is absent from the pinned export"
        ))
    })
}
