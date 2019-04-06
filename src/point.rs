#[derive(Default, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct Point {
    x: i16,
    y: i16,
}

impl Point {
    pub fn new(x: i16, y: i16) -> Self {
        Self { x, y }
    }
}
