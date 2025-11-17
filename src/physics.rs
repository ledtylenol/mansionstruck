use avian2d::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

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

impl KinematicController {
    //need to exclude the parent so we pass the entity
    pub fn collide_n_slide(
        &self,
        vel: Vec2,
        pos: Vec2,
        depth: u32,
        spatial_query: SpatialQuery,
        exclude: Entity,
    ) -> Vec2 {
        if (depth > self.max_bounces) {
            return Vec2::ZERO;
        }

        return vel;
    }
}

//kinematic collide and slide algorithm https://www.youtube.com/watch?v=YR6Q7dUz2uk
pub fn collide_n_slide(
    mut kinematics: Query<(
        Entity,
        &KinematicController,
        &Collider,
        &Transform,
        &mut LinearVelocity,
    )>,
    spatial_query: SpatialQuery,
) {
    for (e, controller, collider, transform, mut vel) in kinematics.iter_mut() {
        let mut depth = 0u32;
        let mut pos = transform.translation;
        while depth < controller.max_bounces {
            let dist = vel.length() + controller.skin_width;

            let filter = SpatialQueryFilter::from_excluded_entities([e]);
            let Ok(dir) = Dir2::new(vel.0) else {
                continue;
            };
            let config = ShapeCastConfig::from_max_distance(dist);
            //exit this loop if it doesn't hit

            let Some(hit) =
                spatial_query.cast_shape(collider, pos.xy(), 0.0, dir, &config, &filter)
            else {
                break;
            };

            let mut snap = vel.0.normalize() * (hit.distance - controller.skin_width);
            let mut leftover = vel.0 - snap;

            //need room for collision to work
            if (snap.length() <= controller.skin_width) {
                snap = Vec2::ZERO;
            }

            let mag = leftover.length();

            leftover = leftover.project_onto(hit.normal1).normalize() * mag;
            pos += snap.extend(pos.z);
            vel.0 = leftover;
            depth += 1;
        }
    }
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
pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(PhysicsPlugins::default())
        .add_systems(FixedUpdate, collide_n_slide);
}
