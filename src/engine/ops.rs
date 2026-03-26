use crate::engine::angle::AngleMode;
use crate::engine::constants;
use crate::engine::error::CalcError;
use crate::engine::stack::CalcState;
use crate::engine::units::{
    atoms_to_display, combine_atoms_mul, convert_tagged_to_unit, derive_display_from_dim,
    parse_unit_expr_atoms, TaggedValue,
};
use crate::engine::value::CalcValue;
use dashu::float::FBig;
use dashu::integer::IBig;

/// All calculator operations. Dispatched via `apply_op`.
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
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
    // Rounding / sign
    Floor,
    Ceil,
    Trunc,
    Round,
    Sign,
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
        // Binary arithmetic — unit-aware intercept
        Op::Add => tagged_binary_op(state, op, do_add),
        Op::Sub => tagged_binary_op(state, op, do_sub),
        Op::Mul => tagged_binary_op(state, op, do_mul),
        Op::Div => tagged_binary_op(state, op, do_div),
        Op::Pow => tagged_binary_op(state, op, do_pow),
        Op::Mod => tagged_binary_op(state, op, do_mod),
        // Binary bitwise — tagged values not supported in bitwise ops
        Op::And => binary_op(state, do_and),
        Op::Or => binary_op(state, do_or),
        Op::Xor => binary_op(state, do_xor),
        Op::Shl => binary_op(state, do_shl),
        Op::Shr => binary_op(state, do_shr),
        // Unary — unit-aware intercept for relevant ops
        Op::Negate => tagged_unary_op(state, op, do_negate),
        Op::Abs => tagged_unary_op(state, op, do_abs),
        Op::Floor => tagged_unary_op(state, op, do_floor),
        Op::Ceil => tagged_unary_op(state, op, do_ceil),
        Op::Trunc => tagged_unary_op(state, op, do_trunc),
        Op::Sign => tagged_unary_op(state, op, do_sign),
        // Compound-unit ops: unit-aware dispatch
        Op::Sqrt => tagged_sqrt_op(state),
        Op::Square => tagged_compound_error_unary(state, do_sq),
        Op::Reciprocal => tagged_reciprocal_op(state),
        Op::Factorial => unary_op(state, do_factorial),
        Op::Not => unary_op(state, do_not),
        // Trig — error if tagged (dimensioned trig is undefined)
        Op::Sin => {
            let m = state.angle_mode;
            tagged_compound_error_unary(state, |v| do_trig(v, m, f64::sin))
        }
        Op::Cos => {
            let m = state.angle_mode;
            tagged_compound_error_unary(state, |v| do_trig(v, m, f64::cos))
        }
        Op::Tan => {
            let m = state.angle_mode;
            tagged_compound_error_unary(state, |v| do_trig(v, m, f64::tan))
        }
        Op::Asin => {
            let m = state.angle_mode;
            tagged_compound_error_unary(state, |v| do_atrig(v, m, f64::asin))
        }
        Op::Acos => {
            let m = state.angle_mode;
            tagged_compound_error_unary(state, |v| do_atrig(v, m, f64::acos))
        }
        Op::Atan => {
            let m = state.angle_mode;
            tagged_compound_error_unary(state, |v| do_atrig(v, m, f64::atan))
        }
        // Log/Exp — error if tagged
        Op::Log10 => tagged_compound_error_unary(state, do_log10),
        Op::Ln => tagged_compound_error_unary(state, do_ln),
        Op::Exp => tagged_compound_error_unary(state, do_exp),
        Op::Exp10 => tagged_compound_error_unary(state, do_exp10),
        // Stack ops — delegate to CalcState (already atomic)
        Op::Swap => state.swap(),
        Op::Dup => state.dup(),
        Op::Drop => state.drop(),
        Op::Rotate => state.rotate(),
        Op::Clear => {
            state.clear();
            Ok(())
        }
        // Rounding / sign — Floor/Ceil/Trunc/Sign handled above with tagged_unary_op
        Op::Round => tagged_binary_op(state, op, do_round),
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

// ── Unit-aware dispatch helpers ──────────────────────────────────────────────

/// Extract an FBig amount from a CalcValue without going through f64.
fn extract_fbig(v: CalcValue) -> FBig {
    match v {
        CalcValue::Float(f) => f,
        CalcValue::Integer(i) => int_to_fbig(&i),
        CalcValue::Tagged(t) => t.amount,
    }
}

/// Derive the display string for a compound unit result of multiplying y_unit × x_unit.
fn derive_mul_unit_display(y_unit: &str, x_unit: &str) -> String {
    let y_atoms = parse_unit_expr_atoms(y_unit).unwrap_or_else(|_| vec![]);
    let x_atoms = parse_unit_expr_atoms(x_unit).unwrap_or_else(|_| vec![]);
    let result = combine_atoms_mul(&y_atoms, &x_atoms);
    if result.is_empty() {
        String::new()
    } else {
        atoms_to_display(&result)
    }
}

