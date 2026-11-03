//! Generic definite-length CBOR-to-Lua decoding (RFC 8949 sections 3.1/3.3).
//! Lua represents numbers as doubles and both string kinds as raw byte strings;
//! null/undefined become nil. Tags and indefinite containers are unsupported,
//! rather than silently losing their meaning. Native edge policy is inferred.

use crate::lua_api::methods::{create_string_bytes, create_table};
use rilua::table_security::unwrap_secret;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val, runtime_error};

pub(super) fn decode(state: &mut LuaState, bytes: &[u8], maximum_depth: usize) -> LuaResult<Val> {
    let mut decoder = Decoder {
        bytes,
        position: 0,
        maximum_depth,
    };
    let result = decoder.value(state, 0)?;
    if decoder.position != bytes.len() {
        return Err(error("trailing data after the CBOR value"));
    }
    Ok(result)
}

fn error(message: &str) -> rilua::LuaError {
    runtime_error(format!("C_EncodingUtil.DeserializeCBOR: {message}"))
}

struct Decoder<'a> {
    bytes: &'a [u8],
    position: usize,
    maximum_depth: usize,
}

impl Decoder<'_> {
    fn take(&mut self, length: usize) -> LuaResult<&[u8]> {
        let end = self
            .position
            .checked_add(length)
            .ok_or_else(|| error("invalid length"))?;
        let bytes = self
            .bytes
            .get(self.position..end)
            .ok_or_else(|| error("truncated value"))?;
        self.position = end;
        Ok(bytes)
    }

    fn argument(&mut self, additional: u8) -> LuaResult<u64> {
        match additional {
            0..=23 => Ok(u64::from(additional)),
            24 => Ok(u64::from(self.take(1)?[0])),
            25 => Ok(u64::from(u16::from_be_bytes(
                self.take(2)?.try_into().unwrap(),
            ))),
            26 => Ok(u64::from(u32::from_be_bytes(
                self.take(4)?.try_into().unwrap(),
            ))),
            27 => Ok(u64::from_be_bytes(self.take(8)?.try_into().unwrap())),
            31 => Err(error("indefinite-length values are unsupported")),
            _ => Err(error("reserved additional information")),
        }
    }

    fn length(&mut self, additional: u8) -> LuaResult<usize> {
        usize::try_from(self.argument(additional)?)
            .map_err(|_| error("length exceeds address space"))
    }

    fn value(&mut self, state: &mut LuaState, depth: usize) -> LuaResult<Val> {
        let initial = self.take(1)?[0];
        let major = initial >> 5;
        let additional = initial & 31;
        match major {
            0 => Ok(Val::Num(self.argument(additional)? as f64)),
            1 => {
                // Add before the sole integer-to-double conversion; rounding
                // the argument first can change negative values beyond 2^53.
                let magnitude = u128::from(self.argument(additional)?) + 1;
                Ok(Val::Num(-(magnitude as f64)))
            }
            2 | 3 => {
                let length = self.length(additional)?;
                Ok(create_string_bytes(state, self.take(length)?))
            }
            4 | 5 => {
                if depth >= self.maximum_depth {
                    return Err(error("EncodingStackSizeLimit exceeded"));
                }
                let length = self.length(additional)?;
                let remaining = self.bytes.len() - self.position;
                let minimum_bytes_per_entry = if major == 5 { 2 } else { 1 };
                if length > remaining / minimum_bytes_per_entry {
                    return Err(error("truncated container"));
                }
                let table = create_table(state);
                let Val::Table(reference) = table else {
                    unreachable!()
                };
                for index in 0..length {
                    let key = if major == 4 {
                        Val::Num((index + 1) as f64)
                    } else {
                        self.value(state, depth + 1)?
                    };
                    let value = self.value(state, depth + 1)?;
                    if matches!(key, Val::Nil) || matches!(key, Val::Num(value) if value.is_nan()) {
                        return Err(error("map key cannot be nil or NaN"));
                    }
                    state
                        .gc
                        .tables
                        .get_mut(reference)
                        .unwrap()
                        .raw_set(key, value, &state.gc.string_arena)
                        .map_err(|_| error("invalid map key"))?;
                    state.gc.barrier_back(reference);
                }
                Ok(table)
            }
            6 => Err(error("tagged values are unsupported")),
            7 => self.simple(additional),
            _ => unreachable!(),
        }
    }

    fn simple(&mut self, additional: u8) -> LuaResult<Val> {
        match additional {
            20 => Ok(Val::Bool(false)),
            21 => Ok(Val::Bool(true)),
            22 | 23 => Ok(Val::Nil),
            25 => Ok(Val::Num(half_float(u16::from_be_bytes(
                self.take(2)?.try_into().unwrap(),
            )))),
            26 => Ok(Val::Num(
                f32::from_bits(u32::from_be_bytes(self.take(4)?.try_into().unwrap())) as f64,
            )),
            27 => Ok(Val::Num(f64::from_bits(u64::from_be_bytes(
                self.take(8)?.try_into().unwrap(),
            )))),
            _ => Err(error("unsupported simple value")),
        }
    }
}

