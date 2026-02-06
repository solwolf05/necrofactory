use bevy_modding::prelude::*;

pub struct TileDef {}

#[derive(Debug, Clone, Copy)]
pub struct Tile {
    id: RegHandle<TileDef>,
    flags: u8,
}

impl Tile {
    pub const MACHINE: u8 = 0b1;

    pub fn new(handle: RegHandle<TileDef>) -> Self {
        Self {
            id: handle,
            flags: 0,
        }
    }

    pub fn add_machine(&mut self) {
        self.flags |= Self::MACHINE;
    }

    pub fn has_machine(&self) -> bool {
        self.flags & Self::MACHINE != 0
    }
}
