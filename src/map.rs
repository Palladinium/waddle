use std::fmt::{self, Display, Formatter};

use slotmap::SecondaryMap;

use crate::String8;

pub mod line_def;
pub mod sector;
pub mod side_def;
pub mod thing;
pub mod udmf;
pub mod vertex;

pub use self::{
    line_def::LineDef, sector::Sector, side_def::SideDef, thing::Thing, vertex::Vertex,
};

use self::{
    line_def::{LineDefMap, RawLineDef},
    sector::SectorMap,
    side_def::{RawSideDef, SideDefMap},
    thing::ThingMap,
    vertex::VertexMap,
};

/// A Doom map, with all entities stored as flat `Vec`s and all references to entities stored as indices.
/// This is very close to the raw representation of a map in a file, but any insertions/deletions require shifting
/// all subsequent indices (and all references to those indices), so it's generally not very ergonomic to modify.
///
/// You can use [RawMap::link] to validate all indices and convert this to a `Map`, which is easier to work with.
#[derive(Debug)]
pub struct RawMap {
    pub name: String8,

    pub vertexes: Vec<Vertex>,
    pub line_defs: Vec<RawLineDef>,
    pub sectors: Vec<Sector>,
    pub side_defs: Vec<RawSideDef>,
    pub things: Vec<Thing>,
}

