//! Reader for the five pinned Forever spell WDC5 layouts.
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

use crate::forever_spell_data::{SourceRow, SourceTable};
use serde_json::{Value, json};

#[derive(Clone, Copy)]
enum Kind {
    Signed,
    Unsigned,
    Float,
    Text,
}
#[derive(Clone, Copy)]
struct Field {
    name: &'static str,
    kind: Kind,
    width: usize,
    count: usize,
}
fn field(name: &'static str, kind: Kind, width: usize, count: usize) -> Field {
    Field {
        name,
        kind,
        width,
        count,
    }
}

fn schema(name: &str) -> Result<(u32, Vec<Field>), String> {
    use Kind::*;
    let s = |n| field(n, Signed, 32, 1);
    let u16f = |n| field(n, Unsigned, 16, 1);
    let f = |n| field(n, Float, 32, 1);
    let schema = match name {
        "Spell" => (
            0xe3d134fb,
            vec![
                field("NameSubtext_lang", Text, 32, 1),
                field("Description_lang", Text, 32, 1),
                field("AuraDescription_lang", Text, 32, 1),
            ],
        ),
        "SpellName" => (0x782ee721, vec![field("Name_lang", Text, 32, 1)]),
        "SpellMisc" => (
            0x434b3607,
            vec![
                field("Attributes", Signed, 32, 17),
                field("DifficultyID", Signed, 16, 1),
                u16f("CastingTimeIndex"),
                u16f("DurationIndex"),
                u16f("PvPDurationIndex"),
                u16f("RangeIndex"),
                field("SchoolMask", Unsigned, 8, 1),
                f("Speed"),
                f("LaunchDelay"),
                f("MinDuration"),
                s("SpellIconFileDataID"),
                s("ActiveIconFileDataID"),
                s("ContentTuningID"),
                s("ShowFutureSpellPlayerConditionID"),
                s("SpellVisualScript"),
                s("ActiveSpellVisualScript"),
            ],
        ),
        "SpellPower" => (
            0x61ad223f,
            vec![
                s("ID"),
                field("OrderIndex", Unsigned, 8, 1),
                s("ManaCost"),
                s("ManaCostPerLevel"),
                s("ManaPerSecond"),
                field("PowerDisplayID", Unsigned, 32, 1),
                s("AltPowerBarID"),
                f("PowerCostPct"),
                f("PowerCostMaxPct"),
                f("OptionalCostPct"),
                f("PowerPctPerSecond"),
                field("PowerType", Signed, 8, 1),
                s("RequiredAuraSpellID"),
                field("OptionalCost", Unsigned, 32, 1),
            ],
        ),
        "SpellEffect" => (
            0x5362e3d4,
            vec![
                field("EffectAura", Signed, 16, 1),
                field("DifficultyID", Signed, 16, 1),
                s("EffectIndex"),
                field("Effect", Unsigned, 32, 1),
                f("EffectAmplitude"),
                s("EffectAttributes"),
                s("EffectAuraPeriod"),
                f("EffectBonusCoefficient"),
                f("EffectChainAmplitude"),
                s("EffectChainTargets"),
                s("EffectItemType"),
                s("EffectMechanic"),
                f("EffectPointsPerResource"),
                f("EffectPos_facing"),
                f("EffectRealPointsPerLevel"),
                s("EffectTriggerSpell"),
                f("BonusCoefficientFromAP"),
                f("PvpMultiplier"),
                f("Coefficient"),
                f("Variance"),
                f("ResourceCoefficient"),
                f("GroupSizeBasePointsCoefficient"),
                f("EffectBasePointsF"),
                s("ScalingClass"),
                s("Node__Field_12_0_0_63534_001"),
                field("EffectMiscValue", Signed, 32, 2),
                field("EffectRadiusIndex", Unsigned, 32, 2),
                field("EffectSpellClassMask", Signed, 32, 4),
                field("ImplicitTarget", Signed, 16, 2),
            ],
        ),
        _ => return Err(format!("unsupported Forever DB2 table {name}")),
    };
    Ok(schema)
}

