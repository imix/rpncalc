use crate::engine::error::CalcError;
use dashu::float::{round::mode::Zero, Context, FBig};
use dashu::integer::IBig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnitCategory {
    Weight,
    Length,
    Temperature,
}

impl UnitCategory {
    pub fn name(&self) -> &'static str {
        match self {
            UnitCategory::Weight => "weight",
            UnitCategory::Length => "length",
            UnitCategory::Temperature => "temperature",
        }
    }
}

/// A physical unit. Linear units have a `to_base` scale factor as an exact
/// decimal string; temperature uses None and is handled by affine conversion.
pub struct Unit {
    pub abbrev: &'static str,
    /// Display abbreviation (may differ from abbrev for aliases).
    pub display: &'static str,
    pub category: UnitCategory,
    /// Scale factor to base unit as exact decimal string. None for temperature (affine).
    pub to_base: Option<&'static str>,
}

/// All recognised units. Aliases (e.g. "F" for "°F") have the same
/// display as their canonical form but a different abbrev.
static UNITS: &[Unit] = &[
    // ── Weight (base: kg) ────────────────────────────────────────────────────
    Unit { abbrev: "oz",  display: "oz",  category: UnitCategory::Weight, to_base: Some("0.028349523125") },
    Unit { abbrev: "lb",  display: "lb",  category: UnitCategory::Weight, to_base: Some("0.45359237") },
    Unit { abbrev: "g",   display: "g",   category: UnitCategory::Weight, to_base: Some("0.001") },
    Unit { abbrev: "kg",  display: "kg",  category: UnitCategory::Weight, to_base: Some("1") },
    // ── Length (base: m) ─────────────────────────────────────────────────────
    Unit { abbrev: "mm",  display: "mm",  category: UnitCategory::Length, to_base: Some("0.001") },
    Unit { abbrev: "cm",  display: "cm",  category: UnitCategory::Length, to_base: Some("0.01") },
    Unit { abbrev: "m",   display: "m",   category: UnitCategory::Length, to_base: Some("1") },
    Unit { abbrev: "km",  display: "km",  category: UnitCategory::Length, to_base: Some("1000") },
    Unit { abbrev: "in",  display: "in",  category: UnitCategory::Length, to_base: Some("0.0254") },
    Unit { abbrev: "ft",  display: "ft",  category: UnitCategory::Length, to_base: Some("0.3048") },
    Unit { abbrev: "yd",  display: "yd",  category: UnitCategory::Length, to_base: Some("0.9144") },
    Unit { abbrev: "mi",  display: "mi",  category: UnitCategory::Length, to_base: Some("1609.344") },
    // ── Temperature (affine) ─────────────────────────────────────────────────
    Unit { abbrev: "°F",   display: "°F",  category: UnitCategory::Temperature, to_base: None },
    Unit { abbrev: "°C",   display: "°C",  category: UnitCategory::Temperature, to_base: None },
    // ASCII aliases — same display as canonical but typable without special chars
    Unit { abbrev: "F",   display: "°F",  category: UnitCategory::Temperature, to_base: None },
    Unit { abbrev: "C",   display: "°C",  category: UnitCategory::Temperature, to_base: None },
    Unit { abbrev: "degF",display: "°F",  category: UnitCategory::Temperature, to_base: None },
    Unit { abbrev: "degC",display: "°C",  category: UnitCategory::Temperature, to_base: None },
];

/// Parse an exact decimal string (e.g. "0.3048") to FBig at 128-bit precision,
/// without routing through f64. Used for unit scale factors.
fn parse_scale(s: &str) -> FBig {
    let (int_s, frac_s) = match s.find('.') {
        Some(pos) => (&s[..pos], &s[pos + 1..]),
        None => (s, ""),
    };
    let decimal_places = frac_s.len() as i64;
    let combined = format!("{}{}", int_s, frac_s);
    let significand: IBig = combined.parse().expect("valid scale constant");
    let ctx = Context::<Zero>::new(128);
    if decimal_places == 0 {
        ctx.convert_int::<2>(significand).value()
    } else {
        let num = ctx.convert_int::<2>(significand).value();
        let den = ctx.convert_int::<2>(IBig::from(10u8).pow(decimal_places as usize)).value();
        ctx.div(num.repr(), den.repr()).value()
    }
}

