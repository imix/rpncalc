use crate::engine::base::Base;
use dashu::float::FBig;
use dashu::integer::IBig;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CalcValue {
    Integer(IBig),
    Float(FBig),
}

impl CalcValue {
    pub fn to_f64(&self) -> f64 {
        match self {
            CalcValue::Integer(n) => n.to_string().parse::<f64>().unwrap_or(f64::NAN),
            CalcValue::Float(f) => f.to_f64().value(),
        }
    }

    pub fn from_f64(val: f64) -> CalcValue {
        // NaN and infinity cannot be represented by FBig; they become ZERO here.
        // Callers producing NaN/infinity from operations should return CalcError instead.
        CalcValue::Float(FBig::try_from(val).unwrap_or(FBig::ZERO))
    }

    #[allow(dead_code)]
    pub fn is_integer(&self) -> bool {
        matches!(self, CalcValue::Integer(_))
    }

    #[allow(dead_code)]
    pub fn to_ibig(&self) -> Option<IBig> {
        match self {
            CalcValue::Integer(n) => Some(n.clone()),
            CalcValue::Float(_) => None,
        }
    }

    /// Display with configurable float precision. Integers ignore precision.
    pub fn display_with_precision(&self, base: Base, precision: usize) -> String {
        match self {
            CalcValue::Integer(_) => self.display_with_base(base),
            CalcValue::Float(f) => format_fbig_prec(f, precision),
        }
    }

    pub fn display_with_base(&self, base: Base) -> String {
        match self {
            CalcValue::Integer(n) => {
                if *n == IBig::ZERO {
                    return "0".to_string();
                }
                let negative = *n < IBig::ZERO;
                let abs_n = if negative { -n.clone() } else { n.clone() };
                let s = format_ibig_base(&abs_n, base);
                match base {
                    Base::Dec => {
                        if negative {
                            format!("-{}", s)
                        } else {
                            s
                        }
                    }
                    Base::Hex => {
                        if negative {
                            format!("-0x{}", s)
                        } else {
                            format!("0x{}", s)
                        }
                    }
                    Base::Oct => {
                        if negative {
                            format!("-0o{}", s)
                        } else {
                            format!("0o{}", s)
                        }
                    }
                    Base::Bin => {
                        if negative {
                            format!("-0b{}", s)
                        } else {
                            format!("0b{}", s)
                        }
                    }
                }
            }
            CalcValue::Float(f) => format_fbig(f),
        }
    }
}

fn format_ibig_base(n: &IBig, base: Base) -> String {
    if *n == IBig::ZERO {
        return "0".to_string();
    }
    let radix = base.radix();
    let mut digits = Vec::new();
    let mut remaining = n.clone();
    let radix_big = IBig::from(radix);
    while remaining > IBig::ZERO {
        let digit = (&remaining % &radix_big)
            .to_string()
            .parse::<u32>()
            .unwrap_or(0);
        let ch = std::char::from_digit(digit, radix).unwrap_or('?');
        digits.push(ch);
        remaining /= &radix_big;
    }
    digits.reverse();
    match base {
        Base::Hex => digits
            .iter()
            .map(|c| c.to_uppercase().next().unwrap_or(*c))
            .collect(),
        _ => digits.iter().collect(),
    }
}

pub(crate) fn format_fbig(f: &FBig) -> String {
    let val = f.to_f64().value();
    if val.is_nan() || val.is_infinite() {
        return format!("{}", val);
    }
    let s = format!("{:.15}", val);
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}

pub(crate) fn format_fbig_prec(f: &FBig, precision: usize) -> String {
    let val = f.to_f64().value();
    if val.is_nan() || val.is_infinite() {
        return format!("{}", val);
    }
    let s = format!("{:.prec$}", val, prec = precision);
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}

impl fmt::Display for CalcValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_with_base(Base::Dec))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_value_integer_serde_roundtrip() {
        let original = CalcValue::Integer(IBig::from(12345));
        let json = serde_json::to_string(&original).expect("serialize");
        let restored: CalcValue = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(restored, CalcValue::Integer(_)));
        assert_eq!(original.to_f64(), restored.to_f64());
    }

    #[test]
    fn test_calc_value_float_serde_roundtrip() {
        let original = CalcValue::from_f64(3.14159);
        let json = serde_json::to_string(&original).expect("serialize");
        let restored: CalcValue = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(restored, CalcValue::Float(_)));
        let diff = (original.to_f64() - restored.to_f64()).abs();
        assert!(diff < 1e-10, "float roundtrip precision lost: {}", diff);
    }

    #[test]
    fn test_calc_value_negative_integer_serde() {
        let original = CalcValue::Integer(IBig::from(-999));
        let json = serde_json::to_string(&original).expect("serialize");
        let restored: CalcValue = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(original.to_f64(), restored.to_f64());
    }

    // ── Story 4.4: precision-aware display ──────────────────────────────────

    #[test]
    fn test_display_with_precision_float_10() {
        // 3.141592653589793 at precision 10 → "3.1415926536" (rounded, trailing zeros trimmed)
        let val = CalcValue::from_f64(std::f64::consts::PI);
        let s = val.display_with_precision(Base::Dec, 10);
        assert_eq!(s, "3.1415926536", "precision 10 PI: got {}", s);
    }

    #[test]
    fn test_display_with_precision_trims_zeros() {
        let val = CalcValue::from_f64(3.0);
        let s = val.display_with_precision(Base::Dec, 5);
        assert_eq!(s, "3", "3.0 at precision 5 should trim to '3', got {}", s);
    }

    #[test]
    fn test_display_with_precision_integer_ignores_precision() {
        let val = CalcValue::Integer(IBig::from(42));
        let s5 = val.display_with_precision(Base::Dec, 5);
        let s15 = val.display_with_precision(Base::Dec, 15);
        assert_eq!(s5, "42");
        assert_eq!(s15, "42");
    }

    #[test]
    fn test_format_fbig_trims_zeros() {
        let f = FBig::try_from(3.0).unwrap();
        assert_eq!(format_fbig(&f), "3");
    }

    #[test]
    fn test_format_fbig_keeps_decimals() {
        let f = FBig::try_from(3.14).unwrap();
        let s = format_fbig(&f);
        assert!(s.starts_with("3.14"), "expected 3.14..., got {}", s);
    }
}
