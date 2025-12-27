use crate::physics::Grounded;
use bevy::prelude::*;
use serde::Deserialize;
use std::marker::PhantomData;
use std::time::Duration;

#[derive(Component, Copy, Clone, Debug, Reflect, Default, Deserialize)]
#[reflect(Component)]
pub struct TimeSince<T> {
    pub time: f32,
    //for generics
    #[reflect(ignore)]
    _phantom: PhantomData<T>,
}
#[derive(Resource, Clone, Debug, Reflect)]
#[reflect(Resource)]
pub struct StopTimer {
    pub timer: Timer,
    pub paused: bool,
}

/// High-level groupings of systems for the app in the `Update` schedule.
/// When adding a new variant, make sure to order it in the `configure_sets`
/// call above.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum AppSystems {
    /// Tick timers.
    TickTimers,
    /// Record player input.
    RecordInput,
    /// Do everything else (consider splitting this into further variants).
    Update,
}

/// Whether or not the game is paused.
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct Pause(pub bool);

/// A system set for systems that shouldn't run while the game is paused.
#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct PausableSystems;

impl StopTimer {
    pub fn tick(&mut self, delta: Duration) {
        self.timer.tick(delta);
    }

    pub fn is_finished(&self) -> bool {
        self.timer.is_finished()
    }
    pub fn just_finished(&self) -> bool {
        self.timer.just_finished()
    }

    pub fn set_duration(&mut self, duration: Duration) {
        self.timer.set_duration(duration);
    }
    pub fn reset(&mut self) {
        self.timer.reset();
    }
}
#[derive(Event, Debug, Reflect, Clone, Copy)]
pub enum TimerEvent {
    Start(Duration),
    Stop,
    Pause,
    Unpause,
}
#[derive(Event)]
pub enum PauseEvent {
    Toggle,
    Enable,
    Disable,
}
pub(crate) fn plugin(app: &mut App) {
    app.insert_resource(StopTimer {
        timer: Timer::from_seconds(
            0.0,
            TimerMode::Once,
        ),
        paused: false,
    })
        .add_systems(Update, tick_pause_timer)
        .add_systems(FixedUpdate, (update_time_since::<Grounded>))
        .add_observer(timer_events)
        .register_type::<StopTimer>()
        .register_type::<TimeSince<Grounded>>()
        .add_observer(handle_pause_event);
}

fn handle_pause_event(obs: On<PauseEvent>, mut virtual_time: ResMut<Time<Virtual>>) {
    match obs.event() {
        PauseEvent::Toggle => {
            let speed = if virtual_time.relative_speed() == 0.0 {
                1.0
            } else {
                0.0
            };
            virtual_time.set_relative_speed(speed);
        }
        PauseEvent::Enable => {
            virtual_time.set_relative_speed(0.0);
        }
        PauseEvent::Disable => {
            virtual_time.set_relative_speed(1.0);
        }
    }
}
fn tick_pause_timer(mut commands: Commands, time: Res<Time<Real>>, mut timer: ResMut<StopTimer>) {
    //don't tick if it's paused
    if timer.paused { return; }
    timer.tick(time.delta());
    if timer.just_finished() {
        commands.trigger(PauseEvent::Disable);
    }
}

fn timer_events(event: On<TimerEvent>, mut commands: Commands, mut timer: ResMut<StopTimer>) {
    info!("timer evented!: {:?}", event.event());
    match event.event() {
        //TODO! customizable overwrite behavior
        &TimerEvent::Start(time) => {
            let duration = timer.timer.duration();
            timer.paused = false;
            timer.set_duration(time.max(duration));
            timer.reset();
            commands.trigger(PauseEvent::Enable);
        }
        &TimerEvent::Stop => {
            timer.paused = true;
            timer.set_duration(Duration::ZERO);
            commands.trigger(PauseEvent::Disable);
        }
        &TimerEvent::Pause => {
            timer.paused = true;
        }
        &TimerEvent::Unpause => {
            timer.paused = false;
        }
    }
}
//generalize yes yes
fn update_time_since<T: Component>(
    mut query: Query<(&mut TimeSince<T>, Option<&T>)>,
    time: Res<Time>,
) {
    for (mut time_since, marker) in query.iter_mut() {
        if marker.is_some() {
            time_since.time = 0.0;
        } else {
            time_since.time += time.delta_secs();
        }
    }
}
