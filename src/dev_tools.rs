//! Development tools for the game. This plugin is only enabled in dev builds.

use crate::screens::Screen;
use bevy::input::common_conditions::input_toggle_active;
use bevy::{dev_tools::states::log_transitions, prelude::*};
use bevy_inspector_egui::bevy_egui::{EguiContext, EguiPlugin, PrimaryEguiContext};

pub(super) fn plugin(app: &mut App) {
    // Log `Screen` state transitions.
    app.add_systems(Update, log_transitions::<Screen>);

    // Toggle the debug overlay for UI.
    //inspect stuff and things
    app.add_plugins((
        EguiPlugin::default(),
        bevy_inspector_egui::quick::WorldInspectorPlugin::new()
            .run_if(input_toggle_active(false, TOGGLE_KEY)),
        //PhysicsDebugPlugin::default(),
    ));
}

const TOGGLE_KEY: KeyCode = KeyCode::Backquote;

fn toggle_debug_ui(
    mut options: ResMut<UiDebugOptions>,
    egui: Single<&mut EguiContext, With<PrimaryEguiContext>>,
) {
    options.toggle();
}
