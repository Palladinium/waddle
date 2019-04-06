use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    convert::TryInto,
    fmt::{self, Display, Formatter},
    io::{Read, Write},
    rc::Rc,
    str::Utf8Error,
};

use itertools::Itertools;
use pest::{iterators::Pair, Parser, Span};
use pest_derive::Parser;

use crate::{
    map::*,
    point::Point,
    string8::{IntoString8Error, String8},
    util::RcRC,
};

#[derive(Debug)]
pub struct PrettyPos {
    line: usize,
    col: usize,
}

impl PrettyPos {
    fn new(span: &Span<'_>) -> Self {
        let (line, col) = span.start_pos().line_col();
        Self { line, col }
    }
}

impl Display for PrettyPos {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "line {}, col {}", self.line, self.col)
    }
}

#[derive(Debug)]
pub enum Error {
    Grammar(Box<dyn GrammarError + 'static>),
    IO(std::io::Error),
    Fmt(fmt::Error),
    Utf8(Utf8Error),
    ParseBool(std::str::ParseBoolError, PrettyPos),
    ParseInt(std::num::ParseIntError, PrettyPos),
    ParseFloat(std::num::ParseFloatError, PrettyPos),
    String8(IntoString8Error, PrettyPos),
    MultipleAssignment(String, PrettyPos),
    InvalidAssignment {
        field: String,
        value: Value,
        pos: PrettyPos,
    },
    InvalidBlock(String, PrettyPos),
    MissingField(&'static str, PrettyPos),
    LineDefSpecial(i16, PrettyPos),
    SectorSpecial(i16, PrettyPos),
}

pub trait GrammarError: std::error::Error {}

struct PestError<R>(pest::error::Error<R>);

impl<R: pest::RuleType> GrammarError for PestError<R> {}

impl<R: pest::RuleType> fmt::Display for PestError<R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<R: pest::RuleType> std::error::Error for PestError<R> {}

impl<R: pest::RuleType> fmt::Debug for PestError<R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Grammar(e) => write!(f, "Error while parsing UDMF textmap: '{}'", e),
            Error::IO(e) => write!(f, "IO Error while reading UDMF textmap: '{}'", e),
            Error::Fmt(e) => write!(f, "Format error while reading UDMF textmap: '{}'", e),
            Error::Utf8(e) => write!(f, "Invalid UTF-8 string: '{}'", e),
            Error::ParseBool(e, p) => write!(f, "Error while parsing bool at {}: '{}'", p, e),
            Error::ParseInt(e, p) => write!(f, "Error while parsing integer at {}: '{}'", p, e),
            Error::ParseFloat(e, p) => write!(f, "Error while parsing float at {}: '{}'", p, e),
            Error::String8(e, p) => write!(f, "Error while parsing string8 at {}: '{}'", p, e),
            Error::MultipleAssignment(s, p) => write!(f, "Multiple assignment at {}: '{}'", p, s),
            Error::InvalidAssignment { field, value, pos } => {
                write!(f, "Invalid assignment at {}: '{} = {}'", pos, field, value)
            }
            Error::InvalidBlock(s, p) => write!(f, "Invalid block at {}: '{}'", p, s),
            Error::MissingField(s, p) => write!(f, "Missing field at {}: '{}'", p, s),
            Error::LineDefSpecial(v, p) => write!(f, "Invalid linedef special at '{}': {}", p, v),
            Error::SectorSpecial(v, p) => write!(f, "Invalid sector special at '{}': {}", p, v),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Grammar(_) => None, // FIXME when I figure out to downcast to a trait object
            Error::IO(e) => Some(e),
            Error::Fmt(e) => Some(e),
            Error::Utf8(e) => Some(e),
            Error::ParseBool(e, _) => Some(e),
            Error::ParseInt(e, _) => Some(e),
            Error::ParseFloat(e, _) => Some(e),
            Error::String8(e, _) => Some(e),
            Error::MultipleAssignment(_, _) => None,
            Error::InvalidAssignment { .. } => None,
            Error::InvalidBlock(_, _) => None,
            Error::MissingField(_, _) => None,
            Error::LineDefSpecial(_, _) => None,
            Error::SectorSpecial(_, _) => None,
        }
    }
}

impl<R: pest::RuleType + 'static> From<pest::error::Error<R>> for Error {
    fn from(e: pest::error::Error<R>) -> Self {
        Error::Grammar(Box::new(PestError(e)))
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IO(e)
    }
}

impl From<std::fmt::Error> for Error {
    fn from(e: std::fmt::Error) -> Self {
        Error::Fmt(e)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(e: std::str::Utf8Error) -> Self {
        Error::Utf8(e)
    }
}

impl<'i> From<(std::str::ParseBoolError, &Span<'i>)> for Error {
    fn from((e, span): (std::str::ParseBoolError, &Span<'i>)) -> Self {
        Error::ParseBool(e, PrettyPos::new(span))
    }
}

impl<'i> From<(std::num::ParseIntError, &Span<'i>)> for Error {
    fn from((e, span): (std::num::ParseIntError, &Span<'i>)) -> Self {
        Error::ParseInt(e, PrettyPos::new(span))
    }
}

impl<'i> From<(std::num::ParseFloatError, &Span<'i>)> for Error {
    fn from((e, span): (std::num::ParseFloatError, &Span<'i>)) -> Self {
        Error::ParseFloat(e, PrettyPos::new(span))
    }
}

impl<'i> From<(IntoString8Error, &Span<'i>)> for Error {
    fn from((e, span): (IntoString8Error, &Span<'i>)) -> Self {
        Error::String8(e, PrettyPos::new(span))
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Parser)]
#[grammar = "map/udmf.pest"]
struct UDMFParser;

trait UDMFObject: Sized {
    fn parse(body: Pair<Rule>) -> Result<Self>;
    fn write<W: UDMFWriter>(&self, writer: &mut W) -> Result<()>;
}

fn parse<O: UDMFObject>(body: Pair<Rule>) -> Result<O> {
    UDMFObject::parse(body)
}

struct LineDefData {
    from_idx: usize,
    to_idx: usize,
    left_side_idx: usize,
    right_side_idx: Option<usize>,

