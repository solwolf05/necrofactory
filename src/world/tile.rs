use crate::modding::Id;

#[derive(Debug)]
pub struct TileDef {
    pub sprite_path: String,
    pub friction: f32,
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

impl Tile {
    #[inline]
    pub fn is_some(&self) -> bool {
        self.id != Id::ZERO
    }

    #[inline]
    pub fn is_none(&self) -> bool {
        self.id == Id::ZERO
    }
}

impl Default for Tile {
    fn default() -> Self {
        Self::new(Id::ZERO)
    }
}
