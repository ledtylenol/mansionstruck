use crate::camera::{CameraReset, ClampFlags, ClampPosition, FollowAxes, FollowWeight, FollowerOf};
use crate::input::{Crouch, InputSettings, Jump, Move, Run};
use crate::physics::{
    ColliderShape, Grounded, IgnoreGrounded, KinematicController, SlideController,
};
use crate::time::{update_time_since, PausableSystems, TimeSince};
use avian2d::prelude::*;
use bevy::asset::io::Writer;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_enhanced_input::prelude::*;
use ron::ser::PrettyConfig;
use serde::Deserialize;
use std::fs::{read_to_string, File, OpenOptions};
use std::io::Write;
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
pub struct Char {
    pub last_pos: f32,
    pub horizontal_dist: f32,
}
impl Default for Char {
    fn default() -> Self {
        Self {
            last_pos: 0.0,
            horizontal_dist: 0.0,
        }
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
    pub char: CharBundle,
    pub controller: KinematicController,
}

#[derive(Bundle, Default, Deserialize)]
pub struct CharBundle {
    #[serde(default)]
    pub char: Char,
    pub move_stats: MoveStats,
    #[serde(default)]
    pub time_since: TimeSince<Grounded>,
    pub ghost_config: GhostConfig,
    #[serde(skip)]
    pub slide: SlideController,
}
impl From<&EntityInstance> for CharBundle {
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
        .register_ldtk_entity::<PlayerBundle>("Char")
        .register_ldtk_entity::<GoalBundle>("Goal")
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                move_mario,
                update_mario_gravity,
                spawn_ghosts,
                manage_ghosts,
            )
                .chain()
                .in_set(PausableSystems),
        )
        //.add_observer(friction)
        .add_observer(handle_mario_startup)
        .add_observer(reset_camera_limits)
        .add_observer(respawn_level);
}

fn reset_camera_limits(
    _trigger: On<CameraReset>,
    mario: Single<&Transform, With<Char>>,
    mut clamp_pos: Single<&mut ClampPosition>,
) {
    info!("Resetting camera limits");
    clamp_pos.min = mario.translation.xy();
}
fn spawn_ghosts(
    mario_query: Single<
        (&Transform, &Sprite, &GhostConfig, &KinematicController),
        (With<Char>, Without<Grounded>),
    >,
    mut commands: Commands,
    time: Res<Time>,
    mut timer: Local<f32>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let (xf, sprite, &GhostConfig(val), KinematicController { velocity: vel }) =
        mario_query.into_inner();
    let (xf, _sprite) = (xf.clone(), sprite.clone());
    if *timer > val && vel.length() > 100.0 {
        let shape = meshes.add(Annulus::new(30.0, 33.0));
        let color = Color::WHITE;
        let time = rand::random_range(0.5..3.0);
        commands.spawn((
            Mesh2d(shape),
            MeshMaterial2d(materials.add(color)),
            xf,
            Ghost { time, start: time },
            Name::new("Ghost"),
        ));
        *timer = 0.0;
    }
    *timer += time.delta_secs();
}

fn manage_ghosts(
    mut ghost_q: Query<(
        Entity,
        &mut Ghost,
        &mut Mesh2d,
        &MeshMaterial2d<ColorMaterial>,
    )>,
    time: Res<Time>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (e, mut ghost, mut mesh, material) in ghost_q.iter_mut() {
        ghost.time -= time.delta_secs();

        if ghost.time <= 0.0 {
            commands.entity(e).despawn();
        }
        let rel = ghost.time / ghost.start;
        //sprite.color = Color::hsva(ops::sin(ghost.time + ghost.start) * 90.0 + 180.0, 1.0, 1.0, ghost.time);
        mesh.0 = meshes.add(Annulus::new(30.0 * rel.powf(5.0), 33.0 * rel.powf(5.0)));
        if let Some(mut mat) = materials.get_mut(material) {
            mat.color = Color::hsva(
                ops::sin(ghost.time + ghost.start) * 90.0 + 180.0,
                1.0,
                1.0,
                rel,
            );
        }
    }
}

fn update_mario_gravity(
    mut query: Query<(&mut GravityScale, &KinematicController), (With<Char>, Without<Grounded>)>,
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
    level: Single<Entity, (With<LevelIid>, Without<Char>)>,
) {
    commands.entity(level.into_inner()).insert(Respawn);
    info!("respawning level");
    commands.trigger(CameraReset);
}
fn move_mario(
    mario: Single<(&mut KinematicController, &MoveStats, Option<&Grounded>), With<Char>>,
    inputs: Single<&ActionValue, With<Action<Move>>>,
    run: Single<&ActionState, With<Action<Run>>>,
    time: Res<Time>,
) {
    let (mut vel, stats, grounded) = mario.into_inner();
    let &ActionValue::Axis2D(axis) = inputs.into_inner() else {
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
    if axis.length() != 0.0 {
        accel = 350.0;
    }
    vel.velocity = vel.velocity.move_towards(
        axis * speed,
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
    e: On<Add, Char>,
    mut commands: Commands,
    input_settings: Res<InputSettings>,
) {
    commands.entity(e.entity).insert(
        actions!(
        Char[
        (
            Action::<Run>::new(),
            Bindings::spawn(SpawnIter(input_settings.run.into_iter()))
        ),
        (
            Action::<Move>::new(),
            DeadZone::default(),
            Bindings::spawn((
            Cardinal::wasd_keys(),
            Axial::left_stick()
            )),

        ),
        (
            Action::<crate::input::Respawn>::new(),
            Bindings::spawn(SpawnIter(input_settings.respawn.into_iter()))
        ),

    ]
    ));
    commands
        .entity(e.entity)
        .insert(FollowAxes::new(FollowAxes::HORIZONTAL | FollowAxes::VERTICAL));
    let cam = commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scale: 0.35,
            scaling_mode: bevy::camera::ScalingMode::FixedVertical {
                viewport_height: 720.0,
            },
            ..OrthographicProjection::default_2d()
        }),
        //TODO spawn at char location instead?
        Transform::from_xyz(1280.0 / 4.0, 238.0, 0.0),
        //per level camera
        ClampFlags(0),
        //TODO this really should use Option<T> for clamping
        ClampPosition {
            min: vec2(f32::NEG_INFINITY, f32::NEG_INFINITY),
            max: vec2(10000000.0, 10000000.0),
        },
        TransformInterpolation,
    )).id();

    commands.entity(e.entity).insert((FollowerOf(cam), FollowWeight(1)));
    info!("camera spawned");
}
