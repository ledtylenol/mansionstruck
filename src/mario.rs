use crate::input::{Crouch, InputSettings, Jump, Move, Run};
use crate::physics::{ColliderShape, Grounded, KinematicController};
use avian2d::prelude::*;
use bevy::asset::ron;
use bevy::asset::ron::error::SpannedResult;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_enhanced_input::prelude::*;
use serde::Deserialize;
use std::fs::read_to_string;

#[derive(Component, Default)]
pub struct Mario {
    pub time_since_space: f32,
    pub moving: bool,
}
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

    pub mario: Mario,
    pub controller: KinematicController,
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
impl From<ColliderBuilder> for ColliderBundle {
    fn from(
        //pattern matching in function definitions...
        ColliderBuilder {
            collider,
            rb,
            velocity,
            rotation_constraints,
            gravity_scale,
            friction,
            density,
        }: ColliderBuilder,
    ) -> Self {
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
        let path = format!(
            "assets/entities/{}.ron",
            entity_instance.identifier.to_lowercase()
        );
        info!("Looking at path: {path}");
        let Some(str) = read_to_string(path).ok() else {
            warn!("did not find an entity file for the identifier");
            return Self::default();
        };
        //str -> Result<ColliderBuilder> -> ColliderBuilder -> ColliderBundle
        ron::de::from_str::<ColliderBuilder>(&str)
            .map_err(|e| warn!("could not parse {e}"))
            .unwrap_or_default()
            .into()
    }
}
#[derive(Default, Bundle, LdtkEntity)]
pub struct GoalBundle {
    #[sprite_sheet]
    sprite_sheet: Sprite,
}

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(LdtkPlugin)
        .add_plugins(super::walls::WallPlugin)
        .insert_resource(LevelSelection::index(0))
        .register_ldtk_entity::<PlayerBundle>("Mario")
        .register_ldtk_entity::<GoalBundle>("Goal")
        .add_systems(Startup, setup)
        .add_systems(Update, (move_mario).chain())
        .add_observer(jump)
        //.add_observer(friction)
        .add_observer(register_input_map);
}

fn jump(
    _jump: On<Fire<Jump>>,
    mario: Single<(&mut KinematicController, &mut Mario), With<Grounded>>,
) {
    let (mut controller, mut mario) = mario.into_inner();
    controller.velocity.y = 350.0;
    mario.time_since_space = 0.0;
}
fn move_mario(
    mario: Single<&mut KinematicController, With<Mario>>,
    inputs: Single<&ActionValue, With<Action<Move>>>,
    time: Res<Time>,
) {
    let mut vel = mario.into_inner();
    match inputs.into_inner() {
        &ActionValue::Axis1D(axis) => {
            let speed = if axis != 0.0 {
                350.0
            } else {
                650.0
            };
            vel.velocity = vel
                .velocity
                .move_towards(vec2(axis * 50.0, vel.velocity.y), time.delta_secs() * speed);
            info!("axis is {axis}");
        }
        av => {
            info!("action is not axis, it is {av:?}");
        }
    }
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

fn register_input_map(
    e: On<Add, Mario>,
    mut commands: Commands,
    input_settings: Res<InputSettings>,
) {
    commands.entity(e.entity).insert(actions!(
        Mario[(
            Action::<Jump>::new(),
            Bindings::spawn(SpawnIter(input_settings.jump.into_iter()))
        ),
        (
            Action::<Run>::new(),
            Bindings::spawn(SpawnIter(input_settings.run.into_iter()))
        ),
        (
            Action::<Crouch>::new(),
            Bindings::spawn(SpawnIter(input_settings.crouch.into_iter()))
        ),
        (
            Action::<Move>::new(),
            Bindings::spawn((
            Bidirectional::new(input_settings.right[0], input_settings.left[0]),
            Bidirectional::new(input_settings.right[1], input_settings.left[1]),
            Bidirectional::new(input_settings.right[2], input_settings.left[2]),
        ))
        )
    ]
    ));
}
