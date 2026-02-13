use std::{fmt::Display, hash::Hash};

use bevy::prelude::*;

use crate::{modding::types::Id, world::tile::Tile};

#[derive(Debug)]
pub struct Chunk {
    tiles: Vec<Tile>,
    pub dirty: bool,
}

impl Chunk {
    pub fn empty() -> Self {
        Self {
            tiles: vec![Tile::new(Id::ZERO); 256],
            dirty: false,
        }
    }

    pub fn insert(&mut self, position: TilePosition, tile: Tile) {
        self.dirty = true;
        self.tiles[position.0 as usize] = tile;
    }

    /// Replace the tile with the default tile.
    pub fn remove(&mut self, position: TilePosition) {
        self.dirty = true;
        self.tiles[position.0 as usize] = Tile::default();
    }

    pub fn get(&self, position: TilePosition) -> &Tile {
        &self.tiles[position.0 as usize]
    }

    pub fn get_mut(&mut self, position: TilePosition) -> &mut Tile {
        self.dirty = true;
        &mut self.tiles[position.0 as usize]
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
