use crate::camera::{CameraReset, ClampFlags, ClampPosition, FollowAxes, FollowerOf};
use crate::input::{Crouch, InputSettings, Jump, Move, Run};
use crate::physics::{ColliderShape, Grounded, KinematicController, TimeSince};
use crate::PausableSystems;
use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_enhanced_input::prelude::*;
use serde::Deserialize;
use std::fs::read_to_string;
use std::time::Duration;


#[derive(Component, Reflect)]
pub struct Ghost {
    time: f32,
    start: f32,
}
#[derive(Component, Reflect, Deserialize)]
pub struct GhostConfig(pub f32);

impl Default for GhostConfig {
    fn default() -> Self {
        GhostConfig(0.1)
    }
}
#[derive(Component, Reflect, Deserialize)]
pub struct Mario {
    pub time_since_space: f32,
    pub last_pos: f32,
    pub horizontal_dist: f32,
    pub jumped: bool,
}
impl Default for Mario {
    fn default() -> Self {
        Self {
            time_since_space: 1000.0,
            last_pos: 0.0,
            horizontal_dist: 0.0,
            jumped: false,
        }
    }
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
#[derive(Component, Reflect, Deserialize, Clone, Debug)]
pub struct MoveStats {
    pub move_speed: f32,
    pub run_speed: f32,
}
impl Default for MoveStats {
    fn default() -> Self {
        MoveStats {
            move_speed: 75.0,
            run_speed: 135.0,
        }
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

    #[from_entity_instance]
    pub mario: MarioBundle,
    pub controller: KinematicController,
}

#[derive(Bundle, Default, Deserialize)]
pub struct MarioBundle {
    #[serde(default)]
    pub mario: Mario,
    pub move_stats: MoveStats,
    pub jump_stats: JumpStats,
    #[serde(default)]
    pub time_since: TimeSince<Grounded>,
    pub ghost_config: GhostConfig,
}
impl From<&EntityInstance> for MarioBundle {
    fn from(entity_instance: &EntityInstance) -> Self {
        let path = format!(
            "assets/entities/{}/entity.ron",
            entity_instance.identifier.to_lowercase()
        );
        info!("Looking at path: {path}");
        let Some(str) = read_to_string(path).ok() else {
            warn!("did not find an entity file for the identifier");
            return Self::default();
        };
        //str -> Result<ColliderBuilder> -> ColliderBuilder -> ColliderBundle
        ron::de::from_str::<_>(&str)
            .map_err(|e| {
                warn!("could not parse {e}");
                e
            })
            .unwrap_or_default()
    }
}
//extra step to convert
#[derive(Clone, Default, Deserialize)]
pub struct ColliderBuilder {
    pub collider: ColliderShape,
    pub rb: RigidBody,
    pub shape_caster: ShapeCasterBuilder,
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
    pub shape_caster: ShapeCaster,
    pub rotation_constraints: LockedAxes,
    pub gravity_scale: GravityScale,
    pub friction: Friction,
    pub grounded: Grounded,
}
#[derive(Deserialize, Clone)]
pub struct ShapeCasterBuilder {
    pub dir: Dir2,
    pub distance: f32,
}

impl ShapeCasterBuilder {
    fn to_shape_cast(&self, collider: &Collider) -> ShapeCaster {
        ShapeCaster::new(collider.clone(), Vec2::ZERO, 0.0, self.dir)
            .with_max_distance(self.distance)
    }
}
impl Default for ShapeCasterBuilder {
    fn default() -> Self {
        Self {
            dir: Dir2::from_xy_unchecked(0.0, -1.0),
            distance: 1.0,
        }
    }
}
impl From<ColliderBuilder> for ColliderBundle {
    fn from(
        //pattern matching in function definitions...
        ColliderBuilder {
            collider,
            rb,
            shape_caster,
            rotation_constraints,
            gravity_scale,
            friction,
        }: ColliderBuilder,
    ) -> Self {
        let collider = collider.into();
        let shape_caster = shape_caster.to_shape_cast(&collider);
        Self {
            collider,
            rb,
            shape_caster,
            rotation_constraints,
            gravity_scale,
            friction,
            ..default()
        }
    }
}
impl From<&EntityInstance> for ColliderBundle {
    fn from(entity_instance: &EntityInstance) -> Self {
        let path = format!(
            "assets/entities/{}/collider.ron",
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
        .add_plugins((super::walls::WallPlugin, crate::camera::plugin))
        .insert_resource(LevelSelection::index(0))
        .register_ldtk_entity::<PlayerBundle>("Mario")
        .register_ldtk_entity::<GoalBundle>("Goal")
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (move_mario, jump, update_sprite, update_mario_gravity, spawn_ghosts, manage_ghosts)
                .chain()
                .in_set(PausableSystems),
        )
        //.add_observer(friction)
        .add_observer(handle_mario_startup)
        .add_observer(respawn_level);
}

fn update_sprite(
    mario: Single<(
        &mut Sprite,
        &mut Mario,
        &Transform,
        &KinematicController,
        Option<&Grounded>,
    )>,
) {
    let (mut sprite, mut mario, tf, controller, grounded) = mario.into_inner();
    let axis = controller.velocity.x;
    let Some(atlas) = &mut sprite.texture_atlas else {
        return;
    };
    if grounded.is_some() {
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
        mario.horizontal_dist = 0.0;
        atlas.index = 5;
    }

    mario.last_pos = tf.translation.x;
}
fn jump(
    jump: Single<(&ActionEvents, &ActionTime), With<Action<Jump>>>,
    mario: Single<(
        &mut KinematicController,
        &mut Mario,
        &mut JumpStats,
        &mut TimeSince<Grounded>,
    )>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let (mut controller, mut mario, mut stats, mut time_since) = mario.into_inner();
    if time_since.time == 0.0 && mario.jumped {
        mario.jumped = false;
        return;
    }
    let (&state, &ActionTime { elapsed_secs, .. }) = jump.into_inner();
    if state.contains(ActionEvents::STARTED)
        || state.contains(ActionEvents::ONGOING) && elapsed_secs < 0.1
    {
        mario.time_since_space = 0.0;
    } else {
        mario.time_since_space += time.delta_secs();
    }
    //don't jump if it's been 0.1s
    //TODO: hardcoded for now, make them components?
    if mario.time_since_space >= 0.1 {
        return;
    }
    if time_since.time >= 0.1 {
        return;
    }
    controller.velocity.y = stats.get_jump_velocity();
    mario.time_since_space = 0.1;
    time_since.time = 0.1;
    //if we dont have an atlas something went very very wrong
    mario.jumped = true;
    commands.trigger(crate::time::TimerEvent::Start(Duration::from_secs_f32(0.1)));
}

fn spawn_ghosts(
    mario_query: Single<(&Transform, &Sprite, &GhostConfig, &KinematicController), (With<Mario>, Without<Grounded>)>,
    mut commands: Commands,
    time: Res<Time>,
    mut timer: Local<f32>,
) {
    let (xf, sprite, &GhostConfig(val), KinematicController { velocity: vel }) = mario_query.into_inner();
    let (xf, sprite) = (xf.clone(), sprite.clone());
    if *timer > val && vel.length() > 100.0 {
        commands.spawn(
            (
                sprite,
                xf,
                Ghost { time: 1.0, start: rand::random_range(-10.0..10.0) },
                Name::new("Ghost")
            )
        );
        *timer = 0.0;
    }
    *timer += time.delta_secs();
}

fn manage_ghosts(
    mut ghost_q: Query<(Entity, &mut Ghost, &mut Sprite)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (e, mut ghost, mut sprite) in ghost_q.iter_mut() {
        ghost.time -= time.delta_secs();
        sprite.color = Color::hsva(ops::sin(ghost.time + ghost.start) * 180.0 + 180.0, ops::cos(ghost.time + ghost.start) * 0.5 + 0.5, 1.0, ghost.time);
        if ghost.time <= 0.0 {
            commands.entity(e).despawn();
        }
    }
}

fn update_mario_gravity(
    mut query: Query<(&mut GravityScale, &KinematicController), (With<Mario>, Without<Grounded>)>,
    jump_query: Query<&mut ActionState, With<Action<Jump>>>,
) {
    let jump_pressed = jump_query.iter().any(|&jump| jump == ActionState::Fired);
    for (mut scale, controller) in query.iter_mut() {
        if !jump_pressed && controller.velocity.y > 0.0 {
            scale.0 = 2.0;
        } else {
            scale.0 = 1.0;
        }
    }
}
fn respawn_level(
    _trigger: On<Start<crate::input::Respawn>>,
    mut commands: Commands,
    level: Single<Entity, With<LevelIid>>,
) {
    commands.entity(level.into_inner()).insert(Respawn);
    info!("respawning level");
    commands.trigger(CameraReset);
}
fn move_mario(
    mario: Single<(&mut KinematicController, &MoveStats, Option<&Grounded>), With<Mario>>,
    inputs: Single<&ActionValue, With<Action<Move>>>,
    run: Single<&ActionState, With<Action<Run>>>,
    time: Res<Time>,
) {
    let (mut vel, stats, grounded) = mario.into_inner();
    let &ActionValue::Axis1D(axis) = inputs.into_inner() else {
        return;
    };
    let speed = if *run.into_inner() == ActionState::Fired {
        stats.run_speed
    } else {
        stats.move_speed
    };
    let mut accel = 650.0;
    if grounded.is_none() {
        accel = 0.0;
    }
    if axis != 0.0 {
        accel = 350.0;
    }
    vel.velocity = vel.velocity.move_towards(
        vec2(axis * speed, vel.velocity.y),
        time.delta_secs() * accel,
    );
}
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("ldtk/mayrio.ldtk").into(),
        ..Default::default()
    });
}

fn handle_mario_startup(
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
        ),
        (
            Action::<crate::input::Respawn>::new(),
            Bindings::spawn(SpawnIter(input_settings.respawn.into_iter()))
        ),

    ]
    ));
    commands
        .entity(e.entity)
        .insert(FollowAxes::new(FollowAxes::HORIZONTAL));
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scale: 0.35,
            scaling_mode: bevy::camera::ScalingMode::FixedVertical {
                viewport_height: 720.0,
            },
            ..OrthographicProjection::default_2d()
        }),
        //TODO spawn at mario location instead?
        Transform::from_xyz(1280.0 / 4.0, 238.0, 0.0),
        FollowerOf(e.entity),
        //per level camera
        ClampFlags(ClampFlags::MIN_X),
        //TODO this really should use Option<T> for clamping
        ClampPosition {
            min: vec2(f32::NEG_INFINITY, f32::NEG_INFINITY),
            max: vec2(10000000.0, 10000000.0),
        },
        TransformInterpolation,
    ));
}