fn bytes(data: &[u8], offset: usize, size: usize) -> Result<&[u8], String> {
    data.get(offset..offset.checked_add(size).ok_or("DB2 offset overflow")?)
        .ok_or_else(|| format!("truncated DB2 at {offset}+{size}"))
}
fn u16_at(data: &[u8], p: usize) -> Result<usize, String> {
    Ok(u16::from_le_bytes(bytes(data, p, 2)?.try_into().unwrap()) as usize)
}
fn u32_at(data: &[u8], p: usize) -> Result<usize, String> {
    Ok(u32::from_le_bytes(bytes(data, p, 4)?.try_into().unwrap()) as usize)
}
fn u64_at(data: &[u8], p: usize) -> Result<u64, String> {
    Ok(u64::from_le_bytes(bytes(data, p, 8)?.try_into().unwrap()))
}
fn bits(data: &[u8], bit: usize, width: usize) -> Result<u64, String> {
    if width > 32 {
        return Err(format!("unsupported field bit width {width}"));
    }
    let mut value = 0u64;
    for i in 0..width {
        value |= (((*data.get((bit + i) / 8).ok_or("truncated record bits")? >> ((bit + i) % 8))
            & 1) as u64)
            << i;
    }
    Ok(value)
}
#[derive(Clone)]
struct Info {
    bit: usize,
    width: usize,
    size: usize,
    compression: usize,
    default: u32,
    array: usize,
    storage: usize,
}
#[derive(Clone)]
struct Section {
    key: u64,
    start: usize,
    count: usize,
    strings: usize,
    end: usize,
    ids: usize,
    relations: usize,
    maps: usize,
    copies: usize,
}
fn section(data: &[u8], n: usize) -> Result<Section, String> {
    let p = 204 + 40 * n;
    Ok(Section {
        key: u64_at(data, p)?,
        start: u32_at(data, p + 8)?,
        count: u32_at(data, p + 12)?,
        strings: u32_at(data, p + 16)?,
        end: u32_at(data, p + 20)?,
        ids: u32_at(data, p + 24)?,
        relations: u32_at(data, p + 28)?,
        maps: u32_at(data, p + 32)?,
        copies: u32_at(data, p + 36)?,
    })
}
fn overlaps(a: &Range<usize>, b: &Range<usize>) -> bool {
    a.start < b.end && b.start < a.end
}
fn guard(range: Range<usize>, zero: &[Range<usize>]) -> Result<Range<usize>, String> {
    if zero.iter().any(|z| overlaps(&range, z)) {
        Err(format!(
            "DB2 source range {range:?} overlaps zero-filled bytes"
        ))
    } else {
        Ok(range)
    }
}
fn checked_string(data: &[u8], p: usize, limit: usize) -> Result<(String, usize), String> {
    let s = bytes(
        data,
        p,
        limit.checked_sub(p).ok_or("invalid string bounds")?,
    )?;
    let n = s
        .iter()
        .position(|&v| v == 0)
        .ok_or("unterminated DB2 string")?;
    Ok((
        std::str::from_utf8(&s[..n])
            .map_err(|e| format!("DB2 string: {e}"))?
            .to_owned(),
        n + 1,
    ))
}
fn numeric(raw: u64, field: Field) -> Result<Value, String> {
    let value = raw & ((1u64 << field.width) - 1);
    Ok(match field.kind {
        Kind::Unsigned => json!(value),
        Kind::Signed => {
            let sign = 1u64 << (field.width - 1);
            json!(if value & sign != 0 {
                (value as i64) - (1i64 << field.width)
            } else {
                value as i64
            })
        }
        Kind::Float => serde_json::Number::from_f64(f32::from_bits(value as u32) as f64)
            .map(Value::Number)
            .ok_or("non-finite DB2 float")?,
        Kind::Text => return Err("unexpected text field".into()),
    })
}
fn infos(data: &[u8], count: usize, sections: usize) -> Result<(Vec<Info>, usize), String> {
    let start = 204 + sections * 40 + count * 4;
    let info_size = u32_at(data, 188)?;
    if info_size != count * 24 {
        return Err("DB2 field descriptor count mismatch".into());
    }
    let palette_size = u32_at(data, 196)?;
    let mut v = Vec::new();
    for i in 0..count {
        let p = start + i * 24;
        v.push(Info {
            bit: u16_at(data, p)?,
            width: u16_at(data, p + 2)?,
            size: u32_at(data, p + 4)?,
            compression: u32_at(data, p + 8)?,
            default: u32_at(data, p + 12)? as u32,
            array: u32_at(data, p + 20)?,
            storage: 0,
        });
    }
    let mut cursor = start + info_size;
    for ty in [3, 4] {
        for info in &mut v {
            if info.compression == ty {
                info.storage = cursor;
                cursor += info.size;
            }
        }
    }
    if cursor != start + info_size + palette_size {
        return Err("DB2 palette size mismatch".into());
    }
    for info in &mut v {
        if info.compression == 2 {
            info.storage = cursor;
            cursor += info.size;
        }
    }
    if cursor != start + info_size + palette_size + u32_at(data, 192)? {
        return Err("DB2 common-data size mismatch".into());
    }
    bytes(data, 0, cursor)?;
    Ok((v, cursor))
}
fn raw_values(
    data: &[u8],
    record: &[u8],
    record_id: u32,
    info: &Info,
    field: Field,
) -> Result<Vec<u64>, String> {
    let raw = if info.compression == 0 && field.count > 1 {
        0
    } else {
        bits(record, info.bit, info.width)?
    };
    match info.compression {
        0 if field.count > 1 => (0..field.count)
            .map(|n| bits(record, info.bit + n * field.width, field.width))
            .collect(),
        0 | 1 => Ok(vec![raw]),
        5 => {
            let signed = if info.width > 0 && raw & (1u64 << (info.width - 1)) != 0 {
                (raw as i64) - (1i64 << info.width)
            } else {
                raw as i64
            };
            Ok(vec![signed as u64])
        }
        2 => {
            let mut value = info.default;
            for p in (info.storage..info.storage + info.size).step_by(8) {
                if u32_at(data, p)? as u32 == record_id {
                    value = u32_at(data, p + 4)? as u32;
                    break;
                }
            }
            Ok(vec![value as u64])
        }
        3 | 4 => {
            let count = if info.compression == 4 { info.array } else { 1 };
            if count != field.count {
                return Err(format!("DB2 array size mismatch for {}", field.name));
            }
            (0..count)
                .map(|i| {
                    u32_at(data, info.storage + ((raw as usize) * count + i) * 4).map(|v| v as u64)
                })
                .collect()
        }
        other => Err(format!("unsupported DB2 field compression {other}")),
    }
}
fn insert_columns(
    out: &mut BTreeMap<String, Value>,
    field: Field,
    values: Vec<Value>,
) -> Result<(), String> {
    if values.len() != field.count {
        return Err(format!("DB2 column count mismatch for {}", field.name));
    }
    for (i, value) in values.into_iter().enumerate() {
        out.insert(
            if field.count == 1 {
                field.name.to_owned()
            } else {
                format!("{}_{}", field.name, i)
            },
            value,
        );
    }
    Ok(())
}

