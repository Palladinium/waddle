macro_rules! assignments {
    ($($name:ident => $key:literal),* $(,)?) => {
        pub mod assignments {
            $(pub const $name: &str = $key;)*

            pub const ALL: &[&str] = &[
                $($name,)*
            ];
        }
    };
}

pub mod global {
    assignments! {
        NAMESPACE => "namespace",
    }

    pub const BLOCKS: &[&str] = &[
        super::vertex::BLOCK,
        super::line_def::BLOCK,
        super::sector::BLOCK,
        super::side_def::BLOCK,
        super::thing::BLOCK,
    ];
}

pub mod vertex {
    pub const BLOCK: &str = "vertex";

    assignments! {
        X => "x",
        Y => "y",
    }
}

pub mod line_def {
    pub const BLOCK: &str = "linedef";

    assignments! {
        FROM_IDX => "v1",
        TO_IDX => "v2",
        LEFT_SIDE_IDX => "sidefront",
        RIGHT_SIDE_IDX => "sideback",
        IMPASSABLE => "blocking",
        BLOCKS_MONSTERS => "blockmonsters",
        TWO_SIDED => "twosided",
        UPPER_UNPEGGED => "dontpegtop",
        LOWER_UNPEGGED => "dontpegbottom",
        SECRET => "secret",
        BLOCKS_SOUND => "blocksound",
        NOT_ON_MAP => "dontdraw",
        ALREADY_ON_MAP => "mapped",
        SPECIAL => "special",
        ARG0 => "arg0",
        ARG1 => "arg1",
        ARG2 => "arg2",
        ARG3 => "arg3",
        ARG4 => "arg4",
        PLAYER_CROSS => "playercross",
        PLAYER_USE => "playeruse",
        MONSTER_CROSS => "monstercross",
        MONSTER_USE => "monsteruse",
        IMPACT => "impact",
        PLAYER_PUSH => "playerpush",
        MONSTER_PUSH => "monsterpush",
        MISSILE_CROSS => "missilecross",
        REPEATS => "repeatspecial",
        MONSTER_ACTIVATE => "monsteractivate",
    }
}

pub mod side_def {
    pub const BLOCK: &str = "sidedef";

    assignments! {
        OFFSET_X => "offsetx",
        OFFSET_Y => "offsety",
        SECTOR_IDX => "sector",
        UPPER_TEXTURE => "texturetop",
        MIDDLE_TEXTURE => "texturemiddle",
        LOWER_TEXTURE => "texturebottom",
    }

    pub const DEFAULT_TEXTURE: &str = "-";
}

pub mod sector {
    pub const BLOCK: &str = "sector";

    assignments! {
        FLOOR_HEIGHT => "heightfloor",
        CEILING_HEIGHT => "heightceiling",
        FLOOR_FLAT => "texturefloor",
        CEILING_FLAT => "textureceiling",
        LIGHT_LEVEL => "lightlevel",
        TAG => "id",
        SPECIAL => "special", // TODO: Double-check
    }

    pub const DEFAULT_LIGHT_LEVEL: u8 = 160;
}

pub mod thing {
    pub const BLOCK: &str = "thing";

    assignments! {
        X => "x",
        Y => "y",
        HEIGHT => "height",
        ANGLE => "angle",
        TYPE => "type",
        SKILL1 => "skill1",
        SKILL2 => "skill2",
        SKILL3 => "skill3",
        SKILL4 => "skill4",
        SKILL5 => "skill5",
        AMBUSH => "ambush",
        SINGLE => "single",
        DM => "dm",
        COOP => "coop",
        MBF_FRIEND => "friend",
        CLASS1 => "class1",
        CLASS2 => "class2",
        CLASS3 => "class3",
        DORMANT => "dormant",
        INVISIBLE => "invisible",
        NPC => "standing",
        TRANSLUCENT => "translucent",
        STRIFE_ALLY => "strifeally",
        SPECIAL => "special", // TODO: Double-check
    }
}
