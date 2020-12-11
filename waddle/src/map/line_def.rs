use bitfield::Bit;
use waddle_derive::LineDefSpecial;

use crate::{
    map::{SideDef, Vertex},
    util::RcRC,
};

#[derive(PartialEq, Eq, Debug)]
pub struct LineDef {
    pub from: RcRC<Vertex>,
    pub to: RcRC<Vertex>,
    pub left_side: RcRC<SideDef>,
    pub right_side: Option<RcRC<SideDef>>,

    pub flags: Flags,
    pub special: Special,
    pub trigger_flags: TriggerFlags,
}

/// Boolean flags associated with a `LineDef`
#[derive(Default, PartialEq, Eq, Hash, Debug, Clone)]
pub struct Flags {
    pub impassable: bool,
    pub blocks_monsters: bool,
    pub two_sided: bool,
    pub upper_unpegged: bool,
    pub lower_unpegged: bool,
    pub secret: bool,
    pub blocks_sound: bool,
    pub not_on_map: bool,
    pub already_on_map: bool,
}

impl From<i16> for Flags {
    fn from(flags: i16) -> Self {
        let flags_bits: u16 = flags as u16;

        Self {
            impassable: flags_bits.bit(0),
            blocks_monsters: flags_bits.bit(1),
            two_sided: flags_bits.bit(2),
            upper_unpegged: flags_bits.bit(3),
            lower_unpegged: flags_bits.bit(4),
            secret: flags_bits.bit(5),
            blocks_sound: flags_bits.bit(6),
            not_on_map: flags_bits.bit(7),
            already_on_map: flags_bits.bit(8),
        }
    }
}

impl From<Flags> for i16 {
    fn from(flags: Flags) -> Self {
        let mut flags_bits: u16 = 0;

        flags_bits.set_bit(0, flags.impassable);
        flags_bits.set_bit(1, flags.blocks_monsters);
        flags_bits.set_bit(2, flags.two_sided);
        flags_bits.set_bit(3, flags.upper_unpegged);
        flags_bits.set_bit(4, flags.lower_unpegged);
        flags_bits.set_bit(5, flags.secret);
        flags_bits.set_bit(6, flags.blocks_sound);
        flags_bits.set_bit(7, flags.not_on_map);
        flags_bits.set_bit(8, flags.already_on_map);

        flags_bits as i16
    }
}

#[derive(Default, PartialEq, Eq, Hash, Debug, Clone)]
pub struct TriggerFlags {
    pub player_cross: bool,
    pub player_use: bool,
    pub monster_cross: bool,
    pub monster_use: bool,
    pub impact: bool,
    pub player_push: bool,
    pub monster_push: bool,
    pub missile_cross: bool,
    pub repeats: bool,
}

/// A special action associated with a `LineDef`
#[derive(PartialEq, Eq, Hash, Debug, LineDefSpecial)]
#[doom_special(DOOMSpecial)]
#[udmf_special(UDMFSpecial)]
#[trigger_flags(TriggerFlags)]
pub enum Special {
    #[udmf(0)]
    #[doom(id = 0, args = (), triggers = [])]
    None,

    #[udmf(10)]
    #[doom(id = 3, args = (tag, 16), triggers = [player_cross])]
    DoorClose {
        tag: i16,
        speed: i16,
        light_tag: i16,
    },
}

impl Default for Special {
    fn default() -> Self {
        Special::None
    }
}

/// A `Special` representation in the UDMF format
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct UDMFSpecial {
    pub value: i16,
    pub args: (i16, i16, i16, i16, i16),
}

impl UDMFSpecial {
    pub fn new(value: i16, args: (i16, i16, i16, i16, i16)) -> Self {
        Self { value, args }
    }
}

/// A `Special` representation in the DOOM format
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct DOOMSpecial {
    pub value: i16,
    pub tag: i16,
}

impl DOOMSpecial {
    pub fn new(value: i16, tag: i16) -> Self {
        Self { value, tag }
    }
}
