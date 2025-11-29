use crate::char_controller::prelude::*;
use crate::mario::{JumpStats, Mario};
use avian2d::math::{AdjustPrecision, AsF32};
use avian2d::prelude::*;
use bevy::color::palettes::tailwind;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

#[derive(Component, Default, Clone, Copy)]
pub struct Grounded(pub f32);
#[derive(Clone, Copy, Deserialize, Serialize)]
pub enum ColliderShape {
    Ball(f32),
    Cuboid(f32, f32),
    Capsule(f32, f32),
}

#[derive(Component, Copy, Clone, Debug, Reflect, Default)]
pub struct KinematicController {
    pub velocity: Vec2,
}
pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(PhysicsPlugins::default().with_length_unit(10.0))
        .add_systems(
            FixedUpdate,
            (check_grounded, apply_gravity, perform_move_and_slide).chain(),
        );
}

pub fn apply_gravity(
    mut kinematics: Query<(&mut KinematicController, Option<&GravityScale>, Option<&mut JumpStats>, &Grounded)>,
    time: Res<Time>,
) {
    for (mut controller, scale, stats, grounded) in kinematics.iter_mut() {
        if grounded.0 == 0.0 { continue; }
        let mut gravity = match stats {
            Some(mut t) => t.get_gravity(controller.velocity.y),
            None => JumpStats::default().get_gravity(controller.velocity.y),
        };
        if let Some(GravityScale(scale)) = scale {
            gravity *= scale;
        }
        controller.velocity.y += gravity * time.delta_secs();
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
fn check_grounded(
    mut char: Query<(Entity, &ShapeHits, Option<&mut Grounded>)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, hits, grounded) in char.iter_mut() {
        let is_grounded = hits.iter().count() > 0;

        if is_grounded {
            commands.entity(entity).insert(Grounded(0.0));
        } else if let Some(mut grounded) = grounded {
            grounded.0 += time.delta_secs();
        }
    }
}
fn perform_move_and_slide(
    mut char: Query<(Entity, &Collider, &mut KinematicController, &mut Transform)>,
    move_and_slide: MoveAndSlide,
    time: Res<Time>,
    mut gizmos: Gizmos,
) {
    for (entity, collider, mut controller, mut transform) in char.iter_mut() {
        let velocity = controller.velocity;
        let filter = SpatialQueryFilter::from_excluded_entities([entity]);
        let mut collisions = HashSet::new();
        let out = move_and_slide.move_and_slide(
            collider,
            transform.translation.xy().adjust_precision(),
            transform
                .rotation
                .to_euler(EulerRot::XYZ)
                .2
                .adjust_precision(),
            velocity,
            time.delta(),
            &MoveAndSlideConfig::default(),
            &filter,
            |hit| {
                collisions.insert(hit.entity);
                if hit.intersects() {
                    gizmos.circle_2d(
                        Isometry2d::from_translation(transform.translation.xy()),
                        33.0,
                        tailwind::RED_600,
                    );
                } else {
                    gizmos.arrow_2d(
                        hit.point.f32(),
                        (hit.point
                            + hit.normal.adjust_precision() * hit.collision_distance
                            / time.delta_secs().adjust_precision())
                            .f32(),
                        tailwind::EMERALD_400,
                    );
                }
                true
            },
        );
        transform.translation = out.position.f32().extend(0.0);
        controller.velocity = out.projected_velocity;
        //info!("{} is colliding with entities: {:?}", entity, collisions);
    }
}