fn string_value(
    data: &[u8],
    sections: &[Section],
    record_index: usize,
    total: usize,
    record_size: usize,
    info: &Info,
    value: u64,
    zero: &[Range<usize>],
) -> Result<String, String> {
    if value == 0 {
        return Ok(String::new());
    }
    let mut logical = (record_index as i64 - total as i64) * record_size as i64
        + (info.bit / 8) as i64
        + value as i64;
    for sec in sections {
        if logical >= 0 && (logical as usize) < sec.strings {
            let base = sec.start + sec.count * record_size;
            let p = base + logical as usize;
            let (value, len) = checked_string(data, p, base + sec.strings)?;
            guard(p..p + len, zero)?;
            return Ok(value);
        }
        logical -= sec.strings as i64;
    }
    Err("DB2 string offset outside string blocks".into())
}
fn read_relations(data: &[u8], p: usize, size: usize) -> Result<BTreeMap<usize, u32>, String> {
    if size == 0 {
        return Ok(BTreeMap::new());
    }
    let count = u32_at(data, p)?;
    if size != 12 + count * 8 {
        return Err("invalid DB2 relationship size".into());
    }
    let mut result = BTreeMap::new();
    for i in 0..count {
        let base = p + 12 + i * 8;
        let id = u32_at(data, base)? as u32;
        let index = u32_at(data, base + 4)?;
        result.insert(index, id);
    }
    Ok(result)
}
fn decode_fixed(
    data: &[u8],
    name: &str,
    selected: &BTreeSet<u32>,
    zero: &[Range<usize>],
    fields: &[Field],
    sections: &[Section],
    info: &[Info],
    guarded: Range<usize>,
) -> Result<Vec<SourceRow>, String> {
    let sec = &sections[0];
    let record_size = u32_at(data, 144)?;
    let total = u32_at(data, 136)?;
    let id_at = sec.start + sec.count * record_size + sec.strings;
    if sec.ids != 0 && sec.ids != sec.count * 4 {
        return Err("invalid DB2 ID list size".into());
    }
    let copy_at = id_at + sec.ids;
    let relations = read_relations(data, copy_at + sec.copies * 8, sec.relations)?;
    let mut records = BTreeMap::new();
    for index in 0..sec.count {
        let at = sec.start + index * record_size;
        let record = bytes(data, at, record_size)?;
        let id = if sec.ids > 0 {
            u32_at(data, id_at + index * 4)? as u32
        } else {
            raw_values(data, record, 0, &info[0], fields[0])?[0] as u32
        };
        records.insert(id, (index, id));
    }
    for i in 0..sec.copies {
        let id = u32_at(data, copy_at + i * 8)? as u32;
        let source = u32_at(data, copy_at + i * 8 + 4)? as u32;
        let record = records
            .get(&source)
            .ok_or_else(|| format!("missing DB2 copy source {source}"))?
            .0;
        records.insert(id, (record, source));
    }
    let mut result = Vec::new();
    for (id, (index, source)) in records {
        let spell_id = if name == "SpellName" {
            id
        } else {
            *relations.get(&index).unwrap_or(&0)
        };
        if !selected.contains(&spell_id) {
            continue;
        }
        let at = sec.start + index * record_size;
        let record = bytes(data, at, record_size)?;
        let mut columns = BTreeMap::from([("ID".to_owned(), json!(id))]);
        for (i, field) in fields.iter().copied().enumerate() {
            if name == "SpellPower" && i == 0 {
                continue;
            }
            let raw = raw_values(data, record, source, &info[i], field)?;
            let mut values = Vec::new();
            for v in raw {
                values.push(match field.kind {
                    Kind::Text => json!(string_value(
                        data,
                        sections,
                        index,
                        total,
                        record_size,
                        &info[i],
                        v,
                        zero
                    )?),
                    _ => numeric(v, field)?,
                });
            }
            insert_columns(&mut columns, field, values)?;
        }
        if name != "SpellName" {
            columns.insert("SpellID".into(), json!(spell_id));
        }
        result.push(SourceRow {
            spell_id,
            record_id: id,
            columns,
            source_ranges: vec![guarded.clone()],
        });
    }
    Ok(result)
}
fn decode_sparse(
    data: &[u8],
    selected: &BTreeSet<u32>,
    fields: &[Field],
    sec: &Section,
    guarded: Range<usize>,
) -> Result<Vec<SourceRow>, String> {
    if sec.strings != 0 || sec.maps != sec.count || (sec.ids != 0 && sec.ids != sec.count * 4) {
        return Err("unsupported sparse Forever Spell section".into());
    }
    let copy_at = sec.end + sec.ids;
    let maps_at = copy_at + sec.copies * 8;
    let ids_at = maps_at + sec.maps * 6 + sec.relations;
    let mut records = BTreeMap::new();
    for i in 0..sec.maps {
        let at = u32_at(data, maps_at + i * 6)?;
        let len = u16_at(data, maps_at + i * 6 + 4)?;
        let id = u32_at(data, ids_at + i * 4)? as u32;
        if at < sec.start || at.checked_add(len).is_none_or(|end| end > sec.end) {
            return Err(format!("invalid sparse Spell row {id}"));
        }
        records.insert(id, (at, len));
    }
    for i in 0..sec.copies {
        let id = u32_at(data, copy_at + i * 8)? as u32;
        let source = u32_at(data, copy_at + i * 8 + 4)? as u32;
        let span = *records
            .get(&source)
            .ok_or_else(|| format!("missing sparse Spell copy source {source}"))?;
        records.insert(id, span);
    }
    let mut result = Vec::new();
    for id in selected {
        if let Some(&(at, len)) = records.get(id) {
            let mut pos = at;
            let end = at + len;
            let mut columns = BTreeMap::from([("ID".into(), json!(id))]);
            for field in fields {
                let (s, used) = checked_string(data, pos, end)?;
                pos += used;
                columns.insert(field.name.into(), json!(s));
            }
            if pos != end
                && (end != ((pos + 3) / 4) * 4
                    || bytes(data, pos, end - pos)?.iter().any(|b| *b != 0))
            {
                return Err(format!("unexpected trailing sparse Spell bytes for {id}"));
            }
            result.push(SourceRow {
                spell_id: *id,
                record_id: *id,
                columns,
                source_ranges: vec![guarded.clone()],
            });
        }
    }
    Ok(result)
}
/// Decode the pinned WDC5 layout. All selected records must reside in its readable first section.
pub fn decode(
    name: &str,
    data: &[u8],
    zero: Vec<Range<usize>>,
    selected: &BTreeSet<u32>,
) -> Result<SourceTable, String> {
    if bytes(data, 0, 4)? != b"WDC5" {
        return Err(format!("{name}: expected WDC5"));
    }
    let (layout, fields) = schema(name)?;
    let actual = u32_at(data, 156)? as u32;
    if actual != layout {
        return Err(format!(
            "{name}: DB2 layout {actual:08X}, expected {layout:08X}"
        ));
    }
    let count = u32_at(data, 140)?;
    let section_count = u32_at(data, 200)?;
    if count != fields.len() || section_count == 0 {
        return Err(format!("{name}: DB2 field/section count mismatch"));
    }
    let sections = (0..section_count)
        .map(|i| section(data, i))
        .collect::<Result<Vec<_>, _>>()?;
    if sections[0].key != 0 {
        return Err(format!("{name}: first DB2 section is encrypted"));
    }
    let (info, metadata_end) = infos(data, count, section_count)?;
    let first_end = sections.get(1).map_or(data.len(), |s| s.start);
    if metadata_end > sections[0].start || first_end > data.len() || first_end < sections[0].start {
        return Err(format!("{name}: invalid first DB2 section bounds"));
    }
    let guarded = guard(0..first_end, &zero)?;
    let flags = u16_at(data, 172)?;
    let rows = match (name, flags) {
        ("Spell", 5) => decode_sparse(data, selected, &fields, &sections[0], guarded)?,
        (_, 0 | 4) => decode_fixed(
            data, name, selected, &zero, &fields, &sections, &info, guarded,
        )?,
        _ => return Err(format!("{name}: unsupported DB2 flags {flags}")),
    };
    let unreadable_records = sections
        .iter()
        .skip(1)
        .filter(|s| zero.iter().any(|z| z.start <= s.start && s.start < z.end))
        .map(|s| s.count)
        .sum();
    Ok(SourceTable {
        layout_hash: layout,
        unreadable_records,
        zero_filled: zero,
        rows,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_record_is_decoded_and_zero_filled_string_is_rejected() {
        let mut db2 = vec![0u8; 284];
        db2[..4].copy_from_slice(b"WDC5");
        for (p, v) in [
            (136, 1u32),
            (140, 1),
            (144, 4),
            (156, 0x782ee721),
            (188, 24),
            (200, 1),
            (212, 272),
            (216, 1),
            (220, 4),
            (228, 4),
        ] {
            db2[p..p + 4].copy_from_slice(&v.to_le_bytes());
        }
        db2[172..174].copy_from_slice(&4u16.to_le_bytes());
        db2[250..252].copy_from_slice(&32u16.to_le_bytes());
        db2[272..276].copy_from_slice(&4u32.to_le_bytes());
        db2[276..280].copy_from_slice(b"hey\0");
        db2[280..284].copy_from_slice(&10157u32.to_le_bytes());
        let ids = BTreeSet::from([10157]);
        let table = decode("SpellName", &db2, vec![], &ids).unwrap();
        assert_eq!(table.rows.len(), 1);
        assert_eq!(table.rows[0].columns["Name_lang"], "hey");
        assert!(
            decode("SpellName", &db2, vec![276..277], &ids)
                .err()
                .unwrap()
                .contains("zero-filled")
        );
    }
}
