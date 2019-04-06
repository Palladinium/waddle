use crate::{Point, String8};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct SideDef {
    pub offset: Point,
    pub upper_texture: String8,
    pub middle_texture: String8,
    pub lower_texture: String8,
}
