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

/// Flags determining how a `LineDef` `Special` may be triggered
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

    /// Compatibility flag defined in the ZDoom UDMF extensions
    pub monsters_activate: bool,
}

/// A special action associated with a `LineDef` or a `Thing`. Can also be called as functions in scripts.
#[derive(PartialEq, Eq, Hash, Debug, LineDefSpecial)]
#[doom_special(DoomSpecial)]
#[udmf_special(UdmfSpecial)]
#[trigger_flags(TriggerFlags)]
pub enum Special {
    #[udmf(0)]
    #[doom(id = 0, args = (), triggers = [])]
    None,

    #[udmf(1)]
    PolyobjStartLine { po: i16, mirror: i16, sound: i16 },

    #[udmf(2)]
    PolyobjRotateLeft {
        po: i16,
        speed: i16,
        // TODO Should be u8
        angle: i16,
    },

    #[udmf(3)]
    PolyobjRotateRight {
        po: i16,
        speed: i16,
        // TODO Should be u8
        angle: i16,
    },

    #[udmf(4)]
    PolyobjMove {
        po: i16,
        speed: i16,
        // TODO Should be u8
        angle: i16,
        dist: i16,
    },

    #[udmf(5)]
    PolyobjExplicitLine {
        po: i16,
        order: i16,
        mirror: i16,
        sound: i16,
    },

    #[udmf(6)]
    PolyobjMoveTimes8 {
        po: i16,
        speed: i16,
        // TODO Should be u8
        angle: i16,
        dist: i16,
    },

    #[udmf(7)]
    PolyobjDoorSwing {
        po: i16,
        speed: i16,
        // TODO Should be u8
        angle: i16,
        delay: i16,
    },

    #[udmf(8)]
    PolyobjDoorSlide {
        po: i16,
        speed: i16,
        // TODO Should be u8
        angle: i16,
        dist: i16,
        delay: i16,
    },

    #[udmf(9)]
    LineHorizon,

    #[udmf(10)]
    #[doom(id = 3, args = (tag, 16), triggers = [player_cross])]
    #[doom(id = 42, args = (tag, 16), triggers = [player_use, repeats])]
    #[doom(id = 50, args = (tag, 16), triggers = [player_use])]
    #[doom(id = 75, args = (tag, 16), triggers = [player_cross, repeats])]
    #[doom(id = 107, args = (tag, 64), triggers = [player_cross, repeats])]
    #[doom(id = 110, args = (tag, 64), triggers = [player_cross])]
    #[doom(id = 113, args = (tag, 64), triggers = [player_use])]
    #[doom(id = 116, args = (tag, 64), triggers = [player_use, repeats])]
    DoorClose {
        tag: i16,
        speed: i16,
        light_tag: i16,
    },

    #[udmf(11)]
    #[doom(id = 2, args = (tag, 16), triggers = [player_cross])]
    #[doom(id = 31, args = (0, 16, tag), triggers = [player_use])]
    #[doom(id = 46, args = (tag, 16), triggers = [impact, missile_cross, monsters_activate, repeats])]
    #[doom(id = 61, args = (tag, 16), triggers = [player_use, repeats])]
    #[doom(id = 86, args = (tag, 16), triggers = [player_cross, repeats])]
    #[doom(id = 103, args = (tag, 16), triggers = [player_use])]
    #[doom(id = 106, args = (tag, 64), triggers = [player_cross, repeats])]
    #[doom(id = 109, args = (tag, 64), triggers = [player_cross])]
    #[doom(id = 112, args = (tag, 64), triggers = [player_use])]
    #[doom(id = 115, args = (tag, 64), triggers = [player_use, repeats])]
    #[doom(id = 118, args = (0, 64, tag), triggers = [player_use])]
    DoorOpen {
        tag: i16,
        speed: i16,
        light_tag: i16,
    },

    #[udmf(12)]
    #[doom(id = 1, args = (0, 16, 150, tag), triggers = [player_use, monsters_activate, repeats])]
    #[doom(id = 4, args = (tag, 16, 150), triggers = [player_cross, monsters_activate])]
    #[doom(id = 29, args = (tag, 16, 150), triggers = [player_use])]
    #[doom(id = 63, args = (tag, 16, 150), triggers = [player_use, repeats])]
    #[doom(id = 90, args = (tag, 16, 150), triggers = [player_cross, repeats])]
    #[doom(id = 105, args = (tag, 64, 150), triggers = [player_cross, repeats])]
    #[doom(id = 108, args = (tag, 64, 150), triggers = [player_cross])]
    #[doom(id = 111, args = (tag, 64, 150), triggers = [player_use])]
    #[doom(id = 114, args = (tag, 64, 150), triggers = [player_use, repeats])]
    #[doom(id = 117, args = (0, 64, 150, tag), triggers = [player_use, repeats])]
    DoorRaise {
        tag: i16,
        speed: i16,
        delay: i16,
        light_tag: i16,
    },

    #[udmf(13)]
    #[doom(id = 26, args = (0, 16, 150, 130, tag), triggers = [player_use, repeats])]
    #[doom(id = 27, args = (0, 16, 150, 131, tag), triggers = [player_use, repeats])]
    #[doom(id = 28, args = (0, 16, 150, 129, tag), triggers = [player_use, repeats])]
    // FIXME Duplicate id 28 on ZDoom wiki page? 10 isn't even a valid lock id, maybe it was supposed to be 100?
    // #[doom(id = 28, args = (0, 16, 150, 10, tag), triggers = [player_use, repeats])]
    #[doom(id = 32, args = (0, 16, 0, 130, tag), triggers = [player_use, monsters_activate])]
    #[doom(id = 33, args = (0, 16, 0, 129, tag), triggers = [player_use, monsters_activate])]
    #[doom(id = 34, args = (0, 16, 0, 131, tag), triggers = [player_use, monsters_activate])]
    #[doom(id = 99, args = (tag, 64, 0, 130), triggers = [player_use, repeats])]
    #[doom(id = 133, args = (tag, 64, 0, 130), triggers = [player_use])]
    #[doom(id = 134, args = (tag, 64, 0, 129), triggers = [player_use, repeats])]
    #[doom(id = 135, args = (tag, 64, 0, 129), triggers = [player_use])]
    #[doom(id = 136, args = (tag, 64, 0, 131), triggers = [player_use, repeats])]
    #[doom(id = 137, args = (tag, 64, 0, 131), triggers = [player_use])]
    DoorRaiseLocked {
        tag: i16,
        speed: i16,
        delay: i16,
        // TODO Should be u8
        lock: i16,
        lighttag: i16,
    },