    flags: line_def::Flags,
    special: line_def::Special,
    trigger_flags: line_def::TriggerFlags,
}

const LINE_DEF_BLOCK: &str = "linedef";
const FROM_IDX_NAME: &str = "v1";
const TO_IDX_NAME: &str = "v2";
const LEFT_SIDE_IDX_NAME: &str = "sidefront";
const RIGHT_SIDE_IDX_NAME: &str = "sideback";
const IMPASSABLE_NAME: &str = "blocking";
const BLOCKS_MONSTERS_NAME: &str = "blockmonsters";
const TWO_SIDED_NAME: &str = "twosided";
const UPPER_UNPEGGED_NAME: &str = "dontpegtop";
const LOWER_UNPEGGED_NAME: &str = "dontpegbottom";
const SECRET_NAME: &str = "secret";
const BLOCKS_SOUND_NAME: &str = "blocksound";
const NOT_ON_MAP_NAME: &str = "dontdraw";
const ALREADY_ON_MAP_NAME: &str = "mapped";
const SPECIAL_NAME: &str = "special";
const ARG0_NAME: &str = "arg0";
const ARG1_NAME: &str = "arg1";
const ARG2_NAME: &str = "arg2";
const ARG3_NAME: &str = "arg3";
const ARG4_NAME: &str = "arg4";
const PLAYER_CROSS_NAME: &str = "playercross";
const PLAYER_USE_NAME: &str = "playeruse";
const MONSTER_CROSS_NAME: &str = "monstercross";
const MONSTER_USE_NAME: &str = "monsteruse";
const IMPACT_NAME: &str = "impact";
const PLAYER_PUSH_NAME: &str = "playerpush";
const MONSTER_PUSH_NAME: &str = "monsterpush";
const MISSILE_CROSS_NAME: &str = "missilecross";
const REPEATS_NAME: &str = "repeatspecial";

impl UDMFObject for LineDefData {
    fn parse(body: Pair<Rule>) -> Result<Self> {
        let mut from_idx: Option<usize> = None;
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

        let default_flags = line_def::Flags::default();
        let default_trigger_flags = line_def::TriggerFlags::default();

        let body_span = body.as_span();

        for expr in body.into_inner() {
            let span = expr.as_span();

            match parse_assignment(expr)? {
                (FROM_IDX_NAME, Value::Int(i)) if i >= 0 => {
                    assign_once(&mut from_idx, i as usize, &span)?
                }
                (TO_IDX_NAME, Value::Int(i)) if i >= 0 => {
                    assign_once(&mut to_idx, i as usize, &span)?
                }
                (LEFT_SIDE_IDX_NAME, Value::Int(i)) if i >= 0 => {
                    assign_once(&mut left_side_idx, i as usize, &span)?
                }
                (RIGHT_SIDE_IDX_NAME, Value::Int(i)) if i >= 0 => {
                    assign_once(&mut right_side_idx, i as usize, &span)?
                }
                (IMPASSABLE_NAME, Value::Bool(b)) => assign_once(&mut impassable, b, &span)?,
                (BLOCKS_MONSTERS_NAME, Value::Bool(b)) => {
                    assign_once(&mut blocks_monsters, b, &span)?
                }
                (TWO_SIDED_NAME, Value::Bool(b)) => assign_once(&mut two_sided, b, &span)?,
                (UPPER_UNPEGGED_NAME, Value::Bool(b)) => {
                    assign_once(&mut upper_unpegged, b, &span)?
                }
                (LOWER_UNPEGGED_NAME, Value::Bool(b)) => {
                    assign_once(&mut lower_unpegged, b, &span)?
                }
                (SECRET_NAME, Value::Bool(b)) => assign_once(&mut secret, b, &span)?,
                (BLOCKS_SOUND_NAME, Value::Bool(b)) => assign_once(&mut blocks_sound, b, &span)?,
                (NOT_ON_MAP_NAME, Value::Bool(b)) => assign_once(&mut not_on_map, b, &span)?,
                (ALREADY_ON_MAP_NAME, Value::Bool(b)) => {
                    assign_once(&mut already_on_map, b, &span)?
                }

                (SPECIAL_NAME, Value::Int(i)) => assign_once(&mut special, i, &span)?,
                (ARG0_NAME, Value::Int(i)) => assign_once(&mut arg0, i, &span)?,
                (ARG1_NAME, Value::Int(i)) => assign_once(&mut arg1, i, &span)?,
                (ARG2_NAME, Value::Int(i)) => assign_once(&mut arg2, i, &span)?,
                (ARG3_NAME, Value::Int(i)) => assign_once(&mut arg3, i, &span)?,
                (ARG4_NAME, Value::Int(i)) => assign_once(&mut arg4, i, &span)?,

                (PLAYER_CROSS_NAME, Value::Bool(b)) => assign_once(&mut player_cross, b, &span)?,
                (PLAYER_USE_NAME, Value::Bool(b)) => assign_once(&mut player_use, b, &span)?,
                (MONSTER_CROSS_NAME, Value::Bool(b)) => assign_once(&mut monster_cross, b, &span)?,
                (MONSTER_USE_NAME, Value::Bool(b)) => assign_once(&mut monster_use, b, &span)?,
                (IMPACT_NAME, Value::Bool(b)) => assign_once(&mut impact, b, &span)?,
                (PLAYER_PUSH_NAME, Value::Bool(b)) => assign_once(&mut player_push, b, &span)?,
                (MONSTER_PUSH_NAME, Value::Bool(b)) => assign_once(&mut monster_push, b, &span)?,
                (MISSILE_CROSS_NAME, Value::Bool(b)) => assign_once(&mut missile_cross, b, &span)?,
                (REPEATS_NAME, Value::Bool(b)) => assign_once(&mut repeats, b, &span)?,

                (k, v) => return invalid_assignment(k, v, &span),
            }
        }

        Ok(Self {
            from_idx: from_idx
                .ok_or_else(|| Error::MissingField(FROM_IDX_NAME, PrettyPos::new(&body_span)))?,

            to_idx: to_idx
                .ok_or_else(|| Error::MissingField(TO_IDX_NAME, PrettyPos::new(&body_span)))?,

            left_side_idx: left_side_idx.ok_or_else(|| {
                Error::MissingField(LEFT_SIDE_IDX_NAME, PrettyPos::new(&body_span))
            })?,

            right_side_idx,

            flags: line_def::Flags {
                impassable: impassable.unwrap_or(default_flags.impassable),
                blocks_monsters: blocks_monsters.unwrap_or(default_flags.blocks_monsters),
                two_sided: two_sided.unwrap_or(default_flags.two_sided),
                upper_unpegged: upper_unpegged.unwrap_or(default_flags.upper_unpegged),
                lower_unpegged: lower_unpegged.unwrap_or(default_flags.lower_unpegged),
                secret: secret.unwrap_or(default_flags.secret),
                blocks_sound: blocks_sound.unwrap_or(default_flags.blocks_sound),
                not_on_map: not_on_map.unwrap_or(default_flags.not_on_map),
                already_on_map: already_on_map.unwrap_or(default_flags.already_on_map),
            },

            special: line_def::UDMFSpecial {
                value: special.unwrap_or(0),
                args: (
                    arg0.unwrap_or(0),
                    arg1.unwrap_or(0),
                    arg2.unwrap_or(0),
                    arg3.unwrap_or(0),
                    arg4.unwrap_or(0),
                ),
            }
            .try_into()
            .map_err(|_| Error::LineDefSpecial(special.unwrap(), PrettyPos::new(&body_span)))?,

            trigger_flags: line_def::TriggerFlags {
                player_cross: player_cross.unwrap_or(default_trigger_flags.player_cross),
                player_use: player_use.unwrap_or(default_trigger_flags.player_use),
                monster_cross: monster_cross.unwrap_or(default_trigger_flags.monster_cross),
                monster_use: monster_use.unwrap_or(default_trigger_flags.monster_use),
                impact: impact.unwrap_or(default_trigger_flags.impact),
                player_push: player_push.unwrap_or(default_trigger_flags.player_push),
                monster_push: monster_push.unwrap_or(default_trigger_flags.monster_push),
                missile_cross: missile_cross.unwrap_or(default_trigger_flags.missile_cross),
                repeats: repeats.unwrap_or(default_trigger_flags.repeats),
            },
        })
    }

