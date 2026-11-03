//! No native CVar metadata is supplied for profiles without a pinned capture.

pub static CVAR_FLAGS: phf::Map<&'static str, u8> = phf::phf_map! {};