    #[udmf(14)]
    DoorAnimated {
        tag: i16,
        speed: i16,
        delay: i16,
        lock: i16,
    },

    #[udmf(15)]
    Autosave,

    #[udmf(16)]
    TransferWallLight {
        lineid: i16,
        // TODO Should be bitfield
        flags: i16,
    },

    #[udmf(17)]
    ThingRaise {
        tid: i16,
        // TODO Should be bool
        nocheck: i16,
    },

    #[udmf(18)]
    StartConversation { talker_tid: i16, facetalker: i16 },

    #[udmf(19)]
    ThingStop { tid: i16 },

    #[udmf(20)]
    FloorLowerByValue { tag: i16, speed: i16, height: i16 },

    #[udmf(21)]
    #[doom(id = 23, args = (tag, 8), triggers = [player_use])]
    #[doom(id = 38, args = (tag, 8), triggers = [player_cross])]
    #[doom(id = 60, args = (tag, 8), triggers = [player_use, repeats])]
    #[doom(id = 82, args = (tag, 8), triggers = [player_cross, repeats])]
    FloorLowerToLowest { tag: i16, speed: i16 },

    #[udmf(22)]
    FloorLowerToNearest { tag: i16, speed: i16 },

    #[udmf(23)]
    #[doom(id = 58, args = (tag, 8, 24), triggers = [player_cross])]
    #[doom(id = 92, args = (tag, 8, 24), triggers = [player_cross, repeats])]
    FloorRaiseByValue { tag: i16, speed: i16, height: i16 },

    #[udmf(24)]
    FloorRaiseToHighest { tag: i16, speed: i16 },

    #[udmf(25)]
    #[doom(id = 18, args = (tag, 8), triggers = [player_use])]
    #[doom(id = 69, args = (tag, 8), triggers = [player_use, repeats])]
    #[doom(id = 119, args = (tag, 8), triggers = [player_cross])]
    #[doom(id = 128, args = (tag, 8), triggers = [player_cross, repeats])]
    #[doom(id = 129, args = (tag, 32), triggers = [player_cross, repeats])]
    #[doom(id = 130, args = (tag, 32), triggers = [player_cross])]
    #[doom(id = 131, args = (tag, 32), triggers = [player_use])]
    #[doom(id = 132, args = (tag, 32), triggers = [player_use, repeats])]
    FloorRaiseToNearest { tag: i16, speed: i16 },

    #[udmf(26)]
    StairsBuildDown {
        tag: i16,
        speed: i16,
        height: i16,
        delay: i16,
        reset: i16,
    },

    #[udmf(27)]
    StairsBuildUp {
        tag: i16,
        speed: i16,
        height: i16,
        delay: i16,
        reset: i16,
    },

    #[udmf(28)]
    FloorRaiseAndCrush {
        tag: i16,
        speed: i16,
        crush: i16,
        // TODO Should be bitflags
        crushmode: i16,
    },

    #[udmf(29)]
    PillarBuild { tag: i16, speed: i16, height: i16 },

    #[udmf(30)]
    PillarOpen {
        tag: i16,
        speed: i16,
        fdist: i16,
        cdist: i16,
    },

    #[udmf(31)]
    StairsBuildDownSync {
        tag: i16,
        speed: i16,
        height: i16,
        reset: i16,
    },

    #[udmf(32)]
    StairsBuildUpSync {
        tag: i16,
        speed: i16,
        height: i16,
        reset: i16,
    },

    #[udmf(33)]
    ForceField,

    #[udmf(34)]
    ClearForceField { tag: i16 },

    #[udmf(35)]
    #[doom(id = 140, args = (tag, 8, 64), triggers = [player_use])]
    FloorRaiseByValueTimes8 { tag: i16, speed: i16, height: i16 },

    #[udmf(36)]
    FloorLowerByValueTimes8 { tag: i16, speed: i16, height: i16 },

    // TODO Check how this is actually used in UDMF - why is the neg argument needed if we can use signed parameters?
    #[udmf(37)]
    FloorMoveToValue {
        tag: i16,
        speed: i16,
        height: i16,
        neg: i16,
    },

    #[udmf(38)]
    CeilingWaggle {
        tag: i16,
        amp: i16,
        freq: i16,
        offset: i16,
        time: i16,
    },

    #[udmf(39)]
    TeleportZombieChanger { tid: i16, tag: i16 },

    #[udmf(40)]
    CeilingLowerByValue { tag: i16, speed: i16, height: i16 },

    #[udmf(41)]
    CeilingRaiseByValue { tag: i16, speed: i16, height: i16 },

    #[udmf(42)]
    CeilingCrushAndRaise {
        tag: i16,
        speed: i16,
        crush: i16,
        // TODO Should be bitflags
        crushmode: i16,
    },

    #[udmf(43)]
    #[doom(id = 44, args = (tag, 8, 0, 2), triggers = [player_cross])]
    #[doom(id = 72, args = (tag, 8, 0, 2), triggers = [player_cross, repeats])]
    CeilingLowerAndCrush {
        tag: i16,
        speed: i16,
        crush: i16,
        // TODO Should be bitflags
        crushmode: i16,
    },

    #[udmf(44)]
    #[doom(id = 57, args = (tag), triggers = [player_cross])]
    #[doom(id = 74, args = (tag), triggers = [player_cross, repeats])]
    CeilingCrushStop { tag: i16 },

    #[udmf(45)]
    CeilingCrushRaiseAndStay {
        tag: i16,
        speed: i16,
        crush: i16,
        // TODO Should be bitflags
        crushmode: i16,
    },

    #[udmf(46)]
    FloorCrushStop { tag: i16 },

    // TODO Check how this is actually used in UDMF - why is the neg argument needed if we can use signed parameters?
    #[udmf(47)]
    CeilingMoveToValue {
        tag: i16,
        speed: i16,
        height: i16,
        neg: i16,
    },

    #[udmf(48)]
    SectorAttach3dMidtex {
        lineid: i16,
        tag: i16,
        // TODO Should be enum
        floororceiling: i16,
    },

    #[udmf(49)]
    GlassBreak {
        // TODO Should be bool
        dontspawnjunk: i16,
        _type: i16,
    },

    #[udmf(50)]
    ExtraFloorLightOnly {
        tag: i16,
        // TODO Should be enum
        _type: i16,
    },

