use crate::modding::types::Id;

#[derive(Debug)]
pub struct TileDef {
    pub sprite_path: String,
}

#[derive(Debug, Clone, Copy)]
pub struct Tile {
    pub id: Id<TileDef>,
}

impl Tile {
    pub fn new(id: Id<TileDef>) -> Self {
        Self { id }
    }
}

impl Default for Tile {
    fn default() -> Self {
        Self::new(Id::ZERO)
    }
}
