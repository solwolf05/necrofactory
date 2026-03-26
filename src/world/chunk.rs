use std::{fmt::Display, hash::Hash};

use bevy::prelude::*;

use crate::world::tile::Tile;

#[derive(Debug)]
pub struct Chunk {
    tiles: Vec<Option<Tile>>,
    pub dirty: bool,
}

impl Chunk {
    pub fn empty() -> Self {
        Self {
            tiles: vec![None; 256],
            dirty: false,
        }
    }

    pub fn insert(&mut self, position: TilePosition, tile: Tile) {
        self.dirty = true;
        self.tiles[position.0 as usize] = Some(tile);
    }

    pub fn remove(&mut self, position: TilePosition) {
        self.dirty = true;
        self.tiles[position.0 as usize] = None;
    }

    pub fn contains(&self, position: TilePosition) -> bool {
        self.tiles[position.0 as usize].is_some()
    }

    pub fn get(&self, position: TilePosition) -> Option<&Tile> {
        self.tiles[position.0 as usize].as_ref()
    }

    pub fn get_mut(&mut self, position: TilePosition) -> Option<&mut Tile> {
        self.dirty = true;
        self.tiles[position.0 as usize].as_mut()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Tile> {
        self.tiles.iter().flatten()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Tile> {
        self.dirty = true;
        self.tiles.iter_mut().flatten()
    }
}

/// A tile's position within a chunk.
///
/// Represented using a u8 for a 1:1 match to a 16x16 grid.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TilePosition(u8);

impl TilePosition {
    pub const ZERO: Self = Self(0);

    pub fn new(value: u8) -> Self {
        Self(value)
    }

    pub fn from_xy(x: u8, y: u8) -> Self {
        if x >= 16 || y >= 16 {
            panic!("tile coordinates out of bounds: ({}, {})", x, y);
        }

        Self(x + y * 16)
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

impl From<TilePosition> for Vec2 {
    fn from(value: TilePosition) -> Self {
        let x = value.x() as f32;
        let y = value.y() as f32;
        Vec2::new(x, y)
    }
}