/// Convenience: parse an integer constant (32, 5, 9 …) to FBig.
fn fbig_int(n: i64) -> FBig {
    let ctx = Context::<Zero>::new(128);
    ctx.convert_int::<2>(IBig::from(n)).value()
}

/// Look up a unit by abbreviation (case-sensitive). Returns the first match.
pub fn lookup_unit(abbrev: &str) -> Option<&'static Unit> {
    UNITS.iter().find(|u| u.abbrev == abbrev)
}

/// Canonical display abbreviation for a given abbreviation string.
/// Returns `abbrev` unchanged if not found.
pub fn canonical_display(abbrev: &str) -> &str {
    lookup_unit(abbrev).map(|u| u.display).unwrap_or(abbrev)
}

/// Convert `amount` (in `from` unit) to `to` unit using FBig arithmetic.
/// Returns `CalcError::IncompatibleUnits` if categories differ.
pub fn convert(amount: FBig, from: &Unit, to: &Unit) -> Result<FBig, CalcError> {
    if from.category != to.category {
        return Err(CalcError::IncompatibleUnits(format!(
            "cannot convert {} to {}",
            from.category.name(),
            to.category.name()
        )));
    }
    if from.abbrev == to.abbrev || from.display == to.display {
        return Ok(amount);
    }
    match from.category {
        UnitCategory::Temperature => convert_temperature(amount, from.display, to.display),
        _ => {
            let from_scale = parse_scale(from.to_base.expect("linear unit must have to_base"));
            let to_scale = parse_scale(to.to_base.expect("linear unit must have to_base"));
            Ok(amount * from_scale / to_scale)
        }
    }
}

fn convert_temperature(amount: FBig, from_display: &str, to_display: &str) -> Result<FBig, CalcError> {
    match (from_display, to_display) {
        ("°F", "°C") => Ok((amount - fbig_int(32)) * fbig_int(5) / fbig_int(9)),
        ("°C", "°F") => Ok(amount * fbig_int(9) / fbig_int(5) + fbig_int(32)),
        _ => Ok(amount),
    }
}

/// A numeric value tagged with a physical unit.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TaggedValue {
    /// The numeric amount in the named unit's scale.
    pub amount: FBig,
    /// Unit abbreviation (e.g. "oz", "°F"). Canonical display is looked up via `canonical_display()`.
    pub unit: String,
}

impl TaggedValue {
    pub fn new(amount: f64, unit: impl Into<String>) -> Self {
        let unit_str = unit.into();
        // Normalise alias to canonical display string
        let display = canonical_display(&unit_str).to_string();
        Self {
            amount: FBig::try_from(amount).unwrap_or(FBig::ZERO),
            unit: display,
        }
    }

