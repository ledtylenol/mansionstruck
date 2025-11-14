use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
#[derive(Default, Bundle, LdtkEntity)]
pub struct PlayerBundle {
    #[sprite_sheet]
    sprite_sheet: Sprite,
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

impl From<&EntityInstance> for ColliderBundle {
    fn from(entity_instance: &EntityInstance) -> Self {
        let rotation_constraints = LockedAxes::ROTATION_LOCKED;

        match entity_instance.identifier.as_ref() {
            "Player" => ColliderBundle {
                collider: Collider::rectangle(6., 14.),
                rb: RigidBody::Dynamic,
                friction: Friction {
                    dynamic_coefficient: 0.0,
                    static_coefficient: 0.0,
                    combine_rule: CoefficientCombine::Min,
                },
                rotation_constraints,
                ..default()
            }
        }
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