    fn write<W: UDMFWriter>(&self, writer: &mut W) -> Result<()> {
        writer.write_block(LINE_DEF_BLOCK, |block| {
            block.write_assignment(FROM_IDX_NAME, &Value::Int(self.from_idx as i16))?;
            block.write_assignment(TO_IDX_NAME, &Value::Int(self.to_idx as i16))?;

            Ok(())
        })
    }
}

pub struct SideDefData {
    pub sector_idx: usize,

    pub offset: Point,
    pub upper_texture: String8,
    pub middle_texture: String8,
    pub lower_texture: String8,
}

const SIDE_DEF_BLOCK: &str = "sidedef";
const OFFSET_X_NAME: &str = "offsetx";
const OFFSET_Y_NAME: &str = "offsety";
const SECTOR_IDX_NAME: &str = "sector";
const UPPER_TEXTURE_NAME: &str = "texturetop";
const MIDDLE_TEXTURE_NAME: &str = "texturemiddle";
const LOWER_TEXTURE_NAME: &str = "texturebottom";
const DEFAULT_TEXTURE: &str = "-";

impl UDMFObject for SideDefData {
    fn parse(body: Pair<Rule>) -> Result<Self> {
        let mut offset_x = None;
        let mut offset_y = None;
        let mut sector_idx = None;
        let mut upper_texture = None;
        let mut middle_texture = None;
        let mut lower_texture = None;

        let body_span = body.as_span();

        for expr in body.into_inner() {
            let span = expr.as_span();

            match parse_assignment(expr)? {
                (OFFSET_X_NAME, Value::Int(i)) => assign_once(&mut offset_x, i, &span)?,
                (OFFSET_Y_NAME, Value::Int(i)) => assign_once(&mut offset_y, i, &span)?,
                (SECTOR_IDX_NAME, Value::Int(i)) if i >= 0 => {
                    assign_once(&mut sector_idx, i as usize, &span)?
                }
                (UPPER_TEXTURE_NAME, Value::Str(s)) => assign_once(
                    &mut upper_texture,
                    s.as_str().try_into().map_err(|e| (e, &span))?,
                    &span,
                )?,
                (MIDDLE_TEXTURE_NAME, Value::Str(s)) => assign_once(
                    &mut middle_texture,
                    s.as_str().try_into().map_err(|e| (e, &span))?,
                    &span,
                )?,
                (LOWER_TEXTURE_NAME, Value::Str(s)) => assign_once(
                    &mut lower_texture,
                    s.as_str().try_into().map_err(|e| (e, &span))?,
                    &span,
                )?,

                (k, v) => return invalid_assignment(k, v, &span),
            }
        }

        Ok(Self {
            offset: Point::new(offset_x.unwrap_or(0), offset_y.unwrap_or(0)),
            sector_idx: sector_idx
                .ok_or_else(|| Error::MissingField(SECTOR_IDX_NAME, PrettyPos::new(&body_span)))?,

            upper_texture: upper_texture.unwrap_or(String8::from_str_unchecked(DEFAULT_TEXTURE)),
            middle_texture: middle_texture.unwrap_or(String8::from_str_unchecked(DEFAULT_TEXTURE)),
            lower_texture: lower_texture.unwrap_or(String8::from_str_unchecked(DEFAULT_TEXTURE)),
        })
    }

