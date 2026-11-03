//! Binary encoding and decoding for C_EncodingUtil. JSON and hex remain
//! separate compatibility surfaces.
//!
//! Contracts: Forever 70707 EncodingUtil{,Constants}Documentation.lua;
//! formats: RFC 4648, RFC 1951/1950/1952 and RFC 8949.
//! Client measurements of fresh CBOR trees and compression checksums:
//! https://github.com/Questie/QuestieDB/blob/master/docs/client-metadata-probes.md
//! Exact native error text and unsupported-input edge behavior remain unproven.

mod cbor;

use std::io::{Read, Write};

use crate::lua_api::methods::{create_string_bytes, table_get};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use base64::Engine;
use base64::alphabet;
use base64::engine::{DecodePaddingMode, GeneralPurpose, GeneralPurposeConfig};
use flate2::{Compression, Decompress, FlushDecompress, Status};
use rilua::table_security::unwrap_secret;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

const DECOMPRESS_SIZE_LIMIT: usize = 104_857_600;
const STACK_SIZE_LIMIT: usize = 100;

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_EncodingUtil")?;
    table_set_rust_fn_static(state, namespace, "EncodeBase64", encode_base64)?;
    table_set_rust_fn_static(state, namespace, "DecodeBase64", decode_base64)?;
    table_set_rust_fn_static(state, namespace, "CompressString", compress_string)?;
    table_set_rust_fn_static(state, namespace, "DecompressString", decompress_string)?;
    table_set_rust_fn_static(state, namespace, "SerializeCBOR", serialize_cbor)?;
    table_set_rust_fn_static(state, namespace, "DeserializeCBOR", deserialize_cbor)
}

fn source_bytes(state: &LuaState) -> LuaResult<Vec<u8>> {
    // These APIs specify AllowedWhenUntainted. Authenticate the original
    // argument before reading its bytes; do not turn binary data into UTF-8.
    let value = unwrap_secret(state, stack_val(state, 1))?;
    let Val::Str(reference) = value else {
        return Err(runtime_error("C_EncodingUtil requires a source string"));
    };
    state
        .gc
        .string_arena
        .get(reference)
        .map(|value| value.data().to_vec())
        .ok_or_else(|| runtime_error("C_EncodingUtil source string is unavailable"))
}

fn optional_enum(state: &LuaState, index: i32, maximum: u8) -> LuaResult<u8> {
    match unwrap_secret(state, stack_val(state, index))? {
        Val::Nil => Ok(0),
        Val::Num(value) if value.fract() == 0.0 && value >= 0.0 && value <= f64::from(maximum) => {
            Ok(value as u8)
        }
        _ => Err(runtime_error(format!(
            "C_EncodingUtil invalid enum at argument {index}"
        ))),
    }
}

fn base64_engine(variant: u8) -> GeneralPurpose {
    let alphabet = if variant == 0 {
        &alphabet::STANDARD
    } else {
        &alphabet::URL_SAFE
    };
    // The documentation allows potential padding; accept padded or unpadded
    // encodings, while rejecting invalid alphabet bytes and nonzero pad bits.
    GeneralPurpose::new(
        alphabet,
        GeneralPurposeConfig::new().with_decode_padding_mode(DecodePaddingMode::Indifferent),
    )
}

fn encode_base64(state: &mut LuaState) -> LuaResult<u32> {
    let source = source_bytes(state)?;
    let variant = optional_enum(state, 2, 1)?;
    let output = base64_engine(variant).encode(source);
    let value = create_string_bytes(state, output.as_bytes());
    state.push(value);
    Ok(1)
}

fn decode_base64(state: &mut LuaState) -> LuaResult<u32> {
    let source = source_bytes(state)?;
    let variant = optional_enum(state, 2, 1)?;
    let output = base64_engine(variant)
        .decode(source)
        .map_err(|error| runtime_error(format!("C_EncodingUtil.DecodeBase64: {error}")))?;
    let value = create_string_bytes(state, &output);
    state.push(value);
    Ok(1)
}

fn compress_string(state: &mut LuaState) -> LuaResult<u32> {
    let source = source_bytes(state)?;
    let method = optional_enum(state, 2, 2)?;
    let level = match optional_enum(state, 3, 2)? {
        1 => Compression::fast(),
        2 => Compression::best(),
        _ => Compression::default(),
    };
    let output = (|| -> std::io::Result<Vec<u8>> {
        match method {
            0 => {
                let mut encoder = flate2::write::DeflateEncoder::new(Vec::new(), level);
                encoder.write_all(&source)?;
                encoder.finish()
            }
            1 => {
                let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), level);
                encoder.write_all(&source)?;
                encoder.finish()
            }
            _ => {
                let mut encoder = flate2::write::GzEncoder::new(Vec::new(), level);
                encoder.write_all(&source)?;
                encoder.finish()
            }
        }
    })()
    .map_err(|error| runtime_error(format!("C_EncodingUtil.CompressString: {error}")))?;
    let value = create_string_bytes(state, &output);
    state.push(value);
    Ok(1)
}