/// Derive the display string for y_unit / x_unit.
fn derive_div_unit_display(y_unit: &str, x_unit: &str) -> String {
    let y_atoms = parse_unit_expr_atoms(y_unit).unwrap_or_else(|_| vec![]);
    let x_neg: Vec<(String, i8)> = parse_unit_expr_atoms(x_unit)
        .unwrap_or_else(|_| vec![])
        .into_iter()
        .map(|(a, e)| (a, -e))
        .collect();
    let result = combine_atoms_mul(&y_atoms, &x_neg);
    if result.is_empty() {
        String::new()
    } else {
        atoms_to_display(&result)
    }
}

/// Binary op with unit-aware pre-dispatch.
/// - Both Tagged: validate/convert compatibility, apply compound unit arithmetic.
/// - One Tagged + one plain: for Add/Sub → error; for Mul/Div → carry unit.
/// - Both plain: delegate directly to `f`.
fn tagged_binary_op(
    state: &mut CalcState,
    op: Op,
    f: impl Fn(CalcValue, CalcValue) -> Result<CalcValue, CalcError>,
) -> Result<(), CalcError> {
    if state.depth() < 2 {
        return Err(CalcError::StackUnderflow);
    }
    let n = state.stack.len();
    let x = state.stack[n - 1].clone(); // position 1 (top)
    let y = state.stack[n - 2].clone(); // position 2

    match (&y, &x) {
        (CalcValue::Tagged(ty), CalcValue::Tagged(tx)) => {
            let ty = ty.clone();
            let tx = tx.clone();

            match op {
                Op::Add | Op::Sub => {
                    // Require identical dimension vectors
                    if tx.dim != ty.dim {
                        return Err(CalcError::IncompatibleUnits(format!(
                            "incompatible units: {} and {}",
                            ty.unit, tx.unit
                        )));
                    }
                    // Convert y to x's unit scale, then apply the operation
                    let converted_y_amount = convert_tagged_to_unit(&ty, &tx)?;
                    let plain_y = CalcValue::Float(converted_y_amount);
                    let plain_x = CalcValue::Float(tx.amount.clone());
                    let plain_result = f(plain_y, plain_x)?;
                    let result = CalcValue::Tagged(TaggedValue {
                        amount: extract_fbig(plain_result),
                        unit: tx.unit.clone(),
                        dim: tx.dim.clone(),
                    });
                    state.stack.truncate(n - 2);
                    state.push(result);
                    Ok(())
                }
                Op::Mul => {
                    // Compound multiplication: amounts multiply, dims add
                    let result_dim = ty.dim.add(&tx.dim);
                    let result_amount = ty.amount.clone() * tx.amount.clone();

                    if result_dim.is_dimensionless() {
                        state.stack.truncate(n - 2);
                        state.push(CalcValue::Float(result_amount));
                        Ok(())
                    } else {
                        let result_unit = derive_mul_unit_display(&ty.unit, &tx.unit);
                        let result_unit = if result_unit.is_empty() {
                            derive_display_from_dim(&result_dim)
                        } else {
                            result_unit
                        };
                        let result = CalcValue::Tagged(TaggedValue {
                            amount: result_amount,
                            unit: result_unit,
                            dim: result_dim,
                        });
                        state.stack.truncate(n - 2);
                        state.push(result);
                        Ok(())
                    }
                }
                Op::Div => {
                    let result_dim = ty.dim.sub(&tx.dim);

                    if result_dim.is_dimensionless() {
                        // Same dimension: convert y to x's scale then divide
                        let converted_y = convert_tagged_to_unit(&ty, &tx)?;
                        let result = f(
                            CalcValue::Float(converted_y),
                            CalcValue::Float(tx.amount.clone()),
                        )?;
                        state.stack.truncate(n - 2);
                        state.push(result);
                        Ok(())
                    } else {
                        // Different dimensions: compound division
                        let result_amount = ty.amount.clone() / tx.amount.clone();
                        let result_unit = derive_div_unit_display(&ty.unit, &tx.unit);
                        let result_unit = if result_unit.is_empty() {
                            derive_display_from_dim(&result_dim)
                        } else {
                            result_unit
                        };
                        let result = CalcValue::Tagged(TaggedValue {
                            amount: result_amount,
                            unit: result_unit,
                            dim: result_dim,
                        });
                        state.stack.truncate(n - 2);
                        state.push(result);
                        Ok(())
                    }
                }
                Op::Pow | Op::Mod | Op::Round => Err(CalcError::IncompatibleUnits(
                    "operation not supported on unit-tagged values".to_string(),
                )),
                _ => binary_op(state, f),
            }
        }

        (CalcValue::Tagged(_), _) | (_, CalcValue::Tagged(_)) => {
            // One tagged, one plain
            match op {
                Op::Add | Op::Sub => Err(CalcError::IncompatibleUnits(
                    "cannot mix unit-tagged and plain values with + or -".to_string(),
                )),
                Op::Mul => {
                    // plain × tagged or tagged × plain → result has tagged's unit
                    let (tagged, plain, tagged_is_x) = match (&y, &x) {
                        (CalcValue::Tagged(t), _) => (t.clone(), x.clone(), false),
                        (_, CalcValue::Tagged(t)) => (t.clone(), y.clone(), true),
                        _ => unreachable!(),
                    };
                    let plain_tagged = CalcValue::Float(tagged.amount.clone());
                    let (a_arg, b_arg) = if tagged_is_x {
                        (plain.clone(), plain_tagged)
                    } else {
                        (plain_tagged, plain.clone())
                    };
                    let plain_result = f(a_arg, b_arg)?;
                    let result = CalcValue::Tagged(TaggedValue {
                        amount: extract_fbig(plain_result),
                        unit: tagged.unit.clone(),
                        dim: tagged.dim.clone(),
                    });
                    state.stack.truncate(n - 2);
                    state.push(result);
                    Ok(())
                }
                Op::Div => {
                    match (&y, &x) {
                        (CalcValue::Tagged(ty), _) => {
                            // tagged(y) / plain(x) → result has tagged's unit
                            let ty = ty.clone();
                            let plain_result = f(
                                CalcValue::Float(ty.amount.clone()),
                                x.clone(),
                            )?;
                            let result = CalcValue::Tagged(TaggedValue {
                                amount: extract_fbig(plain_result),
                                unit: ty.unit.clone(),
                                dim: ty.dim.clone(),
                            });
                            state.stack.truncate(n - 2);
                            state.push(result);
                            Ok(())
                        }
                        (_, CalcValue::Tagged(tx)) => {
                            // plain(y) / tagged(x) → compound result: dim = {0} - tx.dim = negated
                            let tx = tx.clone();
                            let result_dim = tx.dim.negate();
                            let result_amount = f(y.clone(), CalcValue::Float(tx.amount.clone()))?;
                            let result_unit = {
                                let neg_atoms: Vec<(String, i8)> =
                                    parse_unit_expr_atoms(&tx.unit)
                                        .unwrap_or_else(|_| vec![])
                                        .into_iter()
                                        .map(|(a, e)| (a, -e))
                                        .collect();
                                if neg_atoms.is_empty() {
                                    derive_display_from_dim(&result_dim)
                                } else {
                                    atoms_to_display(&neg_atoms)
                                }
                            };
                            let result = CalcValue::Tagged(TaggedValue {
                                amount: extract_fbig(result_amount),
                                unit: result_unit,
                                dim: result_dim,
                            });
                            state.stack.truncate(n - 2);
                            state.push(result);
                            Ok(())
                        }
                        _ => unreachable!(),
                    }
                }
                _ => Err(CalcError::IncompatibleUnits(
                    "operation not supported on unit-tagged values".to_string(),
                )),
            }
        }

        _ => binary_op(state, f),
    }
}

