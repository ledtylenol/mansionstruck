use avian2d::prelude::*;
use bevy::asset::ron;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use serde::Deserialize;
use std::fs::read_to_string;

#[derive(Default, Bundle, LdtkEntity)]
pub struct PlayerBundle {
    #[sprite_sheet]
    sprite_sheet: Sprite,
    #[from_entity_instance]
    collider_bundle: ColliderBundle,
    #[from_entity_instance]
    entity_instance: EntityInstance,
}

#[derive(Bundle, Clone, Default, LdtkIntCell, Deserialize)]
pub struct ColliderBundle {
    pub collider: Collider,
    pub rb: RigidBody,
    pub velocity: LinearVelocity,
    pub rotation_constraints: LockedAxes,
    pub gravity_scale: GravityScale,
    pub friction: Friction,
    pub density: ColliderMassProperties,
}

impl From<&EntityInstance> for ColliderBundle {
    fn from(entity_instance: &EntityInstance) -> Self {
        error!("I AM HERE)");
        let path = format!("assets/entities/{}", entity_instance.identifier.to_lowercase());
        let Some(str) = read_to_string(path).ok() else {
            warn!("did not find an entity file for the identifier");
            return Self::default();
        };
        ron::de::from_str(&str).map_err(|e| warn!("could not parse {e}")).unwrap_or_default()
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
        .add_systems(Startup, setup)
        .insert_resource(LevelSelection::index(0))
        .register_ldtk_entity::<PlayerBundle>("Player")
        .register_ldtk_entity::<GoalBundle>("Goal")
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