    fn write<W: UDMFWriter>(&self, writer: &mut W) -> Result<()> {
        writer.write_block(SIDE_DEF_BLOCK, |block| {
            if self.offset.x != 0 {
                block.write_assignment(OFFSET_X_NAME, &Value::Int(self.offset.x))?;
            }
            if self.offset.y != 0 {
                block.write_assignment(OFFSET_Y_NAME, &Value::Int(self.offset.y))?;
            }

            let upper_texture: &str = (&self.upper_texture).try_into()?;
            if upper_texture != DEFAULT_TEXTURE {
                block
                    .write_assignment(UPPER_TEXTURE_NAME, &Value::Str(upper_texture.to_string()))?;
            }
            let middle_texture: &str = (&self.middle_texture).try_into()?;
            if middle_texture != DEFAULT_TEXTURE {
                block.write_assignment(
                    MIDDLE_TEXTURE_NAME,
                    &Value::Str(middle_texture.to_string()),
                )?;
            }
            let lower_texture: &str = (&self.lower_texture).try_into()?;
            if lower_texture != DEFAULT_TEXTURE {
                block
                    .write_assignment(LOWER_TEXTURE_NAME, &Value::Str(lower_texture.to_string()))?;
            }

            Ok(())
        })
    }
}

pub struct SectorData {
    floor_height: i16,
    ceiling_height: i16,
    floor_flat: String8,
    ceiling_flat: String8,
    light_level: u8,
    special: sector::Special,
    tag: i16,
}

const SECTOR_BLOCK: &str = "sector";
const FLOOR_HEIGHT_NAME: &str = "heightfloor";
const CEILING_HEIGHT_NAME: &str = "heightceiling";
const FLOOR_FLAT_NAME: &str = "texturefloor";
const CEILING_FLAT_NAME: &str = "textureceiling";
const LIGHT_LEVEL_NAME: &str = "lightlevel";
const TAG_NAME: &str = "id";
const DEFAULT_LIGHT_LEVEL: u8 = 160;

impl UDMFObject for SectorData {
    fn parse(body: Pair<Rule>) -> Result<Self> {
        let mut floor_height = None;
        let mut ceiling_height = None;
        let mut floor_flat = None;
        let mut ceiling_flat = None;
        let mut light_level = None;
        let mut special = None;
        let mut tag = None;

        let body_span = body.as_span();

        for expr in body.into_inner() {
            let span = expr.as_span();

            match parse_assignment(expr)? {
                (FLOOR_HEIGHT_NAME, Value::Int(i)) => assign_once(&mut floor_height, i, &span)?,
                (CEILING_HEIGHT_NAME, Value::Int(i)) => assign_once(&mut ceiling_height, i, &span)?,
                (FLOOR_FLAT_NAME, Value::Str(s)) => assign_once(
                    &mut floor_flat,
                    s.as_str().try_into().map_err(|e| (e, &span))?,
                    &span,
                )?,
                (CEILING_FLAT_NAME, Value::Str(s)) => assign_once(
                    &mut ceiling_flat,
                    s.as_str().try_into().map_err(|e| (e, &span))?,
                    &span,
                )?,
                (LIGHT_LEVEL_NAME, Value::Int(i)) if i >= 0 && i < 256 => {
                    assign_once(&mut light_level, i as u8, &span)?
                }
                (SPECIAL_NAME, Value::Int(i)) => assign_once(&mut special, i, &span)?,
                (TAG_NAME, Value::Int(i)) => assign_once(&mut tag, i, &span)?,

                (k, v) => return invalid_assignment(k, v, &span),
            }
        }

        Ok(Self {
            floor_height: floor_height.unwrap_or(0),
            ceiling_height: ceiling_height.unwrap_or(0),

            floor_flat: floor_flat
                .ok_or_else(|| Error::MissingField(FLOOR_FLAT_NAME, PrettyPos::new(&body_span)))?,
            ceiling_flat: ceiling_flat.ok_or_else(|| {
                Error::MissingField(CEILING_FLAT_NAME, PrettyPos::new(&body_span))
            })?,

            light_level: light_level.unwrap_or(DEFAULT_LIGHT_LEVEL),
            special: special
                .unwrap_or(0)
                .try_into()
                .map_err(|e| Error::SectorSpecial(e, PrettyPos::new(&body_span)))?,
            tag: tag.unwrap_or(0),
        })
    }

