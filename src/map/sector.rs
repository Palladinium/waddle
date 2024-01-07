use std::convert::TryFrom;

use slotmap::SlotMap;

use crate::String8;

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct Sector {
    pub floor_height: i16,
    pub ceiling_height: i16,
    pub floor_flat: String8,
    pub ceiling_flat: String8,
    pub light_level: u8,
    pub special: Special,
    pub tag: i16,
}

#[derive(Clone, Copy,Debug, Default, PartialEq, Eq)]
pub enum Special {
    #[default]
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

slotmap::new_key_type! { pub struct SectorKey; }

pub type SectorMap = SlotMap<SectorKey, Sector>;
