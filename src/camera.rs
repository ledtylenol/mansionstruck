use bevy::prelude::*;

#[derive(Component, Reflect)]
#[relationship(relationship_target = CameraFollow)]
pub struct CameraOf(pub Entity);

#[derive(Component, Reflect)]
#[relationship_target(relationship = CameraOf)]
#[require(FollowAxes)]
pub struct CameraFollow(Vec<Entity>);

#[derive(Component, Reflect)]
pub struct FollowAxes(pub u8);

impl FollowAxes {
    pub const HORIZONTAL: u8 = 1;
    pub const VERTICAL: u8 = 2;

    pub const fn new(axes: u8) -> Self {
        assert!(axes == 2 || axes == 1);
        Self(axes)
    }

    pub fn toggle(&mut self, axis: u8) {
        self.0 ^= axis
    }
    pub fn has(&self, axis: u8) -> bool {
        (self.0 & axis) != 0
    }
}

impl Default for FollowAxes {
    fn default() -> Self {
        Self::new(1)
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(PostUpdate, follow_target);
}

pub fn follow_target(
    target: Query<(&Transform, &FollowAxes), With<CameraFollow>>,
    mut follower: Query<(&mut Transform, &CameraOf), Without<CameraFollow>>,
) {
    for (mut transform, camera_of) in follower.iter_mut() {
        let Ok((xf, axes)) = target.get(camera_of.0) else {
            continue;
        };
        if axes.has(FollowAxes::HORIZONTAL) {
            transform.translation.x = xf.translation.x;
        }
        if axes.has(FollowAxes::VERTICAL) {
            transform.translation.y = xf.translation.y;
        }
    }
}
