use std::convert::TryFrom;

use crate::{map::SideDef, util::RcRC, String8};

#[derive(Default, PartialEq, Eq, Debug)]
pub struct Sector {
    pub sides: Vec<RcRC<SideDef>>,

    pub floor_height: i16,
    pub ceiling_height: i16,
    pub floor_flat: String8,
    pub ceiling_flat: String8,
    pub light_level: u8,
    pub special: Special,
    pub tag: i16,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Special {
    None,
}

impl From<Special> for i16 {
    fn from(special: Special) -> Self {
        match special {
            Special::None => 0,
        }
    }
}

impl TryFrom<i16> for Special {
    type Error = i16;

    fn try_from(n: i16) -> Result<Self, Self::Error> {
        match n {
            0 => Ok(Special::None),
            _ => Err(n),
        }
    }
}

impl Default for Special {
    fn default() -> Self {
        Special::None
    }
}
