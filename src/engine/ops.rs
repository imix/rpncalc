use crate::engine::angle::AngleMode;
use crate::engine::constants;
use crate::engine::error::CalcError;
use crate::engine::stack::CalcState;
use crate::engine::value::CalcValue;
use dashu::float::FBig;
use dashu::integer::IBig;

/// All calculator operations. Dispatched via `apply_op`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Op {
    // Binary
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Mod,
    // Unary
    Negate,
    Sqrt,
    Square,
    Reciprocal,
    Abs,
    Factorial,
    // Trig
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    // Log/Exp
    Log10,
    Ln,
    Exp,
    Exp10,
    // Bitwise
    And,
    Or,
    Xor,
    Not,
    Shl,
    Shr,
    // Stack
    Swap,
    Dup,
    Drop,
    Rotate,
    Clear,
    // Constants
    PushPi,
    PushE,
    PushPhi,
}

/// Dispatch an `Op` to the appropriate operation, mutating `state`.
///
/// **Atomicity guarantee:** if `Err` is returned, the stack is unchanged.
pub fn apply_op(state: &mut CalcState, op: Op) -> Result<(), CalcError> {
    match op {
        // Binary arithmetic
        Op::Add => binary_op(state, do_add),
        Op::Sub => binary_op(state, do_sub),
        Op::Mul => binary_op(state, do_mul),
        Op::Div => binary_op(state, do_div),
        Op::Pow => binary_op(state, do_pow),
        Op::Mod => binary_op(state, do_mod),
        // Binary bitwise
        Op::And => binary_op(state, do_and),
        Op::Or => binary_op(state, do_or),
        Op::Xor => binary_op(state, do_xor),
        Op::Shl => binary_op(state, do_shl),
        Op::Shr => binary_op(state, do_shr),
        // Unary
        Op::Negate => unary_op(state, do_negate),
        Op::Sqrt => unary_op(state, do_sqrt),
        Op::Square => unary_op(state, do_sq),
        Op::Reciprocal => unary_op(state, do_reciprocal),
        Op::Abs => unary_op(state, do_abs),
        Op::Factorial => unary_op(state, do_factorial),
        Op::Not => unary_op(state, do_not),
        // Trig — capture angle_mode before mutable borrow
        Op::Sin => {
            let m = state.angle_mode;
            unary_op(state, |v| do_trig(v, m, f64::sin))
        }
        Op::Cos => {
            let m = state.angle_mode;
            unary_op(state, |v| do_trig(v, m, f64::cos))
        }
        Op::Tan => {
            let m = state.angle_mode;
            unary_op(state, |v| do_trig(v, m, f64::tan))
        }
        Op::Asin => {
            let m = state.angle_mode;
            unary_op(state, |v| do_atrig(v, m, f64::asin))
        }
        Op::Acos => {
            let m = state.angle_mode;
            unary_op(state, |v| do_atrig(v, m, f64::acos))
        }
        Op::Atan => {
            let m = state.angle_mode;
            unary_op(state, |v| do_atrig(v, m, f64::atan))
        }
        // Log/Exp
        Op::Log10 => unary_op(state, do_log10),
        Op::Ln => unary_op(state, do_ln),
        Op::Exp => unary_op(state, do_exp),
        Op::Exp10 => unary_op(state, do_exp10),
        // Stack ops — delegate to CalcState (already atomic)
        Op::Swap => state.swap(),
        Op::Dup => state.dup(),
        Op::Drop => state.drop(),
        Op::Rotate => state.rotate(),
        Op::Clear => {
            state.clear();
            Ok(())
        }
        // Constants
        Op::PushPi => {
            state.push(constants::pi());
            Ok(())
        }
        Op::PushE => {
            state.push(constants::euler());
            Ok(())
        }
        Op::PushPhi => {
            state.push(constants::phi());
            Ok(())
        }
    }
}

/// Peek at the top two values, compute, then mutate only on success.
fn binary_op(
    state: &mut CalcState,
    f: impl Fn(CalcValue, CalcValue) -> Result<CalcValue, CalcError>,
) -> Result<(), CalcError> {
    if state.depth() < 2 {
        return Err(CalcError::StackUnderflow);
    }
    // Peek without mutating — atomicity: if f() fails, nothing changes
    let n = state.stack.len();
    let b = state.stack[n - 1].clone(); // top (X)
    let a = state.stack[n - 2].clone(); // second (Y)
    let result = f(a, b)?;
    // Only mutate on success
    state.pop().expect("SAFETY: depth >= 2 verified above");
    state.pop().expect("SAFETY: depth >= 2 verified above");
    state.push(result);
    Ok(())
}