fn half_float(bits: u16) -> f64 {
    let sign = if bits & 0x8000 == 0 { 1.0 } else { -1.0 };
    let exponent = (bits >> 10) & 31;
    let fraction = f64::from(bits & 1023);
    sign * match exponent {
        0 => fraction * 2.0_f64.powi(-24),
        31 if fraction == 0.0 => f64::INFINITY,
        31 => f64::NAN,
        _ => (1.0 + fraction / 1024.0) * 2.0_f64.powi(i32::from(exponent) - 15),
    }
}

/// Inferred wire choices: dense 1..N tables (including empty) are arrays;
/// sparse/mixed tables are maps, nil is null, and strings are byte strings.
/// Map order, sparse-array thresholds and float widths are not native-byte
/// verified. These choices preserve decoded Lua values and key types.
pub(super) fn encode(
    state: &LuaState,
    value: Val,
    maximum_depth: usize,
    ignore_errors: bool,
) -> LuaResult<Vec<u8>> {
    let mut encoder = Encoder {
        output: Vec::new(),
        ancestors: Vec::new(),
        maximum_depth,
        ignore_errors,
    };
    encoder.value(state, value)?;
    Ok(encoder.output)
}

struct Encoder {
    output: Vec<u8>,
    ancestors: Vec<GcRef<Table>>,
    maximum_depth: usize,
    ignore_errors: bool,
}

fn encode_error(message: &str) -> rilua::LuaError {
    runtime_error(format!("C_EncodingUtil.SerializeCBOR: {message}"))
}

impl Encoder {
    fn argument(&mut self, major: u8, value: u64) {
        write_argument(&mut self.output, major, value);
    }

    fn number(&mut self, value: f64) {
        // Preserve doubles outside the contiguous exact-integer range using
        // floating-point wire data. Subtracting one there can round a CBOR
        // negative-integer argument before it has been encoded.
        const MAX_EXACT_INTEGER: f64 = 9_007_199_254_740_992.0;
        let integer = value.fract() == 0.0
            && value.abs() <= MAX_EXACT_INTEGER
            && !(value == 0.0 && value.is_sign_negative());
        if integer && value >= 0.0 {
            self.argument(0, value as u64);
        } else if integer && value < 0.0 {
            self.argument(1, (-1.0 - value) as u64);
        } else {
            self.output.push(0xfb);
            self.output
                .extend_from_slice(&value.to_bits().to_be_bytes());
        }
    }

