use bevy::prelude::*;

#[derive(Component, Reflect)]
#[relationship(relationship_target = FollowTarget)]
pub struct FollowerOf(pub Entity);

#[derive(Component, Reflect)]
#[relationship_target(relationship = FollowerOf)]
#[require(FollowAxes)]
pub struct FollowTarget(Vec<Entity>);

#[derive(Component, Reflect)]
pub struct FollowAxes(pub u8);

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
    app.add_systems(PostUpdate, (update_clamp, follow_target).chain());
}

pub fn update_clamp(
    mut clamp_query: Query<(&mut ClampPosition, &FollowerOf, &ClampFlags)>,
    target_query: Query<(&Transform, &FollowAxes), Without<FollowerOf>>,
) {
    for (mut clamp_pos, &FollowerOf(e), flags) in clamp_query.iter_mut() {
        let Ok((target_xf, axes)) = target_query.get(e) else {
            continue;
        };
        let target_xf = target_xf.translation.xy();
        if axes.has(FollowAxes::HORIZONTAL) {
            if flags.has(ClampFlags::MIN_X) {
                clamp_pos.min.x = clamp_pos.min.x.max(target_xf.x);
            }
            if flags.has(ClampFlags::MAX_X) {
                clamp_pos.max.x = clamp_pos.max.x.min(target_xf.x);
            }
        }
        if axes.has(FollowAxes::VERTICAL) {
            if flags.has(ClampFlags::MIN_Y) {
                clamp_pos.min.y = clamp_pos.min.y.max(target_xf.y);
            }
            if flags.has(ClampFlags::MAX_Y) {
                clamp_pos.max.y = clamp_pos.max.y.min(target_xf.y);
            }
        }
    }
}
pub fn follow_target(
    target: Query<(&Transform, &FollowAxes), With<FollowTarget>>,
    mut follower: Query<
        (&mut Transform, &FollowerOf, Option<&ClampPosition>),
        Without<FollowTarget>,
    >,
) {
    for (mut transform, camera_of, clamp_position) in follower.iter_mut() {
        let Ok((xf, axes)) = target.get(camera_of.0) else {
            continue;
        };
        let mut min_pos = vec2(f32::NEG_INFINITY, f32::NEG_INFINITY);
        let mut max_pos = vec2(f32::INFINITY, f32::INFINITY);
        if let Some(ClampPosition { min, max }) = clamp_position {
            min_pos = *min;
            max_pos = *max;
        }
        if axes.has(FollowAxes::HORIZONTAL) {
            transform.translation.x = xf.translation.x.clamp(min_pos.x, max_pos.x);
        }
        if axes.has(FollowAxes::VERTICAL) {
            transform.translation.y = xf.translation.y.clamp(min_pos.y, max_pos.y);
        }
    }
}
