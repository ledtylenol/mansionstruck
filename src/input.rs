use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
#[derive(Actionlike, Debug, Clone, Copy, Hash, PartialEq, Eq, Reflect)]

pub enum Action {
    #[actionlike(DualAxis)]
    Move,
    Pause,
}

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(InputManagerPlugin::<Action>::default());
}