/// Peek at the top value, compute, then mutate only on success.
fn unary_op(
    state: &mut CalcState,
    f: impl Fn(CalcValue) -> Result<CalcValue, CalcError>,
) -> Result<(), CalcError> {
    // Peek without mutating — atomicity: if f() fails, nothing changes
    let a = state.peek().ok_or(CalcError::StackUnderflow)?.clone();
    let result = f(a)?;
    // Only mutate on success
    state
        .pop()
        .expect("SAFETY: peeked above guarantees depth >= 1");
    state.push(result);
    Ok(())
}

fn do_add(a: CalcValue, b: CalcValue) -> Result<CalcValue, CalcError> {
    match (a, b) {
        (CalcValue::Integer(x), CalcValue::Integer(y)) => Ok(CalcValue::Integer(x + y)),
        (CalcValue::Float(x), CalcValue::Float(y)) => Ok(CalcValue::Float(x + y)),
        (CalcValue::Integer(x), CalcValue::Float(y)) => Ok(CalcValue::Float(int_to_fbig(&x) + y)),
        (CalcValue::Float(x), CalcValue::Integer(y)) => Ok(CalcValue::Float(x + int_to_fbig(&y))),
    }
}

fn do_sub(a: CalcValue, b: CalcValue) -> Result<CalcValue, CalcError> {
    match (a, b) {
        (CalcValue::Integer(x), CalcValue::Integer(y)) => Ok(CalcValue::Integer(x - y)),
        (CalcValue::Float(x), CalcValue::Float(y)) => Ok(CalcValue::Float(x - y)),
        (CalcValue::Integer(x), CalcValue::Float(y)) => Ok(CalcValue::Float(int_to_fbig(&x) - y)),
        (CalcValue::Float(x), CalcValue::Integer(y)) => Ok(CalcValue::Float(x - int_to_fbig(&y))),
    }
}

fn do_mul(a: CalcValue, b: CalcValue) -> Result<CalcValue, CalcError> {
    match (a, b) {
        (CalcValue::Integer(x), CalcValue::Integer(y)) => Ok(CalcValue::Integer(x * y)),
        (CalcValue::Float(x), CalcValue::Float(y)) => Ok(CalcValue::Float(x * y)),
        (CalcValue::Integer(x), CalcValue::Float(y)) => Ok(CalcValue::Float(int_to_fbig(&x) * y)),
        (CalcValue::Float(x), CalcValue::Integer(y)) => Ok(CalcValue::Float(x * int_to_fbig(&y))),
    }
}

fn do_div(a: CalcValue, b: CalcValue) -> Result<CalcValue, CalcError> {
    match (a, b) {
        (CalcValue::Integer(x), CalcValue::Integer(y)) => {
            if y == IBig::ZERO {
                return Err(CalcError::DivisionByZero);
            }
            // Exact integer division stays Integer; otherwise promote to Float
            if &x % &y == IBig::ZERO {
                Ok(CalcValue::Integer(x / y))
            } else {
                Ok(CalcValue::Float(int_to_fbig(&x) / int_to_fbig(&y)))
            }
        }
        (CalcValue::Float(x), CalcValue::Float(y)) => {
            if y.to_f64().value() == 0.0 {
                return Err(CalcError::DivisionByZero);
            }
            Ok(CalcValue::Float(x / y))
        }
        (CalcValue::Integer(x), CalcValue::Float(y)) => {
            if y.to_f64().value() == 0.0 {
                return Err(CalcError::DivisionByZero);
            }
            Ok(CalcValue::Float(int_to_fbig(&x) / y))
        }
        (CalcValue::Float(x), CalcValue::Integer(y)) => {
            if y == IBig::ZERO {
                return Err(CalcError::DivisionByZero);
            }
            Ok(CalcValue::Float(x / int_to_fbig(&y)))
        }
    }
}