impl RawMap {
    pub fn link(&self) -> Result<Map, LinkError> {
        let mut vertexes = VertexMap::with_key();
        let mut line_defs = LineDefMap::with_key();
        let mut sectors = SectorMap::with_key();
        let mut side_defs = SideDefMap::with_key();
        let mut things = ThingMap::with_key();

        let vertex_map: Vec<_> = self
            .vertexes
            .iter()
            .map(|vertex| vertexes.insert(*vertex))
            .collect();

        let sector_map: Vec<_> = self
            .sectors
            .iter()
            .map(|sector| sectors.insert(sector.clone()))
            .collect();

        let side_map: Vec<_> = self
            .side_defs
            .iter()
            .enumerate()
            .map(|(i, side_def)| {
                Ok(side_defs.insert(SideDef {
                    sector: *sector_map.get(usize::from(side_def.sector_idx)).ok_or(
                        LinkError::IndexOutOfRange {
                            referrer: EntityKind::SideDef,
                            referrer_index: i,
                            field: "sector",
                            referee: EntityKind::Sector,
                            referee_index: side_def.sector_idx,
                        },
                    )?,
                    offset: side_def.offset,
                    upper_texture: side_def.upper_texture.clone(),
                    middle_texture: side_def.middle_texture.clone(),
                    lower_texture: side_def.lower_texture.clone(),
                }))
            })
            .collect::<Result<_, _>>()?;

        for (i, line_def) in self.line_defs.iter().enumerate() {
            line_defs.insert(LineDef {
                from: *vertex_map.get(usize::from(line_def.from_idx)).ok_or(
                    LinkError::IndexOutOfRange {
                        referrer: EntityKind::LineDef,
                        referrer_index: i,
                        field: "from",
                        referee: EntityKind::Vertex,
                        referee_index: line_def.from_idx,
                    },
                )?,

                to: *vertex_map.get(usize::from(line_def.to_idx)).ok_or(
                    LinkError::IndexOutOfRange {
                        referrer: EntityKind::LineDef,
                        referrer_index: i,
                        field: "to",
                        referee: EntityKind::Vertex,
                        referee_index: line_def.to_idx,
                    },
                )?,

                left_side: *side_map.get(usize::from(line_def.left_side_idx)).ok_or(
                    LinkError::IndexOutOfRange {
                        referrer: EntityKind::LineDef,
                        referrer_index: i,
                        field: "left_side",
                        referee: EntityKind::SideDef,
                        referee_index: line_def.left_side_idx,
                    },
                )?,

                right_side: line_def
                    .right_side_idx
                    .map(|right_side_idx| {
                        side_map
                            .get(usize::from(right_side_idx))
                            .ok_or(LinkError::IndexOutOfRange {
                                referrer: EntityKind::LineDef,
                                referrer_index: i,
                                field: "right_side",
                                referee: EntityKind::SideDef,
                                referee_index: right_side_idx,
                            })
                            .copied()
                    })
                    .transpose()?,

                flags: line_def.flags.clone(),
                special: line_def.special.clone(),
                trigger_flags: line_def.trigger_flags.clone(),
            });
        }

        for thing in self.things.iter() {
            things.insert(thing.clone());
        }

        Ok(Map {
            name: self.name.clone(),
            vertexes,
            line_defs,
            sectors,
            side_defs,
            things,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EntityKind {
    Vertex,
    LineDef,
    Sector,
    SideDef,
    Thing,
}

impl Display for EntityKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let s = match self {
            EntityKind::Vertex => "vertex",
            EntityKind::LineDef => "line_def",
            EntityKind::Sector => "sector",
            EntityKind::SideDef => "side_def",
            EntityKind::Thing => "thing",
        };

        f.write_str(s)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LinkError {
    #[error(
        "{referrer}[{referrer_index}].{field} refers to invalid {referee} index {referee_index}"
    )]
    IndexOutOfRange {
        referrer: EntityKind,
        referrer_index: usize,
        field: &'static str,
        referee: EntityKind,
        referee_index: u16,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum UnlinkError {
    #[error("{referrer}[{referrer_index}].{field} refers to invalid {referee} key")]
    InvalidKey {
        referrer: EntityKind,
        referrer_index: usize,
        field: &'static str,
        referee: EntityKind,
    },

    #[error("Too many {entity_kind} to fit in an u16")]
    IndexTooLarge { entity_kind: EntityKind },
}

#[derive(Debug)]
pub struct Map {
    pub name: String8,

    pub vertexes: VertexMap,
    pub line_defs: LineDefMap,
    pub sectors: SectorMap,
    pub side_defs: SideDefMap,
    pub things: ThingMap,
}

impl Map {
    pub fn new(name: String8) -> Self {
        Self {
            name,
            vertexes: VertexMap::with_key(),
            line_defs: LineDefMap::with_key(),
            sectors: SectorMap::with_key(),
            side_defs: SideDefMap::with_key(),
            things: ThingMap::with_key(),
        }
    }

    pub fn unlink(&self) -> Result<RawMap, UnlinkError> {
        if self.vertexes.len() > u16::MAX.into() {
            return Err(UnlinkError::IndexTooLarge {
                entity_kind: EntityKind::Vertex,
            });
        }

        if self.line_defs.len() > u16::MAX.into() {
            return Err(UnlinkError::IndexTooLarge {
                entity_kind: EntityKind::LineDef,
            });
        }

        if self.sectors.len() > u16::MAX.into() {
            return Err(UnlinkError::IndexTooLarge {
                entity_kind: EntityKind::Sector,
            });
        }

        if self.side_defs.len() > u16::MAX.into() {
            return Err(UnlinkError::IndexTooLarge {
                entity_kind: EntityKind::SideDef,
            });
        }

        if self.things.len() > u16::MAX.into() {
            return Err(UnlinkError::IndexTooLarge {
                entity_kind: EntityKind::Thing,
            });
        }

        let mut vertex_idx_map = SecondaryMap::with_capacity(self.vertexes.len());
        let mut vertexes = Vec::with_capacity(self.vertexes.len());

        for (i, (vertex_key, vertex)) in self.vertexes.iter().enumerate() {
            vertex_idx_map.insert(vertex_key, i as u16);
            vertexes.push(*vertex);
        }

        let mut sector_idx_map = SecondaryMap::with_capacity(self.sectors.len());
        let mut sectors = Vec::with_capacity(self.sectors.len());

        for (i, (sector_key, sector)) in self.sectors.iter().enumerate() {
            sector_idx_map.insert(sector_key, i as u16);
            sectors.push(sector.clone());
        }

        let mut side_def_idx_map = SecondaryMap::with_capacity(self.side_defs.len());
        let mut side_defs = Vec::with_capacity(self.side_defs.len());

        for (i, (side_def_key, side_def)) in self.side_defs.iter().enumerate() {
            side_def_idx_map.insert(side_def_key, i as u16);

            side_defs.push(RawSideDef {
                sector_idx: *sector_idx_map.get(side_def.sector).ok_or(
                    UnlinkError::InvalidKey {
                        referrer: EntityKind::SideDef,
                        referrer_index: i,
                        field: "sector",
                        referee: EntityKind::Sector,
                    },
                )?,

                offset: side_def.offset,
                upper_texture: side_def.upper_texture.clone(),
                middle_texture: side_def.middle_texture.clone(),
                lower_texture: side_def.lower_texture.clone(),
            });
        }

        let line_defs: Vec<_> = self
            .line_defs
            .values()
            .enumerate()
            .map(|(i, line_def)| {
                Ok(RawLineDef {
                    from_idx: *vertex_idx_map.get(line_def.from).ok_or(
                        UnlinkError::InvalidKey {
                            referrer: EntityKind::LineDef,
                            referrer_index: i,
                            field: "from",
                            referee: EntityKind::Vertex,
                        },
                    )?,

                    to_idx: *vertex_idx_map
                        .get(line_def.to)
                        .ok_or(UnlinkError::InvalidKey {
                            referrer: EntityKind::LineDef,
                            referrer_index: i,
                            field: "to",
                            referee: EntityKind::Vertex,
                        })?,

                    left_side_idx: *side_def_idx_map.get(line_def.left_side).ok_or(
                        UnlinkError::InvalidKey {
                            referrer: EntityKind::LineDef,
                            referrer_index: i,
                            field: "left_side",
                            referee: EntityKind::SideDef,
                        },
                    )?,

                    right_side_idx: line_def
                        .right_side
                        .map(|right_side| {
                            side_def_idx_map
                                .get(right_side)
                                .ok_or(UnlinkError::InvalidKey {
                                    referrer: EntityKind::LineDef,
                                    referrer_index: i,
                                    field: "right_side",
                                    referee: EntityKind::SideDef,
                                })
                                .copied()
                        })
                        .transpose()?,

                    flags: line_def.flags.clone(),
                    special: line_def.special.clone(),
                    trigger_flags: line_def.trigger_flags.clone(),
                })
            })
            .collect::<Result<_, _>>()?;

        let things: Vec<_> = self.things.values().cloned().collect();

        Ok(RawMap {
            name: self.name.clone(),
            vertexes,
            line_defs,
            sectors,
            side_defs,
            things,
        })
    }
}

// TODO: Do I need these?
//impl PartialEq for Map {
//    fn eq(&self, rhs: &Self) -> bool {
//        self.name == rhs.name
//            && itertools::equal(self.linedefs(), rhs.linedefs.iter())
//            && itertools::equal(self.sectors.iter(), rhs.sectors.iter())
//            && itertools::equal(self.things.iter(), rhs.things.iter())
//    }
//}
//
//impl Eq for Map {}

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