    #[udmf(51)]
    SectorSetLink {
        controltag: i16,
        tag: i16,
        // TOTO Should be bool
        surface: i16,
        // TODO Should be bitflags
        _type: i16,
    },

    #[udmf(52)]
    ScrollWall {
        lineid: i16,
        x: i16,
        y: i16,
        // TODO Should be bool
        side: i16,
        // TODO Should be bitflags
        flags: i16,
    },

    // NOTE: Cannot be used on a linedef
    #[udmf(53)]
    LineSetTextureOffset {
        lineid: i16,
        x: i16,
        y: i16,
        // TODO Should be enum
        side: i16,
        // TODO Should be bitflags
        flags: i16,
    },

    #[udmf(54)]
    SectorChangeFlags {
        tag: i16,
        // TODO Should be bitflags
        setflags: i16,
        // TODO Should be bitflags
        clearflags: i16,
    },

    #[udmf(55)]
    LineSetBlocking {
        lineid: i16,
        // TODO Should be bitflags
        setflags: i16,
        // TODO Should be bitflags
        clearflags: i16,
    },

    // NOTE: Cannot be used on a linedef
    #[udmf(56)]
    LineSetTextureScale {
        lineid: i16,
        x: i16,
        y: i16,
        // TODO Should be enum
        side: i16,
        // TODO Should be bitflags
        flags: i16,
    },

    #[udmf(57)]
    SectorSetPortal {
        tag: i16,
        // TODO should be enum
        _type: i16,
        // TODO should be enum
        plane: i16,
        misc: i16,
        alpha: i16,
    },

    #[udmf(58)]
    SectorCopyScroller {
        tag: i16,
        // TODO should be bitflags
        flags: i16,
    },

    #[udmf(59)]
    PolyobjOrMoveToSpot { po: i16, speed: i16, target: i16 },

    #[udmf(60)]
    PlatPerpetualRaise { tag: i16, speed: i16, delay: i16 },

    #[udmf(61)]
    #[doom(id = 54, args = (tag), triggers = [player_cross])]
    #[doom(id = 89, args = (tag), triggers = [player_cross, repeats])]
    PlatStop { tag: i16 },

    #[udmf(62)]
    PlatDownWaitUpStay { tag: i16, speed: i16, delay: i16 },

    #[udmf(63)]
    PlatDownByValue {
        tag: i16,
        speed: i16,
        delay: i16,
        height: i16,
    },

    #[udmf(64)]
    PlatUpWaitDownStay { tag: i16, speed: i16, delay: i16 },

    #[udmf(65)]
    PlatUpByValue {
        tag: i16,
        speed: i16,
        delay: i16,
        height: i16,
    },

    #[udmf(66)]
    FloorLowerInstant { tag: i16, arg1: i16, height: i16 },

    #[udmf(67)]
    FloorRaiseInstant { tag: i16, arg1: i16, height: i16 },

    #[udmf(68)]
    FloorMoveToValueTimes8 {
        tag: i16,
        speed: i16,
        height: i16,
        neg: i16,
    },

    #[udmf(69)]
    CeilingMoveToValueTimes8 {
        tag: i16,
        speed: i16,
        height: i16,
        neg: i16,
    },

    #[udmf(70)]
    #[doom(id = 39, args = (0, tag), triggers = [player_cross, monsters_activate])]
    #[doom(id = 97, args = (0, tag), triggers = [player_cross, repeats, monsters_activate])]
    #[doom(id = 125, args = (0, tag), triggers = [monster_cross])]
    #[doom(id = 126, args = (0, tag), triggers = [monster_cross, repeats])]
    Teleport {
        tid: i16,
        tag: i16,
        // TODO Should be bool
        nosourcefog: i16,
    },

    #[udmf(71)]
    TeleportNoFog {
        tid: i16,
        // TODO Should be enum
        useangle: i16,
        tag: i16,
        // TODO Should be bool
        keepheight: i16,
    },

    #[udmf(72)]
    ThrustThing {
        // TODO Should be u8
        angle: i16,
        force: i16,
        // TODO Should be bool
        nolimit: i16,
        tid: i16,
    },

    #[udmf(73)]
    DamageThing { amount: i16, _mod: i16 },

    #[udmf(74)]
    TeleportNewMap {
        map: i16,
        pos: i16,
        // TODO Should be bool
        face: i16,
    },

    #[udmf(75)]
    TeleportEndGame,

    #[udmf(76)]
    TeleportOther {
        tid: i16,
        destination: i16,
        // TODO Should be bool
        fog: i16,
    },

    #[udmf(77)]
    TeleportGroup {
        tid: i16,
        source: i16,
        destination: i16,
        // TODO Should be bool
        movesource: i16,
        // TODO Should be bool
        fog: i16,
    },

    #[udmf(78)]
    TeleportInSector {
        tag: i16,
        source_tid: i16,
        dest_tid: i16,
        // TODO Should be bool
        fog: i16,
        group_tid: i16,
    },

    #[udmf(79)]
    ThingSetConversation { tid: i16, convid: i16 },

    #[udmf(80)]
    AcsExecute {
        script: i16,
        map: i16,
        s_arg1: i16,
        s_arg2: i16,
        s_arg3: i16,
    },

    #[udmf(81)]
    AcsSuspend { script: i16, map: i16 },

    #[udmf(82)]
    AcsTerminate { script: i16, map: i16 },

    #[udmf(83)]
    AcsLockedExecute {
        script: i16,
        map: i16,
        s_arg1: i16,
        s_arg2: i16,
        // TODO Should be u8
        lock: i16,
    },

    #[udmf(84)]
    AcsExecuteWithResult {
        script: i16,
        s_arg1: i16,
        s_arg2: i16,
        s_arg3: i16,
        s_arg4: i16,
    },

    #[udmf(85)]
    AcsLockedExecuteDoor {
        script: i16,
        map: i16,
        s_arg1: i16,
        s_arg2: i16,
        // TODO Should be u8
        lock: i16,
    },

    #[udmf(86)]
    PolyobjMoveToSpot { po: i16, speed: i16, target: i16 },

    #[udmf(87)]
    PolyobjStop { po: i16 },

    #[udmf(88)]
    PolyobjMoveTo {
        po: i16,
        speed: i16,
        pos_x: i16,
        pos_y: i16,
    },

    #[udmf(89)]
    PolyobjOrMoveTo {
        po: i16,
        speed: i16,
        pos_x: i16,
        pos_y: i16,
    },

    #[udmf(90)]
    PolyobjOrRotateLeft {
        po: i16,
        speed: i16,
        // TODO Should be u8
        angle: i16,
    },