/// Unary sqrt with compound-unit support.
/// - Dimensionless or integer/float: apply sqrt normally.
/// - Tagged with all-even exponents: halve dims, derive unit display.
/// - Tagged with odd exponent: error `non-integer unit exponent after sqrt`.
fn tagged_sqrt_op(state: &mut CalcState) -> Result<(), CalcError> {
    let top = state.peek().ok_or(CalcError::StackUnderflow)?.clone();
    match top {
        CalcValue::Tagged(t) => {
            let new_dim = t.dim.halve().ok_or_else(|| {
                CalcError::IncompatibleUnits("non-integer unit exponent after sqrt".to_string())
            })?;
            let val = t.amount.to_f64().value();
            if val < 0.0 {
                return Err(CalcError::DomainError(
                    "sqrt requires non-negative number".to_string(),
                ));
            }
            let new_amount = FBig::try_from(val.sqrt()).unwrap_or(FBig::ZERO);
            let new_unit = if new_dim.is_dimensionless() {
                String::new()
            } else {
                // Derive display: halve atom exponents if all even, else fall back to dim
                match parse_unit_expr_atoms(&t.unit) {
                    Ok(atoms) if atoms.iter().all(|(_, e)| e % 2 == 0) => {
                        let halved: Vec<(String, i8)> = atoms
                            .into_iter()
                            .map(|(a, e)| (a, e / 2))
                            .filter(|(_, e)| *e != 0)
                            .collect();
                        atoms_to_display(&halved)
                    }
                    _ => derive_display_from_dim(&new_dim),
                }
            };
            if new_dim.is_dimensionless() {
                state.pop().expect("SAFETY: peeked above");
                state.push(CalcValue::Float(new_amount));
            } else {
                let result = CalcValue::Tagged(TaggedValue {
                    amount: new_amount,
                    unit: new_unit,
                    dim: new_dim,
                });
                state.pop().expect("SAFETY: peeked above");
                state.push(result);
            }
            Ok(())
        }
        _ => unary_op(state, do_sqrt),
    }
}

