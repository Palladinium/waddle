use crate::String8;
use generational_arena::Arena;

pub mod line_def;
pub mod sector;
pub mod side_def;
pub mod thing;
pub mod udmf;
pub mod vertex;

pub use self::{
    line_def::LineDef, sector::Sector, side_def::SideDef, thing::Thing, vertex::Vertex,
};

#[derive(Debug)]
pub struct Map {
    pub name: String8,
    pub linedefs: Arena<LineDef>,
    pub sectors: Arena<Sector>,
    pub things: Arena<Thing>,
}

impl Map {
    pub fn new(name: String8) -> Self {
        Self {
            name,
            linedefs: Arena::new(),
            sectors: Arena::new(),
            things: Arena::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_bitfields() {
        let range = i16::min_value()..=i16::max_value();
        assert_eq!(range.len(), 2_usize.pow(16));

        for n in range {
            let cast_u16: u16 = n as u16;
            let ptr_u16: u16 = unsafe { *(&n as *const i16 as *const u16) };

            assert_eq!(
                cast_u16, ptr_u16,
                "Casts for {} don't line up: {}, {}",
                n, cast_u16, ptr_u16
            );
        }
    }
}