    #[udmf(91)]
    PolyobjOrRotateRight {
        po: i16,
        speed: i16,
        // TODO Should be u8
        angle: i16,
    },

    #[udmf(92)]
    PolyobjOrMove {
        po: i16,
        speed: i16,
        // TODO Should be u8
        angle: i16,
        dist: i16,
    },

    #[udmf(93)]
    PolyobjOrMoveTimes8 {
        po: i16,
        speed: i16,
        // TODO Should be u8
        angle: i16,
        dist: i16,
    },

    #[udmf(94)]
    PillarBuildAndCrush {
        tag: i16,
        speed: i16,
        height: i16,
        crush: i16,
        // TODO Should be enum
        crushmode: i16,
    },

    #[udmf(95)]
    FloorAndCeilingLowerByValue { tag: i16, speed: i16, value: i16 },

    #[udmf(96)]
    FloorAndCeilingRaiseByValue { tag: i16, speed: i16, value: i16 },

    #[udmf(97)]
    CeilingLowerAndCrushDist {
        tag: i16,
        speed: i16,
        crush: i16,
        dist: i16,
        // TODO Should be enum
        crushmode: i16,
    },

    #[udmf(98)]
    SectorSetTranslucent {
        tag: i16,
        // TODO Should be enum
        plane: i16,
        amount: i16,
        // TODO Should be enum
        _type: i16,
    },

    #[udmf(99)]
    #[doom(id = 55, args = (tag, 8, 10, 2), triggers = [player_use])]
    #[doom(id = 56, args = (tag, 8, 10, 2), triggers = [player_cross])]
    #[doom(id = 65, args = (tag, 8, 10, 2), triggers = [player_use, repeats])]
    #[doom(id = 94, args = (tag, 8, 10, 2), triggers = [player_cross, repeats])]
    FloorRaiseAndCrushDoom {
        tag: i16,
        speed: i16,
        crush: i16,
        // TODO Should be enum
        crushmode: i16,
    },

    #[udmf(100)]
    #[doom(id = 48, args = (64), triggers = [])]
    ScrollTextureLeft {
        speed: i16,
        // TODO Should be bitflags
        flags: i16,
    },

    #[udmf(101)]
    ScrollTextureRight {
        speed: i16,
        // TODO Should be bitflags
        flags: i16,
    },

    #[udmf(102)]
    ScrollTextureUp {
        speed: i16,
        // TODO Should be bitflags
        flags: i16,
    },

    #[udmf(103)]
    ScrollTextureDown {
        speed: i16,
        // TODO Should be bitflags
        flags: i16,
    },

    #[udmf(104)]
    #[doom(id = 141, args = (tag, 8, 8, 10), triggers = [player_cross])]
    CeilingCrushAndRaiseSilentDist {
        tag: i16,
        dist: i16,
        speed: i16,
        crush: i16,
        // TODO Should be enum
        crushmode: i16,
    },

    #[udmf(105)]
    DoorWaitRaise {
        tag: i16,
        speed: i16,
        delay: i16,
        wait: i16,
        lighttag: i16,
    },

    #[udmf(106)]
    DoorWaitClose {
        tag: i16,
        speed: i16,
        wait: i16,
        lighttag: i16,
    },

    #[udmf(107)]
    LineSetPortalTarget { sourceline: i16, targetline: i16 },
    //
    // UDMF 108 - unused
    //
    #[udmf(109)]
    LightForceLightning {
        // TODO Should be enum
        mode: i16,
    },

    #[udmf(110)]
    LightRaiseByValue { tag: i16, value: i16 },

    #[udmf(111)]
    LightLowerByValue { tag: i16, value: i16 },

    #[udmf(112)]
    #[doom(id = 13, args = (tag, 255), triggers = [player_cross])]
    #[doom(id = 35, args = (tag, 35), triggers = [player_cross])]
    #[doom(id = 79, args = (tag, 35), triggers = [player_cross, repeats])]
    #[doom(id = 81, args = (tag, 255), triggers = [player_cross, repeats])]
    #[doom(id = 138, args = (tag, 255), triggers = [player_use, repeats])]
    #[doom(id = 139, args = (tag, 35), triggers = [player_use, repeats])]
    LightChangeToValue { tag: i16, value: i16 },

    #[udmf(113)]
    LightFade { tag: i16, value: i16, tics: i16 },

    #[udmf(114)]
    LightGlow {
        tag: i16,
        upper: i16,
        lower: i16,
        tics: i16,
    },

    #[udmf(115)]
    LightFlicker { tag: i16, upper: i16, lower: i16 },

    #[udmf(116)]
    LightStrobe {
        tag: i16,
        upper: i16,
        lower: i16,
        u_tics: i16,
        i_tics: i16,
    },

    #[udmf(117)]
    LightStop { tag: i16 },

    #[udmf(118)]
    PlaneCopy {
        front_floor: i16,
        front_ceiling: i16,
        back_floor: i16,
        back_ceiling: i16,
        // TODO Should be bitflags
        share: i16,
    },

    #[udmf(119)]
    ThingDamage { tid: i16, amount: i16, _mod: i16 },

    #[udmf(120)]
    RadiusQuake {
        // TODO Should probably be enum
        intensity: i16,
        duration: i16,
        damrad: i16,
        temrad: i16,
        tid: i16,
    },

    /// Used only in Hexen format
    #[udmf(121)]
    LineSetIdentification {
        lineid: i16,
        moreflags: i16,
        lineid_hi: i16,
    },
    //
    // UDMF 122 - unused
    // UDMF 123 - unused
    // UDMF 124 - unused
    //
    #[udmf(125)]
    ThingMove {
        tid: i16,
        destid: i16,
        // TODO Should be bool
        nofog: i16,
    },
    //
    // UDMF 126 - unused
    //
    #[udmf(127)]
    ThingSetSpecial {
        tid: i16,
        special: i16,
        arg0: i16,
        arg1: i16,
        arg2: i16,
    },

    #[udmf(128)]
    ThrustThingZ {
        tid: i16,
        force: i16,
        // TODO Should be enum
        updown: i16,
        // TODO Should be enum
        setadd: i16,
    },

    #[udmf(129)]
    UsePuzzleItem {
        // TODO Should be enum
        item: i16,
        script: i16,
        s_arg1: i16,
        s_arg2: i16,
        s_arg3: i16,
    },

    #[udmf(130)]
    ThingActivate { tid: i16 },

