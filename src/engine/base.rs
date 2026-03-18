use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum Base {
    Dec,
    Hex,
    Oct,
    Bin,
}

impl Base {
    pub fn cycle(self) -> Base {
        match self {
            Base::Dec => Base::Hex,
            Base::Hex => Base::Oct,
            Base::Oct => Base::Bin,
            Base::Bin => Base::Dec,
        }
    }

    pub fn radix(self) -> u32 {
        match self {
            Base::Dec => 10,
            Base::Hex => 16,
            Base::Oct => 8,
            Base::Bin => 2,
        }
    }
}

impl fmt::Display for Base {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Base::Dec => write!(f, "DEC"),
            Base::Hex => write!(f, "HEX"),
            Base::Oct => write!(f, "OCT"),
            Base::Bin => write!(f, "BIN"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum HexStyle {
    /// Displays as `0xFF`
    ZeroX,
    /// Displays as `$FF`
    Dollar,
    /// Displays as `#FF`
    Hash,
    /// Displays as `FFh`
    Suffix,
}

impl HexStyle {
    pub fn cycle(self) -> HexStyle {
        match self {
            HexStyle::ZeroX => HexStyle::Dollar,
            HexStyle::Dollar => HexStyle::Hash,
            HexStyle::Hash => HexStyle::Suffix,
            HexStyle::Suffix => HexStyle::ZeroX,
        }
    }
}

impl fmt::Display for HexStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HexStyle::ZeroX => write!(f, "0x"),
            HexStyle::Dollar => write!(f, "$"),
            HexStyle::Hash => write!(f, "#"),
            HexStyle::Suffix => write!(f, "h"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hexstyle_cycle_full_roundtrip() {
        let start = HexStyle::ZeroX;
        let cycled = start.cycle().cycle().cycle().cycle();
        assert_eq!(cycled, start);
    }

    #[test]
    fn test_hexstyle_display() {
        assert_eq!(HexStyle::ZeroX.to_string(), "0x");
        assert_eq!(HexStyle::Dollar.to_string(), "$");
        assert_eq!(HexStyle::Hash.to_string(), "#");
        assert_eq!(HexStyle::Suffix.to_string(), "h");
    }
}
