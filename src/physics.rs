use crate::char_controller::prelude::*;
use crate::mario::JumpStats;
use avian2d::math::{AdjustPrecision, AsF32};
use avian2d::prelude::*;
use bevy::color::palettes::tailwind;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

#[derive(Component, Default, Clone, Copy, Reflect)]
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
            (
                check_grounded,
                apply_gravity,
                perform_move_and_slide,
            )
                .chain(),
        );
}

pub fn apply_gravity(
    mut kinematics: Query<
        (
            &mut KinematicController,
            Option<&GravityScale>,
            Option<&mut JumpStats>,
        ),
        Without<Grounded>,
    >,
    time: Res<Time>,
) {
    for (mut controller, scale, stats) in kinematics.iter_mut() {
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
fn check_grounded(mut char: Query<(Entity, &ShapeHits)>, mut commands: Commands) {
    for (entity, hits) in char.iter_mut() {
        let is_grounded = hits.iter().count() > 0;

        if is_grounded {
            commands.entity(entity).insert(Grounded);
        } else {
            commands.entity(entity).try_remove::<Grounded>();
        }
    }
}
fn perform_move_and_slide(
    mut char: Query<(Entity, &Collider, &mut KinematicController, &mut Transform)>,
    mut tile_q: Query<&mut TileColor>,
    tilemap_q: Single<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapTileSize,
        &TilemapType,
        &TileStorage,
        &TilemapAnchor,
    )>,
    move_and_slide: MoveAndSlide,
    time: Res<Time>,
    #[cfg(feature = "dev")] mut gizmos: Gizmos,
) {
    let (size, grid_size, tile_size, map_type, storage, anchor) = tilemap_q.into_inner();
    for (entity, collider, mut controller, mut transform) in char.iter_mut() {
        let velocity = controller.velocity;
        let filter = SpatialQueryFilter::from_excluded_entities([entity]);
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
            #[cfg(feature = "dev")]
            |hit| {
                if let Some(pos) = TilePos::from_world_pos(&(transform.translation.xy() + controller.velocity.normalize() * 16.0 + vec2(-8.0, -8.0)), size, grid_size, tile_size, map_type, anchor)
                    && let Some(entity) = storage.get(&pos) {
                    info!("hit the tile {entity}", );
                    if let Ok(mut color) = tile_q.get_mut(entity) {
                        color.0 = Color::BLACK;
                    }
                }
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
                info!("{}", hit.entity);
                true
            },
            #[cfg(not(feature = "dev"))]
            |hit| true,
        );
        transform.translation = out.position.f32().extend(transform.translation.z);
        controller.velocity = out.projected_velocity;
        //info!("{} is colliding with entities: {:?}", entity, collisions);
    }
}
