use crate::mario::Mario;
use avian2d::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component)]
pub struct Grounded;
#[derive(Clone, Copy, Deserialize, Serialize)]
pub enum ColliderShape {
    Ball(f32),
    Cuboid(f32, f32),
    Capsule(f32, f32),
}

#[derive(Component, Copy, Clone, Debug, Reflect)]
pub struct KinematicController {
    pub max_bounces: u32,
    pub skin_width: f32,
}

impl Default for KinematicController {
    fn default() -> Self {
        let skin_width = 0.015;
        let max_bounces = 5;
        Self {
            skin_width,
            max_bounces,
        }
    }
}
pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(PhysicsPlugins::default())
        .add_systems(FixedUpdate, (apply_gravity, collide_n_slide).chain());
}

pub fn apply_gravity(mut kinematics: Query<(&mut LinearVelocity, Option<&GravityScale>), (With<KinematicController>, Without<Grounded>)>, time: Res<Time>) {
    for (mut linear_velocity, scale) in kinematics.iter_mut() {
        if let Some(GravityScale(gravity)) = scale {
            linear_velocity.y -= 90.0 * gravity * time.delta_secs();
        } else {
            linear_velocity.y -= 90.0 * time.delta_secs();
        }
    }
}
//kinematic collide and slide algorithm https://www.youtube.com/watch?v=YR6Q7dUz2uk
pub fn collide_n_slide(
    mut kinematics: Query<(
        Entity,
        &KinematicController,
        &Collider,
        &mut Transform,
        &mut LinearVelocity,
    )>,
    mut spatial_query: SpatialQuery,
    time: Res<Time>,
) {
    for (e, controller, collider, mut transform, mut vel) in kinematics.iter_mut() {
        let mut depth = 0u32;
        let mut pos = transform.translation;
        let mut filter = SpatialQueryFilter::from_excluded_entities([e]);
        vel.0 = move_n_slide_recursive(e, collider, controller, vel.0, transform.translation.xy(), &mut spatial_query, &mut filter, controller.max_bounces);
        //transform.translation += vel.0.extend(0.0) * time.delta_secs();
    }
}

fn move_n_slide_recursive(e: Entity, collider: &Collider, controller: &KinematicController, vel: Vec2, pos: Vec2, spatial_query: &mut SpatialQuery, filter: &mut SpatialQueryFilter, depth: u32) -> Vec2
{
    if (depth <= 0) {
        return Vec2::ZERO;
    }
    let dist = vel.length() + controller.skin_width;

    let Ok(dir) = Dir2::new(vel) else {
        return vel;
    };
    let config = ShapeCastConfig::from_max_distance(dist);
    //exit this loop if it doesn't hit

    let Some(hit) =
        spatial_query.cast_shape(collider, pos.xy(), 0.0, dir, &config, &filter)
    else {
        return vel;
    };

    let mut snap = vel.normalize() * (hit.distance - controller.skin_width);
    let mut leftover = vel - snap;

    //need room for collision to work
    if (snap.length() <= controller.skin_width) {
        snap = Vec2::ZERO;
    }

    let mag = leftover.length();

    leftover = leftover.project_onto(hit.normal1).normalize() * mag;
    snap + move_n_slide_recursive(e, collider, controller, leftover, pos, spatial_query, filter, depth - 1)
}

impl From<ColliderShape> for Collider {
    fn from(value: ColliderShape) -> Self {
        match value {
            ColliderShape::Ball(radius) => Collider::circle(radius),
            ColliderShape::Cuboid(w, h) => Collider::rectangle(w, h),
            ColliderShape::Capsule(hw, hh) => Collider::capsule(hw / 2., hh / 2.),
        }
    }
}
impl Default for ColliderShape {
    fn default() -> Self {
        ColliderShape::Cuboid(20.0, 20.0)
    }
}
