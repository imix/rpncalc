use std::fmt;

#[derive(Debug, PartialEq)]
pub enum CalcError {
    StackUnderflow,
    DivisionByZero,
    DomainError(String),
    InvalidInput(String),
    NotAnInteger,
}

impl fmt::Display for CalcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CalcError::StackUnderflow => write!(f, "Stack underflow"),
            CalcError::DivisionByZero => write!(f, "Division by zero"),
            CalcError::DomainError(msg) => write!(f, "Domain error: {}", msg),
            CalcError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            CalcError::NotAnInteger => write!(f, "Operation requires an integer"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calcerror_display_messages() {
        assert_eq!(CalcError::StackUnderflow.to_string(), "Stack underflow");
        assert_eq!(CalcError::DivisionByZero.to_string(), "Division by zero");
        assert_eq!(
            CalcError::DomainError("sqrt of negative".to_string()).to_string(),
            "Domain error: sqrt of negative"
        );
        assert_eq!(
            CalcError::InvalidInput("foo".to_string()).to_string(),
            "Invalid input: foo"
        );
        assert_eq!(
            CalcError::NotAnInteger.to_string(),
            "Operation requires an integer"
        );
    }
}
