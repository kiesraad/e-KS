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
    fn single_byte_has_no_trailing_space() {
        assert_eq!(format_hash(&[0xAB], false), "AB");
    }
}
