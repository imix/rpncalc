use crate::engine::value::CalcValue;
use dashu::float::FBig;

pub fn pi() -> CalcValue {
    // 34 significant digits of pi
    let val: f64 = std::f64::consts::PI;
    CalcValue::Float(FBig::try_from(val).unwrap_or(FBig::ZERO))
}

pub fn euler() -> CalcValue {
    // 34 significant digits of e
    let val: f64 = std::f64::consts::E;
    CalcValue::Float(FBig::try_from(val).unwrap_or(FBig::ZERO))
}

pub fn phi() -> CalcValue {
    // Golden ratio: (1 + sqrt(5)) / 2
    let val: f64 = (1.0 + 5.0_f64.sqrt()) / 2.0;
    CalcValue::Float(FBig::try_from(val).unwrap_or(FBig::ZERO))
}
