//! Development tools for the game. This plugin is only enabled in dev builds.

use crate::screens::Screen;
use avian2d::prelude::PhysicsDebugPlugin;
use bevy::{
    dev_tools::states::log_transitions, input::common_conditions::input_just_pressed, prelude::*,
};
use bevy_inspector_egui::bevy_egui::EguiPlugin;

pub(super) fn plugin(app: &mut App) {
    // Log `Screen` state transitions.
    app.add_systems(Update, log_transitions::<Screen>);

    // Toggle the debug overlay for UI.
    app.add_systems(
        Update,
        toggle_debug_ui.run_if(input_just_pressed(TOGGLE_KEY)),
    );
    //inspect stuff and things
    app.add_plugins((
        EguiPlugin::default(),
        bevy_inspector_egui::quick::WorldInspectorPlugin::new(),
        PhysicsDebugPlugin::default(),
    ));
}

const TOGGLE_KEY: KeyCode = KeyCode::Backquote;

fn toggle_debug_ui(mut options: ResMut<UiDebugOptions>) {
    options.toggle();
}