    #[udmf(131)]
    ThingDeactivate { tid: i16 },

    #[udmf(132)]
    ThingRemove { tid: i16 },

    #[udmf(133)]
    ThingDestroy {
        tid: i16,
        // TODO Should be bool
        extreme: i16,
        tag: i16,
    },

    #[udmf(134)]
    ThingProjectile {
        tid: i16,
        _type: i16,
        // TODO Should be u8
        angle: i16,
        speed: i16,
        vspeed: i16,
    },

    #[udmf(135)]
    ThingSpawn {
        tid: i16,
        _type: i16,
        // TODO Should be u8
        angle: i16,
        newtid: i16,
    },

    #[udmf(136)]
    ThingProjectileGravity {
        tid: i16,
        _type: i16,
        // TODO Should be u8
        angle: i16,
        speed: i16,
        vspeed: i16,
    },

    #[udmf(137)]
    ThingSpawnNoFog {
        tid: i16,
        _type: i16,
        // TODO Should be u8
        angle: i16,
        newtid: i16,
    },

    #[udmf(138)]
    FloorWaggle {
        tag: i16,
        amp: i16,
        freq: i16,
        offset: i16,
        time: i16,
    },

    #[udmf(139)]
    ThingSpawnFacing {
        tid: i16,
        _type: i16,
        // TODO Should be bool
        nofog: i16,
        newtid: i16,
    },

    #[udmf(140)]
    SectorChangeSound { tag: i16, newsequence: i16 },
    //
    // UDMF 141 - unused
    // UDMF 142 - unused
    // UDMF 143 - unused
    // UDMF 144 - unused
    //
    #[udmf(145)]
    PlayerSetTeam { team: i16 },
    //
    // UDMF 146 - unused
    // UDMF 147 - unused
    // UDMF 148 - unused
    // UDMF 149 - unused
    // UDMF 150 - unused
    // UDMF 151 - unused
    //
    #[udmf(152)]
    TeamScore { points: i16, nogrin: i16 },

    #[udmf(153)]
    TeamGivePoints {
        // TODO Should be enum
        team: i16,
        points: i16,
        // TODO Should be bool
        announce: i16,
    },

    #[udmf(154)]
    TeleportNoStop {
        tid: i16,
        sectortag: i16,
        // TODO Should be bool
        nofog: i16,
    },
    //
    // UDMF 155 - unused
    //
    #[udmf(156)]
    LineSetPortal {
        targetline: i16,
        thisline: i16,
        // TODO Should be enum
        _type: i16,
        // TODO Should be enum
        planeanchor: i16,
    },

    #[udmf(157)]
    SetGlobalFogParameter {
        // TODO Should be enum
        property: i16,
        value: i16,
    },

    #[udmf(158)]
    FsExecute {
        scriptnumber: i16,
        // TODO Should be enum
        side: i16,
        keynum: i16,
        // TODO Should be enum
        message: i16,
    },

    #[udmf(159)]
    SectorSetPlaneReflection {
        tag: i16,
        // TODO Should be u8
        floor: i16,
        // TODO Should be u8
        ceiling: i16,
    },

    #[udmf(160)]
    SectorSet3dFloor {
        tag: i16,
        // TODO Should be bitflags
        _type: i16,
        // TODO Should be bitflags
        flags: i16,
        // TODO Should be u8
        alpha: i16,
        // TODO Should be something else - perhaps enum?
        hitag_lineid: i16,
    },

    #[udmf(161)]
    SectorSetContents {
        _type: i16,
        translucency: i16,
        flags: i16,
    },
    //
    // UDMF 162 - unused
    // UDMF 163 - unused
    // UDMF 164 - unused
    // UDMF 165 - unused
    // UDMF 166 - unused
    // UDMF 167 - unused
    //
    #[udmf(168)]
    CeilingCrushAndRaiseDist {
        tag: i16,
        dist: i16,
        speed: i16,
        crush: i16,
        // TODO Should be enum
        crushmode: i16,
    },

    #[udmf(169)]
    GenericCrusher2 {
        tag: i16,
        dspeed: i16,
        uspeed: i16,
        // TODO Should be bool
        silent: i16,
        crush: i16,
    },

    #[udmf(170)]
    /// NOTE: Can only be used from a script
    SectorSetCeilingScale2 {
        tag: i16,
        // TODO Check type
        u_fixed: i16,
        // TODO Check type
        v_fixed: i16,
    },

    #[udmf(171)]
    /// NOTE: Can only be used from a script
    SectorSetFloorScale2 {
        tag: i16,
        // TODO Check type
        u_fixed: i16,
        // TODO Check type
        v_fixed: i16,
    },

    #[udmf(172)]
    PlatUpNearestWaitDownStay { tag: i16, speed: i16, delay: i16 },

    #[udmf(173)]
    NoiseAlert { target_tid: i16, emitter_tid: i16 },

    #[udmf(174)]
    SendToCommunicator {
        voc_id: i16,
        // TODO Should be bool
        front_only: i16,
        identify: i16,
        // TODO Should be bool
        nolog: i16,
    },

    #[udmf(175)]
    ThingProjectileIntercept {
        tid: i16,
        _type: i16,
        // TODO Should be u8
        speed: i16,
        target: i16,
        newtid: i16,
    },

    #[udmf(176)]
    ThingChangeTid { oldtid: i16, newtid: i16 },

    #[udmf(177)]
    ThingHate {
        hater: i16,
        hatee: i16,
        // TODO Should be enum
        _type: i16,
    },

    #[udmf(178)]
    ThingProjectileAimed {
        tid: i16,
        _type: i16,
        speed: i16,
        target: i16,
        newtid: i16,
    },

    #[udmf(179)]
    ChangeSkill {
        // TODO Should be enum
        skill: i16,
    },

    #[udmf(180)]
    ThingSetTranslation { tid: i16, translation: i16 },

    #[udmf(181)]
    PlaneAlign {
        // TODO Should be enum
        floor: i16,
        // TODO Should be enum
        ceiling: i16,
        lineid: i16,
    },

    #[udmf(182)]
    LineMirror,

    #[udmf(183)]
    LineAlignCeiling {
        lineid: i16,
        // TODO Should be enum
        side: i16,
    },

    #[udmf(184)]
    LineAlignFloor {
        lineid: i16,
        // TODO Should be enum
        side: i16,
    },

    #[udmf(185)]
    SectorSetRotation { tag: i16, floor: i16, ceiling: i16 },

