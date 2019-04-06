use std::io::{Read, Write};

use crate::String8;

struct ACSLibrary;
struct ColorMap;
struct Filter;
struct Flat;
struct Graphic;
struct HiRes;
struct Music;
struct Patch;
struct Sound;
struct Sprite;
struct Texture;
struct Voice;
struct Voxel;

pub struct Wad {
    acs_libraries: Vec<ACSLibrary>,
}

impl Wad {}
