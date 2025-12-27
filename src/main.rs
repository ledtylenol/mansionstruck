// Support configuring Bevy lints within code.
#![cfg_attr(bevy_lint, feature(register_tool), register_tool(bevy))]
// Disable console on Windows for non-dev builds.
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

mod asset_tracking;
mod audio;
#[cfg(feature = "dev")]
mod dev_tools;
mod input;
mod mario;
mod physics;
mod screens;
mod ui;
mod walls;

mod camera;
mod char_controller;
mod time;

use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_cobweb_ui::prelude::*;
use seldom_state::prelude::*;
use crate::time::{AppSystems, PausableSystems, Pause};

#[derive(Copy, Clone, Component)]
struct RotateComp;

fn main() -> AppExit {
    App::new().add_plugins(AppPlugin).run()
}

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // Add Bevy plugins.
        app.add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics on web build on itch.
                    // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Window {
                        title: "Projg".to_string(),
                        fit_canvas_to_parent: true,
                        ..default()
                    }
                        .into(),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        );

        // Add other plugins.
        app.add_plugins((
            asset_tracking::plugin,
            audio::plugin,
            screens::plugin,
            ui::plugin,
            input::plugin,
            mario::plugin,
            physics::plugin,
            #[cfg(feature = "dev")]
            dev_tools::plugin,
            CobwebUiPlugin,
            StateMachinePlugin::default(),
            time::plugin,
        ))
            .load("ui/main.cob");

        // Order new `AppSystems` variants by adding them here:
        app.configure_sets(
            Update,
            (
                AppSystems::TickTimers,
                AppSystems::RecordInput,
                AppSystems::Update,
            )
                .chain(),
        );

        // Set up the `Pause` state.
        app.init_state::<Pause>();
        app.configure_sets(Update, PausableSystems.run_if(in_state(Pause(false))));
    }
}
