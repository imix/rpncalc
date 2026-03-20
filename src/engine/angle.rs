use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum AngleMode {
    Deg,
    Rad,
    Grad,
}

impl AngleMode {
    pub fn to_radians(self, val: f64) -> f64 {
        match self {
            AngleMode::Rad => val,
            AngleMode::Deg => val * std::f64::consts::PI / 180.0,
            AngleMode::Grad => val * std::f64::consts::PI / 200.0,
        }
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn from_radians(self, val: f64) -> f64 {
        match self {
            AngleMode::Rad => val,
            AngleMode::Deg => val * 180.0 / std::f64::consts::PI,
            AngleMode::Grad => val * 200.0 / std::f64::consts::PI,
        }
    }

    #[allow(dead_code)]
    pub fn cycle(self) -> AngleMode {
        match self {
            AngleMode::Deg => AngleMode::Rad,
            AngleMode::Rad => AngleMode::Grad,
            AngleMode::Grad => AngleMode::Deg,
        }
    }
}

impl fmt::Display for AngleMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AngleMode::Deg => write!(f, "DEG"),
            AngleMode::Rad => write!(f, "RAD"),
            AngleMode::Grad => write!(f, "GRD"),
        }
    }
}
