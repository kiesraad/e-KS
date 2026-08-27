fn append_char(out: &mut String, c: char) {
    match c {
        // ASCII letters/digits
        'A'..='Z' | 'a'..='z' | '0'..='9' => out.push(c),

        // Separators to be normalized to '-'
        ' ' | '-' | '_' | '/' | '\\' | ':' | ';' | ',' | '.' | '+' | '=' | '(' | ')' | '['
        | ']' | '{' | '}' | '|' => out.push('-'),

        // Latin-1 Supplement + Extended-A letters
        'À' | 'Á' | 'Â' | 'Ã' | 'Ä' | 'Å' | 'Ā' | 'Ă' | 'Ą' => out.push('A'),
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ă' | 'ą' => out.push('a'),

        'Æ' => out.push_str("AE"),
        'æ' => out.push_str("ae"),

        'Ç' | 'Ć' | 'Ĉ' | 'Ċ' | 'Č' => out.push('C'),
        'ç' | 'ć' | 'ĉ' | 'ċ' | 'č' => out.push('c'),

        'Ð' | 'Ď' | 'Đ' => out.push('D'),
        'ð' | 'ď' | 'đ' => out.push('d'),

        'È' | 'É' | 'Ê' | 'Ë' | 'Ē' | 'Ĕ' | 'Ė' | 'Ę' | 'Ě' => out.push('E'),
        'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ĕ' | 'ė' | 'ę' | 'ě' => out.push('e'),

        'Ĝ' | 'Ğ' | 'Ġ' | 'Ģ' => out.push('G'),
        'ĝ' | 'ğ' | 'ġ' | 'ģ' => out.push('g'),

        'Ĥ' | 'Ħ' => out.push('H'),
        'ĥ' | 'ħ' => out.push('h'),

        'Ì' | 'Í' | 'Î' | 'Ï' | 'Ĩ' | 'Ī' | 'Ĭ' | 'Į' | 'İ' => out.push('I'),
        'ì' | 'í' | 'î' | 'ï' | 'ĩ' | 'ī' | 'ĭ' | 'į' | 'ı' => out.push('i'),

        'Ĳ' => out.push_str("IJ"),
        'ĳ' => out.push_str("ij"),

        'Ĵ' => out.push('J'),
        'ĵ' => out.push('j'),

        'Ķ' => out.push('K'),
        'ķ' | 'ĸ' => out.push('k'),

        'Ĺ' | 'Ļ' | 'Ľ' | 'Ŀ' | 'Ł' => out.push('L'),
        'ĺ' | 'ļ' | 'ľ' | 'ŀ' | 'ł' => out.push('l'),

        'Ñ' | 'Ń' | 'Ņ' | 'Ň' | 'Ŋ' => out.push('N'),
        'ñ' | 'ń' | 'ņ' | 'ň' | 'ŉ' | 'ŋ' => out.push('n'),

        'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' | 'Ø' | 'Ō' | 'Ŏ' | 'Ő' => out.push('O'),
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'ō' | 'ŏ' | 'ő' => out.push('o'),

        'Œ' => out.push_str("OE"),
        'œ' => out.push_str("oe"),

        'Ŕ' | 'Ŗ' | 'Ř' => out.push('R'),
        'ŕ' | 'ŗ' | 'ř' => out.push('r'),

        'Ś' | 'Ŝ' | 'Ş' | 'Š' => out.push('S'),
        'ś' | 'ŝ' | 'ş' | 'š' => out.push('s'),

        'ß' => out.push_str("ss"),

        'Þ' => out.push_str("TH"),
        'þ' => out.push_str("th"),

        'Ţ' | 'Ť' | 'Ŧ' => out.push('T'),
        'ţ' | 'ť' | 'ŧ' => out.push('t'),

        'Ù' | 'Ú' | 'Û' | 'Ü' | 'Ũ' | 'Ū' | 'Ŭ' | 'Ů' | 'Ű' | 'Ų' => out.push('U'),
        'ù' | 'ú' | 'û' | 'ü' | 'ũ' | 'ū' | 'ŭ' | 'ů' | 'ű' | 'ų' => out.push('u'),

        'Ŵ' => out.push('W'),
        'ŵ' => out.push('w'),

        'Ý' | 'Ŷ' | 'Ÿ' => out.push('Y'),
        'ý' | 'ÿ' | 'ŷ' => out.push('y'),

        'Ź' | 'Ż' | 'Ž' => out.push('Z'),
        'ź' | 'ż' | 'ž' => out.push('z'),

        // other symbols are ignored
        _ => {}
    }
}

/// Slugify a string to normalize all special characters.
///
/// All characters are converted to letters, digits, and dashes to prevent issues with special characters in URLs and filenames.
pub fn slugify_teletex(input: &str, only_lowercase: bool) -> String {
    let mut output = String::with_capacity(input.len());

    // replace with normalized characters (letters, digits, and dashes)
    input.chars().for_each(|c| append_char(&mut output, c));

    // remove consecutive '-' characters
    let output = output.split("-").filter(|s| !s.is_empty());
    if only_lowercase {
        output
            .map(str::to_ascii_lowercase)
            .collect::<Vec<_>>()
            .join("-")
    } else {
        output.collect::<Vec<_>>().join("-")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify_teletex() {
        let input = "Héllo_Ŵörłð! \n\r\t Þis įŝ-å tésŧ.   ßlûğïfÿ/teletex: 123ħŕæĳœ--žĵçķĲ/þë-ñÊŵ?q=ĆÄÐŒÑĶÏĢŁØŰŔŽŤĤÆĴŸŞ";

        let expected = "hello-world-this-is-a-test-sslugify-teletex-123hraeijoe-zjckij-the-newq-cadoenkiglourzthaejys";
        assert_eq!(slugify_teletex(input, true), expected);

        let expected = "Hello-World-THis-is-a-test-sslugify-teletex-123hraeijoe-zjckIJ-the-nEwq-CADOENKIGLOURZTHAEJYS";
        assert_eq!(slugify_teletex(input, false), expected);
    }
}
