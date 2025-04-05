use bevy::prelude::*;

#[derive(Component, Deref, DerefMut, Clone, Debug)]
pub struct Transition(pub Timer);
