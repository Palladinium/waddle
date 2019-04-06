use std::{
    cell::RefCell,
    convert::TryInto,
    fmt,
    io::{Read, Write},
    rc::Rc,
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

impl fmt::Display for PrettyPos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "line {}, col {}", self.line, self.col)
    }
}

#[derive(Debug)]
pub enum Error {
    Grammar(Box<dyn GrammarError + 'static>),
    IO(std::io::Error),
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
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Grammar(_) => None, // FIXME when I figure out to downcast to a trait object
            Error::IO(e) => Some(e),
            Error::ParseBool(e, _) => Some(e),
            Error::ParseInt(e, _) => Some(e),
            Error::ParseFloat(e, _) => Some(e),
            Error::String8(e, _) => Some(e),
            Error::MultipleAssignment(_, _) => None,
            Error::InvalidAssignment { .. } => None,
            Error::InvalidBlock(_, _) => None,
            Error::MissingField(_, _) => None,
            Error::LineDefSpecial(_, _) => None,
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
}

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

        let default_flags = line_def::Flags::default();

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

const OFFSET_X_NAME: &str = "offsetx";
const OFFSET_Y_NAME: &str = "offsety";
const SECTOR_IDX_NAME: &str = "sector";
const UPPER_TEXTURE_NAME: &str = "texturetop";
const MIDDLE_TEXTURE_NAME: &str = "texturemiddle";
const LOWER_TEXTURE_NAME: &str = "texturebottom";

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

            upper_texture: upper_texture.unwrap_or(String8::from_str_unchecked("-")),
            middle_texture: middle_texture.unwrap_or(String8::from_str_unchecked("-")),
            lower_texture: lower_texture.unwrap_or(String8::from_str_unchecked("-")),
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

const FLOOR_HEIGHT_NAME: &str = "heightfloor";
const CEILING_HEIGHT_NAME: &str = "heightceiling";
const FLOOR_FLAT_NAME: &str = "texturefloor";
const CEILING_FLAT_NAME: &str = "textureceiling";
const LIGHT_LEVEL_NAME: &str = "lightlevel";
const TAG_NAME: &str = "id";

impl UDMFObject for SectorData {
    fn parse(body: Pair<Rule>) -> Result<Self> {
        let mut floor_height = None;
        let mut ceiling_height = None;
        let mut floor_flat = None;
        let mut ceiling_flat = None;
        let mut light_level = None;
        let mut tag = None;

        // FIXME special

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

            light_level: light_level.unwrap_or(160),
            special: sector::Special::default(),
            tag: tag.unwrap_or(0),
        })
    }
}

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
}

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
            Value::Str(v) => write!(f, "{}", v),
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

impl Map {
    pub fn write_udmf_textmap<W: Write>(&self, writer: &mut W) -> Result<()> {
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
                            "vertex" => vertices.push(Rc::new(RefCell::new(parse(block)?))),
                            "linedef" => linedef_data.push(parse(block)?),
                            "sector" => sector_data.push(parse(block)?),
                            "sidedef" => sidedef_data.push(parse(block)?),
                            "thing" => things.push(parse(block)?),
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
