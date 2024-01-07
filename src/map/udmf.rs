use std::{
    convert::TryInto,
    fmt::{self, Display, Formatter},
    io::{self, Read, Write},
    ops::{Range, RangeInclusive},
};

use miette::Diagnostic;
use winnow::Located;

pub mod ast;
mod consts;
mod parse;

use crate::{
    map::{line_def::RawLineDef, side_def::RawSideDef, *},
    number::Number,
    point::Point,
    string8::{IntoString8Error, String8},
};

use self::ast::GlobalExpr;

#[derive(Clone, Debug)]
pub struct Identifier(String);

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}

#[derive(Debug, thiserror::Error, Diagnostic)]
pub enum LoadError {
    #[error("Parse error: {0}")]
    Parse(winnow::error::ContextError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Compile(#[from] Box<CompileError>),

    #[error(transparent)]
    Link(#[from] LinkError),
}

#[derive(Debug, thiserror::Error, Diagnostic)]
pub enum CompileError {
    #[error("Invalid string8: {error}")]
    String8 {
        #[source]
        error: IntoString8Error,
        #[label("This string8 is invalid")]
        span: Range<usize>,
    },

    #[error("{identifier} was assigned to multiple times")]
    MultipleAssignment {
        identifier: Identifier,
        #[label("{identifier} was previously assigned here...")]
        previous_span: Range<usize>,
        #[label("... and later assigned again here")]
        span: Range<usize>,
    },

    #[error("{identifier} was assigned a value of the wrong type")]
    InvalidAssignmentType {
        identifier: Identifier,
        value: Value,
        expected: ValidValueTypes,
        #[label("{identifier} expects {expected}...")]
        identifier_span: Range<usize>,
        #[label("...but {value} was assigned to it")]
        value_span: Range<usize>,
    },

    #[error("{identifier} must be in the range {range:?}")]
    OutOfRange {
        identifier: Identifier,
        range: RangeInclusive<i32>,
        #[label("This value is out of range")]
        span: Range<usize>,
    },

    #[error("{identifier} is not a valid assignment here")]
    InvalidAssignment {
        identifier: Identifier,
        valid: ValidIdentifiers,
        #[label("Valid assignments here are {valid}")]
        span: Range<usize>,
    },

    #[error("{identifier} is not a valid block here")]
    InvalidBlock {
        identifier: Identifier,
        valid: ValidIdentifiers,
        #[label("Valid blocks here are {valid}")]
        span: Range<usize>,
    },

    #[error("Some required assignments were missing")]
    MissingAssignments {
        missing: MissingAssignments,
        #[label("{missing}")]
        span: Range<usize>,
    },

    /// The args must be tuples since Range does not impl Copy
    #[error("{value} is not a recognized linedef/thing special")]
    LineDefSpecial {
        value: i16,
        #[label("This linedef special")]
        special_span: Range<usize>,

        #[label("... and this argument")]
        arg0_span: Option<(usize, usize)>,
        #[label("... and this argument")]
        arg1_span: Option<(usize, usize)>,
        #[label("... and this argument")]
        arg2_span: Option<(usize, usize)>,
        #[label("... and this argument")]
        arg3_span: Option<(usize, usize)>,
        #[label("... and this argument")]
        arg4_span: Option<(usize, usize)>,
    },

    #[error("{value} is not a recognized sector special")]
    SectorSpecial {
        value: i16,
        #[label("This sector special is invalid")]
        span: Range<usize>,
    },
}

#[derive(Debug)]
pub struct ValidIdentifiers(&'static [&'static str]);

impl Display for ValidIdentifiers {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Some((first, rest)) = self.0.split_first() else {
            return f.write_str("<none>");
        };

        f.write_str(first)?;

        for item in rest {
            write!(f, ", {item}")?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct MissingAssignments(Vec<&'static str>);

impl Display for MissingAssignments {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Some((first, rest)) = self.0.split_first() else {
            return f.write_str("none???");
        };

        f.write_str(first)?;

        for item in rest {
            write!(f, ", {item}")?;
        }

        f.write_str(" were not assigned")?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct ValidValueTypes(&'static [ValueType]);

impl Display for ValidValueTypes {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Some((first, rest)) = self.0.split_first() else {
            return f.write_str("none???");
        };

        if let Some((last, rest)) = rest.split_last() {
            write!(f, "one of {first}")?;

            for item in rest {
                write!(f, ", {item}")?;
            }

            write!(f, " or {last}")?;
        } else {
            write!(f, "{first}")?;
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WriteError {
    #[error(transparent)]
    Unlink(#[from] UnlinkError),

    #[error("Invalid UTF-8 in String8")]
    String8Utf8(#[source] std::str::Utf8Error),

    #[error("IO error")]
    Io(#[from] io::Error),
}

/// A map entity which is expressed as a block in UDMF
pub trait UdmfBlock: Sized {
    fn compile(block: &ast::Block) -> Result<Self, Box<CompileError>>;
    fn write<W: UdmfWriter>(&self, writer: &mut W) -> Result<(), WriteError>;
}

impl UdmfBlock for RawLineDef {
    fn compile(block: &ast::Block) -> Result<Self, Box<CompileError>> {
        use consts::line_def::assignments as a;

        let mut from_idx = None;
        let mut to_idx = None;
        let mut left_side_idx = None;
        let mut right_side_idx = None;

        let mut impassable = None;
        let mut blocks_monsters = None;
        let mut two_sided = None;
        let mut upper_unpegged = None;
        let mut lower_unpegged = None;
        let mut secret = None;
        let mut blocks_sound = None;
        let mut not_on_map = None;
        let mut already_on_map = None;

        let mut special = None;
        let mut arg0 = None;
        let mut arg1 = None;
        let mut arg2 = None;
        let mut arg3 = None;
        let mut arg4 = None;

        let mut player_cross = None;
        let mut player_use = None;
        let mut monster_cross = None;
        let mut monster_use = None;
        let mut impact = None;
        let mut player_push = None;
        let mut monster_push = None;
        let mut missile_cross = None;
        let mut repeats = None;
        let mut monster_activate = None;

        let default_flags = line_def::Flags::default();
        let default_trigger_flags = line_def::TriggerFlags::default();

        for assignment in &block.assignments {
            match assignment.item.identifier.item.0.as_str() {
                a::FROM_IDX => assign_once(&mut from_idx, expect_u16_value, assignment)?,
                a::TO_IDX => assign_once(&mut to_idx, expect_u16_value, assignment)?,
                a::LEFT_SIDE_IDX => assign_once(&mut left_side_idx, expect_u16_value, assignment)?,
                a::RIGHT_SIDE_IDX => {
                    assign_once(&mut right_side_idx, expect_u16_value, assignment)?
                }
                a::IMPASSABLE => assign_once(&mut impassable, expect_bool_value, assignment)?,
                a::BLOCKS_MONSTERS => {
                    assign_once(&mut blocks_monsters, expect_bool_value, assignment)?
                }
                a::TWO_SIDED => assign_once(&mut two_sided, expect_bool_value, assignment)?,
                a::UPPER_UNPEGGED => {
                    assign_once(&mut upper_unpegged, expect_bool_value, assignment)?
                }
                a::LOWER_UNPEGGED => {
                    assign_once(&mut lower_unpegged, expect_bool_value, assignment)?
                }
                a::SECRET => assign_once(&mut secret, expect_bool_value, assignment)?,
                a::BLOCKS_SOUND => assign_once(&mut blocks_sound, expect_bool_value, assignment)?,
                a::NOT_ON_MAP => assign_once(&mut not_on_map, expect_bool_value, assignment)?,
                a::ALREADY_ON_MAP => {
                    assign_once(&mut already_on_map, expect_bool_value, assignment)?
                }
                a::SPECIAL => assign_once(&mut special, expect_i16_value, assignment)?,
                a::ARG0 => assign_once(&mut arg0, expect_i16_value, assignment)?,
                a::ARG1 => assign_once(&mut arg1, expect_i16_value, assignment)?,
                a::ARG2 => assign_once(&mut arg2, expect_i16_value, assignment)?,
                a::ARG3 => assign_once(&mut arg3, expect_i16_value, assignment)?,
                a::ARG4 => assign_once(&mut arg4, expect_i16_value, assignment)?,
                a::PLAYER_CROSS => assign_once(&mut player_cross, expect_bool_value, assignment)?,
                a::PLAYER_USE => assign_once(&mut player_use, expect_bool_value, assignment)?,
                a::MONSTER_CROSS => assign_once(&mut monster_cross, expect_bool_value, assignment)?,
                a::MONSTER_USE => assign_once(&mut monster_use, expect_bool_value, assignment)?,
                a::IMPACT => assign_once(&mut impact, expect_bool_value, assignment)?,
                a::PLAYER_PUSH => assign_once(&mut player_push, expect_bool_value, assignment)?,
                a::MONSTER_PUSH => assign_once(&mut monster_push, expect_bool_value, assignment)?,
                a::MISSILE_CROSS => assign_once(&mut missile_cross, expect_bool_value, assignment)?,
                a::REPEATS => assign_once(&mut repeats, expect_bool_value, assignment)?,
                a::MONSTER_ACTIVATE => {
                    assign_once(&mut monster_activate, expect_bool_value, assignment)?
                }

                _ => {
                    return Err(Box::new(CompileError::InvalidAssignment {
                        identifier: assignment.item.identifier.item.clone(),
                        valid: ValidIdentifiers(a::ALL),
                        span: assignment.span.clone(),
                    }))
                }
            }
        }

        let mut missing_assignments = Vec::new();

        if from_idx.is_none() {
            missing_assignments.push(a::FROM_IDX);
        }

        if to_idx.is_none() {
            missing_assignments.push(a::TO_IDX);
        }

        if left_side_idx.is_none() {
            missing_assignments.push(a::LEFT_SIDE_IDX);
        }

        if !missing_assignments.is_empty() {
            return Err(Box::new(CompileError::MissingAssignments {
                missing: MissingAssignments(missing_assignments),
                span: block.identifier.span.clone(),
            }));
        }

        let special = if let Some((value, span)) = special {
            let (arg0, arg0_span) = arg0.unzip();
            let (arg1, arg1_span) = arg1.unzip();
            let (arg2, arg2_span) = arg2.unzip();
            let (arg3, arg3_span) = arg3.unzip();
            let (arg4, arg4_span) = arg4.unzip();

            let udmf_special = line_def::UdmfSpecial {
                value,
                args: [
                    arg0.unwrap_or(0),
                    arg1.unwrap_or(0),
                    arg2.unwrap_or(0),
                    arg3.unwrap_or(0),
                    arg4.unwrap_or(0),
                ],
            };

            line_def::Special::try_from(udmf_special).map_err(|_| {
                Box::new(CompileError::LineDefSpecial {
                    value,
                    special_span: span,
                    arg0_span: arg0_span.map(|r| (r.start, r.end)),
                    arg1_span: arg1_span.map(|r| (r.start, r.end)),
                    arg2_span: arg2_span.map(|r| (r.start, r.end)),
                    arg3_span: arg3_span.map(|r| (r.start, r.end)),
                    arg4_span: arg4_span.map(|r| (r.start, r.end)),
                })
            })?
        } else {
            line_def::Special::None
        };

        Ok(Self {
            from_idx: from_idx.unwrap().0,
            to_idx: to_idx.unwrap().0,
            left_side_idx: left_side_idx.unwrap().0,
            right_side_idx: right_side_idx.map(|v| v.0),

            flags: line_def::Flags {
                impassable: impassable.map(|v| v.0).unwrap_or(default_flags.impassable),
                blocks_monsters: blocks_monsters
                    .map(|v| v.0)
                    .unwrap_or(default_flags.blocks_monsters),
                two_sided: two_sided.map(|v| v.0).unwrap_or(default_flags.two_sided),
                upper_unpegged: upper_unpegged
                    .map(|v| v.0)
                    .unwrap_or(default_flags.upper_unpegged),
                lower_unpegged: lower_unpegged
                    .map(|v| v.0)
                    .unwrap_or(default_flags.lower_unpegged),
                secret: secret.map(|v| v.0).unwrap_or(default_flags.secret),
                blocks_sound: blocks_sound
                    .map(|v| v.0)
                    .unwrap_or(default_flags.blocks_sound),
                not_on_map: not_on_map.map(|v| v.0).unwrap_or(default_flags.not_on_map),
                already_on_map: already_on_map
                    .map(|v| v.0)
                    .unwrap_or(default_flags.already_on_map),
            },

            special,

            trigger_flags: line_def::TriggerFlags {
                player_cross: player_cross
                    .map(|v| v.0)
                    .unwrap_or(default_trigger_flags.player_cross),
                player_use: player_use
                    .map(|v| v.0)
                    .unwrap_or(default_trigger_flags.player_use),
                monster_cross: monster_cross
                    .map(|v| v.0)
                    .unwrap_or(default_trigger_flags.monster_cross),
                monster_use: monster_use
                    .map(|v| v.0)
                    .unwrap_or(default_trigger_flags.monster_use),
                impact: impact.map(|v| v.0).unwrap_or(default_trigger_flags.impact),
                player_push: player_push
                    .map(|v| v.0)
                    .unwrap_or(default_trigger_flags.player_push),
                monster_push: monster_push
                    .map(|v| v.0)
                    .unwrap_or(default_trigger_flags.monster_push),
                missile_cross: missile_cross
                    .map(|v| v.0)
                    .unwrap_or(default_trigger_flags.missile_cross),
                repeats: repeats
                    .map(|v| v.0)
                    .unwrap_or(default_trigger_flags.repeats),
                monsters_activate: monster_activate
                    .map(|v| v.0)
                    .unwrap_or(default_trigger_flags.monsters_activate),
            },
        })
    }

    fn write<W: UdmfWriter>(&self, writer: &mut W) -> Result<(), WriteError> {
        writer.write_block(consts::line_def::BLOCK, |block| {
            block.write_assignment(
                consts::line_def::assignments::FROM_IDX,
                &Value::Int(i32::from(self.from_idx)),
            )?;
            block.write_assignment(
                consts::line_def::assignments::TO_IDX,
                &Value::Int(i32::from(self.to_idx)),
            )?;

            // TODO: The rest of the owl

            Ok(())
        })
    }
}

impl UdmfBlock for RawSideDef {
    fn compile(block: &ast::Block) -> Result<Self, Box<CompileError>> {
        use consts::side_def::assignments as a;

        let mut offset_x = None;
        let mut offset_y = None;
        let mut sector_idx = None;
        let mut upper_texture = None;
        let mut middle_texture = None;
        let mut lower_texture = None;

        for assignment in &block.assignments {
            match assignment.item.identifier.item.0.as_str() {
                a::OFFSET_X => assign_once(&mut offset_x, expect_i16_value, assignment)?,
                a::OFFSET_Y => assign_once(&mut offset_y, expect_i16_value, assignment)?,
                a::SECTOR_IDX => assign_once(&mut sector_idx, expect_u16_value, assignment)?,
                a::UPPER_TEXTURE => assign_once(&mut upper_texture, expect_str8_value, assignment)?,
                a::MIDDLE_TEXTURE => {
                    assign_once(&mut middle_texture, expect_str8_value, assignment)?
                }
                a::LOWER_TEXTURE => assign_once(&mut lower_texture, expect_str8_value, assignment)?,

                _ => {
                    return Err(Box::new(CompileError::InvalidAssignment {
                        identifier: assignment.item.identifier.item.clone(),
                        valid: ValidIdentifiers(a::ALL),
                        span: assignment.span.clone(),
                    }))
                }
            }
        }

        let mut missing_assignments = Vec::new();

        if sector_idx.is_none() {
            missing_assignments.push(a::SECTOR_IDX);
        }

        if !missing_assignments.is_empty() {
            return Err(Box::new(CompileError::MissingAssignments {
                missing: MissingAssignments(missing_assignments),
                span: block.identifier.span.clone(),
            }));
        }

        Ok(Self {
            offset: Point::new(
                offset_x.map(|v| v.0).unwrap_or(0),
                offset_y.map(|v| v.0).unwrap_or(0),
            ),
            sector_idx: sector_idx.unwrap().0,

            upper_texture: upper_texture
                .map(|v| v.0)
                .unwrap_or(String8::new_unchecked(consts::side_def::DEFAULT_TEXTURE)),
            middle_texture: middle_texture
                .map(|v| v.0)
                .unwrap_or(String8::new_unchecked(consts::side_def::DEFAULT_TEXTURE)),
            lower_texture: lower_texture
                .map(|v| v.0)
                .unwrap_or(String8::new_unchecked(consts::side_def::DEFAULT_TEXTURE)),
        })
    }

    fn write<W: UdmfWriter>(&self, writer: &mut W) -> Result<(), WriteError> {
        use consts::side_def::assignments as a;

        writer.write_block(consts::side_def::BLOCK, |block| {
            if self.offset.x != 0 {
                block.write_assignment(a::OFFSET_X, &Value::Int(i32::from(self.offset.x)))?;
            }

            if self.offset.y != 0 {
                block.write_assignment(a::OFFSET_Y, &Value::Int(i32::from(self.offset.y)))?;
            }

            let upper_texture: &str = (&self.upper_texture)
                .try_into()
                .map_err(WriteError::String8Utf8)?;

            if upper_texture != consts::side_def::DEFAULT_TEXTURE {
                block.write_assignment(a::UPPER_TEXTURE, &Value::Str(upper_texture.to_string()))?;
            }

            let middle_texture: &str = (&self.middle_texture)
                .try_into()
                .map_err(WriteError::String8Utf8)?;

            if middle_texture != consts::side_def::DEFAULT_TEXTURE {
                block
                    .write_assignment(a::MIDDLE_TEXTURE, &Value::Str(middle_texture.to_string()))?;
            }

            let lower_texture: &str = (&self.lower_texture)
                .try_into()
                .map_err(WriteError::String8Utf8)?;

            if lower_texture != consts::side_def::DEFAULT_TEXTURE {
                block.write_assignment(a::LOWER_TEXTURE, &Value::Str(lower_texture.to_string()))?;
            }

            Ok(())
        })
    }
}

impl UdmfBlock for Sector {
    fn compile(block: &ast::Block) -> Result<Self, Box<CompileError>> {
        use consts::sector::assignments as a;

        let mut floor_height = None;
        let mut ceiling_height = None;
        let mut floor_flat = None;
        let mut ceiling_flat = None;
        let mut light_level = None;
        let mut special = None;
        let mut tag = None;

        for assignment in &block.assignments {
            match assignment.item.identifier.item.0.as_str() {
                a::FLOOR_HEIGHT => assign_once(&mut floor_height, expect_i16_value, assignment)?,
                a::CEILING_HEIGHT => {
                    assign_once(&mut ceiling_height, expect_i16_value, assignment)?
                }
                a::FLOOR_FLAT => assign_once(&mut floor_flat, expect_str8_value, assignment)?,
                a::CEILING_FLAT => assign_once(&mut ceiling_flat, expect_str8_value, assignment)?,
                a::LIGHT_LEVEL => assign_once(&mut light_level, expect_u8_value, assignment)?,
                a::SPECIAL => assign_once(&mut special, expect_i16_value, assignment)?,
                a::TAG => assign_once(&mut tag, expect_i16_value, assignment)?,

                _ => {
                    return Err(Box::new(CompileError::InvalidAssignment {
                        identifier: assignment.item.identifier.item.clone(),
                        valid: ValidIdentifiers(a::ALL),
                        span: assignment.span.clone(),
                    }))
                }
            }
        }

        let mut missing_assignments = Vec::new();

        if floor_flat.is_none() {
            missing_assignments.push(a::FLOOR_FLAT);
        }

        if ceiling_flat.is_none() {
            missing_assignments.push(a::CEILING_FLAT);
        }

        if !missing_assignments.is_empty() {
            return Err(Box::new(CompileError::MissingAssignments {
                missing: MissingAssignments(missing_assignments),
                span: block.identifier.span.clone(),
            }));
        }

        let special = if let Some((value, span)) = special {
            value
                .try_into()
                .map_err(|_| Box::new(CompileError::SectorSpecial { value, span }))?
        } else {
            sector::Special::None
        };

        Ok(Self {
            floor_height: floor_height.map(|v| v.0).unwrap_or(0),
            ceiling_height: ceiling_height.map(|v| v.0).unwrap_or(0),

            floor_flat: floor_flat.unwrap().0,
            ceiling_flat: ceiling_flat.unwrap().0,

            light_level: light_level
                .map(|v| v.0)
                .unwrap_or(consts::sector::DEFAULT_LIGHT_LEVEL),
            special,
            tag: tag.map(|v| v.0).unwrap_or(0),
        })
    }

    fn write<W: UdmfWriter>(&self, writer: &mut W) -> Result<(), WriteError> {
        use consts::sector::assignments as a;

        writer.write_block(consts::sector::BLOCK, |block| {
            if self.floor_height != 0 {
                block
                    .write_assignment(a::FLOOR_HEIGHT, &Value::Int(i32::from(self.floor_height)))?;
            }
            if self.ceiling_height != 0 {
                block.write_assignment(
                    a::CEILING_HEIGHT,
                    &Value::Int(i32::from(self.ceiling_height)),
                )?;
            }

            block.write_assignment(
                a::FLOOR_FLAT,
                &Value::Str(
                    self.floor_flat
                        .try_as_str()
                        .map_err(WriteError::String8Utf8)?
                        .to_owned(),
                ),
            )?;
            block.write_assignment(
                a::CEILING_FLAT,
                &Value::Str(
                    self.ceiling_flat
                        .try_as_str()
                        .map_err(WriteError::String8Utf8)?
                        .to_owned(),
                ),
            )?;

            if self.light_level != consts::sector::DEFAULT_LIGHT_LEVEL {
                block.write_assignment(a::LIGHT_LEVEL, &Value::Int(i32::from(self.light_level)))?;
            }
            let special: i16 = self.special.into();
            if special != 0 {
                block.write_assignment(a::SPECIAL, &Value::Int(i32::from(special)))?;
            }

            if self.tag != 0 {
                block.write_assignment(a::TAG, &Value::Int(i32::from(self.tag)))?;
            }

            Ok(())
        })
    }
}

impl UdmfBlock for Vertex {
    fn compile(block: &ast::Block) -> Result<Self, Box<CompileError>> {
        use consts::vertex::assignments as a;

        let mut x = None;
        let mut y = None;

        for assignment in &block.assignments {
            match assignment.item.identifier.item.0.as_str() {
                a::X => assign_once(&mut x, expect_number_value, assignment)?,
                a::Y => assign_once(&mut y, expect_number_value, assignment)?,

                _ => {
                    return Err(Box::new(CompileError::InvalidAssignment {
                        identifier: assignment.item.identifier.item.clone(),
                        valid: ValidIdentifiers(a::ALL),
                        span: assignment.span.clone(),
                    }))
                }
            }
        }

        let mut missing_assignments = Vec::new();

        if x.is_none() {
            missing_assignments.push(a::X);
        }

        if y.is_none() {
            missing_assignments.push(a::Y);
        }

        if !missing_assignments.is_empty() {
            return Err(Box::new(CompileError::MissingAssignments {
                missing: MissingAssignments(missing_assignments),
                span: block.identifier.span.clone(),
            }));
        }

        Ok(Self {
            position: Point {
                x: x.unwrap().0,
                y: y.unwrap().0,
            },
        })
    }

    fn write<W: UdmfWriter>(&self, writer: &mut W) -> Result<(), WriteError> {
        use consts::vertex::assignments as a;

        writer.write_block(consts::vertex::BLOCK, |block| {
            block.write_assignment(a::X, &self.position.x.into())?;
            block.write_assignment(a::Y, &self.position.y.into())?;

            Ok(())
        })
    }
}

impl UdmfBlock for Thing {
    fn compile(block: &ast::Block) -> Result<Self, Box<CompileError>> {
        use consts::thing::assignments as a;

        let mut x = None;
        let mut y = None;

        let mut height = None;
        let mut angle = None;
        let mut type_ = None;

        let mut skill1 = None;
        let mut skill2 = None;
        let mut skill3 = None;
        let mut skill4 = None;
        let mut skill5 = None;
        let mut ambush = None;
        let mut single = None;
        let mut dm = None;
        let mut coop = None;
        let mut mbf_friend = None;
        let mut dormant = None;
        let mut class1 = None;
        let mut class2 = None;
        let mut class3 = None;
        let mut npc = None;
        let mut strife_ally = None;
        let mut translucent = None;
        let mut invisible = None;

        // FIXME Special

        let default_flags = thing::Flags::default();

        for assignment in &block.assignments {
            match assignment.item.identifier.item.0.as_str() {
                a::X => assign_once(&mut x, expect_number_value, assignment)?,
                a::Y => assign_once(&mut y, expect_number_value, assignment)?,
                a::ANGLE => assign_once(&mut angle, expect_i16_value, assignment)?,
                a::HEIGHT => assign_once(&mut height, expect_i16_value, assignment)?,
                a::TYPE => assign_once(&mut type_, expect_i16_value, assignment)?,

                a::SKILL1 => assign_once(&mut skill1, expect_bool_value, assignment)?,
                a::SKILL2 => assign_once(&mut skill2, expect_bool_value, assignment)?,
                a::SKILL3 => assign_once(&mut skill3, expect_bool_value, assignment)?,
                a::SKILL4 => assign_once(&mut skill4, expect_bool_value, assignment)?,
                a::SKILL5 => assign_once(&mut skill5, expect_bool_value, assignment)?,

                a::AMBUSH => assign_once(&mut ambush, expect_bool_value, assignment)?,

                a::CLASS1 => assign_once(&mut class1, expect_bool_value, assignment)?,
                a::CLASS2 => assign_once(&mut class2, expect_bool_value, assignment)?,
                a::CLASS3 => assign_once(&mut class3, expect_bool_value, assignment)?,

                a::MBF_FRIEND => assign_once(&mut mbf_friend, expect_bool_value, assignment)?,
                a::DORMANT => assign_once(&mut dormant, expect_bool_value, assignment)?,
                a::COOP => assign_once(&mut coop, expect_bool_value, assignment)?,
                a::DM => assign_once(&mut dm, expect_bool_value, assignment)?,
                a::INVISIBLE => assign_once(&mut invisible, expect_bool_value, assignment)?,
                a::NPC => assign_once(&mut npc, expect_bool_value, assignment)?,
                a::SINGLE => assign_once(&mut single, expect_bool_value, assignment)?,
                a::STRIFE_ALLY => assign_once(&mut strife_ally, expect_bool_value, assignment)?,
                a::TRANSLUCENT => assign_once(&mut translucent, expect_bool_value, assignment)?,

                _ => {
                    return Err(Box::new(CompileError::InvalidAssignment {
                        identifier: assignment.item.identifier.item.clone(),
                        valid: ValidIdentifiers(a::ALL),
                        span: assignment.span.clone(),
                    }))
                }
            }
        }

        let mut missing_assignments = Vec::new();

        if x.is_none() {
            missing_assignments.push(a::X);
        }

        if y.is_none() {
            missing_assignments.push(a::Y);
        }

        if type_.is_none() {
            missing_assignments.push(a::TYPE);
        }

        if !missing_assignments.is_empty() {
            return Err(Box::new(CompileError::MissingAssignments {
                missing: MissingAssignments(missing_assignments),
                span: block.identifier.span.clone(),
            }));
        }

        Ok(Self {
            position: Point {
                x: x.unwrap().0,
                y: y.unwrap().0,
            },

            angle: angle.map(|v| v.0).unwrap_or(0),
            height: height.map(|v| v.0).unwrap_or(0),

            type_: type_.unwrap().0,

            flags: thing::Flags {
                skill1: skill1.map(|v| v.0).unwrap_or(default_flags.skill1),
                skill2: skill2.map(|v| v.0).unwrap_or(default_flags.skill2),
                skill3: skill3.map(|v| v.0).unwrap_or(default_flags.skill3),
                skill4: skill4.map(|v| v.0).unwrap_or(default_flags.skill4),
                skill5: skill5.map(|v| v.0).unwrap_or(default_flags.skill5),

                ambush: ambush.map(|v| v.0).unwrap_or(default_flags.ambush),

                class1: class1.map(|v| v.0).unwrap_or(default_flags.class1),
                class2: class2.map(|v| v.0).unwrap_or(default_flags.class2),
                class3: class3.map(|v| v.0).unwrap_or(default_flags.class3),

                mbf_friend: mbf_friend.map(|v| v.0).unwrap_or(default_flags.mbf_friend),
                dormant: dormant.map(|v| v.0).unwrap_or(default_flags.dormant),
                coop: coop.map(|v| v.0).unwrap_or(default_flags.coop),
                dm: dm.map(|v| v.0).unwrap_or(default_flags.dm),
                invisible: invisible.map(|v| v.0).unwrap_or(default_flags.invisible),

                npc: npc.map(|v| v.0).unwrap_or(default_flags.npc),
                single: single.map(|v| v.0).unwrap_or(default_flags.single),
                strife_ally: strife_ally
                    .map(|v| v.0)
                    .unwrap_or(default_flags.strife_ally),
                translucent: translucent
                    .map(|v| v.0)
                    .unwrap_or(default_flags.translucent),
            },

            special: thing::Special::None,
        })
    }

    fn write<W: UdmfWriter>(&self, writer: &mut W) -> Result<(), WriteError> {
        use consts::thing::assignments as a;

        writer.write_block(consts::thing::BLOCK, |block| {
            if self.height != 0 {
                block.write_assignment(a::HEIGHT, &Value::Int(i32::from(self.height)))?;
            }
            if self.angle != 0 {
                block.write_assignment(a::ANGLE, &Value::Int(i32::from(self.angle)))?;
            }

            block.write_assignment(a::TYPE, &Value::Int(i32::from(self.type_)))?;

            let default_flags = thing::Flags::default();

            if self.flags.skill1 != default_flags.skill1 {
                block.write_assignment(a::SKILL1, &Value::Bool(self.flags.skill1))?;
            }
            if self.flags.skill2 != default_flags.skill2 {
                block.write_assignment(a::SKILL2, &Value::Bool(self.flags.skill2))?;
            }
            if self.flags.skill3 != default_flags.skill3 {
                block.write_assignment(a::SKILL3, &Value::Bool(self.flags.skill3))?;
            }
            if self.flags.skill4 != default_flags.skill4 {
                block.write_assignment(a::SKILL4, &Value::Bool(self.flags.skill4))?;
            }
            if self.flags.skill5 != default_flags.skill5 {
                block.write_assignment(a::SKILL5, &Value::Bool(self.flags.skill5))?;
            }
            if self.flags.ambush != default_flags.ambush {
                block.write_assignment(a::AMBUSH, &Value::Bool(self.flags.ambush))?;
            }
            if self.flags.single != default_flags.single {
                block.write_assignment(a::SINGLE, &Value::Bool(self.flags.single))?;
            }
            if self.flags.dm != default_flags.dm {
                block.write_assignment(a::DM, &Value::Bool(self.flags.dm))?;
            }
            if self.flags.coop != default_flags.coop {
                block.write_assignment(a::COOP, &Value::Bool(self.flags.coop))?;
            }
            if self.flags.mbf_friend != default_flags.mbf_friend {
                block.write_assignment(a::MBF_FRIEND, &Value::Bool(self.flags.mbf_friend))?;
            }
            if self.flags.class1 != default_flags.class1 {
                block.write_assignment(a::CLASS1, &Value::Bool(self.flags.class1))?;
            }
            if self.flags.class2 != default_flags.class2 {
                block.write_assignment(a::CLASS2, &Value::Bool(self.flags.class2))?;
            }
            if self.flags.class3 != default_flags.class3 {
                block.write_assignment(a::CLASS3, &Value::Bool(self.flags.class3))?;
            }
            if self.flags.dormant != default_flags.dormant {
                block.write_assignment(a::DORMANT, &Value::Bool(self.flags.dormant))?;
            }
            if self.flags.invisible != default_flags.invisible {
                block.write_assignment(a::INVISIBLE, &Value::Bool(self.flags.invisible))?;
            }
            if self.flags.npc != default_flags.npc {
                block.write_assignment(a::NPC, &Value::Bool(self.flags.npc))?;
            }
            if self.flags.translucent != default_flags.translucent {
                block.write_assignment(a::TRANSLUCENT, &Value::Bool(self.flags.translucent))?;
            }
            if self.flags.strife_ally != default_flags.strife_ally {
                block.write_assignment(a::STRIFE_ALLY, &Value::Bool(self.flags.strife_ally))?;
            }

            Ok(())
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ValueType {
    Int,
    Float,
    Str,
    Bool,
}

impl Display for ValueType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            ValueType::Int => "integer",
            ValueType::Float => "float",
            ValueType::Str => "string",
            ValueType::Bool => "boolean",
        };

        f.write_str(s)
    }
}

// TODO: Move to AST?
#[derive(Clone, Debug)]
pub enum Value {
    Int(i32),
    Float(f64),
    Str(String),
    Bool(bool),
}

impl From<Number> for Value {
    fn from(n: Number) -> Self {
        match n {
            Number::Int(i) => Self::Int(i),
            Number::Float(f) => Self::Float(f),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Int(v) => write!(f, "{}", v),
            Value::Float(v) => write!(f, "{}", v),
            Value::Str(v) => write!(f, "\"{}\"", v),
            Value::Bool(v) => write!(f, "{}", v),
        }
    }
}

fn expect_u16_value(
    assignment: &ast::Spanned<ast::AssignmentExpr>,
) -> Result<u16, Box<CompileError>> {
    let n = expect_int_value(assignment)?;

    u16::try_from(n).map_err(|_| {
        Box::new(CompileError::OutOfRange {
            identifier: assignment.item.identifier.item.clone(),
            range: i32::from(u16::MIN)..=i32::from(u16::MAX),
            span: assignment.item.value.span.clone(),
        })
    })
}

fn expect_i16_value(
    assignment: &ast::Spanned<ast::AssignmentExpr>,
) -> Result<i16, Box<CompileError>> {
    let n = expect_int_value(assignment)?;

    i16::try_from(n).map_err(|_| {
        Box::new(CompileError::OutOfRange {
            identifier: assignment.item.identifier.item.clone(),
            range: i32::from(i16::MIN)..=i32::from(i16::MAX),
            span: assignment.item.value.span.clone(),
        })
    })
}

fn expect_u8_value(
    assignment: &ast::Spanned<ast::AssignmentExpr>,
) -> Result<u8, Box<CompileError>> {
    let n = expect_int_value(assignment)?;

    u8::try_from(n).map_err(|_| {
        Box::new(CompileError::OutOfRange {
            identifier: assignment.item.identifier.item.clone(),
            range: i32::from(u8::MIN)..=i32::from(u8::MAX),
            span: assignment.item.value.span.clone(),
        })
    })
}

fn expect_int_value(
    assignment: &ast::Spanned<ast::AssignmentExpr>,
) -> Result<i32, Box<CompileError>> {
    if let Value::Int(value) = &assignment.item.value.item {
        Ok(*value)
    } else {
        Err(Box::new(CompileError::InvalidAssignmentType {
            identifier: assignment.item.identifier.item.clone(),
            value: assignment.item.value.item.clone(),
            expected: ValidValueTypes(&[ValueType::Int]),
            identifier_span: assignment.item.identifier.span.clone(),
            value_span: assignment.item.value.span.clone(),
        }))
    }
}

fn expect_bool_value(
    assignment: &ast::Spanned<ast::AssignmentExpr>,
) -> Result<bool, Box<CompileError>> {
    if let Value::Bool(value) = &assignment.item.value.item {
        Ok(*value)
    } else {
        Err(Box::new(CompileError::InvalidAssignmentType {
            identifier: assignment.item.identifier.item.clone(),
            value: assignment.item.value.item.clone(),
            expected: ValidValueTypes(&[ValueType::Bool]),
            identifier_span: assignment.item.identifier.span.clone(),
            value_span: assignment.item.value.span.clone(),
        }))
    }
}

fn expect_str_value(
    assignment: &ast::Spanned<ast::AssignmentExpr>,
) -> Result<String, Box<CompileError>> {
    if let Value::Str(value) = &assignment.item.value.item {
        Ok(value.clone())
    } else {
        Err(Box::new(CompileError::InvalidAssignmentType {
            identifier: assignment.item.identifier.item.clone(),
            value: assignment.item.value.item.clone(),
            expected: ValidValueTypes(&[ValueType::Str]),
            identifier_span: assignment.item.identifier.span.clone(),
            value_span: assignment.item.value.span.clone(),
        }))
    }
}

fn expect_str8_value(
    assignment: &ast::Spanned<ast::AssignmentExpr>,
) -> Result<String8, Box<CompileError>> {
    if let Value::Str(value) = &assignment.item.value.item {
        String8::new(value).map_err(|e| {
            Box::new(CompileError::String8 {
                error: e,
                span: assignment.item.value.span.clone(),
            })
        })
    } else {
        Err(Box::new(CompileError::InvalidAssignmentType {
            identifier: assignment.item.identifier.item.clone(),
            value: assignment.item.value.item.clone(),
            expected: ValidValueTypes(&[ValueType::Str]),
            identifier_span: assignment.item.identifier.span.clone(),
            value_span: assignment.item.value.span.clone(),
        }))
    }
}

fn expect_number_value(
    assignment: &ast::Spanned<ast::AssignmentExpr>,
) -> Result<Number, Box<CompileError>> {
    match &assignment.item.value.item {
        Value::Int(i) => Ok(Number::Int(*i)),
        Value::Float(f) => Ok(Number::Float(*f)),
        _ => Err(Box::new(CompileError::InvalidAssignmentType {
            identifier: assignment.item.identifier.item.clone(),
            value: assignment.item.value.item.clone(),
            expected: ValidValueTypes(&[ValueType::Int, ValueType::Float]),
            identifier_span: assignment.item.identifier.span.clone(),
            value_span: assignment.item.value.span.clone(),
        })),
    }
}

fn assign_once<T, F>(
    opt: &mut Option<(T, Range<usize>)>,
    expect: F,
    assignment: &ast::Spanned<ast::AssignmentExpr>,
) -> Result<(), Box<CompileError>>
where
    F: Fn(&ast::Spanned<ast::AssignmentExpr>) -> Result<T, Box<CompileError>>,
{
    if let Some((_, previous_span)) = opt {
        Err(Box::new(CompileError::MultipleAssignment {
            identifier: assignment.item.identifier.item.clone(),
            previous_span: previous_span.clone(),
            span: assignment.span.clone(),
        }))
    } else {
        let value = expect(assignment)?;
        *opt = Some((value, assignment.span.clone()));
        Ok(())
    }
}

// TODO: Rewrite this to take ast types
pub trait UdmfWriter: Sized {
    type Writer: Write;
    fn writer(&mut self) -> &mut Self::Writer;

    fn indent(&self) -> usize;

    fn write_comment(&mut self, text: &str) -> Result<(), WriteError> {
        let indent = self.indent();
        writeln!(self.writer(), "{:2$}//{}", "", text, indent)?;
        Ok(())
    }

    fn write_blank_line(&mut self) -> Result<(), WriteError> {
        writeln!(self.writer())?;
        Ok(())
    }

    fn write_assignment(&mut self, key: &str, value: &Value) -> Result<(), WriteError> {
        let indent = self.indent();
        writeln!(self.writer(), "{:3$}{}={};", "", key, value, indent)?;
        Ok(())
    }

    fn write_block<F, E>(&mut self, key: &str, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut UdmfBlockWriter<Self>) -> Result<(), E>,
        E: From<WriteError>,
    {
        let mut block_writer = UdmfBlockWriter(self);
        block_writer.start(key)?;
        f(&mut block_writer)?;
        block_writer.end()?;

        Ok(())
    }
}

pub struct UdmfBlockWriter<'w, W>(&'w mut W);

impl<'w, W: UdmfWriter> UdmfBlockWriter<'w, W> {
    fn start(&mut self, key: &str) -> Result<(), WriteError> {
        let indent = self.0.indent();
        writeln!(self.0.writer(), "{:2$}{} {{", "", key, indent)?;
        Ok(())
    }

    fn end(&mut self) -> Result<(), WriteError> {
        let indent = self.0.indent();
        writeln!(self.0.writer(), "{:1$}}}", "", indent)?;
        Ok(())
    }
}

impl<'w, W: UdmfWriter> UdmfWriter for UdmfBlockWriter<'w, W> {
    type Writer = W::Writer;

    fn writer(&mut self) -> &mut Self::Writer {
        self.0.writer()
    }

    fn indent(&self) -> usize {
        self.0.indent() + 2
    }
}

impl<W: Write> UdmfWriter for W {
    type Writer = Self;

    fn writer(&mut self) -> &mut Self::Writer {
        self
    }

    fn indent(&self) -> usize {
        0
    }
}

impl Map {
    pub fn write_udmf_textmap<W: Write>(&self, writer: &mut W) -> Result<(), WriteError> {
        let raw_map = self.unlink()?;

        writer.write_comment(&format!(
            "Written by {} v{}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        ))?;

        writer.write_assignment("namespace", &Value::Str("zdoom".to_string()))?;

        writer.write_comment("Vertexes")?;
        for (i, vertex) in raw_map.vertexes.iter().enumerate() {
            writer.write_comment(&format!("#{}", i))?;
            vertex.write(writer)?;
            writer.write_blank_line()?;
        }

        writer.write_comment("Line Defs")?;
        for (i, line_def) in raw_map.line_defs.iter().enumerate() {
            writer.write_comment(&format!("#{}", i))?;
            line_def.write(writer)?;
            writer.write_blank_line()?;
        }

        writer.write_comment("Sectors")?;
        for (i, sector) in raw_map.sectors.iter().enumerate() {
            writer.write_comment(&format!("#{}", i))?;
            sector.write(writer)?;
            writer.write_blank_line()?;
        }

        writer.write_comment("Side Defs")?;
        for (i, side_def) in raw_map.side_defs.iter().enumerate() {
            writer.write_comment(&format!("#{}", i))?;
            side_def.write(writer)?;
            writer.write_blank_line()?;
        }

        writer.write_comment("Things")?;
        for (i, thing) in raw_map.things.iter().enumerate() {
            writer.write_comment(&format!("#{}", i))?;
            thing.write(writer)?;
            writer.write_blank_line()?;
        }

        Ok(())
    }

    pub fn load_udmf_textmap<R: Read>(name: String8, contents: &str) -> Result<Self, LoadError> {
        let translation_unit =
            parse::parse_translation_unit(&mut Located::new(contents)).map_err(|e| {
                LoadError::Parse(e.into_inner().expect("Incomplete parse error not expected"))
            })?;
        let raw_map = compile_udmf_translation_unit(&translation_unit, name)?;
        let map = raw_map.link()?;

        Ok(map)
    }
}

fn compile_udmf_translation_unit(
    translation_unit: &ast::TranslationUnit,
    name: String8,
) -> Result<RawMap, Box<CompileError>> {
    use consts::global::assignments as a;

    let mut namespace = None;

    let mut vertexes: Vec<Vertex> = Vec::new();
    let mut line_defs: Vec<RawLineDef> = Vec::new();
    let mut side_defs: Vec<RawSideDef> = Vec::new();
    let mut sectors: Vec<Sector> = Vec::new();
    let mut things: Vec<Thing> = Vec::new();

    for global_expression in &translation_unit.expressions {
        match global_expression {
            GlobalExpr::AssignmentExpr(assignment) => {
                match assignment.item.identifier.item.0.as_str() {
                    a::NAMESPACE => assign_once(&mut namespace, expect_str_value, assignment)?,

                    _ => {
                        return Err(Box::new(CompileError::InvalidAssignment {
                            identifier: assignment.item.identifier.item.clone(),
                            valid: ValidIdentifiers(a::ALL),
                            span: assignment.span.clone(),
                        }))
                    }
                }
            }

            GlobalExpr::Block(block) => match block.item.identifier.item.0.as_str() {
                consts::vertex::BLOCK => vertexes.push(Vertex::compile(&block.item)?),
                consts::line_def::BLOCK => line_defs.push(RawLineDef::compile(&block.item)?),
                consts::sector::BLOCK => sectors.push(Sector::compile(&block.item)?),
                consts::side_def::BLOCK => side_defs.push(RawSideDef::compile(&block.item)?),
                consts::thing::BLOCK => things.push(Thing::compile(&block.item)?),

                _ => {
                    return Err(Box::new(CompileError::InvalidBlock {
                        identifier: block.item.identifier.item.clone(),
                        valid: ValidIdentifiers(consts::global::BLOCKS),
                        span: block.item.identifier.span.clone(),
                    }))
                }
            },
        }
    }

    Ok(RawMap {
        name,
        vertexes,
        line_defs,
        side_defs,
        sectors,
        things,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{convert::TryInto, io::Cursor};

    use pretty_assertions::assert_eq;

    #[test]
    fn udmf_parsing() {
        let s = include_str!("udmf_test.txt");

        let result =
            Map::load_udmf_textmap("foo".try_into().unwrap(), &mut Cursor::new(s)).unwrap();

        let mut expected = Map::new("foo".try_into().unwrap());

        let vertices: Vec<_> = [
            Point::new(-96, 32),
            Point::new(64, -64),
            Point::new(128, 64),
            Point::new(-64, 96),
        ]
        .iter()
        .map(|&position| Rc::new(RefCell::new(Vertex { position })))
        .collect();

        let sidedefs = vec![
            Rc::new(RefCell::new(SideDef {
                upper_texture: String8::new_unchecked("-"),
                middle_texture: String8::new_unchecked("STONE2"),
                lower_texture: String8::new_unchecked("-"),
                offset: Point::new(0, 0),
            }));
            4
        ];

        expected.linedefs.insert(LineDef {
            from: vertices[1].clone(),
            to: vertices[0].clone(),
            left_side: sidedefs[0].clone(),
            right_side: None,
            special: line_def::Special::default(),
            flags: line_def::Flags {
                impassable: true,
                ..line_def::Flags::default()
            },
            trigger_flags: line_def::TriggerFlags::default(),
        });
        expected.linedefs.insert(LineDef {
            from: vertices[2].clone(),
            to: vertices[1].clone(),
            left_side: sidedefs[3].clone(),
            right_side: None,
            special: line_def::Special::default(),
            flags: line_def::Flags {
                impassable: true,
                ..line_def::Flags::default()
            },
            trigger_flags: line_def::TriggerFlags::default(),
        });
        expected.linedefs.insert(LineDef {
            from: vertices[3].clone(),
            to: vertices[2].clone(),
            left_side: sidedefs[2].clone(),
            right_side: None,
            special: line_def::Special::default(),
            flags: line_def::Flags {
                impassable: true,
                ..line_def::Flags::default()
            },
            trigger_flags: line_def::TriggerFlags::default(),
        });
        expected.linedefs.insert(LineDef {
            from: vertices[0].clone(),
            to: vertices[3].clone(),
            left_side: sidedefs[1].clone(),
            right_side: None,
            special: line_def::Special::default(),
            flags: line_def::Flags {
                impassable: true,
                ..line_def::Flags::default()
            },
            trigger_flags: line_def::TriggerFlags::default(),
        });

        expected.sectors.insert(Sector {
            sides: sidedefs[0..4].iter().cloned().collect(),
            floor_flat: String8::from_str_unchecked("MFLR8_1"),
            ceiling_flat: String8::from_str_unchecked("MFLR8_1"),
            ceiling_height: 128,
            floor_height: 0,
            light_level: 160,
            special: sector::Special::default(),
            tag: 0,
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn udmf_linedef_specials() {
        for value in i16::min_value()..=i16::max_value() {
            for args_len in 0..5 {
                let mut args = [0; 5];

                for i in 0..args_len {
                    args[i] = 1;
                }

                let udmf_special = line_def::UdmfSpecial::new(value, args);

                let result: std::result::Result<line_def::Special, _> = udmf_special.try_into();

                if let Ok(special) = result {
                    let converted: line_def::UdmfSpecial = special.into();
                    assert_eq!(converted, udmf_special);
                }
            }
        }
    }
}
