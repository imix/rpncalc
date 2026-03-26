use crate::engine::error::CalcError;
use dashu::float::{round::mode::Zero, Context, FBig};
use dashu::integer::IBig;
use serde::{Deserialize, Serialize};

/// Dimension vector: signed integer exponents for the seven SI base dimensions.
/// All-zeros represents a dimensionless value.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DimensionVector {
    #[serde(default, skip_serializing_if = "is_zero")]
    pub kg: i8, // mass
    #[serde(default, skip_serializing_if = "is_zero")]
    pub m: i8, // length
    #[serde(default, skip_serializing_if = "is_zero")]
    pub s: i8, // time
    #[serde(default, skip_serializing_if = "is_zero", rename = "A")]
    pub a: i8, // electric current
    #[serde(default, skip_serializing_if = "is_zero", rename = "K")]
    pub k: i8, // thermodynamic temperature
    #[serde(default, skip_serializing_if = "is_zero")]
    pub mol: i8, // amount of substance
    #[serde(default, skip_serializing_if = "is_zero")]
    pub cd: i8, // luminous intensity
}

fn is_zero(n: &i8) -> bool {
    *n == 0
}

impl DimensionVector {
    pub fn is_dimensionless(&self) -> bool {
        *self == Self::default()
    }

    /// Add dimension exponents — used for multiplication.
    pub fn add(&self, other: &Self) -> Self {
        Self {
            kg: self.kg + other.kg,
            m: self.m + other.m,
            s: self.s + other.s,
            a: self.a + other.a,
            k: self.k + other.k,
            mol: self.mol + other.mol,
            cd: self.cd + other.cd,
        }
    }

    /// Subtract dimension exponents — used for division.
    pub fn sub(&self, other: &Self) -> Self {
        Self {
            kg: self.kg - other.kg,
            m: self.m - other.m,
            s: self.s - other.s,
            a: self.a - other.a,
            k: self.k - other.k,
            mol: self.mol - other.mol,
            cd: self.cd - other.cd,
        }
    }

    /// Negate all exponents — used for reciprocal (1/x).
    pub fn negate(&self) -> Self {
        Self {
            kg: -self.kg,
            m: -self.m,
            s: -self.s,
            a: -self.a,
            k: -self.k,
            mol: -self.mol,
            cd: -self.cd,
        }
    }