    #[udmf(186)]
    SectorSetCeilingPanning {
        tag: i16,
        u_int: i16,
        u_frac: i16,
        v_int: i16,
        v_frac: i16,
    },

    #[udmf(187)]
    SectorSetFloorPanning {
        tag: i16,
        u_int: i16,
        u_frac: i16,
        v_int: i16,
        v_frac: i16,
    },

    #[udmf(188)]
    SectorSetCeilingScale {
        tag: i16,
        u_int: i16,
        u_frac: i16,
        v_int: i16,
        v_frac: i16,
    },

    #[udmf(189)]
    SectorSetFloorScale {
        tag: i16,
        u_int: i16,
        u_frac: i16,
        v_int: i16,
        v_frac: i16,
    },

    #[udmf(190)]
    StaticInit {
        tag: i16,
        // TODO Should be enum
        prop: i16,
        flip_ceiling: i16,
        movetype: i16,
    },

    #[udmf(191)]
    SetPlayerProperty {
        // TODO Should be enum
        who: i16,
        // TODO Should be enum
        set: i16,
        // TODO Should be enum
        which: i16,
    },

    #[udmf(192)]
    CeilingLowerToHighestFloor { tag: i16, speed: i16 },

    #[udmf(193)]
    CeilingLowerInstant { tag: i16, arg1: i16, height: i16 },

    #[udmf(194)]
    CeilingRaiseInstant { tag: i16, arg1: i16, height: i16 },

    #[udmf(195)]
    CeilingCrushRaiseAndStayA {
        tag: i16,
        dspeed: i16,
        uspeed: i16,
        crush: i16,
        // TODO Should be enum
        crushmode: i16,
    },

    #[udmf(196)]
    CeilingCrushAndRaiseA {
        tag: i16,
        dspeed: i16,
        uspeed: i16,
        crush: i16,
        // TODO Should be enum
        crushmode: i16,
    },

    #[udmf(197)]
    CeilingCrushAndRaiseSilentA {
        tag: i16,
        dspeed: i16,
        uspeed: i16,
        crush: i16,
        // TODO Should be enum
        crushmode: i16,
    },

    #[udmf(198)]
    CeilingRaiseByValueTimes8 { tag: i16, speed: i16, height: i16 },

    #[udmf(199)]
    CeilingLowerByValueTimes8 { tag: i16, speed: i16, height: i16 },

    #[udmf(200)]
    GenericFloor {
        tag: i16,
        speed: i16,
        height: i16,
        // TODO Should be enum
        target: i16,
        // TODO Should be bitflags
        flags: i16,
    },

    #[udmf(201)]
    GenericCeiling {
        tag: i16,
        speed: i16,
        height: i16,
        // TODO Should be enum
        target: i16,
        // TODO Should be bitflags
        flags: i16,
    },

    #[udmf(202)]
    GenericDoor {
        tag: i16,
        speed: i16,
        // TODO Should be enum + bitflags
        kind: i16,
        delay: i16,
        lock: i16,
    },

    #[udmf(203)]
    GenericLift {
        tag: i16,
        speed: i16,
        delay: i16,
        // TODO Should be enum
        _type: i16,
        height: i16,
    },

    #[udmf(204)]
    GenericStairs {
        tag: i16,
        speed: i16,
        height: i16,
        // TODO Should be bitflags
        flags: i16,
        reset: i16,
    },

    #[udmf(205)]
    GenericCrusher {
        tag: i16,
        dspeed: i16,
        uspeed: i16,
        // TODO Should be bool
        silent: i16,
        crush: i16,
    },

    #[udmf(206)]
    #[doom(id = 10, args = (tag, 32, 105, 0), triggers = [player_cross, monsters_activate])]
    #[doom(id = 21, args = (tag, 32, 105), triggers = [player_use])]
    #[doom(id = 62, args = (tag, 32, 105, 0), triggers = [player_use, repeats])]
    #[doom(id = 88, args = (tag, 32, 105, 0), triggers = [player_cross, repeats, monsters_activate])]
    #[doom(id = 120, args = (tag, 64, 105, 0), triggers = [player_cross, repeats])]
    #[doom(id = 121, args = (tag, 64, 105, 0), triggers = [player_cross])]
    #[doom(id = 122, args = (tag, 64, 105, 0), triggers = [player_use])]
    #[doom(id = 123, args = (tag, 64, 105, 0), triggers = [player_use, repeats])]
    PlatDownWaitUpStayLip {
        tag: i16,
        speed: i16,
        delay: i16,
        lip: i16,
        // TODO Should be enum
        sound: i16,
    },

    #[udmf(207)]
    #[doom(id = 53, args = (tag, 8, 105, 0), triggers = [player_cross])]
    #[doom(id = 87, args = (tag, 8, 105, 0), triggers = [player_cross, repeats])]
    PlatPerpetualRaiseLip {
        tag: i16,
        speed: i16,
        delay: i16,
        lip: i16,
    },

    #[udmf(208)]
    TranslucentLine {
        lineid: i16,
        amount: i16,
        // TODO Should be bool
        additive: i16,
        // TODO Should be bitflags
        moreflags: i16,
    },

    #[udmf(209)]
    TransferHeights {
        tag: i16,
        // TODO Should be bitflags
        flags: i16,
    },

    #[udmf(210)]
    TransferFloorLight { tag: i16 },

    #[udmf(211)]
    TransferCeilingLight { tag: i16 },

    #[udmf(212)]
    SectorSetColor {
        tag: i16,
        // TODO Should be u8
        r: i16,
        // TODO Should be u8
        g: i16,
        // TODO Should be u8
        b: i16,
        // TODO Should be u8
        desat: i16,
    },

    #[udmf(213)]
    SectorSetFade {
        tag: i16,
        // TODO Should be u8
        r: i16,
        // TODO Should be u8
        g: i16,
        // TODO Should be u8
        b: i16,
    },

    #[udmf(214)]
    SectorSetDamage {
        tag: i16,
        amount: i16,
        // TODO Should be enum
        _mod: i16,
        interval: i16,
        leaky: i16,
    },

    #[udmf(215)]
    TeleportLine {
        thisid: i16,
        destid: i16,
        // TODO Should be bool
        flip: i16,
    },

    #[udmf(216)]
    SectorSetGravity { tag: i16, ipart: i16, fpart: i16 },

    #[udmf(217)]
    #[doom(id = 7, args = (tag, 2, 8), triggers = [player_use])]
    #[doom(id = 8, args = (tag, 2, 8), triggers = [player_cross])]
    StairsBuildUpDoom {
        tag: i16,
        speed: i16,
        height: i16,
        delay: i16,
        reset: i16,
    },