    fn value(&mut self, state: &LuaState, value: Val) -> LuaResult<()> {
        // Secret values remain subject to AllowedWhenUntainted even when
        // unsupported-value replacement is requested.
        match unwrap_secret(state, value)? {
            Val::Nil => self.output.push(0xf6),
            Val::Bool(value) => self.output.push(if value { 0xf5 } else { 0xf4 }),
            Val::Num(value) => self.number(value),
            Val::Str(reference) => {
                let bytes = state
                    .gc
                    .string_arena
                    .get(reference)
                    .ok_or_else(|| encode_error("string is unavailable"))?
                    .data();
                self.argument(2, bytes.len() as u64);
                self.output.extend_from_slice(bytes);
            }
            Val::Table(reference) => self.table(state, reference)?,
            _ if self.ignore_errors => self.output.push(0xf7),
            _ => return Err(encode_error("unsupported Lua value")),
        }
        Ok(())
    }

    fn table(&mut self, state: &LuaState, reference: GcRef<Table>) -> LuaResult<()> {
        if self.ancestors.len() >= self.maximum_depth {
            return Err(encode_error("EncodingStackSizeLimit exceeded"));
        }
        if self.ancestors.contains(&reference) {
            return Err(encode_error("recursive table reference"));
        }
        let table = state
            .gc
            .tables
            .get(reference)
            .ok_or_else(|| encode_error("table is unavailable"))?;
        let mut entries: Vec<(Val, Val)> = table
            .array_slice()
            .iter()
            .copied()
            .enumerate()
            .filter(|(_, value)| !matches!(value, Val::Nil))
            .map(|(index, value)| (Val::Num((index + 1) as f64), value))
            .chain(
                table
                    .hash_entries()
                    .into_iter()
                    .filter(|(_, value)| !matches!(value, Val::Nil)),
            )
            .collect();
        let dense = entries.iter().all(|(key, _)| {
            matches!(key, Val::Num(index)
            if index.fract() == 0.0 && *index >= 1.0 && *index <= entries.len() as f64)
        });
        self.ancestors.push(reference);
        self.argument(if dense { 4 } else { 5 }, entries.len() as u64);
        if dense {
            entries.sort_unstable_by(|(left, _), (right, _)| {
                let (Val::Num(left), Val::Num(right)) = (left, right) else {
                    unreachable!()
                };
                left.partial_cmp(right).unwrap()
            });
            for (_, value) in entries {
                self.value(state, value)?;
            }
        } else {
            for (key, value) in entries {
                let key = unwrap_secret(state, key)?;
                if !matches!(
                    key,
                    Val::Bool(_) | Val::Num(_) | Val::Str(_) | Val::Table(_)
                ) {
                    return Err(encode_error("unsupported map key"));
                }
                self.value(state, key)?;
                self.value(state, value)?;
            }
        }
        self.ancestors.pop();
        Ok(())
    }
}

fn write_argument(output: &mut Vec<u8>, major: u8, value: u64) {
    let prefix = major << 5;
    match value {
        0..=23 => output.push(prefix | value as u8),
        24..=255 => output.extend_from_slice(&[prefix | 24, value as u8]),
        256..=65535 => {
            output.push(prefix | 25);
            output.extend_from_slice(&(value as u16).to_be_bytes());
        }
        65536..=4294967295 => {
            output.push(prefix | 26);
            output.extend_from_slice(&(value as u32).to_be_bytes());
        }
        _ => {
            output.push(prefix | 27);
            output.extend_from_slice(&value.to_be_bytes());
        }
    }
}

pub(super) fn encode_id_flag_maps(maps: &[(u32, Vec<u32>)]) -> Vec<u8> {
    let mut output = Vec::new();
    write_argument(&mut output, 5, maps.len() as u64);
    for (id, flags) in maps {
        write_argument(&mut output, 0, u64::from(*id));
        write_argument(&mut output, 5, flags.len() as u64);
        for flag in flags {
            write_argument(&mut output, 0, u64::from(*flag));
            write_argument(&mut output, 0, 1);
        }
    }
    output
}
