use avian2d::prelude::*;
use bevy::prelude::*;


pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(PhysicsPlugins::default());
}