fn do_pow(a: CalcValue, b: CalcValue) -> Result<CalcValue, CalcError> {
    // Integer^non-negative-integer: exact result
    if let (CalcValue::Integer(ref x), CalcValue::Integer(ref y)) = (&a, &b) {
        if *y >= IBig::ZERO {
            let exp_str = y.to_string();
            if let Ok(exp_u32) = exp_str.parse::<u32>() {
                if exp_u32 <= 1000 {
                    return Ok(CalcValue::Integer(x.clone().pow(exp_u32 as usize)));
                }
            }
        }
    }
    let result = a.to_f64().powf(b.to_f64());
    if result.is_infinite() || result.is_nan() {
        return Err(CalcError::DomainError(
            "pow result out of range".to_string(),
        ));
    }
    Ok(CalcValue::from_f64(result))
}

fn do_mod(a: CalcValue, b: CalcValue) -> Result<CalcValue, CalcError> {
    match (a, b) {
        (CalcValue::Integer(x), CalcValue::Integer(y)) => {
            if y == IBig::ZERO {
                return Err(CalcError::DivisionByZero);
            }
            Ok(CalcValue::Integer(x % y))
        }
        (a, b) => {
            let bv = b.to_f64();
            if bv == 0.0 {
                return Err(CalcError::DivisionByZero);
            }
            Ok(CalcValue::from_f64(a.to_f64() % bv))
        }
    }
}

fn do_negate(v: CalcValue) -> Result<CalcValue, CalcError> {
    match v {
        CalcValue::Integer(x) => Ok(CalcValue::Integer(-x)),
        CalcValue::Float(x) => Ok(CalcValue::Float(-x)),
    }
}

fn do_sqrt(v: CalcValue) -> Result<CalcValue, CalcError> {
    let val = v.to_f64();
    if val < 0.0 {
        return Err(CalcError::DomainError(
            "sqrt requires non-negative number".to_string(),
        ));
    }
    Ok(CalcValue::from_f64(val.sqrt()))
}

fn do_sq(v: CalcValue) -> Result<CalcValue, CalcError> {
    match v {
        CalcValue::Integer(x) => Ok(CalcValue::Integer(x.clone() * x)),
        CalcValue::Float(x) => Ok(CalcValue::Float(x.clone() * x)),
    }
}

fn do_reciprocal(v: CalcValue) -> Result<CalcValue, CalcError> {
    let val = v.to_f64();
    if val == 0.0 {
        return Err(CalcError::DivisionByZero);
    }
    Ok(CalcValue::from_f64(1.0 / val))
}

fn do_abs(v: CalcValue) -> Result<CalcValue, CalcError> {
    match v {
        CalcValue::Integer(x) => {
            if x < IBig::ZERO {
                Ok(CalcValue::Integer(-x))
            } else {
                Ok(CalcValue::Integer(x))
            }
        }
        CalcValue::Float(x) => {
            // Use to_f64().value() — FBig::to_string() returns binary, not decimal
            let val = x.to_f64().value();
            Ok(CalcValue::from_f64(val.abs()))
        }
    }
}

fn do_factorial(v: CalcValue) -> Result<CalcValue, CalcError> {
    match v {
        CalcValue::Integer(x) => {
            if x < IBig::ZERO {
                return Err(CalcError::DomainError(
                    "factorial requires non-negative integer".to_string(),
                ));
            }
            let n: u64 = x
                .to_string()
                .parse()
                .map_err(|_| CalcError::DomainError("factorial argument too large".to_string()))?;
            if n > 10000 {
                return Err(CalcError::DomainError(
                    "factorial argument too large".to_string(),
                ));
            }
            let mut result = IBig::from(1u32);
            for i in 2..=n {
                result *= IBig::from(i);
            }
            Ok(CalcValue::Integer(result))
        }
        CalcValue::Float(_) => Err(CalcError::NotAnInteger),
    }
}

fn do_not(v: CalcValue) -> Result<CalcValue, CalcError> {
    match v {
        CalcValue::Integer(x) => Ok(CalcValue::Integer(!x)),
        CalcValue::Float(_) => Err(CalcError::NotAnInteger),
    }
}

fn do_and(a: CalcValue, b: CalcValue) -> Result<CalcValue, CalcError> {
    match (a, b) {
        (CalcValue::Integer(x), CalcValue::Integer(y)) => Ok(CalcValue::Integer(x & y)),
        _ => Err(CalcError::NotAnInteger),
    }
}

