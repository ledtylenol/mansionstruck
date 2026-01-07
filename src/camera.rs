use bevy::prelude::*;

#[derive(Event, Copy, Clone)]
pub struct CameraReset;
#[derive(Component, Reflect)]
#[relationship(relationship_target = FollowTargets)]
pub struct FollowerOf(pub Entity);

#[derive(Component, Reflect)]
#[relationship_target(relationship = FollowerOf)]
#[require(FollowAxes)]
pub struct FollowTargets(Vec<Entity>);

#[derive(Component, Reflect)]
pub struct FollowAxes(pub u8);
#[derive(Component, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct FollowWeight(pub u8);

#[derive(Component, Reflect)]
#[require(ClampFlags)]
pub struct ClampPosition {
    pub min: Vec2,
    pub max: Vec2,
}

#[derive(Component, Reflect, Default, Clone, Copy, Eq, PartialEq)]
pub struct ClampFlags(pub u8);

//TODO replace with bitflags!
impl ClampFlags {
    pub const MIN_Y: u8 = 0b0001;
    pub const MIN_X: u8 = 0b0010;
    pub const MAX_Y: u8 = 0b0100;
    pub const MAX_X: u8 = 0b1000;
    pub const ALL: u8 = 0b1111;

    pub const fn has(&self, rhs: u8) -> bool {
        self.0 & rhs != 0
    }
}

//TODO replace with bitflags!
impl FollowAxes {
    pub const HORIZONTAL: u8 = 1;
    pub const VERTICAL: u8 = 2;

    pub const fn new(axes: u8) -> Self {
        assert!(axes == 2 || axes == 1 || axes == 3);
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
    app.add_systems(PostUpdate, follow_targets);
}

pub fn follow_targets(
    mut cam_query: Query<(Entity, &mut Transform), With<FollowTargets>>,
    follower_query: Query<&FollowTargets>,
    target_query: Query<(&Transform, &FollowWeight), Without<FollowTargets>>,
) {
    for (e, mut transform) in cam_query.iter_mut() {
        transform.translation.x = 0.0;
        transform.translation.y = 0.0;
        let ancs = follower_query.iter_descendants(e);
        let sum: u8 = ancs.filter_map(|e| target_query.get(e).ok()).map(|(_, &FollowWeight(num))| num).sum();
        for e in follower_query.iter_descendants(e) {
            let Ok((xf, &FollowWeight(weight))) = target_query.get(e) else { continue; };
            let ratio = (weight as f32 / sum as f32);
            transform.translation.x += xf.translation.x * ratio;
            transform.translation.y += xf.translation.y * ratio;
        }
    }
}