    fn write<W: UDMFWriter>(&self, writer: &mut W) -> Result<()> {
        writer.write_block(SECTOR_BLOCK, |block| {
            if self.floor_height != 0 {
                block.write_assignment(FLOOR_HEIGHT_NAME, &Value::Int(self.floor_height))?;
            }
            if self.ceiling_height != 0 {
                block.write_assignment(CEILING_HEIGHT_NAME, &Value::Int(self.ceiling_height))?;
            }

            block.write_assignment(
                FLOOR_FLAT_NAME,
                &Value::Str(self.floor_flat.try_as_str()?.to_owned()),
            )?;
            block.write_assignment(
                CEILING_FLAT_NAME,
                &Value::Str(self.ceiling_flat.try_as_str()?.to_owned()),
            )?;

            if self.light_level != DEFAULT_LIGHT_LEVEL {
                block.write_assignment(LIGHT_LEVEL_NAME, &Value::Int(self.light_level as i16))?;
            }
            let special: i16 = self.special.into();
            if special != 0 {
                block.write_assignment(SPECIAL_NAME, &Value::Int(special))?;
            }

            if self.tag != 0 {
                block.write_assignment(TAG_NAME, &Value::Int(self.tag))?;
            }

            Ok(())
        })
    }
}

const VERTEX_BLOCK: &str = "vertex";
const X_NAME: &str = "x";
const Y_NAME: &str = "y";

impl UDMFObject for Vertex {
    fn parse(body: Pair<Rule>) -> Result<Self> {
        let mut x = None;
        let mut y = None;

        let body_span = body.as_span();

        for expr in body.into_inner() {
            let span = expr.as_span();

            match parse_assignment(expr)? {
                (X_NAME, Value::Int(i)) => assign_once(&mut x, i, &span)?,
                (X_NAME, Value::Float(f)) if f.fract() == 0.0 => {
                    assign_once(&mut x, f as i16, &span)?
                }
                (Y_NAME, Value::Int(i)) => assign_once(&mut y, i, &span)?,
                (Y_NAME, Value::Float(f)) if f.fract() == 0.0 => {
                    assign_once(&mut y, f as i16, &span)?
                }

                (k, v) => return invalid_assignment(k, v, &span),
            }
        }

        Ok(Self {
            position: Point::new(
                x.ok_or_else(|| Error::MissingField(X_NAME, PrettyPos::new(&body_span)))?,
                y.ok_or_else(|| Error::MissingField(Y_NAME, PrettyPos::new(&body_span)))?,
            ),
        })
    }

    fn write<W: UDMFWriter>(&self, writer: &mut W) -> Result<()> {
        writer.write_block(VERTEX_BLOCK, |block| {
            block.write_assignment(X_NAME, &Value::Float(self.position.x.into()))?;
            block.write_assignment(Y_NAME, &Value::Float(self.position.y.into()))?;

            Ok(())
        })
    }
}

const THING_BLOCK: &str = "thing";
const HEIGHT_NAME: &str = "height";
const ANGLE_NAME: &str = "angle";
const TYPE_NAME: &str = "type";
const SKILL1_NAME: &str = "skill1";
const SKILL2_NAME: &str = "skill2";
const SKILL3_NAME: &str = "skill3";
const SKILL4_NAME: &str = "skill4";
const SKILL5_NAME: &str = "skill5";
const AMBUSH_NAME: &str = "ambush";
const SINGLE_NAME: &str = "single";
const DM_NAME: &str = "dm";
const COOP_NAME: &str = "coop";
const MBF_FRIEND_NAME: &str = "friend";
const CLASS1_NAME: &str = "class1";
const CLASS2_NAME: &str = "class2";
const CLASS3_NAME: &str = "class3";
const DORMANT_NAME: &str = "dormant";
const INVISIBLE_NAME: &str = "invisible";
const NPC_NAME: &str = "standing";
const TRANSLUCENT_NAME: &str = "translucent";
const STRIFE_ALLY_NAME: &str = "strifeally";

impl UDMFObject for Thing {
    fn parse(body: Pair<Rule>) -> Result<Self> {
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

        let body_span = body.as_span();

        for expr in body.into_inner() {
            let span = expr.as_span();

            match parse_assignment(expr)? {
                (X_NAME, Value::Int(i)) => assign_once(&mut x, i, &span)?,
                (X_NAME, Value::Float(f)) if f.fract() == 0.0 => {
                    assign_once(&mut x, f as i16, &span)?
                }
                (Y_NAME, Value::Int(i)) => assign_once(&mut y, i, &span)?,
                (Y_NAME, Value::Float(f)) if f.fract() == 0.0 => {
                    assign_once(&mut y, f as i16, &span)?
                }

                (ANGLE_NAME, Value::Int(i)) => assign_once(&mut angle, i, &span)?,
                (HEIGHT_NAME, Value::Int(i)) => assign_once(&mut height, i, &span)?,
                (TYPE_NAME, Value::Int(i)) => assign_once(&mut type_, i, &span)?,

                (SKILL1_NAME, Value::Bool(b)) => assign_once(&mut skill1, b, &span)?,
                (SKILL2_NAME, Value::Bool(b)) => assign_once(&mut skill2, b, &span)?,
                (SKILL3_NAME, Value::Bool(b)) => assign_once(&mut skill3, b, &span)?,
                (SKILL4_NAME, Value::Bool(b)) => assign_once(&mut skill4, b, &span)?,
                (SKILL5_NAME, Value::Bool(b)) => assign_once(&mut skill5, b, &span)?,

                (AMBUSH_NAME, Value::Bool(b)) => assign_once(&mut ambush, b, &span)?,

                (CLASS1_NAME, Value::Bool(b)) => assign_once(&mut class1, b, &span)?,
                (CLASS2_NAME, Value::Bool(b)) => assign_once(&mut class2, b, &span)?,
                (CLASS3_NAME, Value::Bool(b)) => assign_once(&mut class3, b, &span)?,

                (MBF_FRIEND_NAME, Value::Bool(b)) => assign_once(&mut mbf_friend, b, &span)?,
                (DORMANT_NAME, Value::Bool(b)) => assign_once(&mut dormant, b, &span)?,
                (COOP_NAME, Value::Bool(b)) => assign_once(&mut coop, b, &span)?,
                (DM_NAME, Value::Bool(b)) => assign_once(&mut dm, b, &span)?,
                (INVISIBLE_NAME, Value::Bool(b)) => assign_once(&mut invisible, b, &span)?,
                (NPC_NAME, Value::Bool(b)) => assign_once(&mut npc, b, &span)?,
                (SINGLE_NAME, Value::Bool(b)) => assign_once(&mut single, b, &span)?,
                (STRIFE_ALLY_NAME, Value::Bool(b)) => assign_once(&mut strife_ally, b, &span)?,
                (TRANSLUCENT_NAME, Value::Bool(b)) => assign_once(&mut translucent, b, &span)?,

                (k, v) => return invalid_assignment(k, v, &span),
            }
        }

        Ok(Self {
            position: Point::new(
                x.ok_or_else(|| Error::MissingField(X_NAME, PrettyPos::new(&body_span)))?,
                y.ok_or_else(|| Error::MissingField(Y_NAME, PrettyPos::new(&body_span)))?,
            ),

            angle: angle.unwrap_or(0),
            height: height.unwrap_or(0),

            type_: type_
                .ok_or_else(|| Error::MissingField(TYPE_NAME, PrettyPos::new(&body_span)))?,

            flags: thing::Flags {
                skill1: skill1.unwrap_or(default_flags.skill1),
                skill2: skill2.unwrap_or(default_flags.skill2),
                skill3: skill3.unwrap_or(default_flags.skill3),
                skill4: skill4.unwrap_or(default_flags.skill4),
                skill5: skill5.unwrap_or(default_flags.skill5),

                ambush: ambush.unwrap_or(default_flags.ambush),

                class1: class1.unwrap_or(default_flags.class1),
                class2: class2.unwrap_or(default_flags.class2),
                class3: class3.unwrap_or(default_flags.class3),

                mbf_friend: mbf_friend.unwrap_or(default_flags.mbf_friend),
                dormant: dormant.unwrap_or(default_flags.dormant),
                coop: coop.unwrap_or(default_flags.coop),
                dm: dm.unwrap_or(default_flags.dm),
                invisible: invisible.unwrap_or(default_flags.invisible),

                npc: npc.unwrap_or(default_flags.npc),
                single: single.unwrap_or(default_flags.single),
                strife_ally: strife_ally.unwrap_or(default_flags.strife_ally),
                translucent: translucent.unwrap_or(default_flags.translucent),
            },

            special: thing::Special::None,
        })
    }

