use crate::char_controller::prelude::*;
use crate::mario::Mario;
use avian2d::math::{AdjustPrecision, AsF32};
use avian2d::prelude::*;
use bevy::platform::collections::HashSet;
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
    mut kinematics: Query<(&mut KinematicController, Option<&GravityScale>), Without<Grounded>>,
    time: Res<Time>,
) {
    for (mut controller, scale) in kinematics.iter_mut() {
        if let Some(GravityScale(gravity)) = scale {
            controller.velocity.y -= 90.0 * gravity * time.delta_secs();
        } else {
            controller.velocity.y -= 90.0 * time.delta_secs();
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
fn check_grounded(
    char: Query<(
        Entity,
        &Collider,
        &Transform,
        Option<&Grounded>,
    )>,
    move_and_slide: MoveAndSlide,
    mut commands: Commands,
) {
    for (entity, collider, transform, grounded) in char.iter() {
        let velocity = vec2(0.0, -1.0);
        let filter = SpatialQueryFilter::from_excluded_entities([entity]);
        let out = move_and_slide.cast_move(
            collider,
            transform.translation.xy().adjust_precision(),
            transform
                .rotation
                .to_euler(EulerRot::XYZ)
                .2
                .adjust_precision(),
            velocity,
            MoveAndSlideConfig::default().skin_width,
            &filter,
        );
        if out.is_none() && grounded.is_some() {
            commands.entity(entity).remove::<Grounded>();
            info!("removing grounded for entity {entity}");
        } else if out.is_some()
            && grounded.is_none()
        {
            info!("applying grounded to entity {entity}");
            commands.entity(entity).insert(Grounded);
        }
    }
}
fn perform_move_and_slide(
    mut char: Query<(Entity, &Collider, &mut KinematicController, &mut Transform)>,
    move_and_slide: MoveAndSlide,
    time: Res<Time>,
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
                true
            },
        );
        transform.translation = out.position.f32().extend(0.0);
        controller.velocity = out.projected_velocity;
        //info!("{} is colliding with entities: {:?}", entity, collisions);
    }
}
