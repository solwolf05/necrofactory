use std::{fmt::Display, hash::Hash};

use crate::world::tile::Tile;

#[derive(Debug)]
pub struct Chunk {
    tiles: [Option<Tile>; 256],
}

impl Chunk {
    pub fn empty() -> Self {
        Self { tiles: [None; 256] }
    }

    pub fn insert(&mut self, position: TilePosition, tile: Tile) {
        self.tiles[position.0 as usize] = Some(tile);
    }

    pub fn remove(&mut self, position: TilePosition) {
        self.tiles[position.0 as usize] = None;
    }

    pub fn get(&self, position: TilePosition) -> Option<&Tile> {
        self.tiles[position.0 as usize].as_ref()
    }
}

/// A tile's position within a chunk.
///
/// Represented using a u8 for a 1:1 match to a 16x16 grid.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TilePosition(u8);

impl TilePosition {
    pub fn new(value: u8) -> Self {
        Self(value)
    }

    pub fn from_xy(x: u8, y: u8) -> Option<Self> {
        if x >= 16 || y >= 16 {
            return None;
        }

        Some(Self(x + y * 16))
    }

    pub fn x(&self) -> u8 {
        self.0 % 16
    }

    pub fn y(&self) -> u8 {
        self.0 / 16
    }

    pub fn to_xy(&self) -> (u8, u8) {
        (self.0 % 16, self.0 / 16)
    }
}

impl Display for TilePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (x, y) = self.to_xy();
        write!(f, "({}, {})", x, y)
    }
}