/// Unary reciprocal (1/x) with compound-unit support.
/// - Plain values: compute 1/x normally.
/// - Tagged: negate all dimension exponents, compute 1/amount, derive new unit display.
fn tagged_reciprocal_op(state: &mut CalcState) -> Result<(), CalcError> {
    let top = state.peek().ok_or(CalcError::StackUnderflow)?.clone();
    match top {
        CalcValue::Tagged(t) => {
            let val = t.amount.to_f64().value();
            if val == 0.0 {
                return Err(CalcError::DivisionByZero);
            }
            let new_amount = FBig::try_from(1.0 / val).unwrap_or(FBig::ZERO);
            let new_dim = t.dim.negate();
            let new_unit = if new_dim.is_dimensionless() {
                String::new()
            } else {
                match parse_unit_expr_atoms(&t.unit) {
                    Ok(atoms) => {
                        let negated: Vec<(String, i8)> =
                            atoms.into_iter().map(|(a, e)| (a, -e)).collect();
                        atoms_to_display(&negated)
                    }
                    _ => derive_display_from_dim(&new_dim),
                }
            };
            if new_dim.is_dimensionless() {
                state.pop().expect("SAFETY: peeked above");
                state.push(CalcValue::Float(new_amount));
            } else {
                let result = CalcValue::Tagged(TaggedValue {
                    amount: new_amount,
                    unit: new_unit,
                    dim: new_dim,
                });
                state.pop().expect("SAFETY: peeked above");
                state.push(result);
            }
            Ok(())
        }
        _ => unary_op(state, do_reciprocal),
    }
}

/// Unary op with unit-aware pre-dispatch. Ops that preserve units (negate, abs, floor,
/// ceil, trunc, round, sign) extract the amount, apply `f`, and re-wrap with the unit.
fn tagged_unary_op(
    state: &mut CalcState,
    op: Op,
    f: impl Fn(CalcValue) -> Result<CalcValue, CalcError>,
) -> Result<(), CalcError> {
    let top = state.peek().ok_or(CalcError::StackUnderflow)?.clone();
    match top {
        CalcValue::Tagged(ref t) => {
            let plain = CalcValue::Float(t.amount.clone());
            let plain_result = f(plain)?;
            let unit = t.unit.clone();
            let result = match op {
                // Sign returns dimensionless (+1, 0, -1)
                Op::Sign => plain_result,
                _ => CalcValue::Tagged(TaggedValue {
                    amount: FBig::try_from(plain_result.to_f64()).unwrap_or(FBig::ZERO),
                    unit,
                    dim: t.dim.clone(),
                }),
            };
            state.pop().expect("SAFETY: peeked above");
            state.push(result);
            Ok(())
        }
        _ => unary_op(state, f),
    }
}

/// Unary op that errors if the top value is unit-tagged (would produce a compound unit).
fn tagged_compound_error_unary(
    state: &mut CalcState,
    f: impl Fn(CalcValue) -> Result<CalcValue, CalcError>,
) -> Result<(), CalcError> {
    let top = state.peek().ok_or(CalcError::StackUnderflow)?;
    if matches!(top, CalcValue::Tagged(_)) {
        return Err(CalcError::IncompatibleUnits(
            "compound unit not supported".to_string(),
        ));
    }
    unary_op(state, f)
}

fn do_add(a: CalcValue, b: CalcValue) -> Result<CalcValue, CalcError> {
    match (a, b) {
        (CalcValue::Integer(x), CalcValue::Integer(y)) => Ok(CalcValue::Integer(x + y)),
        (CalcValue::Float(x), CalcValue::Float(y)) => Ok(CalcValue::Float(x + y)),
        (CalcValue::Integer(x), CalcValue::Float(y)) => Ok(CalcValue::Float(int_to_fbig(&x) + y)),
        (CalcValue::Float(x), CalcValue::Integer(y)) => Ok(CalcValue::Float(x + int_to_fbig(&y))),
        _ => unreachable!("Tagged values are intercepted by tagged_binary_op"),
    }
}

fn do_sub(a: CalcValue, b: CalcValue) -> Result<CalcValue, CalcError> {
    match (a, b) {
        (CalcValue::Integer(x), CalcValue::Integer(y)) => Ok(CalcValue::Integer(x - y)),
        (CalcValue::Float(x), CalcValue::Float(y)) => Ok(CalcValue::Float(x - y)),
        (CalcValue::Integer(x), CalcValue::Float(y)) => Ok(CalcValue::Float(int_to_fbig(&x) - y)),
        (CalcValue::Float(x), CalcValue::Integer(y)) => Ok(CalcValue::Float(x - int_to_fbig(&y))),
        _ => unreachable!("Tagged values are intercepted by tagged_binary_op"),
    }
}

