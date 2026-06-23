/// Format a hash as uppercase hex with a space after every 4 characters.
pub fn format_hash(hash: &[u8], half: bool) -> String {
    let bytes = if half { &hash[..hash.len() / 2] } else { hash };
    let mut out = String::with_capacity(bytes.len() * 2 + bytes.len() / 2);
    for (i, byte) in bytes.iter().enumerate() {
        if i > 0 && i % 2 == 0 {
            out.push(' ');
        }
        out.push_str(&format!("{byte:02X}"));
    }
    out
}

/// Parse a hash entered as hex back into its raw bytes.
///
/// The inverse of [`format_hash`]: whitespace is ignored and hex digits are
/// case-insensitive, so a value copied straight from a rendered hash (e.g.
/// `"F381 3DE7 96D3 8033 …"`) parses cleanly. The result can be shorter than a
/// full 32-byte chain hash — [`format_hash`] renders only the first half by
/// default — and is matched as a prefix when looking the event up.
///
/// Returns `None` for an odd number of hex digits, a non-hex character, more
/// than 32 bytes (longer than a chain hash), or shorter than 4 bytes.
///
/// The 4 byte minimum length requirement makes it unlikely that you accidentally
/// end up with a different event if a typo happens, but does not protect against
/// trying to find events with brute-force.
pub fn parse_hash_prefix(input: &str) -> Option<Vec<u8>> {
    let digits: Vec<char> = input.chars().filter(|c| !c.is_whitespace()).collect();
    if digits.len() < 8 || digits.len() > 64 || !digits.len().is_multiple_of(2) {
        return None;
    }

    digits
        .chunks(2)
        .map(|pair| {
            let hi = pair[0].to_digit(16)?;
            let lo = pair[1].to_digit(16)?;
            Some((hi * 16 + lo) as u8)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_32_byte_hash_with_spaces_every_4_chars() {
        let hash: [u8; 32] = [
            0xF3, 0x81, 0x3D, 0xE7, 0x96, 0xD3, 0x80, 0x33, 0xFA, 0xF5, 0x8D, 0x2C, 0xE6, 0x94,
            0x61, 0xF0, 0x91, 0x84, 0x44, 0x6B, 0x54, 0x15, 0x8D, 0x5D, 0x67, 0x4A, 0xB7, 0xBC,
            0xE9, 0x2C, 0xE9, 0x8A,
        ];
        assert_eq!(
            format_hash(&hash, false),
            "F381 3DE7 96D3 8033 FAF5 8D2C E694 61F0 9184 446B 5415 8D5D 674A B7BC E92C E98A"
        );
    }

    #[test]
    fn formats_half_of_32_byte_hash() {
        let hash: [u8; 32] = [
            0xF3, 0x81, 0x3D, 0xE7, 0x96, 0xD3, 0x80, 0x33, 0xFA, 0xF5, 0x8D, 0x2C, 0xE6, 0x94,
            0x61, 0xF0, 0x91, 0x84, 0x44, 0x6B, 0x54, 0x15, 0x8D, 0x5D, 0x67, 0x4A, 0xB7, 0xBC,
            0xE9, 0x2C, 0xE9, 0x8A,
        ];
        assert_eq!(
            format_hash(&hash, true),
            "F381 3DE7 96D3 8033 FAF5 8D2C E694 61F0"
        );
    }

    #[test]
    fn empty_hash_produces_empty_string() {
        assert_eq!(format_hash(&[], false), "");
    }

    #[test]
    fn parse_hash_round_trips_a_formatted_hash() {
        let hash: [u8; 32] = [
            0xF3, 0x81, 0x3D, 0xE7, 0x96, 0xD3, 0x80, 0x33, 0xFA, 0xF5, 0x8D, 0x2C, 0xE6, 0x94,
            0x61, 0xF0, 0x91, 0x84, 0x44, 0x6B, 0x54, 0x15, 0x8D, 0x5D, 0x67, 0x4A, 0xB7, 0xBC,
            0xE9, 0x2C, 0xE9, 0x8A,
        ];

        assert_eq!(parse_hash_prefix(&format_hash(&hash, false)).unwrap(), hash);
        // The half-hash that documents render parses to the first 16 bytes.
        assert_eq!(
            parse_hash_prefix(&format_hash(&hash, true)).unwrap(),
            hash[..16]
        );
    }

    #[test]
    fn parse_hash_is_case_insensitive_and_ignores_whitespace() {
        assert_eq!(
            parse_hash_prefix("de ad\tBE\nef").unwrap(),
            [0xDE, 0xAD, 0xBE, 0xEF]
        );
    }

    #[test]
    fn parse_hash_rejects_malformed_input() {
        assert_eq!(parse_hash_prefix(""), None);
        assert_eq!(parse_hash_prefix("   "), None);
        assert_eq!(parse_hash_prefix("abc"), None); // odd digit count
        assert_eq!(parse_hash_prefix("zz"), None); // non-hex
        assert_eq!(parse_hash_prefix(&"a".repeat(6)), None); // shorter than 4 bytes
        assert_eq!(parse_hash_prefix(&"a".repeat(66)), None); // longer than 32 bytes
    }

    #[test]
    fn single_byte_has_no_trailing_space() {
        assert_eq!(format_hash(&[0xAB], false), "AB");
    }
}
