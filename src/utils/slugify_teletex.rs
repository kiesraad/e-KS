fn append_char(out: &mut String, c: char) {
    match c {
        // ASCII letters/digits
        'A'..='Z' => out.push(c.to_ascii_lowercase()),
        'a'..='z' | '0'..='9' => out.push(c),

        // Separators to be normalized to '-'
        ' ' | '-' | '_' | '/' | '\\' | ':' | ';' | ',' | '.' | '+' | '=' | '(' | ')' | '['
        | ']' | '{' | '}' | '|' => out.push('-'),

        // Latin-1 Supplement + Extended-A letters
        'À' | 'Á' | 'Â' | 'Ã' | 'Ä' | 'Å' | 'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'Ā' | 'Ă' | 'Ą'
        | 'ā' | 'ă' | 'ą' => out.push('a'),

        'Æ' | 'æ' => out.push_str("ae"),

        'Ç' | 'Ć' | 'Ĉ' | 'Ċ' | 'Č' | 'ç' | 'ć' | 'ĉ' | 'ċ' | 'č' => out.push('c'),

        'Ð' | 'Ď' | 'Đ' | 'ð' | 'ď' | 'đ' => out.push('d'),

        'È' | 'É' | 'Ê' | 'Ë' | 'Ē' | 'Ĕ' | 'Ė' | 'Ę' | 'Ě' | 'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ĕ'
        | 'ė' | 'ę' | 'ě' => out.push('e'),

        'Ĝ' | 'Ğ' | 'Ġ' | 'Ģ' | 'ĝ' | 'ğ' | 'ġ' | 'ģ' => out.push('g'),

        'Ĥ' | 'Ħ' | 'ĥ' | 'ħ' => out.push('h'),

        'Ì' | 'Í' | 'Î' | 'Ï' | 'Ĩ' | 'Ī' | 'Ĭ' | 'Į' | 'İ' | 'ì' | 'í' | 'î' | 'ï' | 'ĩ' | 'ī'
        | 'ĭ' | 'į' | 'ı' => out.push('i'),

        'Ĳ' | 'ĳ' => out.push_str("ij"),

        'Ĵ' | 'ĵ' => out.push('j'),

        'Ķ' | 'ķ' | 'ĸ' => out.push('k'),

        'Ĺ' | 'Ļ' | 'Ľ' | 'Ŀ' | 'Ł' | 'ĺ' | 'ļ' | 'ľ' | 'ŀ' | 'ł' => out.push('l'),

        'Ñ' | 'Ń' | 'Ņ' | 'Ň' | 'Ŋ' | 'ñ' | 'ń' | 'ņ' | 'ň' | 'ŉ' | 'ŋ' => out.push('n'),

        'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' | 'Ø' | 'Ō' | 'Ŏ' | 'Ő' | 'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø'
        | 'ō' | 'ŏ' | 'ő' => out.push('o'),

        'Œ' | 'œ' => out.push_str("oe"),

        'Ŕ' | 'Ŗ' | 'Ř' | 'ŕ' | 'ŗ' | 'ř' => out.push('r'),

        'Ś' | 'Ŝ' | 'Ş' | 'Š' | 'ś' | 'ŝ' | 'ş' | 'š' => out.push('s'),

        'ß' => out.push_str("ss"),

        'Þ' | 'þ' => out.push_str("th"),

        'Ţ' | 'Ť' | 'Ŧ' | 'ţ' | 'ť' | 'ŧ' => out.push('t'),

        'Ù' | 'Ú' | 'Û' | 'Ü' | 'Ũ' | 'Ū' | 'Ŭ' | 'Ů' | 'Ű' | 'Ų' | 'ù' | 'ú' | 'û' | 'ü' | 'ũ'
        | 'ū' | 'ŭ' | 'ů' | 'ű' | 'ų' => out.push('u'),

        'Ŵ' | 'ŵ' => out.push('w'),

        'Ý' | 'Ŷ' | 'Ÿ' | 'ý' | 'ÿ' | 'ŷ' => out.push('y'),

        'Ź' | 'Ż' | 'Ž' | 'ź' | 'ż' | 'ž' => out.push('z'),

        // other symbols are ignored
        _ => {}
    }
}

/// Slugify a string to normalize all special characters.
///
/// All characters are converted to lowercase letters, digits, and dashes to prevent issues with special characters in URLs and filenames.
pub fn slugify_teletex(input: &str) -> String {
    let mut output = String::with_capacity(input.len());

    // replace with normalized characters (lowercase letters, digits, and dashes)
    input.chars().for_each(|c| append_char(&mut output, c));

    // remove consecutive '-' characters
    output
        .split("-")
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify_teletex() {
        let input = "Héllo_Ŵörłð! \n\r\t Þis įŝ-å tésŧ.   ßlûgïfÿ/teletex: 123ŕæĳœ--žĵķ";
        let expected = "hello-world-this-is-a-test-sslugify-teletex-123raeijoe-zjk";
        assert_eq!(slugify_teletex(input), expected);
    }
}
