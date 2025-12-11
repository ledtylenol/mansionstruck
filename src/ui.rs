use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_cobweb_ui::prelude::*;

#[derive(Component, Default, PartialEq, Reflect)]
struct MainInterface;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(LoadState::Done), build_ui)
        .register_component_type::<MainInterface>();
}
fn spawn_respawn_button(mut c: Commands, mut s: SceneBuilder) {
    c.ui_root()
        .spawn_scene(("ui/main.cob", "respawn_scene"), &mut s, |scene_handle| {
            let entity = scene_handle.id();
            scene_handle.on_pressed(move |mut commands: Commands| {
                commands.get_entity(entity)?.despawn();
                commands.run_system_cached(build_ui);
                OK
            });
        });
}
pub fn build_ui(mut commands: Commands, mut s: SceneBuilder) {
    commands
        .ui_root()
        .spawn_scene(("ui/main.cob", "main_scene"), &mut s, |sc| {
            sc.get("cell::text").update_text("Runtime!");

            for i in 0..=10 {
                sc.spawn_scene(("ui/main.cob", "number_text"), |sc| {
                    sc.edit("cell::text", |sc| {
                        sc.update_text(i.to_string());
                        sc.on_pressed(move || println!("you pressed {i}"));
                    });
                });
            }
            sc.spawn_scene(("ui/main.cob", "despawn_button"), |sc| {
                sc.on_pressed(
                    |mut commands: Commands, interface: Single<Entity, With<MainInterface>>| {
                        commands.get_entity(interface.into_inner())?.despawn();
                        commands.run_system_cached(spawn_respawn_button);
                        OK
                    },
                );
            });
            sc.spawn_scene(("ui/main.cob", "exit_button"), |sc| {
                sc.on_pressed(
                    |mut commands: Commands, interface: Single<Entity, With<PrimaryWindow>>| {
                        commands.get_entity(interface.into_inner())?.despawn();
                        OK
                    },
                );
            });
        });
}
