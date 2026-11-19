//! Generated Retail AreaTable name index; see `area_names_retail.provenance.json`.
//! VERIFIED (DB2 12.1.5.70783); names are looked up lazily from the data file.

const AREA_NAME_DATA: &[u8] = include_bytes!("area_names_retail.bin");
const HEADER_BYTES: usize = 4;
const ENTRY_BYTES: usize = 8;

pub fn get_area_name(id: u32) -> Option<&'static str> {
    let count = read_u32(0)? as usize;
    let names_start = HEADER_BYTES.checked_add(count.checked_mul(ENTRY_BYTES)?)?;
    if names_start > AREA_NAME_DATA.len() {
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
    let name_data = AREA_NAME_DATA.get(name_start..)?;
    let name_end = name_data.iter().position(|byte| *byte == 0)?;
    std::str::from_utf8(&name_data[..name_end]).ok()
}

fn read_u32(offset: usize) -> Option<u32> {
    let end = offset.checked_add(4)?;
    let bytes = AREA_NAME_DATA.get(offset..end)?;
    Some(u32::from_le_bytes(bytes.try_into().ok()?))
}
