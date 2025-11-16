use crate::physics::ColliderShape;
use avian2d::prelude::*;
use bevy::asset::ron;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use serde::Deserialize;
use std::fs::read_to_string;

#[derive(Default, Bundle, LdtkEntity)]
pub struct PlayerBundle {
    #[sprite_sheet]
    pub sprite_sheet: Sprite,
    #[from_entity_instance]
    pub collider_bundle: ColliderBundle,
    #[from_entity_instance]
    pub entity_instance: EntityInstance,
    #[worldly]
    pub worldly: Worldly,
}

//extra step to convert
#[derive(Clone, Default, Deserialize)]
pub struct ColliderBuilder {
    pub collider: ColliderShape,
    pub rb: RigidBody,
    pub velocity: LinearVelocity,
    #[serde(default)]
    pub rotation_constraints: LockedAxes,
    #[serde(default)]
    pub gravity_scale: GravityScale,
    #[serde(default)]
    pub friction: Friction,
    #[serde(default)]
    pub density: ColliderMassProperties,
}
#[derive(Bundle, Clone, Default, LdtkIntCell)]
pub struct ColliderBundle {
    pub collider: Collider,
    pub rb: RigidBody,
    pub velocity: LinearVelocity,
    pub rotation_constraints: LockedAxes,
    pub gravity_scale: GravityScale,
    pub friction: Friction,
    pub density: ColliderMassProperties,
}

impl From<ColliderShape> for Collider {
    fn from(value: ColliderShape) -> Self {
        match value {
            ColliderShape::Ball(radius) => Collider::circle(radius),
            ColliderShape::Cuboid(w, h) => Collider::rectangle(w, h),
            ColliderShape::Capsule(hw, hh) => Collider::capsule(hw, hh)
        }
    }
}
impl From<ColliderBuilder> for ColliderBundle {
    fn from(ColliderBuilder { collider, rb, velocity, rotation_constraints, gravity_scale, friction, density }: ColliderBuilder) -> Self {
        let collider = collider.into();
        Self {
            collider,
            rb,
            velocity,
            rotation_constraints,
            gravity_scale,
            friction,
            density,
        }
    }
}

impl From<&EntityInstance> for ColliderBundle {
    fn from(entity_instance: &EntityInstance) -> Self {
        error!("I AM HERE)");
        let path = format!("assets/entities/{}.ron", entity_instance.identifier.to_lowercase());
        info!("Looking at path: {path}");
        let Some(str) = read_to_string(path).ok() else {
            warn!("did not find an entity file for the identifier");
            return Self::default();
        };
        //str -> Result<ColliderBuilder> -> ColliderBuilder -> ColliderBundle
        ron::de::from_str::<ColliderBuilder>(&str).map_err(|e| warn!("could not parse {e}")).unwrap_or_default().into()
    }
}
#[derive(Default, Bundle, LdtkEntity)]
pub struct GoalBundle {
    #[sprite_sheet]
    sprite_sheet: Sprite,
}
pub(crate) fn plugin(app: &mut App) {
    app
        .add_plugins(LdtkPlugin)
        .insert_resource(LevelSelection::index(0))
        .register_ldtk_entity::<PlayerBundle>("Mario")
        .register_ldtk_entity::<GoalBundle>("Goal")
        .add_systems(Startup, setup)
    ;
}


fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scale: 0.5,
            ..OrthographicProjection::default_2d()
        }),
        Transform::from_xyz(1280.0 / 4.0, 720.0 / 4.0, 0.0),
    ));

    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("ldtk/mayrio.ldtk").into(),
        ..Default::default()
    });
}