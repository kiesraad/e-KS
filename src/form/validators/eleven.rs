use crate::form::ValidationError;

pub fn validate_eleven_check() -> impl Fn(&str) -> Result<String, ValidationError> {
    move |value: &str| {
        let trimmed_value = value.trim();

        if trimmed_value.is_empty() {
            return Err(ValidationError::ValueShouldNotBeEmpty);
        }

        if trimmed_value.len() < 9 {
            return Err(ValidationError::ValueTooShort(trimmed_value.len(), 9));
        }

        if trimmed_value.len() > 9 {
            return Err(ValidationError::ValueTooLong(trimmed_value.len(), 9));
        }

        let mut checksum = 0;
        for (i, digit) in trimmed_value.chars().rev().enumerate() {
            checksum += (if i == 0 { -1 } else { i as i32 + 1 })
                * (digit.to_digit(10).ok_or(ValidationError::InvalidValue)?) as i32;
        }

        if checksum % 11 != 0 {
            return Err(ValidationError::InvalidChecksum);
        }

        Ok(trimmed_value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// All "valid" BSNs here have been randomly generated to match the "elfproef"
    #[test]
    fn accepts_valid_bsns() {
        assert_eq!(validate_eleven_check()("625576044").unwrap(), "625576044");

        let valid_bsns = [
            " 223395560 ",
            "  581964640",
            "870854306",
            "362861171",
            "375681024",
            "371593347",
            "439789679",
            "744389926",
            "829982073",
            "128460027",
            "012345672", // 8 digit BSN prefixed with 0
        ];
        for bsn in valid_bsns {
            assert_eq!(validate_eleven_check()(bsn).unwrap().len(), 9);
        }
    }

    #[test]
    fn deny_invalid_bsns() {
        let invalid_bsns = [
            "403727903",
            "223339816",
            "641808384",
            "728589738",
            "755177829",
            "819525755",
            "297648605",
            "608987805",
            "574126804",
            "613571295",
        ];
        for bsn in invalid_bsns {
            let err = validate_eleven_check()(bsn).unwrap_err();
            assert_eq!(err, ValidationError::InvalidChecksum);
        }
    }

    #[test]
    fn deny_invalid_symbols() {
        let invalid_bsns = [
            "40372790A",
            "B23339816",
            "64808384!",
            "728.58738",
            "75-177-89",
        ];
        for bsn in invalid_bsns {
            let err = validate_eleven_check()(bsn).unwrap_err();
            assert_eq!(err, ValidationError::InvalidValue);
        }
    }

    #[test]
    fn deny_invalid_length() {
        let err = validate_eleven_check()("0123456789").unwrap_err();
        assert_eq!(err, ValidationError::ValueTooLong(10, 9));

        let err = validate_eleven_check()("01234567").unwrap_err();
        assert_eq!(err, ValidationError::ValueTooShort(8, 9));
    }
}
