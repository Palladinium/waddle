use crate::Point;

#[derive(Default, PartialEq, Eq, Debug, Hash, PartialOrd, Ord, Clone, Copy)]
pub struct Vertex {
    pub position: Point,
}