    fn write<W: UDMFWriter>(&self, writer: &mut W) -> Result<()> {
        writer.write_block(THING_BLOCK, |block| {
            if self.height != 0 {
                block.write_assignment(HEIGHT_NAME, &Value::Int(self.height))?;
            }
            if self.angle != 0 {
                block.write_assignment(ANGLE_NAME, &Value::Int(self.angle))?;
            }

            block.write_assignment(TYPE_NAME, &Value::Int(self.type_))?;

            let default_flags = thing::Flags::default();

            if self.flags.skill1 != default_flags.skill1 {
                block.write_assignment(SKILL1_NAME, &Value::Bool(self.flags.skill1))?;
            }
            if self.flags.skill2 != default_flags.skill2 {
                block.write_assignment(SKILL2_NAME, &Value::Bool(self.flags.skill2))?;
            }
            if self.flags.skill3 != default_flags.skill3 {
                block.write_assignment(SKILL3_NAME, &Value::Bool(self.flags.skill3))?;
            }
            if self.flags.skill4 != default_flags.skill4 {
                block.write_assignment(SKILL4_NAME, &Value::Bool(self.flags.skill4))?;
            }
            if self.flags.skill5 != default_flags.skill5 {
                block.write_assignment(SKILL5_NAME, &Value::Bool(self.flags.skill5))?;
            }
            if self.flags.ambush != default_flags.ambush {
                block.write_assignment(AMBUSH_NAME, &Value::Bool(self.flags.ambush))?;
            }
            if self.flags.single != default_flags.single {
                block.write_assignment(SINGLE_NAME, &Value::Bool(self.flags.single))?;
            }
            if self.flags.dm != default_flags.dm {
                block.write_assignment(DM_NAME, &Value::Bool(self.flags.dm))?;
            }
            if self.flags.coop != default_flags.coop {
                block.write_assignment(COOP_NAME, &Value::Bool(self.flags.coop))?;
            }
            if self.flags.mbf_friend != default_flags.mbf_friend {
                block.write_assignment(MBF_FRIEND_NAME, &Value::Bool(self.flags.mbf_friend))?;
            }
            if self.flags.class1 != default_flags.class1 {
                block.write_assignment(CLASS1_NAME, &Value::Bool(self.flags.class1))?;
            }
            if self.flags.class2 != default_flags.class2 {
                block.write_assignment(CLASS2_NAME, &Value::Bool(self.flags.class2))?;
            }
            if self.flags.class3 != default_flags.class3 {
                block.write_assignment(CLASS3_NAME, &Value::Bool(self.flags.class3))?;
            }
            if self.flags.dormant != default_flags.dormant {
                block.write_assignment(DORMANT_NAME, &Value::Bool(self.flags.dormant))?;
            }
            if self.flags.invisible != default_flags.invisible {
                block.write_assignment(INVISIBLE_NAME, &Value::Bool(self.flags.invisible))?;
            }
            if self.flags.npc != default_flags.npc {
                block.write_assignment(NPC_NAME, &Value::Bool(self.flags.npc))?;
            }
            if self.flags.translucent != default_flags.translucent {
                block.write_assignment(TRANSLUCENT_NAME, &Value::Bool(self.flags.translucent))?;
            }
            if self.flags.strife_ally != default_flags.strife_ally {
                block.write_assignment(STRIFE_ALLY_NAME, &Value::Bool(self.flags.strife_ally))?;
            }

            Ok(())
        })
    }
}

#[derive(Debug)]
pub enum Value {
    Int(i16),
    Float(f64),
    Str(String),
    Bool(bool),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Int(v) => write!(f, "{}", v),
            Value::Float(v) => write!(f, "{}", v),
            Value::Str(v) => write!(f, "\"{}\"", v),
            Value::Bool(v) => write!(f, "{}", v),
        }
    }
}