fn decompress_string(state: &mut LuaState) -> LuaResult<u32> {
    let source = source_bytes(state)?;
    let method = optional_enum(state, 2, 2)?;
    let output = if method == 2 {
        let decoder = flate2::read::GzDecoder::new(source.as_slice());
        let mut output = Vec::new();
        decoder
            .take(DECOMPRESS_SIZE_LIMIT as u64 + 1)
            .read_to_end(&mut output)
            .map_err(|error| runtime_error(format!("C_EncodingUtil.DecompressString: {error}")))?;
        ensure_output_limit(output.len())?;
        output
    } else {
        inflate(&source, method == 1)?
    };
    let value = create_string_bytes(state, &output);
    state.push(value);
    Ok(1)
}

fn ensure_output_limit(length: usize) -> LuaResult<()> {
    if length > DECOMPRESS_SIZE_LIMIT {
        return Err(runtime_error(
            "C_EncodingUtil.DecompressString exceeds EncodingDecompressSizeLimit",
        ));
    }
    Ok(())
}

fn inflate(source: &[u8], zlib: bool) -> LuaResult<Vec<u8>> {
    let mut decoder = Decompress::new(zlib);
    let mut output = Vec::new();
    loop {
        let input_before = decoder.total_in();
        let output_before = decoder.total_out();
        let mut chunk = [0; 8192];
        let status = decoder
            .decompress(
                &source[input_before as usize..],
                &mut chunk,
                // A fixed-size output buffer may fill before StreamEnd.
                // Finish requires enough room for all remaining output in
                // some backends; stream until the decoder reports completion.
                FlushDecompress::None,
            )
            .map_err(|error| runtime_error(format!("C_EncodingUtil.DecompressString: {error}")))?;
        let produced = (decoder.total_out() - output_before) as usize;
        ensure_output_limit(output.len() + produced)?;
        output.extend_from_slice(&chunk[..produced]);
        if status == Status::StreamEnd {
            return Ok(output);
        }
        if decoder.total_in() == input_before && produced == 0 {
            return Err(runtime_error(
                "C_EncodingUtil.DecompressString: incomplete compressed stream",
            ));
        }
    }
}

fn deserialize_cbor(state: &mut LuaState) -> LuaResult<u32> {
    let source = source_bytes(state)?;
    if source.is_empty() {
        // The native Forever CVar default is empty and selected CvarUtil.lua
        // uses this API unguarded before `or {}`. Infer one nil for the empty
        // sentinel from that consumer and the documented nilable return.
        state.push(Val::Nil);
        return Ok(1);
    }
    let value = cbor::decode(state, &source, STACK_SIZE_LIMIT)?;
    state.push(value);
    Ok(1)
}

/// Write a validated legacy persisted ID → flag-map value in the actual CVar
/// wire format. CVar storage owns detection and validation of legacy data.
pub(crate) fn encode_persisted_id_flag_maps(maps: &[(u32, Vec<u32>)]) -> String {
    base64_engine(0).encode(cbor::encode_id_flag_maps(maps))
}

fn serialize_cbor(state: &mut LuaState) -> LuaResult<u32> {
    let value = unwrap_secret(state, stack_val(state, 1))?;
    let options = unwrap_secret(state, stack_val(state, 2))?;
    let ignore_errors = match options {
        Val::Nil => false,
        Val::Table(_) => {
            let flag = table_get(state, options, "ignoreSerializationErrors");
            match unwrap_secret(state, flag)? {
                Val::Nil | Val::Bool(false) => false,
                Val::Bool(true) => true,
                _ => {
                    return Err(runtime_error(
                        "C_EncodingUtil.SerializeCBOR: ignoreSerializationErrors must be a boolean",
                    ));
                }
            }
        }
        _ => {
            return Err(runtime_error(
                "C_EncodingUtil.SerializeCBOR: options must be a table or nil",
            ));
        }
    };
    let output = cbor::encode(state, value, STACK_SIZE_LIMIT, ignore_errors)?;
    let value = create_string_bytes(state, &output);
    state.push(value);
    Ok(1)
}