fn do_mul(a: CalcValue, b: CalcValue) -> Result<CalcValue, CalcError> {
    match (a, b) {
        (CalcValue::Integer(x), CalcValue::Integer(y)) => Ok(CalcValue::Integer(x * y)),
        (CalcValue::Float(x), CalcValue::Float(y)) => Ok(CalcValue::Float(x * y)),
        (CalcValue::Integer(x), CalcValue::Float(y)) => Ok(CalcValue::Float(int_to_fbig(&x) * y)),
        (CalcValue::Float(x), CalcValue::Integer(y)) => Ok(CalcValue::Float(x * int_to_fbig(&y))),
        _ => unreachable!("Tagged values are intercepted by tagged_binary_op"),
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
        _ => unreachable!("Tagged values are intercepted by tagged_binary_op"),
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
        CalcValue::Tagged(_) => unreachable!("Tagged handled by tagged_unary_op"),
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
        CalcValue::Tagged(_) => unreachable!("Tagged blocked by tagged_compound_error_unary"),
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
        CalcValue::Tagged(_) => unreachable!("Tagged handled by tagged_unary_op"),
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
            let n: u64 = u64::try_from(&x)
                .map_err(|_| CalcError::DomainError("factorial argument too large".to_string()))?;
            let mut result = IBig::from(1u32);
            for i in 2..=n {
                result *= IBig::from(i);
            }
            Ok(CalcValue::Integer(result))
        }
        CalcValue::Float(_) | CalcValue::Tagged(_) => Err(CalcError::NotAnInteger),
    }
}