    #[udmf(218)]
    SectorSetWind {
        tag: i16,
        amount: i16,
        angle: i16,
        // TODO Should be bool
        useline: i16,
    },

    #[udmf(219)]
    SectorSetFriction {
        tag: i16, // TODO Should be u8
        amount: i16,
    },

    #[udmf(220)]
    SectorSetCurrent {
        tag: i16,
        amount: i16,
        angle: i16,
        // TODO Should be bool
        useline: i16,
    },

    #[udmf(221)]
    ScrollTextureBoth {
        lineid: i16,
        left: i16,
        right: i16,
        down: i16,
        up: i16,
    },

    #[udmf(222)]
    ScrollTextureModel {
        lineid: i16,
        // TODO Should be bitfield
        scrollbits: i16,
    },

    #[udmf(223)]
    /// NOTE: This cannot be used in a script, as the script version takes different arguments
    ScrollFloor {
        tag: i16,
        // TODO Should be bitflags
        scrollbits: i16,
        // TODO Should be enum
        _type: i16,
        x_move: i16,
        y_move: i16,
    },

    #[udmf(224)]
    /// NOTE: This cannot be used in a script, as the script version takes different arguments
    ScrollCeiling {
        tag: i16,
        // TODO Should be bitflags
        scrollbits: i16,
        unused: i16,
        x_move: i16,
        y_move: i16,
    },

    #[udmf(225)]
    ScrollTextureOffsets {
        // TODO Should be bitflags
        flags: i16,
    },

    #[udmf(226)]
    AcsExecuteAlways {
        script: i16,
        map: i16,
        s_arg1: i16,
        s_arg2: i16,
        s_arg3: i16,
    },

    #[udmf(227)]
    PointPushSetForce {
        tag: i16,
        tid: i16,
        amount: i16,
        // TODO Should be bool
        useline: i16,
    },

    #[udmf(228)]
    #[doom(id = 20, args = (tag, 4), triggers = [player_use])]
    #[doom(id = 22, args = (tag, 4), triggers = [player_cross])]
    #[doom(id = 47, args = (tag, 4), triggers = [impact, missile_cross])]
    #[doom(id = 68, args = (tag, 4), triggers = [player_use, repeats])]
    #[doom(id = 95, args = (tag, 4), triggers = [player_cross, repeats])]
    PlatRaiseAndStayTx0 {
        tag: i16,
        speed: i16,
        // TODO Should be enum
        lockout: i16,
    },

    #[udmf(229)]
    ThingSetGoal {
        tid: i16,
        goal: i16,
        delay: i16,
        // TODO Should be bool
        dontchasetarget: i16,
    },

    #[udmf(230)]
    #[doom(id = 14, args = (tag, 4, 4), triggers = [player_use])]
    #[doom(id = 15, args = (tag, 4, 3), triggers = [player_use])]
    #[doom(id = 66, args = (tag, 4, 3), triggers = [player_use, repeats])]
    #[doom(id = 67, args = (tag, 4, 4), triggers = [player_use, repeats])]
    PlatUpByValueStayTx { tag: i16, speed: i16, height: i16 },

    #[udmf(231)]
    PlatToggleCeiling { tag: i16 },

    #[udmf(232)]
    #[doom(id = 17, args = (tag, 5, 35), triggers = [player_cross])]
    LightStrobeDoom { tag: i16, u_tics: i16, i_tics: i16 },

    #[udmf(233)]
    #[doom(id = 104, args = (tag), triggers = [player_cross])]
    LightMinNeighbor { tag: i16 },

    #[udmf(234)]
    #[doom(id = 12, args = (tag), triggers = [player_cross])]
    #[doom(id = 80, args = (tag), triggers = [player_cross, repeats])]
    LightMaxNeighbor { tag: i16 },

    #[udmf(235)]
    FloorTransferTrigger { tag: i16 },

    #[udmf(236)]
    FloorTransferNumeric { tag: i16 },

    #[udmf(237)]
    ChangeCamera {
        tid: i16,
        // TODO Should be enum
        who: i16,
        // TODO Should be bool
        revert: i16,
    },

    #[udmf(238)]
    #[doom(id = 5, args = (tag, 8), triggers = [player_cross])]
    #[doom(id = 24, args = (tag, 8), triggers = [impact, missile_cross])]
    #[doom(id = 64, args = (tag, 8), triggers = [player_use, repeats])]
    #[doom(id = 91, args = (tag, 8), triggers = [player_cross, repeats])]
    #[doom(id = 101, args = (tag, 8), triggers = [player_use])]
    FloorRaiseToLowestCeiling { tag: i16, speed: i16 },

    #[udmf(239)]
    #[doom(id = 59, args = (tag, 8, 24), triggers = [player_cross])]
    #[doom(id = 93, args = (tag, 8, 24), triggers = [player_cross, repeats])]
    FloorRaiseByValueTxTy { tag: i16, speed: i16, height: i16 },

    #[udmf(240)]
    #[doom(id = 30, args = (tag, 8), triggers = [player_cross])]
    #[doom(id = 96, args = (tag, 8), triggers = [player_cross, repeats])]
    FloorRaiseByTexture { tag: i16, speed: i16 },

    #[udmf(241)]
    #[doom(id = 37, args = (tag, 8), triggers = [player_cross])]
    #[doom(id = 84, args = (tag, 8), triggers = [player_cross, repeats])]
    FloorLowerToLowestTxTy { tag: i16, speed: i16 },

    #[udmf(242)]
    #[doom(id = 19, args = (tag, 8, 128), triggers = [player_cross])]
    #[doom(id = 36, args = (tag, 32, 136), triggers = [player_cross])]
    #[doom(id = 45, args = (tag, 8, 128), triggers = [player_use, repeats])]
    #[doom(id = 70, args = (tag, 32, 136), triggers = [player_use, repeats])]
    #[doom(id = 71, args = (tag, 32, 136), triggers = [player_use])]
    #[doom(id = 83, args = (tag, 8, 128), triggers = [player_cross, repeats])]
    #[doom(id = 98, args = (tag, 32, 136), triggers = [player_cross, repeats])]
    #[doom(id = 102, args = (tag, 8, 128), triggers = [player_use])]
    FloorLowerToHighest {
        tag: i16,
        speed: i16,
        adjust: i16,
        force_adjust: i16,
    },