fn do_or(a: CalcValue, b: CalcValue) -> Result<CalcValue, CalcError> {
    match (a, b) {
        (CalcValue::Integer(x), CalcValue::Integer(y)) => Ok(CalcValue::Integer(x | y)),
        _ => Err(CalcError::NotAnInteger),
    }
}

fn do_xor(a: CalcValue, b: CalcValue) -> Result<CalcValue, CalcError> {
    match (a, b) {
        (CalcValue::Integer(x), CalcValue::Integer(y)) => Ok(CalcValue::Integer(x ^ y)),
        _ => Err(CalcError::NotAnInteger),
    }
}

fn do_shl(a: CalcValue, b: CalcValue) -> Result<CalcValue, CalcError> {
    match (a, b) {
        (CalcValue::Integer(x), CalcValue::Integer(y)) => {
            let shift: usize = y
                .to_string()
                .parse()
                .map_err(|_| CalcError::DomainError("shift amount too large".to_string()))?;
            Ok(CalcValue::Integer(x << shift))
        }
        _ => Err(CalcError::NotAnInteger),
    }
}

fn do_shr(a: CalcValue, b: CalcValue) -> Result<CalcValue, CalcError> {
    match (a, b) {
        (CalcValue::Integer(x), CalcValue::Integer(y)) => {
            let shift: usize = y
                .to_string()
                .parse()
                .map_err(|_| CalcError::DomainError("shift amount too large".to_string()))?;
            Ok(CalcValue::Integer(x >> shift))
        }
        _ => Err(CalcError::NotAnInteger),
    }
}

fn do_trig(v: CalcValue, mode: AngleMode, f: fn(f64) -> f64) -> Result<CalcValue, CalcError> {
    let rad = mode.to_radians(v.to_f64());
    Ok(CalcValue::from_f64(f(rad)))
}

fn do_atrig(v: CalcValue, mode: AngleMode, f: fn(f64) -> f64) -> Result<CalcValue, CalcError> {
    let rad = f(v.to_f64());
    if rad.is_nan() {
        return Err(CalcError::DomainError("value out of domain".to_string()));
    }
    Ok(CalcValue::from_f64(mode.from_radians(rad)))
}

fn do_log10(v: CalcValue) -> Result<CalcValue, CalcError> {
    let val = v.to_f64();
    if val <= 0.0 {
        return Err(CalcError::DomainError(
            "log requires positive number".to_string(),
        ));
    }
    Ok(CalcValue::from_f64(val.log10()))
}

fn do_ln(v: CalcValue) -> Result<CalcValue, CalcError> {
    let val = v.to_f64();
    if val <= 0.0 {
        return Err(CalcError::DomainError(
            "ln requires positive number".to_string(),
        ));
    }
    Ok(CalcValue::from_f64(val.ln()))
}

fn do_exp(v: CalcValue) -> Result<CalcValue, CalcError> {
    let result = v.to_f64().exp();
    if result.is_infinite() || result.is_nan() {
        return Err(CalcError::DomainError(
            "exp result out of range".to_string(),
        ));
    }
    Ok(CalcValue::from_f64(result))
}

fn do_exp10(v: CalcValue) -> Result<CalcValue, CalcError> {
    let result = 10.0_f64.powf(v.to_f64());
    if result.is_infinite() || result.is_nan() {
        return Err(CalcError::DomainError(
            "exp10 result out of range".to_string(),
        ));
    }
    Ok(CalcValue::from_f64(result))
}

