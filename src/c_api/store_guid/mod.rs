//! VERIFIED R88w.w2 scanners/limits; symbolic identity approved by main #1310.
//! UNKNOWN separator exceptions, exhaustion, WorldLayer progression and sentinels
//! are deliberately not assigned identities. This is not native GUID bit packing.
mod dispatch;
mod scanners;
pub use scanners::opaque_key;
use scanners::{decimal, hexadecimal};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Identity {
    pub kind: &'static str,
    pub form: &'static str,
    pub fields: Vec<u64>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Known(Identity),
    KnownEmpty,
    Unknown,
}
use Outcome::*;

pub fn parse(bytes: &[u8]) -> Outcome {
    if !bytes.is_ascii() || bytes.contains(&0) {
        return Unknown;
    }
    let (kind, payload) = dispatch::select_parser(bytes);
    match kind {
        "Uniq" => uniq(payload),
        "Player" => player(payload),
        "Housing" => housing(payload),
        "ClubFinder" => club_finder(payload),
        "WorldLayer" => Unknown,
        _ => regular(kind, payload),
    }
}

fn uniq(bytes: &[u8]) -> Outcome {
    if bytes.iter().take(16).all(|b| *b == b'0') {
        return KnownEmpty;
    }
    if bytes.eq_ignore_ascii_case(b"WOWGUID_")
        || bytes.eq_ignore_ascii_case(b"WOWGUID_NULL")
        || bytes.eq_ignore_ascii_case(b"WOWGUID_FROM_STRING_FAILED")
    {
        return KnownEmpty;
    }
    // VERIFIED: all remaining default forms start with WOWGUID_ (R88w.w2).
    // Compare only available bytes: a matching short marker remains UNKNOWN.
    if bytes
        .iter()
        .zip(b"WOWGUID_")
        .any(|(actual, expected)| !actual.eq_ignore_ascii_case(expected))
    {
        return KnownEmpty;
    }
    // UNKNOWN sentinel priority and short matching marker bounds (#1310).
    Unknown
}

fn known(kind: &'static str, form: &'static str, fields: Vec<u64>) -> Outcome {
    Known(Identity { kind, form, fields })
}

fn regular(kind: &'static str, bytes: &[u8]) -> Outcome {
    let Some((limits, hex_limit)) = schema(kind) else {
        return Unknown;
    };
    let canonical = if kind == "LMMParty" { "Party" } else { kind };
    let mut result = fields(canonical, "fields", bytes, limits, hex_limit);
    if matches!(kind, "StaticDoor" | "Transport") {
        if let Known(identity) = &mut result {
            if identity.fields[0] > 1048575 {
                identity.fields[0] = 0;
            }
        }
    }
    result
}

// Canonical '-' separators are supported. Other skip-byte cases stay UNKNOWN,
// never KnownEmpty: the native exception set has not been established.
fn fields(
    kind: &'static str,
    form: &'static str,
    bytes: &[u8],
    limits: &[u64],
    hex_limit: Option<u64>,
) -> Outcome {
    let (mut values, bytes) = match decimal_fields(bytes, limits, hex_limit.is_some()) {
        Ok(result) => result,
        Err(outcome) => return outcome,
    };
    if let Some(limit) = hex_limit {
        if bytes.is_empty() {
            return Unknown;
        }
        let read = hexadecimal(bytes);
        if read.consumed == 0 && kind != "Party" {
            return Unknown;
        }
        if read.value > limit {
            return KnownEmpty;
        }
        values.push(read.value);
    }
    known(kind, form, values)
}

fn decimal_fields<'a>(
    mut bytes: &'a [u8],
    limits: &[u64],
    has_hex: bool,
) -> Result<(Vec<u64>, &'a [u8]), Outcome> {
    let mut values = Vec::new();
    for (index, &limit) in limits.iter().enumerate() {
        if bytes.is_empty() {
            return Err(Unknown);
        }
        let read = decimal(bytes);
        if read.value > limit {
            return Err(KnownEmpty);
        }
        values.push(read.value);
        bytes = &bytes[read.consumed..];
        if index + 1 < limits.len() || has_hex {
            let Some(rest) = bytes.strip_prefix(b"-") else {
                return Err(Unknown);
            };
            bytes = rest;
        } else if read.consumed == 0 {
            return Err(Unknown);
        }
    }
    Ok((values, bytes))
}

