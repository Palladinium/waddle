use slotmap::SlotMap;

use crate::Point;

#[derive(Default, PartialEq, Debug, PartialOrd, Clone, Copy)]
pub struct Vertex {
    pub position: Point,
}

slotmap::new_key_type! { pub struct VertexKey; }

pub type VertexMap = SlotMap<VertexKey, Vertex>;
