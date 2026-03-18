use crate::engine::value::CalcValue;
use crate::engine::CalcError;
use dashu::float::FBig;
use dashu::integer::IBig;

pub fn parse_value(input: &str) -> Result<CalcValue, CalcError> {
    // Remove underscores (digit separators)
    let clean: String = input.chars().filter(|&c| c != '_').collect();

    if clean.is_empty() {
        return Err(CalcError::InvalidInput("Empty input".to_string()));
    }

    // Check for explicit base prefixes
    if let Some(rest) = clean
        .strip_prefix("0x")
        .or_else(|| clean.strip_prefix("0X"))
    {
        return parse_hex(rest);
    }
    if let Some(rest) = clean
        .strip_prefix("0o")
        .or_else(|| clean.strip_prefix("0O"))
    {
        return parse_octal(rest);
    }
    if let Some(rest) = clean
        .strip_prefix("0b")
        .or_else(|| clean.strip_prefix("0B"))
    {
        return parse_binary(rest);
    }
    // Handle negative with prefix
    if let Some(rest) = clean
        .strip_prefix("-0x")
        .or_else(|| clean.strip_prefix("-0X"))
    {
        return parse_hex(rest).map(negate);
    }
    if let Some(rest) = clean
        .strip_prefix("-0o")
        .or_else(|| clean.strip_prefix("-0O"))
    {
        return parse_octal(rest).map(negate);
    }
    if let Some(rest) = clean
        .strip_prefix("-0b")
        .or_else(|| clean.strip_prefix("-0B"))
    {
        return parse_binary(rest).map(negate);
    }

    // Check if it's a float (contains '.' or 'e'/'E')
    let lower = clean.to_lowercase();
    if lower.contains('.') || lower.contains('e') {
        parse_float(&clean)
    } else {
        parse_integer(&clean)
    }
}

fn parse_hex(s: &str) -> Result<CalcValue, CalcError> {
    IBig::from_str_radix(s, 16)
        .map(CalcValue::Integer)
        .map_err(|_| CalcError::InvalidInput(format!("Invalid hex number: 0x{}", s)))
}

fn parse_octal(s: &str) -> Result<CalcValue, CalcError> {
    IBig::from_str_radix(s, 8)
        .map(CalcValue::Integer)
        .map_err(|_| CalcError::InvalidInput(format!("Invalid octal number: 0o{}", s)))
}

fn parse_binary(s: &str) -> Result<CalcValue, CalcError> {
    IBig::from_str_radix(s, 2)
        .map(CalcValue::Integer)
        .map_err(|_| CalcError::InvalidInput(format!("Invalid binary number: 0b{}", s)))
}

fn parse_integer(s: &str) -> Result<CalcValue, CalcError> {
    s.parse::<IBig>()
        .map(CalcValue::Integer)
        .map_err(|_| CalcError::InvalidInput(format!("Invalid integer: {}", s)))
}

fn parse_float(s: &str) -> Result<CalcValue, CalcError> {
    s.parse::<f64>()
        .map_err(|_| CalcError::InvalidInput(format!("Invalid number: {}", s)))
        .and_then(|f| {
            if f.is_nan() || f.is_infinite() {
                Err(CalcError::InvalidInput(
                    "Invalid floating point value".to_string(),
                ))
            } else {
                FBig::try_from(f)
                    .map(CalcValue::Float)
                    .map_err(|_| CalcError::InvalidInput("Could not convert to float".to_string()))
            }
        })
}

fn negate(v: CalcValue) -> CalcValue {
    match v {
        CalcValue::Integer(n) => CalcValue::Integer(-n),
        CalcValue::Float(f) => CalcValue::Float(-f),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::CalcError;
    use dashu::integer::IBig;

    fn int_val(n: i64) -> CalcValue {
        CalcValue::Integer(IBig::from(n))
    }

    // ── integers ────────────────────────────────────────────────────────────

    #[test]
    fn test_parse_integer_positive() {
        assert_eq!(parse_value("42"), Ok(int_val(42)));
    }

    #[test]
    fn test_parse_integer_negative() {
        assert_eq!(parse_value("-17"), Ok(int_val(-17)));
    }

    #[test]
    fn test_parse_integer_zero() {
        assert_eq!(parse_value("0"), Ok(int_val(0)));
    }

    // ── floats ──────────────────────────────────────────────────────────────

    #[test]
    fn test_parse_float_decimal() {
        assert!(matches!(parse_value("3.14"), Ok(CalcValue::Float(_))));
    }

    #[test]
    fn test_parse_float_scientific() {
        assert!(matches!(parse_value("1.5e-3"), Ok(CalcValue::Float(_))));
    }

    #[test]
    fn test_parse_float_scientific_positive_exp() {
        assert!(matches!(parse_value("1e10"), Ok(CalcValue::Float(_))));
    }

    // ── hex ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_parse_hex_uppercase() {
        assert_eq!(parse_value("0xFF"), Ok(int_val(255)));
    }

    #[test]
    fn test_parse_hex_lowercase() {
        assert_eq!(parse_value("0xff"), Ok(int_val(255)));
    }

    #[test]
    fn test_parse_hex_prefix_uppercase() {
        assert_eq!(parse_value("0XFF"), Ok(int_val(255)));
    }

    #[test]
    fn test_parse_hex_negative() {
        assert_eq!(parse_value("-0xFF"), Ok(int_val(-255)));
    }

    // ── octal ────────────────────────────────────────────────────────────────

    #[test]
    fn test_parse_octal() {
        assert_eq!(parse_value("0o377"), Ok(int_val(255)));
    }

    #[test]
    fn test_parse_octal_prefix_uppercase() {
        assert_eq!(parse_value("0O377"), Ok(int_val(255)));
    }

    #[test]
    fn test_parse_octal_negative() {
        assert_eq!(parse_value("-0o10"), Ok(int_val(-8)));
    }

    // ── binary ───────────────────────────────────────────────────────────────

    #[test]
    fn test_parse_binary() {
        assert_eq!(parse_value("0b11111111"), Ok(int_val(255)));
    }

    #[test]
    fn test_parse_binary_prefix_uppercase() {
        assert_eq!(parse_value("0B101"), Ok(int_val(5)));
    }

    #[test]
    fn test_parse_binary_negative() {
        assert_eq!(parse_value("-0b101"), Ok(int_val(-5)));
    }

    // ── digit separators ────────────────────────────────────────────────────

    #[test]
    fn test_parse_digit_separators_integer() {
        assert_eq!(parse_value("1_000_000"), Ok(int_val(1_000_000)));
    }

    #[test]
    fn test_parse_digit_separators_hex() {
        assert_eq!(parse_value("0xFF_FF"), Ok(int_val(65535)));
    }

    // ── errors ───────────────────────────────────────────────────────────────

    #[test]
    fn test_parse_empty_string() {
        assert!(matches!(parse_value(""), Err(CalcError::InvalidInput(_))));
    }

    #[test]
    fn test_parse_garbage() {
        assert!(matches!(
            parse_value("abc"),
            Err(CalcError::InvalidInput(_))
        ));
    }

    #[test]
    fn test_parse_invalid_hex() {
        assert!(matches!(
            parse_value("0xGG"),
            Err(CalcError::InvalidInput(_))
        ));
    }

    #[test]
    fn test_parse_invalid_octal() {
        assert!(matches!(
            parse_value("0o99"),
            Err(CalcError::InvalidInput(_))
        ));
    }

    #[test]
    fn test_parse_invalid_binary() {
        assert!(matches!(
            parse_value("0b2"),
            Err(CalcError::InvalidInput(_))
        ));
    }
}