fn int_to_fbig(n: &IBig) -> FBig {
    let s = n.to_string();
    s.parse::<f64>()
        .ok()
        .and_then(|f| FBig::try_from(f).ok())
        .unwrap_or(FBig::ZERO)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::angle::AngleMode;
    use crate::engine::stack::CalcState;
    use dashu::integer::IBig;

    fn int(n: i64) -> CalcValue {
        CalcValue::Integer(IBig::from(n))
    }

    fn float(f: f64) -> CalcValue {
        CalcValue::from_f64(f)
    }

    fn state_with(vals: &[i64]) -> CalcState {
        let mut s = CalcState::new();
        for &v in vals {
            s.push(int(v));
        }
        s
    }

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    fn top_f64(s: &CalcState) -> f64 {
        s.peek().unwrap().to_f64()
    }

    // ── Op::constructible ────────────────────────────────────────────────────

    #[test]
    fn test_op_constructible_and_comparable() {
        let _ = Op::Sin;
        assert_ne!(Op::Add, Op::Sub);
    }

    // ── Binary arithmetic ────────────────────────────────────────────────────

    #[test]
    fn test_add_int_int() {
        let mut s = state_with(&[3, 4]);
        apply_op(&mut s, Op::Add).unwrap();
        assert_eq!(s.peek(), Some(&int(7)));
        assert_eq!(s.depth(), 1);
    }

    #[test]
    fn test_sub_int_int() {
        let mut s = state_with(&[10, 3]);
        apply_op(&mut s, Op::Sub).unwrap();
        assert_eq!(s.peek(), Some(&int(7)));
    }

    #[test]
    fn test_mul_int_int() {
        let mut s = state_with(&[6, 7]);
        apply_op(&mut s, Op::Mul).unwrap();
        assert_eq!(s.peek(), Some(&int(42)));
    }

    #[test]
    fn test_div_exact_stays_integer() {
        let mut s = state_with(&[12, 4]);
        apply_op(&mut s, Op::Div).unwrap();
        assert_eq!(s.peek(), Some(&int(3)));
    }

    #[test]
    fn test_div_non_exact_promotes_to_float() {
        let mut s = state_with(&[10, 3]);
        apply_op(&mut s, Op::Div).unwrap();
        assert!(matches!(s.peek(), Some(CalcValue::Float(_))));
        assert!(approx_eq(top_f64(&s), 10.0 / 3.0));
    }

    #[test]
    fn test_div_by_zero_returns_error_stack_unchanged() {
        let mut s = state_with(&[5, 0]);
        let err = apply_op(&mut s, Op::Div).unwrap_err();
        assert!(matches!(err, CalcError::DivisionByZero));
        assert_eq!(s.depth(), 2);
        assert_eq!(s.peek(), Some(&int(0)));
    }

    #[test]
    fn test_pow_integer_exact() {
        let mut s = state_with(&[2, 10]);
        apply_op(&mut s, Op::Pow).unwrap();
        assert_eq!(s.peek(), Some(&int(1024)));
    }

    #[test]
    fn test_mod_integer() {
        let mut s = state_with(&[17, 5]);
        apply_op(&mut s, Op::Mod).unwrap();
        assert_eq!(s.peek(), Some(&int(2)));
    }

    #[test]
    fn test_pow_overflow_domain_error_stack_unchanged() {
        let mut s = CalcState::new();
        s.push(float(1e300));
        s.push(float(1e300)); // 1e300^1e300 overflows to infinity
        let err = apply_op(&mut s, Op::Pow).unwrap_err();
        assert!(matches!(err, CalcError::DomainError(_)));
        assert_eq!(s.depth(), 2);
    }

    #[test]
    fn test_exp_overflow_domain_error_stack_unchanged() {
        let mut s = CalcState::new();
        s.push(float(1e300)); // e^1e300 overflows
        let err = apply_op(&mut s, Op::Exp).unwrap_err();
        assert!(matches!(err, CalcError::DomainError(_)));
        assert_eq!(s.depth(), 1);
    }

    #[test]
    fn test_exp10_overflow_domain_error_stack_unchanged() {
        let mut s = CalcState::new();
        s.push(float(1e300)); // 10^1e300 overflows
        let err = apply_op(&mut s, Op::Exp10).unwrap_err();
        assert!(matches!(err, CalcError::DomainError(_)));
        assert_eq!(s.depth(), 1);
    }

    #[test]
    fn test_mod_by_zero_error_stack_unchanged() {
        let mut s = state_with(&[7, 0]);
        let err = apply_op(&mut s, Op::Mod).unwrap_err();
        assert!(matches!(err, CalcError::DivisionByZero));
        assert_eq!(s.depth(), 2);
    }

    #[test]
    fn test_add_mixed_int_float() {
        let mut s = CalcState::new();
        s.push(int(3));
        s.push(float(1.5));
        apply_op(&mut s, Op::Add).unwrap();
        assert!(approx_eq(top_f64(&s), 4.5));
    }

    #[test]
    fn test_binary_op_underflow_stack_unchanged() {
        let mut s = state_with(&[1]);
        let err = apply_op(&mut s, Op::Add).unwrap_err();
        assert!(matches!(err, CalcError::StackUnderflow));
        assert_eq!(s.depth(), 1);
        assert_eq!(s.peek(), Some(&int(1)));
    }

    // ── Unary arithmetic ─────────────────────────────────────────────────────

    #[test]
    fn test_negate_integer() {
        let mut s = state_with(&[5]);
        apply_op(&mut s, Op::Negate).unwrap();
        assert_eq!(s.peek(), Some(&int(-5)));
    }

    #[test]
    fn test_negate_float() {
        let mut s = CalcState::new();
        s.push(float(2.5));
        apply_op(&mut s, Op::Negate).unwrap();
        assert!(approx_eq(top_f64(&s), -2.5));
    }

    #[test]
    fn test_sqrt_integer() {
        let mut s = state_with(&[9]);
        apply_op(&mut s, Op::Sqrt).unwrap();
        assert!(approx_eq(top_f64(&s), 3.0));
    }

    #[test]
    fn test_sqrt_negative_domain_error_stack_unchanged() {
        let mut s = state_with(&[-1]);
        let err = apply_op(&mut s, Op::Sqrt).unwrap_err();
        assert!(matches!(err, CalcError::DomainError(_)));
        assert_eq!(s.depth(), 1);
        assert_eq!(s.peek(), Some(&int(-1)));
    }

    #[test]
    fn test_square_integer() {
        let mut s = state_with(&[7]);
        apply_op(&mut s, Op::Square).unwrap();
        assert_eq!(s.peek(), Some(&int(49)));
    }

    #[test]
    fn test_reciprocal() {
        let mut s = state_with(&[4]);
        apply_op(&mut s, Op::Reciprocal).unwrap();
        assert!(approx_eq(top_f64(&s), 0.25));
    }

    #[test]
    fn test_reciprocal_zero_error_stack_unchanged() {
        let mut s = state_with(&[0]);
        let err = apply_op(&mut s, Op::Reciprocal).unwrap_err();
        assert!(matches!(err, CalcError::DivisionByZero));
        assert_eq!(s.depth(), 1);
    }

    #[test]
    fn test_abs_negative_integer() {
        let mut s = state_with(&[-5]);
        apply_op(&mut s, Op::Abs).unwrap();
        assert_eq!(s.peek(), Some(&int(5)));
    }

    #[test]
    fn test_abs_negative_float() {
        let mut s = CalcState::new();
        s.push(float(-2.5));
        apply_op(&mut s, Op::Abs).unwrap();
        assert!(approx_eq(top_f64(&s), 2.5));
    }

    #[test]
    fn test_factorial_five() {
        let mut s = state_with(&[5]);
        apply_op(&mut s, Op::Factorial).unwrap();
        assert_eq!(s.peek(), Some(&int(120)));
    }

    #[test]
    fn test_factorial_zero() {
        let mut s = state_with(&[0]);
        apply_op(&mut s, Op::Factorial).unwrap();
        assert_eq!(s.peek(), Some(&int(1)));
    }

    #[test]
    fn test_factorial_negative_domain_error_stack_unchanged() {
        let mut s = state_with(&[-1]);
        let err = apply_op(&mut s, Op::Factorial).unwrap_err();
        assert!(matches!(err, CalcError::DomainError(_)));
        assert_eq!(s.depth(), 1);
    }

    #[test]
    fn test_factorial_float_not_integer_error() {
        let mut s = CalcState::new();
        s.push(float(5.0));
        let err = apply_op(&mut s, Op::Factorial).unwrap_err();
        assert!(matches!(err, CalcError::NotAnInteger));
        assert_eq!(s.depth(), 1);
    }

    #[test]
    fn test_unary_op_underflow_stack_unchanged() {
        let mut s = CalcState::new();
        let err = apply_op(&mut s, Op::Sqrt).unwrap_err();
        assert!(matches!(err, CalcError::StackUnderflow));
        assert_eq!(s.depth(), 0);
    }

    // ── Trig ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_sin_90_deg() {
        let mut s = state_with(&[90]);
        apply_op(&mut s, Op::Sin).unwrap();
        assert!(approx_eq(top_f64(&s), 1.0));
    }

    #[test]
    fn test_cos_0_deg() {
        let mut s = state_with(&[0]);
        apply_op(&mut s, Op::Cos).unwrap();
        assert!(approx_eq(top_f64(&s), 1.0));
    }

    #[test]
    fn test_tan_45_deg() {
        let mut s = state_with(&[45]);
        apply_op(&mut s, Op::Tan).unwrap();
        assert!(approx_eq(top_f64(&s), 1.0));
    }

    #[test]
    fn test_sin_rad_mode() {
        let mut s = CalcState::new();
        s.angle_mode = AngleMode::Rad;
        s.push(float(std::f64::consts::PI / 2.0));
        apply_op(&mut s, Op::Sin).unwrap();
        assert!(approx_eq(top_f64(&s), 1.0));
    }

    #[test]
    fn test_sin_grad_mode() {
        let mut s = CalcState::new();
        s.angle_mode = AngleMode::Grad;
        s.push(int(100)); // 100 grad = 90 deg
        apply_op(&mut s, Op::Sin).unwrap();
        assert!(approx_eq(top_f64(&s), 1.0));
    }

    #[test]
    fn test_asin_one_deg() {
        let mut s = CalcState::new();
        s.push(float(1.0));
        apply_op(&mut s, Op::Asin).unwrap();
        assert!(approx_eq(top_f64(&s), 90.0));
    }

    #[test]
    fn test_acos_one_deg() {
        let mut s = CalcState::new();
        s.push(float(1.0));
        apply_op(&mut s, Op::Acos).unwrap();
        assert!(approx_eq(top_f64(&s), 0.0));
    }

    #[test]
    fn test_atan_one_deg() {
        let mut s = CalcState::new();
        s.push(float(1.0));
        apply_op(&mut s, Op::Atan).unwrap();
        assert!(approx_eq(top_f64(&s), 45.0));
    }

    #[test]
    fn test_asin_domain_error_stack_unchanged() {
        let mut s = CalcState::new();
        s.push(float(2.0)); // asin(2) is NaN
        let err = apply_op(&mut s, Op::Asin).unwrap_err();
        assert!(matches!(err, CalcError::DomainError(_)));
        assert_eq!(s.depth(), 1);
        assert!(approx_eq(top_f64(&s), 2.0));
    }

    // ── Log/Exp ──────────────────────────────────────────────────────────────

    #[test]
    fn test_ln_of_e() {
        let mut s = CalcState::new();
        s.push(float(std::f64::consts::E));
        apply_op(&mut s, Op::Ln).unwrap();
        assert!(approx_eq(top_f64(&s), 1.0));
    }

    #[test]
    fn test_log10_100() {
        let mut s = state_with(&[100]);
        apply_op(&mut s, Op::Log10).unwrap();
        assert!(approx_eq(top_f64(&s), 2.0));
    }

    #[test]
    fn test_exp_1() {
        let mut s = state_with(&[1]);
        apply_op(&mut s, Op::Exp).unwrap();
        assert!(approx_eq(top_f64(&s), std::f64::consts::E));
    }

    #[test]
    fn test_exp10_2() {
        let mut s = state_with(&[2]);
        apply_op(&mut s, Op::Exp10).unwrap();
        assert!(approx_eq(top_f64(&s), 100.0));
    }

    #[test]
    fn test_ln_zero_domain_error_stack_unchanged() {
        let mut s = state_with(&[0]);
        let err = apply_op(&mut s, Op::Ln).unwrap_err();
        assert!(matches!(err, CalcError::DomainError(_)));
        assert_eq!(s.depth(), 1);
    }

    #[test]
    fn test_ln_negative_domain_error_stack_unchanged() {
        let mut s = state_with(&[-5]);
        let err = apply_op(&mut s, Op::Ln).unwrap_err();
        assert!(matches!(err, CalcError::DomainError(_)));
        assert_eq!(s.depth(), 1);
    }

    #[test]
    fn test_log10_zero_domain_error_stack_unchanged() {
        let mut s = state_with(&[0]);
        let err = apply_op(&mut s, Op::Log10).unwrap_err();
        assert!(matches!(err, CalcError::DomainError(_)));
        assert_eq!(s.depth(), 1);
    }

    // ── Bitwise ──────────────────────────────────────────────────────────────

    #[test]
    fn test_and_integers() {
        let mut s = state_with(&[0b1100, 0b1010]);
        apply_op(&mut s, Op::And).unwrap();
        assert_eq!(s.peek(), Some(&int(0b1000)));
    }

    #[test]
    fn test_or_integers() {
        let mut s = state_with(&[0b1100, 0b1010]);
        apply_op(&mut s, Op::Or).unwrap();
        assert_eq!(s.peek(), Some(&int(0b1110)));
    }

    #[test]
    fn test_xor_integers() {
        let mut s = state_with(&[0b1100, 0b1010]);
        apply_op(&mut s, Op::Xor).unwrap();
        assert_eq!(s.peek(), Some(&int(0b0110)));
    }

    #[test]
    fn test_shl_integer() {
        let mut s = state_with(&[1, 4]);
        apply_op(&mut s, Op::Shl).unwrap();
        assert_eq!(s.peek(), Some(&int(16)));
    }

    #[test]
    fn test_shr_integer() {
        let mut s = state_with(&[16, 2]);
        apply_op(&mut s, Op::Shr).unwrap();
        assert_eq!(s.peek(), Some(&int(4)));
    }

    #[test]
    fn test_not_integer() {
        let mut s = state_with(&[0]);
        apply_op(&mut s, Op::Not).unwrap();
        // !0 = -1 for IBig (two's complement signed)
        assert_eq!(s.peek(), Some(&int(-1)));
    }

    #[test]
    fn test_and_float_not_integer_error_stack_unchanged() {
        let mut s = CalcState::new();
        s.push(float(3.0));
        s.push(int(2));
        let err = apply_op(&mut s, Op::And).unwrap_err();
        assert!(matches!(err, CalcError::NotAnInteger));
        assert_eq!(s.depth(), 2);
    }

    #[test]
    fn test_not_float_not_integer_error_stack_unchanged() {
        let mut s = CalcState::new();
        s.push(float(3.0));
        let err = apply_op(&mut s, Op::Not).unwrap_err();
        assert!(matches!(err, CalcError::NotAnInteger));
        assert_eq!(s.depth(), 1);
    }

    // ── Constants ────────────────────────────────────────────────────────────

    #[test]
    fn test_push_pi() {
        let mut s = CalcState::new();
        apply_op(&mut s, Op::PushPi).unwrap();
        assert_eq!(s.depth(), 1);
        assert!(approx_eq(top_f64(&s), std::f64::consts::PI));
    }

    #[test]
    fn test_push_e() {
        let mut s = CalcState::new();
        apply_op(&mut s, Op::PushE).unwrap();
        assert!(approx_eq(top_f64(&s), std::f64::consts::E));
    }

    #[test]
    fn test_push_phi() {
        let mut s = CalcState::new();
        apply_op(&mut s, Op::PushPhi).unwrap();
        let phi = (1.0 + 5.0_f64.sqrt()) / 2.0;
        assert!(approx_eq(top_f64(&s), phi));
    }

    // ── Stack ops via apply_op ────────────────────────────────────────────────

    #[test]
    fn test_swap_via_apply_op() {
        let mut s = state_with(&[1, 2]);
        apply_op(&mut s, Op::Swap).unwrap();
        assert_eq!(s.peek(), Some(&int(1)));
    }

    #[test]
    fn test_dup_via_apply_op() {
        let mut s = state_with(&[5]);
        apply_op(&mut s, Op::Dup).unwrap();
        assert_eq!(s.depth(), 2);
        assert_eq!(s.peek(), Some(&int(5)));
    }

    #[test]
    fn test_drop_via_apply_op() {
        let mut s = state_with(&[1, 2]);
        apply_op(&mut s, Op::Drop).unwrap();
        assert_eq!(s.depth(), 1);
        assert_eq!(s.peek(), Some(&int(1)));
    }

    #[test]
    fn test_rotate_via_apply_op() {
        let mut s = state_with(&[1, 2, 3]);
        apply_op(&mut s, Op::Rotate).unwrap();
        assert_eq!(s.peek(), Some(&int(2)));
    }

    #[test]
    fn test_clear_via_apply_op() {
        let mut s = state_with(&[1, 2, 3]);
        apply_op(&mut s, Op::Clear).unwrap();
        assert!(s.is_empty());
    }

    #[test]
    fn test_swap_underflow_via_apply_op() {
        let mut s = state_with(&[1]);
        let err = apply_op(&mut s, Op::Swap).unwrap_err();
        assert!(matches!(err, CalcError::StackUnderflow));
        assert_eq!(s.depth(), 1);
    }
}