    /// Return the static Unit definition, if the unit is known.
    pub fn unit_def(&self) -> Option<&'static Unit> {
        lookup_unit(&self.unit)
    }

    /// Convert this tagged value to a different unit abbreviation.
    pub fn convert_to(&self, target_abbrev: &str) -> Result<TaggedValue, CalcError> {
        let from = self.unit_def().ok_or_else(|| {
            CalcError::IncompatibleUnits(format!("unknown unit: {}", self.unit))
        })?;
        let target_display = canonical_display(target_abbrev);
        let to = lookup_unit(target_display).ok_or_else(|| {
            CalcError::InvalidInput(format!("unknown unit: {}", target_abbrev))
        })?;
        let converted = convert(self.amount.clone(), from, to)?;
        Ok(TaggedValue { amount: converted, unit: target_display.to_string() })
    }

    pub fn display(&self) -> String {
        format!("{} {}", crate::engine::value::format_fbig(&self.amount), self.unit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: build an FBig from an f64 for test inputs.
    fn fbig(v: f64) -> FBig {
        FBig::try_from(v).unwrap()
    }

    // ── lookup ───────────────────────────────────────────────────────────────

    #[test]
    fn test_lookup_known_unit() {
        assert!(lookup_unit("oz").is_some());
        assert!(lookup_unit("g").is_some());
        assert!(lookup_unit("°F").is_some());
        assert!(lookup_unit("F").is_some()); // alias
    }

    #[test]
    fn test_lookup_unknown_unit() {
        assert!(lookup_unit("fathoms").is_none());
        assert!(lookup_unit("psi").is_none());
        assert!(lookup_unit("OZ").is_none()); // case-sensitive
    }

    #[test]
    fn test_canonical_display_alias() {
        assert_eq!(canonical_display("F"), "°F");
        assert_eq!(canonical_display("C"), "°C");
        assert_eq!(canonical_display("degF"), "°F");
        assert_eq!(canonical_display("degC"), "°C");
    }

    #[test]
    fn test_canonical_display_canonical() {
        assert_eq!(canonical_display("oz"), "oz");
        assert_eq!(canonical_display("°F"), "°F");
    }

    // ── weight conversion ────────────────────────────────────────────────────

    #[test]
    fn test_oz_to_g() {
        // AC-3: 1.9 oz → ~53.86 g
        let oz = lookup_unit("oz").unwrap();
        let g = lookup_unit("g").unwrap();
        let result = convert(fbig(1.9), oz, g).unwrap();
        assert!((result.to_f64().value() - 53.8640939).abs() < 0.001,
            "1.9 oz in grams = {}", result.to_f64().value());
    }

    #[test]
    fn test_g_to_oz() {
        let g = lookup_unit("g").unwrap();
        let oz = lookup_unit("oz").unwrap();
        let result = convert(fbig(53.86), g, oz).unwrap();
        assert!((result.to_f64().value() - 1.9).abs() < 0.01,
            "53.86 g in oz = {}", result.to_f64().value());
    }

    #[test]
    fn test_lb_to_g() {
        // AC-16: 1 lb → 453.592 g
        let lb = lookup_unit("lb").unwrap();
        let g = lookup_unit("g").unwrap();
        let result = convert(fbig(1.0), lb, g).unwrap();
        assert!((result.to_f64().value() - 453.59237).abs() < 0.001,
            "1 lb in g = {}", result.to_f64().value());
    }

    #[test]
    fn test_oz_to_kg() {
        let oz = lookup_unit("oz").unwrap();
        let kg = lookup_unit("kg").unwrap();
        let result = convert(fbig(1.0), oz, kg).unwrap();
        assert!((result.to_f64().value() - 0.028349523125).abs() < 1e-9,
            "1 oz in kg = {}", result.to_f64().value());
    }

    // ── length conversion ────────────────────────────────────────────────────

    #[test]
    fn test_ft_to_m() {
        // AC-4: 6 ft → 1.8288 m
        let ft = lookup_unit("ft").unwrap();
        let m = lookup_unit("m").unwrap();
        let result = convert(fbig(6.0), ft, m).unwrap();
        assert!((result.to_f64().value() - 1.8288).abs() < 1e-9,
            "6 ft in m = {}", result.to_f64().value());
    }

    #[test]
    fn test_in_to_cm() {
        let inch = lookup_unit("in").unwrap();
        let cm = lookup_unit("cm").unwrap();
        let result = convert(fbig(1.0), inch, cm).unwrap();
        assert!((result.to_f64().value() - 2.54).abs() < 1e-9,
            "1 in in cm = {}", result.to_f64().value());
    }

    #[test]
    fn test_mi_to_km() {
        let mi = lookup_unit("mi").unwrap();
        let km = lookup_unit("km").unwrap();
        let result = convert(fbig(1.0), mi, km).unwrap();
        assert!((result.to_f64().value() - 1.609344).abs() < 1e-6,
            "1 mi in km = {}", result.to_f64().value());
    }

    // ── length conversion no noise ────────────────────────────────────────────

    #[test]
    fn test_ft_to_cm_no_noise() {
        // 1.223 ft → cm should display as 37.27704, not 37.27704000000001
        let ft = lookup_unit("ft").unwrap();
        let cm = lookup_unit("cm").unwrap();
        let result = convert(parse_scale("1.223"), ft, cm).unwrap();
        let displayed = crate::engine::value::format_fbig(&result);
        assert_eq!(displayed, "37.27704",
            "expected clean 37.27704, got {}", displayed);
    }

    #[test]
    fn test_ft_cm_ft_roundtrip_no_noise() {
        // 3.2 ft → cm → ft should round-trip cleanly
        let ft = lookup_unit("ft").unwrap();
        let cm = lookup_unit("cm").unwrap();
        let start = parse_scale("3.2");
        let in_cm = convert(start, ft, cm).unwrap();
        let back = convert(in_cm, cm, ft).unwrap();
        let displayed = crate::engine::value::format_fbig(&back);
        assert_eq!(displayed, "3.2",
            "round-trip 3.2 ft→cm→ft, got {}", displayed);
    }

    // ── temperature conversion ────────────────────────────────────────────────

    #[test]
    fn test_f_to_c() {
        // AC-5: 98.6 °F → 37 °C
        let f = lookup_unit("°F").unwrap();
        let c = lookup_unit("°C").unwrap();
        let result = convert(fbig(98.6), f, c).unwrap();
        assert!((result.to_f64().value() - 37.0).abs() < 0.001,
            "98.6 °F in °C = {}", result.to_f64().value());
    }

    #[test]
    fn test_c_to_f() {
        // AC-6: 100 °C → 212 °F
        let c = lookup_unit("°C").unwrap();
        let f = lookup_unit("°F").unwrap();
        let result = convert(fbig(100.0), c, f).unwrap();
        assert!((result.to_f64().value() - 212.0).abs() < 0.001,
            "100 °C in °F = {}", result.to_f64().value());
    }

    #[test]
    fn test_f_to_c_freezing() {
        let f = lookup_unit("°F").unwrap();
        let c = lookup_unit("°C").unwrap();
        let result = convert(fbig(32.0), f, c).unwrap();
        assert!(result.to_f64().value().abs() < 1e-9,
            "32 °F = 0 °C, got {}", result.to_f64().value());
    }

    #[test]
    fn test_temperature_alias_f_to_c() {
        // "F" alias should resolve to °F for conversion
        let tagged = TaggedValue::new(98.6, "F");
        assert_eq!(tagged.unit, "°F");
        let converted = tagged.convert_to("C").unwrap();
        assert_eq!(converted.unit, "°C");
        assert!((converted.amount.to_f64().value() - 37.0).abs() < 0.001);
    }

    // ── incompatible categories ───────────────────────────────────────────────

    #[test]
    fn test_incompatible_weight_to_length() {
        let oz = lookup_unit("oz").unwrap();
        let m = lookup_unit("m").unwrap();
        assert!(matches!(convert(fbig(1.0), oz, m), Err(CalcError::IncompatibleUnits(_))));
    }

    #[test]
    fn test_incompatible_weight_to_temperature() {
        let g = lookup_unit("g").unwrap();
        let f = lookup_unit("°F").unwrap();
        assert!(matches!(convert(fbig(1.0), g, f), Err(CalcError::IncompatibleUnits(_))));
    }

    // ── same unit (no conversion) ─────────────────────────────────────────────

    #[test]
    fn test_same_unit_no_op() {
        let oz = lookup_unit("oz").unwrap();
        let result = convert(fbig(1.9), oz, oz).unwrap();
        assert!((result.to_f64().value() - 1.9).abs() < 1e-10);
    }

    // ── TaggedValue ──────────────────────────────────────────────────────────

    #[test]
    fn test_tagged_value_new_normalises_alias() {
        let t = TaggedValue::new(98.6, "F");
        assert_eq!(t.unit, "°F");
        assert!((t.amount.to_f64().value() - 98.6).abs() < 1e-10);
    }

    #[test]
    fn test_tagged_value_display() {
        let t = TaggedValue::new(1.9, "oz");
        assert_eq!(t.display(), "1.9 oz");
    }

    #[test]
    fn test_tagged_value_convert_to() {
        let t = TaggedValue::new(1.9, "oz");
        let converted = t.convert_to("g").unwrap();
        assert_eq!(converted.unit, "g");
        assert!((converted.amount.to_f64().value() - 53.86).abs() < 0.01);
    }

    #[test]
    fn test_tagged_value_convert_to_incompatible() {
        let t = TaggedValue::new(1.9, "oz");
        assert!(matches!(t.convert_to("m"), Err(CalcError::IncompatibleUnits(_))));
    }

    #[test]
    fn test_tagged_value_serde_roundtrip() {
        let t = TaggedValue::new(1.9, "oz");
        let json = serde_json::to_string(&t).expect("serialize");
        let restored: TaggedValue = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(t, restored);
    }
}
