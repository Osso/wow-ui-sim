//! R88w.w2 ASCII-default-locale scanners. No GUID bit packing or lookup identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scanned {
    pub value: u64,
    pub consumed: usize,
}

pub fn decimal(bytes: &[u8]) -> Scanned {
    let mut out = Scanned {
        value: 0,
        consumed: 0,
    };
    for &byte in bytes.iter().take(20) {
        if !byte.is_ascii_digit() {
            break;
        }
        out.value = out
            .value
            .wrapping_mul(10)
            .wrapping_add(u64::from(byte - b'0'));
        out.consumed += 1;
    }
    out
}

pub fn hexadecimal(bytes: &[u8]) -> Scanned {
    let prefixed = bytes.len() > 2 && bytes[0] == b'0' && matches!(bytes[1], b'x' | b'X');
    let start = if prefixed { 2 } else { 0 };
    let mut out = Scanned {
        value: 0,
        consumed: start,
    };
    for &byte in bytes[start..].iter().take(16) {
        let Some(digit) = hex_digit(byte) else {
            break;
        };
        out.value = out.value * 16 + u64::from(digit);
        out.consumed += 1;
    }
    out
}

fn hex_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// Exact-case prefix and ASCII digits are VERIFIED for the default comparison locale.
/// Non-ASCII native digit behavior in other locales is deliberately not inferred.
pub fn opaque_key(bytes: &[u8]) -> Option<u32> {
    let digits = bytes.strip_prefix(b"Opaque-")?;
    if digits.is_empty() || !digits.iter().all(u8::is_ascii_digit) {
        return None;
    }
    Some(digits.iter().fold(0u32, |key, byte| {
        key.wrapping_mul(10).wrapping_add(u32::from(byte - b'0'))
    }))
}

#[cfg(all(test, feature = "optional-tests"))]
mod tests {
    use super::*;

    #[test]
    fn decimal_consumes_twenty_digits_wraps_and_allows_no_digit() {
        for (input, value, consumed) in [
            ("18446744073709551616!", 0, 20),
            ("18446744073709551617!", 1, 20),
            ("123456789012345678901", 12345678901234567890, 20),
            ("Z", 0, 0),
        ] {
            assert_eq!(decimal(input.as_bytes()), Scanned { value, consumed });
        }
    }
    #[test]
    fn hexadecimal_limits_digits_and_distinguishes_prefix_advance() {
        for (input, value, consumed) in [
            ("0xZ", 0, 2),
            ("0X12trailing", 18, 4),
            ("0x", 0, 1),
            ("ffffffffffffffffF", u64::MAX, 16),
            ("Z", 0, 0),
        ] {
            assert_eq!(hexadecimal(input.as_bytes()), Scanned { value, consumed });
        }
    }

    #[test]
    fn opaque_prefix_is_exact_and_key_wraps_at_u32() {
        assert_eq!(opaque_key(b"Opaque-1"), Some(1));
        assert_eq!(opaque_key(b"Opaque-4294967297"), Some(1));
        assert_eq!(opaque_key(b"Opaque-2"), Some(2));
        for absent in [b"Opaque-".as_slice(), b"opaque-1", b"Opaque-1Z", b"Other-1"] {
            assert_eq!(opaque_key(absent), None);
        }
    }
}
