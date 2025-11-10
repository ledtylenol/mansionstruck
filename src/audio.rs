use bevy::prelude::*;
use bevy_fmod::prelude::*;
pub(super) fn plugin(app: &mut App) {
    // fmod how I love you
    app.add_plugins(FmodPlugin::new(&[]));
}
