use bevy::prelude::*;

#[derive(Default, States, Clone, Copy, Ord, PartialOrd, PartialEq, Eq, Hash, Debug)]
pub enum Screen {
    #[default]
    Load,

    Menu,
    Game,
}

pub(crate) fn plugin(app: &mut App) {
    app.init_state::<Screen>();
}
