//! Generated Retail SpellName index; see `spell_names_retail.provenance.json`.
//! VERIFIED (DB2 12.1.5.70783); the data file stores all readable non-empty names.

const SPELL_NAME_DATA: &[u8] = include_bytes!("spell_names_retail.bin");
const HEADER_BYTES: usize = 4;
const ENTRY_BYTES: usize = 8;

pub fn get_spell_name(id: u32) -> Option<&'static str> {
    let count = read_u32(0)? as usize;
    let names_start = HEADER_BYTES.checked_add(count.checked_mul(ENTRY_BYTES)?)?;
    if names_start > SPELL_NAME_DATA.len() {
        return None;
    }
    let mut low = 0;
    let mut high = count;
    while low < high {
        let middle = low + (high - low) / 2;
        let entry = HEADER_BYTES + middle * ENTRY_BYTES;
        if read_u32(entry)? < id {
            low = middle + 1;
        } else {
            high = middle;
        }
    }
    if low == count {
        return None;
    }
    let entry = HEADER_BYTES + low * ENTRY_BYTES;
    if read_u32(entry)? != id {
        return None;
    }
    let name_offset = read_u32(entry + 4)? as usize;
    let name_start = names_start.checked_add(name_offset)?;
    let name_data = SPELL_NAME_DATA.get(name_start..)?;
    let name_end = name_data.iter().position(|byte| *byte == 0)?;
    std::str::from_utf8(&name_data[..name_end]).ok()
}

fn read_u32(offset: usize) -> Option<u32> {
    let end = offset.checked_add(4)?;
    let bytes = SPELL_NAME_DATA.get(offset..end)?;
    Some(u32::from_le_bytes(bytes.try_into().ok()?))
}

#[cfg(test)]
mod tests {
    use super::get_spell_name;
    use std::time::Instant;

    #[test]
    fn binary_name_lookup_matches_distinct_db2_rows() {
        let started = Instant::now();
        let heart = get_spell_name(275825);
        eprintln!("first_spell_name_lookup_ns={}", started.elapsed().as_nanos());
        assert_eq!(heart, Some("Heart of Azeroth"));
        let repeated_started = Instant::now();
        for _ in 0..999 {
            let id = std::hint::black_box(275825);
            std::hint::black_box(get_spell_name(id));
        }
        let repeated_ns = repeated_started.elapsed().as_nanos();
        eprintln!("spell_name_lookup_ns_per_call_2_to_1000={}", repeated_ns / 999);
        assert_eq!(get_spell_name(133137), Some("Active"));
        assert_eq!(get_spell_name(u32::MAX), None);
    }
}