fn player(bytes: &[u8]) -> Outcome {
    let realm = decimal(bytes);
    if realm.value > 65535 {
        return KnownEmpty;
    }
    let Some(rest) = bytes[realm.consumed..].strip_prefix(b"-") else {
        return if bytes.len() > realm.consumed {
            KnownEmpty
        } else {
            Unknown
        };
    };
    if rest.get(1) == Some(&b'-') {
        return selected_player(realm.value, rest);
    }
    player_hex("omitted-selector", vec![realm.value], rest)
}

fn selected_player(realm: u64, bytes: &[u8]) -> Outcome {
    let form = match bytes[0] {
        b'0' => "selector-0",
        b'1' => "selector-1",
        b'2' => "selector-2",
        b'3' => "selector-3",
        _ => return KnownEmpty,
    };
    let mut values = vec![realm];
    let mut rest = &bytes[2..];
    if bytes[0] == b'2' {
        let read = decimal(rest);
        if read.value > 16777215 {
            return KnownEmpty;
        }
        values.push(read.value);
        let Some(tail) = rest[read.consumed..].strip_prefix(b"-") else {
            return if rest.len() > read.consumed {
                KnownEmpty
            } else {
                Unknown
            };
        };
        rest = tail;
    }
    player_hex(form, values, rest)
}

fn player_hex(form: &'static str, mut values: Vec<u64>, bytes: &[u8]) -> Outcome {
    if bytes.is_empty() {
        return Unknown;
    }
    let read = hexadecimal(bytes);
    if read.consumed == 0 {
        return KnownEmpty;
    }
    values.push(read.value);
    known("Player", form, values)
}

fn club_finder(bytes: &[u8]) -> Outcome {
    if bytes.is_empty() {
        return Unknown;
    }
    if decimal(bytes).value == 1 {
        fields(
            "ClubFinder",
            "decimal",
            bytes,
            &[255, 4294967295, 65535, u64::MAX],
            None,
        )
    } else {
        fields(
            "ClubFinder",
            "hex",
            bytes,
            &[255, 4294967295],
            Some(u64::MAX),
        )
    }
}

fn housing(bytes: &[u8]) -> Outcome {
    if bytes.is_empty() {
        return Unknown;
    }
    let (form, limits, hex): (&str, &[u64], _) = match decimal(bytes).value {
        1 => ("1", &[5, 65535, 4294967295], Some(u64::MAX)),
        2 => ("2", &[5, 4294967295], Some(u64::MAX)),
        3 => ("3", &[5, 32767, u64::MAX, 63], None),
        4 => ("4", &[5, 32767, 4294967295], Some(u64::MAX)),
        5 => ("5", &[5, 4294967295], Some(u64::MAX)),
        _ => return KnownEmpty,
    };
    fields("Housing", form, bytes, limits, hex)
}