impl UDMFObject for Value {
    fn parse(pair: Pair<Rule>) -> Result<Self> {
        let span = pair.as_span();

        Ok(match pair.as_rule() {
            Rule::integer_decimal => {
                Value::Int(i16::from_str_radix(pair.as_str(), 10).map_err(|e| (e, &span))?)
            }
            Rule::integer_octal => {
                Value::Int(i16::from_str_radix(&pair.as_str()[1..], 8).map_err(|e| (e, &span))?)
            }
            Rule::integer_hex => {
                Value::Int(i16::from_str_radix(&pair.as_str()[2..], 16).map_err(|e| (e, &span))?)
            }
            Rule::float => Value::Float(pair.as_str().parse().map_err(|e| (e, &span))?),
            Rule::bool => Value::Bool(pair.as_str().parse().map_err(|e| (e, &span))?),
            Rule::quoted_string => {
                Value::Str(pair.into_inner().nth(0).unwrap().as_str().to_owned())
            }
            _ => panic!("Invalid rule as value: {:?}", pair.as_rule()),
        })
    }

    fn write<W: UDMFWriter>(&self, writer: &mut W) -> Result<()> {
        Ok(write!(writer.writer(), "{}", self)?)
    }
}

fn parse_assignment(pair: Pair<Rule>) -> Result<(&str, Value)> {
    debug_assert_eq!(pair.as_rule(), Rule::assignment_expr);

    let (ident, val) = pair.into_inner().next_tuple().unwrap();
    Ok((ident.as_str(), parse(val)?))
}

fn parse_block(pair: Pair<Rule>) -> (&str, Pair<Rule>) {
    let (ident, body) = pair.into_inner().next_tuple().unwrap();

    debug_assert_eq!(ident.as_rule(), Rule::identifier);
    debug_assert_eq!(body.as_rule(), Rule::block_body);

    (ident.as_str(), body)
}

fn assign_once<T>(opt: &mut Option<T>, value: T, span: &Span<'_>) -> Result<()> {
    if opt.is_some() {
        Err(Error::MultipleAssignment(
            span.as_str().to_owned(),
            PrettyPos::new(span),
        ))
    } else {
        *opt = Some(value);
        Ok(())
    }
}

fn invalid_assignment<T>(field: &str, value: Value, span: &Span<'_>) -> Result<T> {
    Err(Error::InvalidAssignment {
        field: field.to_owned(),
        value,
        pos: PrettyPos::new(span),
    })
}

fn invalid_block<T>(ident: &str, span: &Span<'_>) -> Result<T> {
    Err(Error::InvalidBlock(ident.to_owned(), PrettyPos::new(span)))
}

trait UDMFWriter: Sized {
    type Writer: Write;
    fn writer(&mut self) -> &mut Self::Writer;

    fn indent(&self) -> usize;

    fn write_comment(&mut self, text: &str) -> Result<()> {
        let indent = self.indent();
        Ok(writeln!(self.writer(), "{:2$}//{}", "", text, indent)?)
    }

    fn write_assignment(&mut self, key: &str, value: &Value) -> Result<()> {
        let indent = self.indent();
        Ok(writeln!(
            self.writer(),
            "{:3$}{}={};",
            "",
            key,
            value,
            indent
        )?)
    }

    fn write_block<F>(&mut self, key: &str, mut f: F) -> Result<()>
    where
        F: FnMut(&mut UDMFBlockWriter<Self>) -> Result<()>,
    {
        let mut block_writer = UDMFBlockWriter(self);
        block_writer.start(key)?;
        f(&mut block_writer)?;
        block_writer.end()
    }
}

struct UDMFBlockWriter<'w, W>(&'w mut W);

impl<'w, W: UDMFWriter> UDMFBlockWriter<'w, W> {
    fn start(&mut self, key: &str) -> Result<()> {
        let indent = self.0.indent();
        Ok(writeln!(self.0.writer(), "{:2$}{} {{", "", key, indent)?)
    }

    fn end(&mut self) -> Result<()> {
        let indent = self.0.indent();
        Ok(writeln!(self.0.writer(), "{:1$}}}", "", indent)?)
    }
}

impl<'w, W: UDMFWriter> UDMFWriter for UDMFBlockWriter<'w, W> {
    type Writer = W::Writer;

    fn writer(&mut self) -> &mut Self::Writer {
        self.0.writer()
    }

    fn indent(&self) -> usize {
        self.0.indent() + 2
    }
}

impl<W: Write> UDMFWriter for W {
    type Writer = Self;

    fn writer(&mut self) -> &mut Self::Writer {
        self
    }

    fn indent(&self) -> usize {
        0
    }
}

impl Map {
    pub fn write_udmf_textmap<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_comment("Written by this thing")?; // FIXME name
        writer.write_assignment("namespace", &Value::Str("zdoom".to_string()))?;

