use crate::input::{Crouch, InputSettings, Jump, Move, Run};
use crate::physics::{ColliderShape, Grounded, KinematicController};
use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_enhanced_input::prelude::*;
use serde::Deserialize;
use std::fs::read_to_string;

#[derive(Component, Default, Reflect)]
pub struct Mario {
    pub time_since_space: f32,
    pub last_pos: f32,
    pub horizontal_dist: f32,
    pub ping_pong: i32,
}

impl Mario {
    fn get_next_sprite_pos(&mut self, idx: i32) -> i32 {
        (idx + 1) % 3 + 1
    }
}
#[derive(Component, Reflect, Deserialize, Clone, Debug)]
pub struct JumpStats {
    jump_time: f32,
    fall_time: f32,
    jump_height: f32,
    #[serde(default)]
    jump_velocity: Option<f32>,
    #[serde(default)]
    jump_gravity: Option<f32>,
    #[serde(default)]
    fall_gravity: Option<f32>,
}
impl JumpStats {
    pub fn new(jump_height: f32, jump_time: f32, fall_time: f32) -> Self {
        let jump_velocity = Some((2.0 * jump_height) / jump_time);
        let jump_gravity = Some((-2.0 * jump_height) / (jump_time * jump_time));
        let fall_gravity = Some((-2.0 * jump_height) / (fall_time * fall_time));

        let stats = Self {
            jump_time,
            fall_time,
            jump_height,
            jump_velocity,
            jump_gravity,
            fall_gravity,
        };
        info!("Jump stats made: {stats:?} ");
        stats
    }

    pub fn get_gravity(&mut self, y_vel: f32) -> f32 {
        if self.jump_gravity.is_none() || self.fall_gravity.is_none() {
            self.calculate_params();
        }
        //should never fail
        if y_vel > 0.0 {
            self.jump_gravity.unwrap_or(-160.0)
        } else {
            self.fall_gravity.unwrap_or(-160.0)
        }
    }
    pub fn get_jump_velocity(&mut self) -> f32 {
        if self.jump_velocity.is_none() {
            self.calculate_params();
        }
        //should also never fail unless something goes catastrophically wrong
        self.jump_velocity.unwrap_or(2.0 * 80.0)
    }

    pub fn set_height(&mut self, jump_height: f32) {
        self.jump_height = jump_height;
        self.calculate_params();
    }

    pub fn set_jump_time(&mut self, jump_time: f32) {
        self.jump_time = jump_time;
        self.calculate_params();
    }

    pub fn set_fall_time(&mut self, fall_time: f32) {
        self.fall_time = fall_time;
        self.calculate_params();
    }

    fn calculate_params(&mut self) {
        self.jump_velocity = Some((2.0 * self.jump_height) / self.jump_time);
        self.jump_gravity = Some((-2.0 * self.jump_height) / (self.jump_time * self.jump_time));
        self.fall_gravity = Some((-2.0 * self.jump_height) / (self.fall_time * self.fall_time));
    }
}
impl Default for JumpStats {
    fn default() -> Self {
        Self::new(80.0, 1.0, 1.0)
    }
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
    //TODO remove this as we dont use it
    pub velocity: LinearVelocity,
    pub jump_stats: JumpStats,
    #[serde(default)]
    pub rotation_constraints: LockedAxes,
    //get_gravity * scale
    #[serde(default)]
    pub gravity_scale: GravityScale,
    #[serde(default)]
    pub friction: Friction,
}
#[derive(Bundle, Clone, Default, LdtkIntCell)]
pub struct ColliderBundle {
    pub collider: Collider,
    pub rb: RigidBody,
    pub velocity: LinearVelocity,
    pub jump_stats: JumpStats,
    pub rotation_constraints: LockedAxes,
    pub gravity_scale: GravityScale,
    pub friction: Friction,
}
impl From<ColliderBuilder> for ColliderBundle {
    fn from(
        //pattern matching in function definitions...
        ColliderBuilder {
            collider,
            rb,
            velocity,
            jump_stats,
            rotation_constraints,
            gravity_scale,
            friction,
        }: ColliderBuilder,
    ) -> Self {
        let collider = collider.into();
        Self {
            collider,
            rb,
            velocity,
            jump_stats,
            rotation_constraints,
            gravity_scale,
            friction,
        }
    }
}
impl From<&EntityInstance> for ColliderBundle {
    fn from(entity_instance: &EntityInstance) -> Self {
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
            .map_err(|e| {
                warn!("could not parse {e}");
                e
            })
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
        .add_systems(Update, (move_mario, update_sprite).chain())
        .add_observer(jump)
        //.add_observer(friction)
        .add_observer(register_input_map);
}

fn update_sprite(
    mario: Single<(&mut Sprite, &mut Mario, &Transform, Option<&Grounded>)>,
    input: Single<&ActionValue, With<Action<Move>>>,
) {
    let (mut sprite, mut mario, tf, grounded) = mario.into_inner();
    let &ActionValue::Axis1D(axis) = input.into_inner() else {
        return;
    };
    if grounded.is_some() {
        let Some(atlas) = &mut sprite.texture_atlas else {
            return;
        };
        if axis != 0.0 {
            mario.horizontal_dist += (tf.translation.x - mario.last_pos).abs();
            if mario.horizontal_dist > 5.0 {
                atlas.index = mario.get_next_sprite_pos(atlas.index as i32) as usize;
                mario.horizontal_dist = 0.0
            }
            sprite.flip_x = axis < 0.0;
        } else {
            atlas.index = 0;
        }
    } else {
        mario.horizontal_dist = 0.0
    }

    mario.last_pos = tf.translation.x;
}
fn jump(
    _jump: On<Fire<Jump>>,
    mario: Single<(&mut KinematicController, &mut Mario, &mut JumpStats), With<Grounded>>,
) {
    let (mut controller, mut mario, mut stats) = mario.into_inner();
    controller.velocity.y = stats.get_jump_velocity();
    mario.time_since_space = 0.0;
}
fn move_mario(
    mario: Single<(&mut KinematicController, Option<&Grounded>), With<Mario>>,
    inputs: Single<&ActionValue, With<Action<Move>>>,
    time: Res<Time>,
) {
    let (mut vel, grounded) = mario.into_inner();
    match inputs.into_inner() {
        &ActionValue::Axis1D(axis) => {
            let mut speed = 650.0;
            if grounded.is_none() {
                speed = 0.0;
            }
            if axis != 0.0 {
                speed = 350.0;
            }
            vel.velocity = vel
                .velocity
                .move_towards(vec2(axis * 50.0, vel.velocity.y), time.delta_secs() * speed);
        }
        _ => (),
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
