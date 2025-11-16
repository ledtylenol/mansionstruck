use avian2d::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Deserialize, Serialize)]
pub enum ColliderShape {
    Ball(f32),
    Cuboid(f32, f32),
    Capsule(f32, f32),
}
impl Default for ColliderShape {
    fn default() -> Self {
        ColliderShape::Cuboid(20.0, 20.0)
    }
}
pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(PhysicsPlugins::default());
}