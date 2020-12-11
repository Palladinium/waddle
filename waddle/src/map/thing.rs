use crate::Point;

#[derive(PartialEq, Eq, Debug)]
pub struct Flags {
    pub skill1: bool,
    pub skill2: bool,
    pub skill3: bool,
    pub skill4: bool,
    pub skill5: bool,
    pub ambush: bool,
    pub single: bool,
    pub dm: bool,
    pub coop: bool,

    pub mbf_friend: bool,

    pub dormant: bool,
    pub class1: bool,
    pub class2: bool,
    pub class3: bool,
    pub npc: bool,
    pub strife_ally: bool,
    pub translucent: bool,
    pub invisible: bool,
}

impl Default for Flags {
    fn default() -> Self {
        Self {
            skill1: true,
            skill2: true,
            skill3: true,
            skill4: true,
            skill5: true,
            ambush: true,
            single: true,
            dm: true,
            coop: true,
            mbf_friend: false,
            dormant: false,
            class1: false,
            class2: false,
            class3: false,
            npc: false,
            strife_ally: false,
            translucent: false,
            invisible: false,
        }
    }
}

impl Flags {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum Special {
    None,
}

impl Default for Special {
    fn default() -> Self {
        Special::None
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Thing {
    pub position: Point,
    pub height: i16,
    pub angle: i16,
    pub type_: i16,
    pub flags: Flags,
    pub special: Special,
}
