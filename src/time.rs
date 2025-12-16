use bevy::prelude::*;
use std::time::Duration;

#[derive(Resource, Clone, Debug, Reflect)]
#[reflect(Resource)]
pub struct StopTimer {
    pub timer: Timer,
    pub paused: bool,
}

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
pub enum Pause {
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
        .add_observer(timer_events)
        .register_type::<StopTimer>()
        .add_observer(toggle_pause);
}

fn toggle_pause(obs: On<Pause>, mut virtual_time: ResMut<Time<Virtual>>) {
    match obs.event() {
        Pause::Toggle => {
            let speed = if virtual_time.relative_speed() == 0.0 {
                1.0
            } else {
                0.0
            };
            virtual_time.set_relative_speed(speed);
        }
        Pause::Enable => {
            virtual_time.set_relative_speed(0.0);
        }
        Pause::Disable => {
            virtual_time.set_relative_speed(1.0);
        }
    }
}
fn tick_pause_timer(mut commands: Commands, time: Res<Time<Real>>, mut timer: ResMut<StopTimer>) {
    //don't tick if it's paused
    if timer.paused { return; }
    timer.tick(time.delta());
    if timer.just_finished() {
        commands.trigger(Pause::Disable);
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
            commands.trigger(Pause::Enable);
        }
        &TimerEvent::Stop => {
            timer.paused = true;
            timer.set_duration(Duration::ZERO);
            commands.trigger(Pause::Disable);
        }
        &TimerEvent::Pause => {
            timer.paused = true;
        }
        &TimerEvent::Unpause => {
            timer.paused = false;
        }
    }
}
