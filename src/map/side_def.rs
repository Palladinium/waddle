use slotmap::SlotMap;

use crate::{map::sector::SectorKey, Point, String8};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct RawSideDef {
    pub sector_idx: u16,

    pub offset: Point<i16>,
    pub upper_texture: String8,
    pub middle_texture: String8,
    pub lower_texture: String8,
}

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct SideDef {
    pub sector: SectorKey,

    pub offset: Point<i16>,
    pub upper_texture: String8,
    pub middle_texture: String8,
    pub lower_texture: String8,
}

slotmap::new_key_type! { pub struct SideDefKey; }

pub type SideDefMap = SlotMap<SideDefKey, SideDef>;