    #[udmf(243)]
    #[doom(id = 11, args = (0), triggers = [player_use])]
    #[doom(id = 52, args = (0), triggers = [player_cross])]
    ExitNormal { pos: i16 },

    #[udmf(244)]
    #[doom(id = 51, args = (0), triggers = [player_use])]
    #[doom(id = 124, args = (0), triggers = [player_cross])]
    ExitSecret { pos: i16 },

    #[udmf(245)]
    ElevatorRaiseToNearest { tag: i16, speed: i16 },

    #[udmf(246)]
    ElevatorMoveToFloor { tag: i16, speed: i16 },

    #[udmf(247)]
    ElevatorLowerToNearest { tag: i16, speed: i16 },

    #[udmf(248)]
    HealThing { amount: i16, max: i16 },

    #[udmf(249)]
    #[doom(id = 16, args = (tag, 16, 240), triggers = [player_cross])]
    #[doom(id = 76, args = (tag, 16, 240), triggers = [player_cross, repeats])]
    DoorCloseWaitOpen {
        tag: i16,
        speed: i16,
        delay: i16,
        lighttag: i16,
    },

    #[udmf(250)]
    #[doom(id = 9, args = (tag, 4, 4), triggers = [player_use])]
    FloorDonut { ptag: i16, pspeed: i16, sspeed: i16 },

    #[udmf(251)]
    FloorAndCeilingLowerRaise {
        tag: i16,
        fspeed: i16,
        cspeed: i16,
        // TODO should be enum
        boomemu: i16,
    },

    #[udmf(252)]
    CeilingRaiseToNearest { tag: i16, speed: i16 },

    #[udmf(253)]
    CeilingLowerToLowest { tag: i16, speed: i16 },

    #[udmf(254)]
    #[doom(id = 41, args = (tag, 8), triggers = [player_use])]
    #[doom(id = 43, args = (tag, 8), triggers = [player_use, repeats])]
    CeilingLowerToFloor { tag: i16, speed: i16 },

    #[udmf(255)]
    CeilingCrushRaiseAndStaySilA {
        tag: i16,
        dspeed: i16,
        uspeed: i16,
        crush: i16,
        // TODO Should be enum
        crusmode: i16,
    },

    #[udmf(256)]
    FloorLowerToHighestEE { tag: i16, speed: i16, change: i16 },

    #[udmf(257)]
    FloorRaiseToLowest { tag: i16, change: i16, crush: i16 },

    #[udmf(258)]
    FloorLowerToLowestCeiling { tag: i16, speed: i16, change: i16 },

    #[udmf(259)]
    FloorRaiseToCeiling {
        tag: i16,
        speed: i16,
        change: i16,
        crush: i16,
        gap: i16,
    },

    #[udmf(260)]
    FloorToCeilingInstant {
        tag: i16,
        change: i16,
        crush: i16,
        gap: i16,
    },

    #[udmf(261)]
    FloorLowerByTexture {
        tag: i16,
        speed: i16,
        change: i16,
        crush: i16,
    },

    #[udmf(262)]
    CeilingRaiseToHighest { tag: i16, speed: i16, change: i16 },

    #[udmf(263)]
    CeilingToHighestInstant { tag: i16, change: i16, crush: i16 },

    #[udmf(264)]
    CeilingLowerToNearest {
        tag: i16,
        speed: i16,
        change: i16,
        crush: i16,
    },

    #[udmf(265)]
    CeilingRaiseToLowest { tag: i16, speed: i16, change: i16 },

    #[udmf(266)]
    CeilingRaiseToHighestFloor { tag: i16, speed: i16, change: i16 },

    #[udmf(267)]
    CeilingToFloorInstant {
        tag: i16,
        change: i16,
        crush: i16,
        gap: i16,
    },

    #[udmf(268)]
    CeilingRaiseByTexture { tag: i16, speed: i16, change: i16 },

    #[udmf(269)]
    CeilingLowerByTexture {
        tag: i16,
        speed: i16,
        change: i16,
        crush: i16,
    },

    #[udmf(270)]
    StairsBuildDownDoom {
        tag: i16,
        speed: i16,
        height: i16,
        delay: i16,
        reset: i16,
    },

    #[udmf(271)]
    StairsBuildUpDoomSync {
        tag: i16,
        speed: i16,
        height: i16,
        reset: i16,
    },

    #[udmf(272)]
    StairsBuildDownDoomSync {
        tag: i16,
        speed: i16,
        height: i16,
        reset: i16,
    },

    #[udmf(273)]
    #[doom(id = 100, args = (tag, 32, 16, 0, 0), triggers = [player_cross])]
    #[doom(id = 127, args = (tag, 32, 16, 0, 0), triggers = [player_use])]
    StairsBuildUpDoomCrush {
        tag: i16,
        speed: i16,
        height: i16,
        delay: i16,
        reset: i16,
    },

    #[udmf(274)]
    DoorAnmatedClose { tag: i16, speed: i16 },

    #[udmf(275)]
    FloorStop { tag: i16 },

    #[udmf(276)]
    CeilingStop { tag: i16 },

    #[udmf(277)]
    SectorSetFloorGlow {
        tag: i16,
        height: i16,
        // TODO Should be u8
        r: i16,
        // TODO Should be u8
        g: i16,
        // TODO Should be u8
        b: i16,
    },

    #[udmf(278)]
    SectorSetCeilingGlow {
        tag: i16,
        height: i16,
        // TODO Should be u8
        r: i16,
        // TODO Should be u8
        g: i16,
        // TODO Should be u8
        b: i16,
    },

    #[udmf(279)]
    FloorMoveToValueAndCrush {
        tag: i16,
        speed: i16,
        height: i16,
        crush: i16,
        // TODO Should be enum
        crushmode: i16,
    },

    #[udmf(280)]
    CeilingMoveToValueAndCrush {
        tag: i16,
        speed: i16,
        height: i16,
        crush: i16,
        // TODO Should be enum
        crushmode: i16,
    },
}

impl Default for Special {
    fn default() -> Self {
        Special::None
    }
}

/// A `Special` representation in the UDMF format
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct UdmfSpecial {
    pub value: i16,
    pub args: [i16; 5],
}

impl UdmfSpecial {
    pub fn new(value: i16, args: [i16; 5]) -> Self {
        Self { value, args }
    }
}

/// A `Special` representation in the DOOM format
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct DoomSpecial {
    pub value: i16,
    pub tag: i16,
}

impl DoomSpecial {
    pub fn new(value: i16, tag: i16) -> Self {
        Self { value, tag }
    }
}