    /// Halve all exponents — used for sqrt. Returns `None` if any exponent is odd.
    pub fn halve(&self) -> Option<Self> {
        if self.kg % 2 != 0
            || self.m % 2 != 0
            || self.s % 2 != 0
            || self.a % 2 != 0
            || self.k % 2 != 0
            || self.mol % 2 != 0
            || self.cd % 2 != 0
        {
            return None;
        }
        Some(Self {
            kg: self.kg / 2,
            m: self.m / 2,
            s: self.s / 2,
            a: self.a / 2,
            k: self.k / 2,
            mol: self.mol / 2,
            cd: self.cd / 2,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnitCategory {
    Weight,
    Length,
    Temperature,
    Time,
}

impl UnitCategory {
    pub fn name(&self) -> &'static str {
        match self {
            UnitCategory::Weight => "weight",
            UnitCategory::Length => "length",
            UnitCategory::Temperature => "temperature",
            UnitCategory::Time => "time",
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
    /// SI dimension vector for this unit.
    pub dim: DimensionVector,
}

/// All recognised units. Aliases (e.g. "F" for "°F") have the same
/// display as their canonical form but a different abbrev.
static UNITS: &[Unit] = &[
    // ── Weight (base: kg) ────────────────────────────────────────────────────
    Unit { abbrev: "oz",  display: "oz",  category: UnitCategory::Weight,      to_base: Some("0.028349523125"), dim: DimensionVector { kg: 1, m: 0, s: 0, a: 0, k: 0, mol: 0, cd: 0 } },
    Unit { abbrev: "lb",  display: "lb",  category: UnitCategory::Weight,      to_base: Some("0.45359237"),     dim: DimensionVector { kg: 1, m: 0, s: 0, a: 0, k: 0, mol: 0, cd: 0 } },
    Unit { abbrev: "g",   display: "g",   category: UnitCategory::Weight,      to_base: Some("0.001"),          dim: DimensionVector { kg: 1, m: 0, s: 0, a: 0, k: 0, mol: 0, cd: 0 } },
    Unit { abbrev: "kg",  display: "kg",  category: UnitCategory::Weight,      to_base: Some("1"),              dim: DimensionVector { kg: 1, m: 0, s: 0, a: 0, k: 0, mol: 0, cd: 0 } },
    // ── Length (base: m) ─────────────────────────────────────────────────────
    Unit { abbrev: "mm",  display: "mm",  category: UnitCategory::Length,      to_base: Some("0.001"),          dim: DimensionVector { kg: 0, m: 1, s: 0, a: 0, k: 0, mol: 0, cd: 0 } },
    Unit { abbrev: "cm",  display: "cm",  category: UnitCategory::Length,      to_base: Some("0.01"),           dim: DimensionVector { kg: 0, m: 1, s: 0, a: 0, k: 0, mol: 0, cd: 0 } },
    Unit { abbrev: "m",   display: "m",   category: UnitCategory::Length,      to_base: Some("1"),              dim: DimensionVector { kg: 0, m: 1, s: 0, a: 0, k: 0, mol: 0, cd: 0 } },
    Unit { abbrev: "km",  display: "km",  category: UnitCategory::Length,      to_base: Some("1000"),           dim: DimensionVector { kg: 0, m: 1, s: 0, a: 0, k: 0, mol: 0, cd: 0 } },
    Unit { abbrev: "in",  display: "in",  category: UnitCategory::Length,      to_base: Some("0.0254"),         dim: DimensionVector { kg: 0, m: 1, s: 0, a: 0, k: 0, mol: 0, cd: 0 } },
    Unit { abbrev: "ft",  display: "ft",  category: UnitCategory::Length,      to_base: Some("0.3048"),         dim: DimensionVector { kg: 0, m: 1, s: 0, a: 0, k: 0, mol: 0, cd: 0 } },
    Unit { abbrev: "yd",  display: "yd",  category: UnitCategory::Length,      to_base: Some("0.9144"),         dim: DimensionVector { kg: 0, m: 1, s: 0, a: 0, k: 0, mol: 0, cd: 0 } },
    Unit { abbrev: "mi",  display: "mi",  category: UnitCategory::Length,      to_base: Some("1609.344"),       dim: DimensionVector { kg: 0, m: 1, s: 0, a: 0, k: 0, mol: 0, cd: 0 } },
    // ── Time (base: s) ───────────────────────────────────────────────────────
    Unit { abbrev: "s",   display: "s",   category: UnitCategory::Time,        to_base: Some("1"),              dim: DimensionVector { kg: 0, m: 0, s: 1, a: 0, k: 0, mol: 0, cd: 0 } },
    Unit { abbrev: "min", display: "min", category: UnitCategory::Time,        to_base: Some("60"),             dim: DimensionVector { kg: 0, m: 0, s: 1, a: 0, k: 0, mol: 0, cd: 0 } },
    Unit { abbrev: "h",   display: "h",   category: UnitCategory::Time,        to_base: Some("3600"),           dim: DimensionVector { kg: 0, m: 0, s: 1, a: 0, k: 0, mol: 0, cd: 0 } },
    // ── Temperature (affine) ─────────────────────────────────────────────────
    Unit { abbrev: "°F",  display: "°F",  category: UnitCategory::Temperature, to_base: None,                   dim: DimensionVector { kg: 0, m: 0, s: 0, a: 0, k: 1, mol: 0, cd: 0 } },
    Unit { abbrev: "°C",  display: "°C",  category: UnitCategory::Temperature, to_base: None,                   dim: DimensionVector { kg: 0, m: 0, s: 0, a: 0, k: 1, mol: 0, cd: 0 } },
    // ASCII aliases — same display as canonical but typable without special chars
    Unit { abbrev: "F",   display: "°F",  category: UnitCategory::Temperature, to_base: None,                   dim: DimensionVector { kg: 0, m: 0, s: 0, a: 0, k: 1, mol: 0, cd: 0 } },
    Unit { abbrev: "C",   display: "°C",  category: UnitCategory::Temperature, to_base: None,                   dim: DimensionVector { kg: 0, m: 0, s: 0, a: 0, k: 1, mol: 0, cd: 0 } },
    Unit { abbrev: "degF",display: "°F",  category: UnitCategory::Temperature, to_base: None,                   dim: DimensionVector { kg: 0, m: 0, s: 0, a: 0, k: 1, mol: 0, cd: 0 } },
    Unit { abbrev: "degC",display: "°C",  category: UnitCategory::Temperature, to_base: None,                   dim: DimensionVector { kg: 0, m: 0, s: 0, a: 0, k: 1, mol: 0, cd: 0 } },
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
    /// SI dimension vector. Populated from the unit registry; used for arithmetic type-checking.
    /// `#[serde(default)]` allows old session files (no `dim` field) to deserialise without error.
    #[serde(default)]
    pub dim: DimensionVector,
}

impl TaggedValue {
    pub fn new(amount: f64, unit: impl Into<String>) -> Self {
        let unit_str = unit.into();
        // Normalise alias to canonical display string
        let display = canonical_display(&unit_str).to_string();
        let dim = lookup_unit(&display)
            .map(|u| u.dim.clone())
            .unwrap_or_default();
        Self {
            amount: FBig::try_from(amount).unwrap_or(FBig::ZERO),
            unit: display,
            dim,
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
        Ok(TaggedValue {
            amount: converted,
            unit: target_display.to_string(),
            dim: to.dim.clone(),
        })
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

    // ── AC-2: unit registry SI dimensions ───────────────────────────────────

    #[test]
    fn test_registry_si_dimensions() {
        let mass_dim = DimensionVector { kg: 1, ..Default::default() };
        let len_dim = DimensionVector { m: 1, ..Default::default() };
        let temp_dim = DimensionVector { k: 1, ..Default::default() };
        let time_dim = DimensionVector { s: 1, ..Default::default() };

        for abbrev in &["oz", "lb", "g", "kg"] {
            let u = lookup_unit(abbrev).unwrap_or_else(|| panic!("unit {} not found", abbrev));
            assert_eq!(u.dim, mass_dim, "{} should have mass dim", abbrev);
        }
        for abbrev in &["mm", "cm", "m", "km", "ft", "in", "yd", "mi"] {
            let u = lookup_unit(abbrev).unwrap_or_else(|| panic!("unit {} not found", abbrev));
            assert_eq!(u.dim, len_dim, "{} should have length dim", abbrev);
        }
        for abbrev in &["°F", "°C"] {
            let u = lookup_unit(abbrev).unwrap_or_else(|| panic!("unit {} not found", abbrev));
            assert_eq!(u.dim, temp_dim, "{} should have temperature dim", abbrev);
        }
        let s_unit = lookup_unit("s").expect("s not found");
        assert_eq!(s_unit.dim, time_dim);
    }

    // ── AC-3: TaggedValue serde round-trip preserves dim ────────────────────

    #[test]
    fn test_tagged_value_dim_serde_roundtrip() {
        // Simulate a compound dim (m:1, s:-1) that compound-unit-operations will produce.
        let t = TaggedValue {
            amount: fbig(27.78),
            unit: "m/s".to_string(),
            dim: DimensionVector { m: 1, s: -1, ..Default::default() },
        };
        let json = serde_json::to_string(&t).expect("serialize");
        let restored: TaggedValue = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.dim, t.dim);
        assert_eq!(restored.unit, "m/s");
    }

    #[test]
    fn test_tagged_value_new_populates_dim() {
        let oz = TaggedValue::new(1.9, "oz");
        assert_eq!(oz.dim, DimensionVector { kg: 1, ..Default::default() });

        let ft = TaggedValue::new(6.0, "ft");
        assert_eq!(ft.dim, DimensionVector { m: 1, ..Default::default() });

        let f = TaggedValue::new(98.6, "F");
        assert_eq!(f.dim, DimensionVector { k: 1, ..Default::default() });
    }

    #[test]
    fn test_convert_to_preserves_dim() {
        let oz = TaggedValue::new(1.9, "oz");
        let g = oz.convert_to("g").unwrap();
        assert_eq!(g.dim, DimensionVector { kg: 1, ..Default::default() });
    }

    // ── DimensionVector arithmetic ───────────────────────────────────────────

    #[test]
    fn test_dimension_vector_arithmetic() {
        let mass = DimensionVector { kg: 1, ..Default::default() };
        let accel = DimensionVector { m: 1, s: -2, ..Default::default() };

        // force = mass × acceleration: {kg:1} + {m:1, s:-2} = {kg:1, m:1, s:-2}
        let force = mass.add(&accel);
        assert_eq!(force, DimensionVector { kg: 1, m: 1, s: -2, ..Default::default() });

        // dimensionless from same-unit division
        assert!(mass.sub(&mass).is_dimensionless());

        // reciprocal
        let recip = accel.negate();
        assert_eq!(recip, DimensionVector { m: -1, s: 2, ..Default::default() });

        // sqrt of area {m:2} → {m:1}
        let area = DimensionVector { m: 2, ..Default::default() };
        assert_eq!(area.halve(), Some(DimensionVector { m: 1, ..Default::default() }));

        // sqrt of speed {m:1, s:-1} → None (odd exponent)
        let speed = DimensionVector { m: 1, s: -1, ..Default::default() };
        assert_eq!(speed.halve(), None);
    }
}
