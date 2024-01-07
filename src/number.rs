use std::fmt::{self, Display, Formatter};

/// The various Doom specifications are sometimes inconsistent about the representations of numbers.
/// For example, VERTEXES in the original WAD format are 2-byte integers, but in UDMF they're floats (although in practice integers work too).
/// This type allows one to interoperate the various formats without losing precision.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum Number {
    Int(i32),
    Float(f64),
}

impl Number {
    pub fn as_int(self) -> Option<i32> {
        match self {
            Self::Int(i) => Some(i),
            Self::Float(_) => None,
        }
    }

    pub fn as_float(self) -> Option<f64> {
        match self {
            Self::Int(_) => None,
            Self::Float(f) => Some(f),
        }
    }

    pub fn into_int(self) -> i32 {
        match self {
            Self::Int(i) => i,
            Self::Float(f) => f as i32,
        }
    }

    pub fn into_float(self) -> f64 {
        match self {
            Self::Int(i) => i as f64,
            Self::Float(f) => f,
        }
    }

    pub fn is_zero(self) -> bool {
        match self {
            Number::Int(i) => i == 0,
            Number::Float(f) => f == 0.0,
        }
    }
}

impl Default for Number {
    fn default() -> Self {
        Self::Int(0)
    }
}

impl From<i32> for Number {
    fn from(i: i32) -> Self {
        Self::Int(i)
    }
}

impl From<f64> for Number {
    fn from(f: f64) -> Self {
        Self::Float(f)
    }
}

impl Display for Number {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(i) => write!(formatter, "{i}"),
            Self::Float(f) => write!(formatter, "{f}"),
        }
    }
}