fn do_not(v: CalcValue) -> Result<CalcValue, CalcError> {
    match v {
        CalcValue::Integer(x) => Ok(CalcValue::Integer(!x)),
        CalcValue::Float(_) | CalcValue::Tagged(_) => Err(CalcError::NotAnInteger),
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

fn do_floor(v: CalcValue) -> Result<CalcValue, CalcError> {
    match v {
        CalcValue::Integer(_) => Ok(v),
        CalcValue::Float(f) => {
            let x = f.to_f64().value();
            if !x.is_finite() {
                return Err(CalcError::DomainError("value not finite".to_string()));
            }
            Ok(CalcValue::from_f64(x.floor()))
        }
        CalcValue::Tagged(_) => unreachable!("Tagged handled by tagged_unary_op"),
    }
}

fn do_ceil(v: CalcValue) -> Result<CalcValue, CalcError> {
    match v {
        CalcValue::Integer(_) => Ok(v),
        CalcValue::Float(f) => {
            let x = f.to_f64().value();
            if !x.is_finite() {
                return Err(CalcError::DomainError("value not finite".to_string()));
            }
            Ok(CalcValue::from_f64(x.ceil()))
        }
        CalcValue::Tagged(_) => unreachable!("Tagged handled by tagged_unary_op"),
    }
}

fn do_trunc(v: CalcValue) -> Result<CalcValue, CalcError> {
    match v {
        CalcValue::Integer(_) => Ok(v),
        CalcValue::Float(f) => {
            let x = f.to_f64().value();
            if !x.is_finite() {
                return Err(CalcError::DomainError("value not finite".to_string()));
            }
            Ok(CalcValue::from_f64(x.trunc()))
        }
        CalcValue::Tagged(_) => unreachable!("Tagged handled by tagged_unary_op"),
    }
}

fn do_sign(v: CalcValue) -> Result<CalcValue, CalcError> {
    match v {
        CalcValue::Integer(ref x) => {
            if *x < IBig::ZERO {
                Ok(CalcValue::Integer(IBig::from(-1i32)))
            } else if *x == IBig::ZERO {
                Ok(CalcValue::Integer(IBig::ZERO))
            } else {
                Ok(CalcValue::Integer(IBig::from(1i32)))
            }
        }
        CalcValue::Float(f) => {
            let x = f.to_f64().value();
            if x < 0.0 {
                Ok(CalcValue::Integer(IBig::from(-1i32)))
            } else if x == 0.0 {
                Ok(CalcValue::Integer(IBig::ZERO))
            } else {
                Ok(CalcValue::Integer(IBig::from(1i32)))
            }
        }
        CalcValue::Tagged(_) => unreachable!("Tagged handled by tagged_unary_op"),
    }
}

/// Round `value` to `precision` decimal places.
/// Precision may be negative (rounds to nearest 10^|n|).
/// Precision must be integer-valued.
fn do_round(value: CalcValue, precision: CalcValue) -> Result<CalcValue, CalcError> {
    let n: i32 = match &precision {
        CalcValue::Integer(i) => i
            .to_string()
            .parse::<i32>()
            .map_err(|_| CalcError::DomainError("precision out of range".to_string()))?,
        CalcValue::Float(f) => {
            let x = f.to_f64().value();
            if x.fract() != 0.0 {
                return Err(CalcError::NotAnInteger);
            }
            x as i32
        }
        CalcValue::Tagged(_) => return Err(CalcError::NotAnInteger),
    };
    match value {
        CalcValue::Integer(_) if n >= 0 => Ok(value),
        CalcValue::Integer(ref i) => {
            let scale = 10f64.powi(-n);
            let v = i.to_string().parse::<f64>().unwrap_or(0.0);
            Ok(CalcValue::from_f64((v / scale).round() * scale))
        }
        CalcValue::Float(f) => {
            let x = f.to_f64().value();
            if !x.is_finite() {
                return Err(CalcError::DomainError("value not finite".to_string()));
            }
            let scale = 10f64.powi(n);
            Ok(CalcValue::from_f64((x * scale).round() / scale))
        }
        CalcValue::Tagged(_) => unreachable!("Tagged handled by tagged_binary_op"),
    }
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

    // ── Rounding / sign ops ──────────────────────────────────────────────────

    // AC-1: FLOOR rounds positive float down
    #[test]
    fn test_floor_positive_float() {
        let mut s = CalcState::new();
        s.push(float(2.7));
        apply_op(&mut s, Op::Floor).unwrap();
        assert!(approx_eq(top_f64(&s), 2.0));
        assert_eq!(s.depth(), 1);
    }

    // AC-2: FLOOR of negative float rounds toward −∞ (not toward zero)
    #[test]
    fn test_floor_negative_float() {
        let mut s = CalcState::new();
        s.push(float(-2.3));
        apply_op(&mut s, Op::Floor).unwrap();
        assert!(approx_eq(top_f64(&s), -3.0), "floor(-2.3) should be -3");
    }

    // FLOOR of integer is identity
    #[test]
    fn test_floor_integer_identity() {
        let mut s = state_with(&[5]);
        apply_op(&mut s, Op::Floor).unwrap();
        assert_eq!(s.peek(), Some(&int(5)));
    }

    // AC-3: CEIL rounds positive float up
    #[test]
    fn test_ceil_positive_float() {
        let mut s = CalcState::new();
        s.push(float(2.1));
        apply_op(&mut s, Op::Ceil).unwrap();
        assert!(approx_eq(top_f64(&s), 3.0));
    }

    // CEIL of negative float rounds toward zero
    #[test]
    fn test_ceil_negative_float() {
        let mut s = CalcState::new();
        s.push(float(-2.3));
        apply_op(&mut s, Op::Ceil).unwrap();
        assert!(approx_eq(top_f64(&s), -2.0), "ceil(-2.3) should be -2");
    }

    // AC-4: TRUNC truncates toward zero (not same as floor for negatives)
    #[test]
    fn test_trunc_negative_float() {
        let mut s = CalcState::new();
        s.push(float(-2.7));
        apply_op(&mut s, Op::Trunc).unwrap();
        assert!(approx_eq(top_f64(&s), -2.0), "trunc(-2.7) should be -2, not -3");
    }

    #[test]
    fn test_trunc_positive_float() {
        let mut s = CalcState::new();
        s.push(float(3.9));
        apply_op(&mut s, Op::Trunc).unwrap();
        assert!(approx_eq(top_f64(&s), 3.0));
    }

    // AC-5: ROUND rounds to n decimal places (positive precision)
    #[test]
    fn test_round_positive_precision() {
        let mut s = CalcState::new();
        s.push(float(3.14159));
        s.push(int(3)); // precision = 3
        apply_op(&mut s, Op::Round).unwrap();
        assert!(approx_eq(top_f64(&s), 3.142), "round(3.14159, 3) should be 3.142");
        assert_eq!(s.depth(), 1);
    }

    // AC-6: ROUND with negative precision rounds to power of 10
    #[test]
    fn test_round_negative_precision() {
        let mut s = CalcState::new();
        s.push(float(1234.5));
        s.push(int(-2)); // round to nearest 100
        apply_op(&mut s, Op::Round).unwrap();
        assert!(approx_eq(top_f64(&s), 1200.0), "round(1234.5, -2) should be 1200");
        assert_eq!(s.depth(), 1);
    }

    // AC-7: SIGN of negative value returns −1
    #[test]
    fn test_sign_negative() {
        let mut s = CalcState::new();
        s.push(float(-5.0));
        apply_op(&mut s, Op::Sign).unwrap();
        assert_eq!(s.peek(), Some(&int(-1)));
    }

    // AC-8: SIGN of zero returns 0
    #[test]
    fn test_sign_zero() {
        let mut s = state_with(&[0]);
        apply_op(&mut s, Op::Sign).unwrap();
        assert_eq!(s.peek(), Some(&int(0)));
    }

    // AC-9: SIGN of positive value returns +1
    #[test]
    fn test_sign_positive() {
        let mut s = state_with(&[42]);
        apply_op(&mut s, Op::Sign).unwrap();
        assert_eq!(s.peek(), Some(&int(1)));
    }

    // AC-11: stack underflow on unary rounding ops
    #[test]
    fn test_floor_underflow() {
        let mut s = CalcState::new();
        let err = apply_op(&mut s, Op::Floor).unwrap_err();
        assert!(matches!(err, CalcError::StackUnderflow));
    }

    #[test]
    fn test_sign_underflow() {
        let mut s = CalcState::new();
        let err = apply_op(&mut s, Op::Sign).unwrap_err();
        assert!(matches!(err, CalcError::StackUnderflow));
    }

    // AC-12: ROUND underflow with only 1 item
    #[test]
    fn test_round_underflow_single_item() {
        let mut s = state_with(&[3]);
        let err = apply_op(&mut s, Op::Round).unwrap_err();
        assert!(matches!(err, CalcError::StackUnderflow));
        assert_eq!(s.depth(), 1);
    }

    // AC-13: ROUND with non-integer precision shows NotAnInteger error
    #[test]
    fn test_round_non_integer_precision() {
        let mut s = CalcState::new();
        s.push(float(3.14));
        s.push(float(2.5)); // non-integer precision
        let err = apply_op(&mut s, Op::Round).unwrap_err();
        assert!(matches!(err, CalcError::NotAnInteger));
        assert_eq!(s.depth(), 2); // stack unchanged
    }

    // ── Compound unit operations ──────────────────────────────────────────────

    fn tagged(amount: f64, unit: &str) -> CalcValue {
        CalcValue::Tagged(crate::engine::units::TaggedValue::new(amount, unit))
    }

    fn compound_tagged(amount: f64, unit: &str, dim: crate::engine::units::DimensionVector) -> CalcValue {
        CalcValue::Tagged(crate::engine::units::TaggedValue::new_compound(
            dashu::float::FBig::try_from(amount).unwrap(),
            unit.to_string(),
            dim,
        ))
    }

    fn speed_dim() -> crate::engine::units::DimensionVector {
        crate::engine::units::DimensionVector { m: 1, s: -1, ..Default::default() }
    }
    fn accel_dim() -> crate::engine::units::DimensionVector {
        crate::engine::units::DimensionVector { m: 1, s: -2, ..Default::default() }
    }
    fn area_dim() -> crate::engine::units::DimensionVector {
        crate::engine::units::DimensionVector { m: 2, ..Default::default() }
    }

    // AC-3: Derive speed by dividing distance by time
    #[test]
    fn test_compound_div_speed() {
        let mut s = CalcState::new();
        s.push(tagged(100.0, "km")); // y: 100 km
        s.push(tagged(2.0, "h"));   // x: 2 h
        apply_op(&mut s, Op::Div).unwrap();
        assert_eq!(s.depth(), 1);
        match s.peek().unwrap() {
            CalcValue::Tagged(tv) => {
                assert_eq!(tv.unit, "km/h");
                assert!((tv.amount.to_f64().value() - 50.0).abs() < 1e-6,
                    "100km / 2h = {}", tv.amount.to_f64().value());
            }
            v => panic!("expected Tagged, got {:?}", v),
        }
    }

    // AC-4: speed × time → distance (dimension cancellation)
    #[test]
    fn test_compound_mul_cancellation() {
        let mut s = CalcState::new();
        s.push(compound_tagged(50.0, "km/h", speed_dim())); // y: 50 km/h
        s.push(tagged(2.0, "h"));  // x: 2 h
        apply_op(&mut s, Op::Mul).unwrap();
        assert_eq!(s.depth(), 1);
        match s.peek().unwrap() {
            CalcValue::Tagged(tv) => {
                assert_eq!(tv.unit, "km");
                assert!((tv.amount.to_f64().value() - 100.0).abs() < 1e-6,
                    "50 km/h * 2 h = {}", tv.amount.to_f64().value());
            }
            v => panic!("expected Tagged, got {:?}", v),
        }
    }

    // AC-5: area from two length values
    #[test]
    fn test_compound_mul_area() {
        let mut s = CalcState::new();
        s.push(tagged(5.0, "m"));
        s.push(tagged(3.0, "m"));
        apply_op(&mut s, Op::Mul).unwrap();
        assert_eq!(s.depth(), 1);
        match s.peek().unwrap() {
            CalcValue::Tagged(tv) => {
                assert_eq!(tv.unit, "m2");
                assert!((tv.amount.to_f64().value() - 15.0).abs() < 1e-9);
            }
            v => panic!("expected Tagged, got {:?}", v),
        }
    }

    // AC-6: force from mass × acceleration
    #[test]
    fn test_compound_mul_force() {
        let mut s = CalcState::new();
        s.push(tagged(80.0, "kg")); // y: 80 kg
        s.push(compound_tagged(9.8, "m/s2", accel_dim())); // x: 9.8 m/s2
        apply_op(&mut s, Op::Mul).unwrap();
        assert_eq!(s.depth(), 1);
        match s.peek().unwrap() {
            CalcValue::Tagged(tv) => {
                assert_eq!(tv.unit, "kg*m/s2");
                assert!((tv.amount.to_f64().value() - 784.0).abs() < 0.01,
                    "80 kg * 9.8 m/s2 = {}", tv.amount.to_f64().value());
            }
            v => panic!("expected Tagged, got {:?}", v),
        }
    }

    // AC-7: dimensionless result from same-compound-unit division
    #[test]
    fn test_compound_div_dimensionless() {
        let mut s = CalcState::new();
        s.push(compound_tagged(10.0, "m/s", speed_dim())); // y
        s.push(compound_tagged(5.0, "m/s", speed_dim()));  // x
        apply_op(&mut s, Op::Div).unwrap();
        assert_eq!(s.depth(), 1);
        let result = s.peek().unwrap();
        assert!(!matches!(result, CalcValue::Tagged(_)), "expected plain number, got Tagged");
        assert!((result.to_f64() - 2.0).abs() < 1e-9);
    }

    // AC-8: scalar × compound unit preserves unit
    #[test]
    fn test_compound_mul_scalar() {
        let mut s = CalcState::new();
        s.push(compound_tagged(9.8, "m/s2", accel_dim())); // y: 9.8 m/s2
        s.push(float(2.0)); // x: 2 (plain)
        apply_op(&mut s, Op::Mul).unwrap();
        assert_eq!(s.depth(), 1);
        match s.peek().unwrap() {
            CalcValue::Tagged(tv) => {
                assert_eq!(tv.unit, "m/s2");
                assert!((tv.amount.to_f64().value() - 19.6).abs() < 1e-6);
            }
            v => panic!("expected Tagged, got {:?}", v),
        }
    }

    // AC-9: sqrt reduces even-exponent compound unit
    #[test]
    fn test_compound_sqrt_area() {
        let mut s = CalcState::new();
        s.push(compound_tagged(25.0, "m2", area_dim()));
        apply_op(&mut s, Op::Sqrt).unwrap();
        assert_eq!(s.depth(), 1);
        match s.peek().unwrap() {
            CalcValue::Tagged(tv) => {
                assert_eq!(tv.unit, "m");
                assert!((tv.amount.to_f64().value() - 5.0).abs() < 1e-6);
            }
            v => panic!("expected Tagged, got {:?}", v),
        }
    }

    // AC-10: sqrt on odd-exponent compound unit → error
    #[test]
    fn test_compound_sqrt_odd_error() {
        let mut s = CalcState::new();
        s.push(compound_tagged(4.0, "m/s", speed_dim()));
        let err = apply_op(&mut s, Op::Sqrt).unwrap_err();
        assert!(matches!(&err, CalcError::IncompatibleUnits(e) if e.contains("non-integer")),
            "got: {:?}", err);
        assert_eq!(s.depth(), 1); // stack unchanged
    }

    // AC-11: incompatible compound units in addition → error
    #[test]
    fn test_compound_add_incompatible() {
        let mut s = CalcState::new();
        s.push(compound_tagged(1.0, "m/s", speed_dim()));
        s.push(compound_tagged(1.0, "m/s2", accel_dim()));
        let err = apply_op(&mut s, Op::Add).unwrap_err();
        assert!(matches!(err, CalcError::IncompatibleUnits(_)));
        assert_eq!(s.depth(), 2); // stack unchanged
    }

    // AC-15: add two same-compound-unit values
    #[test]
    fn test_compound_add_same_unit() {
        let mut s = CalcState::new();
        s.push(compound_tagged(1.0, "m/s", speed_dim()));
        s.push(compound_tagged(2.0, "m/s", speed_dim()));
        apply_op(&mut s, Op::Add).unwrap();
        assert_eq!(s.depth(), 1);
        match s.peek().unwrap() {
            CalcValue::Tagged(tv) => {
                assert_eq!(tv.unit, "m/s");
                assert!((tv.amount.to_f64().value() - 3.0).abs() < 1e-9);
            }
            v => panic!("expected Tagged, got {:?}", v),
        }
    }

    // AC-16: reciprocal of a compound-unit value
    #[test]
    fn test_compound_reciprocal() {
        let mut s = CalcState::new();
        s.push(compound_tagged(4.0, "m/s2", accel_dim()));
        apply_op(&mut s, Op::Reciprocal).unwrap();
        assert_eq!(s.depth(), 1);
        match s.peek().unwrap() {
            CalcValue::Tagged(tv) => {
                assert_eq!(tv.unit, "s2/m");
                assert!((tv.amount.to_f64().value() - 0.25).abs() < 1e-9);
            }
            v => panic!("expected Tagged, got {:?}", v),
        }
    }
}
