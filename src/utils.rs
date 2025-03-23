use bevy::prelude::*;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy, Deref, DerefMut, Default)]
pub struct HashableVec2(Vec2);

impl HashableVec2 {
    pub fn new(vec: Vec2) -> Self {
        Self(vec)
    }

    pub fn vec2(&self) -> Vec2 {
        self.0
    }
}

impl PartialEq for HashableVec2 {
    fn eq(&self, other: &Self) -> bool {
        self.0.x == other.0.x && self.0.y == other.0.y
    }
}

impl Eq for HashableVec2 {}

impl Hash for HashableVec2 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Convert to bits for hashing
        let x_bits = self.0.x.to_bits();
        let y_bits = self.0.y.to_bits();
        x_bits.hash(state);
        y_bits.hash(state);
    }
}

impl From<Vec2> for HashableVec2 {
    fn from(vec: Vec2) -> Self {
        HashableVec2(vec)
    }
}

impl From<HashableVec2> for Vec2 {
    fn from(hashable: HashableVec2) -> Self {
        hashable.0
    }
}