        let vertices: BTreeMap<Vertex, usize> = self
            .linedefs
            .iter()
            .flat_map(|l| vec![l.from.borrow().clone(), l.to.borrow().clone()])
            .collect::<BTreeSet<_>>()
            .into_iter()
            .enumerate()
            .map(|(i, v)| (v, i))
            .collect();

        /*
                let sidedefs: BTreeMap<SideDef> = self
                    .linedefs
                    .iter()
                    .flat_map(|l| {
                        if let Some(right) = l.right_side {
                            vec![l.left_side.borrow().clone(), right.borrow().clone()]
                        } else {
                            vec![l.left_side.borrow().clone()]
                        }
                    })
                    .collect();
        */

        for (vertex, i) in vertices {
            writer.write_comment(&format!("#{}", i))?;
            vertex.write(writer)?;
        }

        Ok(())
    }

    pub fn load_udmf_textmap<R: Read>(name: String8, reader: &mut R) -> Result<Self> {
        let mut buf = String::new();
        reader.read_to_string(&mut buf)?;

        let translation_units = UDMFParser::parse(Rule::translation_unit, &buf)?;

        let mut namespace = None;
        let mut vertices: Vec<RcRC<Vertex>> = Vec::new();
        let mut linedef_data: Vec<LineDefData> = Vec::new();
        let mut sidedef_data: Vec<SideDefData> = Vec::new();
        let mut sector_data: Vec<SectorData> = Vec::new();
        let mut things: Vec<Thing> = Vec::new();

        for translation_unit in translation_units {
            for global_expression in translation_unit.into_inner() {
                let span = global_expression.as_span();

                match global_expression.as_rule() {
                    Rule::assignment_expr => match parse_assignment(global_expression)? {
                        ("namespace", Value::Str(s)) => assign_once(&mut namespace, s, &span)?,
                        (k, v) => return invalid_assignment(k, v, &span),
                    },

                    Rule::block => {
                        let (ident, block) = parse_block(global_expression);

                        match ident {
                            VERTEX_BLOCK => vertices.push(Rc::new(RefCell::new(parse(block)?))),
                            LINE_DEF_BLOCK => linedef_data.push(parse(block)?),
                            SECTOR_BLOCK => sector_data.push(parse(block)?),
                            SIDE_DEF_BLOCK => sidedef_data.push(parse(block)?),
                            THING_BLOCK => things.push(parse(block)?),
                            id => return invalid_block(id, &span),
                        }
                    }

                    Rule::EOI => {}

                    _ => panic!(
                        "Invalid rule as global expression: {:?}",
                        global_expression.as_rule()
                    ),
                }
            }
        }

        let mut sector_side_map: Vec<Vec<usize>> = vec![Vec::new(); sector_data.len()];

        let sidedefs: Vec<_> = sidedef_data
            .into_iter()
            .enumerate()
            .map(|(i, sdd)| {
                sector_side_map[sdd.sector_idx].push(i);

                Rc::new(RefCell::new(SideDef {
                    offset: sdd.offset,
                    upper_texture: sdd.upper_texture,
                    middle_texture: sdd.middle_texture,
                    lower_texture: sdd.lower_texture,
                }))
            })
            .collect();

        let linedefs = linedef_data
            .into_iter()
            .map(|ld| LineDef {
                from: vertices[ld.from_idx].clone(),
                to: vertices[ld.to_idx].clone(),
                left_side: sidedefs[ld.left_side_idx].clone(),
                right_side: ld.right_side_idx.map(|i| sidedefs[i].clone()),

                flags: ld.flags,
                special: ld.special,
                trigger_flags: ld.trigger_flags,
            })
            .collect();

        let sectors = sector_data
            .into_iter()
            .zip(sector_side_map.into_iter())
            .map(|(sd, sides_indices)| Sector {
                sides: sides_indices
                    .into_iter()
                    .map(|i| sidedefs[i].clone())
                    .collect(),

                floor_height: sd.floor_height,
                ceiling_height: sd.ceiling_height,
                floor_flat: sd.floor_flat,
                ceiling_flat: sd.ceiling_flat,
                light_level: sd.light_level,
                special: sd.special,
                tag: sd.tag,
            })
            .collect();

        Ok(Self {
            name,
            linedefs,
            sectors,
            things,
        })
    }
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
                upper_texture: String8::from_str_unchecked("-"),
                middle_texture: String8::from_str_unchecked("STONE2"),
                lower_texture: String8::from_str_unchecked("-"),
                offset: Point::new(0, 0),
            }));
            4
        ];

        expected.linedefs.push(LineDef {
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
        expected.linedefs.push(LineDef {
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
        expected.linedefs.push(LineDef {
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
        expected.linedefs.push(LineDef {
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

        expected.sectors.push(Sector {
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
            let udmf_special = line_def::UDMFSpecial::new(value, (1, 2, 3, 4, 5));

            let result: std::result::Result<line_def::Special, _> = udmf_special.try_into();

            if let Ok(special) = result {
                let converted: line_def::UDMFSpecial = special.into();

                assert_eq!(converted.value, udmf_special.value);

                if converted.args.0 != 0 {
                    assert_eq!(converted.args.0, udmf_special.args.0);
                }
                if converted.args.1 != 0 {
                    assert_eq!(converted.args.1, udmf_special.args.1);
                }
                if converted.args.2 != 0 {
                    assert_eq!(converted.args.2, udmf_special.args.2);
                }
                if converted.args.3 != 0 {
                    assert_eq!(converted.args.3, udmf_special.args.3);
                }
                if converted.args.4 != 0 {
                    assert_eq!(converted.args.4, udmf_special.args.4);
                }
            }
        }
    }
}
