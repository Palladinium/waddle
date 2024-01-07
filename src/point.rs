use crate::number::Number;

#[derive(Default, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct Point<T = Number> {
    pub x: T,
    pub y: T,
}

impl<T> Point<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}