fn schema(kind: &str) -> Option<(&'static [u64], Option<u64>)> {
    SCHEMAS
        .iter()
        .find(|(names, _, _)| names.contains(&kind))
        .map(|(_, limits, hex)| (*limits, *hex))
}
const SCHEMAS: &[(&[&str], &[u64], Option<u64>)] = &[
    (
        &["Item", "ToolsClient"],
        &[65535, 16777215],
        Some(18446744073709551615),
    ),
    (
        &[
            "WorldTransaction",
            "Conversation",
            "Creature",
            "Vehicle",
            "Pet",
            "GameObject",
            "DynamicObject",
            "AreaTrigger",
            "Corpse",
            "LootObject",
            "SceneObject",
            "Scenario",
            "AIGroup",
            "DynamicDoor",
            "Vignette",
            "CallForHelp",
            "AIResource",
            "AILock",
            "AILockTicket",
            "Cast",
            "MeshObject",
            "Entity",
        ],
        &[63, 65535, 8191, 16777215, 8388607],
        Some(1099511627775),
    ),
    (
        &["StaticDoor", "Transport"],
        &[18446744073709551615],
        Some(18446744073709551615),
    ),
    (&["ClientActor"], &[65535, 65535, 4294967295], None),
    (
        &["ChatChannel"],
        &[65535, 1, 1, 65535, 63, 1],
        Some(18446744073709551615),
    ),
    (
        &[
            "Party",
            "PetBattle",
            "UniqUserClient",
            "ClientSession",
            "ClientConnection",
            "LMMParty",
        ],
        &[65535, 4294967295],
        Some(18446744073709551615),
    ),
    (
        &["Guild", "ArenaTeam"],
        &[65535],
        Some(18446744073709551615),
    ),
    (
        &[
            "WowAccount",
            "BNetAccount",
            "GMTask",
            "RaidGroup",
            "Spell",
            "Mail",
            "UserRouter",
            "PVPQueueGroup",
            "UserClient",
            "BattlePet",
            "CommerceObj",
        ],
        &[288230376151711743],
        Some(18446744073709551615),
    ),
    (
        &["MobileSession"],
        &[65535, 511],
        Some(18446744073709551615),
    ),
    (&["WebObj"], &[65535, 31, 3], Some(18446744073709551615)),
    (
        &["LFGObject"],
        &[15, 15, 15, 255, 1],
        Some(18446744073709551615),
    ),
    (&["LFGList"], &[15], Some(18446744073709551615)),
    (
        &["LMMLobby"],
        &[65535, 65535, 255, 255],
        Some(18446744073709551615),
    ),
];
#[cfg(all(test, feature = "optional-tests"))]
mod tests {
    use super::*;
    #[test]
    fn backed_player_values_and_verified_failure() {
        assert_eq!(
            parse(b"Player-1-12-FF"),
            known("Player", "omitted-selector", vec![1, 0x12])
        );
        assert_eq!(
            parse(b"Player-1-0xZ"),
            known("Player", "omitted-selector", vec![1, 0])
        );
        assert_ne!(parse(b"Player-1-12"), parse(b"Player-2-12"));
        assert_eq!(parse(b"Player-65536-12"), KnownEmpty);
    }
    #[test]
    fn verified_lmm_party_kind_equivalence() {
        assert_eq!(parse(b"LMMParty-1-2-FF"), parse(b"Party-1-2-FF"));
    }
    #[test]
    fn bounded_limits_and_represented_zero() {
        assert_eq!(parse(b"Guild-65536-1"), KnownEmpty);
        assert_eq!(parse(b"MobileSession-1-512-1"), KnownEmpty);
        assert_eq!(parse(b"StaticDoor-1048576-1"), parse(b"StaticDoor-0-1"));
        assert_eq!(parse(b"Transport-1048577-2"), parse(b"Transport-0-2"));
        assert_eq!(
            parse(b"Party-1-2-Z"),
            known("Party", "fields", vec![1, 2, 0])
        );
    }
    #[test]
    fn scanner_rules_reach_symbolic_identity() {
        assert_eq!(parse(b"pLaYeR-1-fF"), parse(b"Player-1-FF"));
        assert_eq!(parse(b"Party--2-FF"), parse(b"Party-0-2-FF"));
        assert_eq!(
            parse(b"Player-18446744073709551616-1"),
            parse(b"Player-0-1")
        );
        assert_eq!(
            parse(b"Player-1-FFFFFFFFFFFFFFFFF"),
            parse(b"Player-1-FFFFFFFFFFFFFFFF")
        );
    }
    #[test]
    fn definite_default_marker_mismatches_are_failed_reads() {
        assert_eq!(parse(b"bad"), KnownEmpty);
        assert_eq!(parse(b"UnknownType-1-2"), KnownEmpty);
        assert_eq!(parse(b"Opaque-1"), KnownEmpty);
        assert_eq!(parse(b"Opaque-4294967297"), KnownEmpty);
        assert_eq!(parse(b"0Z"), KnownEmpty);
    }
    #[test]
    fn verified_empty_forms() {
        assert_eq!(parse(b""), KnownEmpty);
        assert_eq!(parse(b"0000000000000000trailing"), KnownEmpty);
        assert_eq!(parse(b"WOWGUID_FROM_STRING_FAILED"), KnownEmpty);
    }
}
