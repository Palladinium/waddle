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

#[derive(PartialEq, Eq, Debug)]
pub enum Special {
    None,
}

impl Default for Special {
    fn default() -> Self {
        Special::None
    }
}
