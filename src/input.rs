use crate::mario::Char;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;

#[derive(Resource, Debug, Reflect, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct InputSettings {
    pub jump: [Binding; 3],
    pub run: [Binding; 3],
    pub respawn: [Binding; 3],
}

impl InputSettings {
    fn read(path: &str) -> Result<Self, Box<dyn Error>> {
        let string = fs::read_to_string(path)?;
        let settings = ron::from_str(&string)?;
        Ok(settings)
    }

    fn write(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let string = ron::ser::to_string_pretty(self, PrettyConfig::default())?;
        fs::write(path, string)?;
        Ok(())
    }

    fn clear(&mut self) {
        self.respawn.fill(Binding::None);
        self.jump.fill(Binding::None);
        self.run.fill(Binding::None);
    }
}

impl Default for InputSettings {
    fn default() -> Self {
        Self {
            jump: [
                KeyCode::Space.into(),
                GamepadButton::South.into(),
                Binding::None,
            ],
            run: [
                KeyCode::ShiftLeft.into(),
                GamepadButton::LeftTrigger.into(),
                Binding::None,
            ],
            respawn: [KeyCode::KeyR.into(), Binding::None, Binding::None],
        }
    }
}
#[derive(InputAction)]
#[action_output(Vec2)]
pub struct Move;

#[derive(InputAction)]
#[action_output(bool)]
pub struct Jump;
#[derive(InputAction)]
#[action_output(bool)]
pub struct Run;
#[derive(InputAction)]
#[action_output(bool)]
pub struct Crouch;

#[derive(InputAction)]
#[action_output(bool)]
pub struct Respawn;

pub(crate) fn plugin(app: &mut App) {
    let input = InputSettings::default();
    app.add_plugins(EnhancedInputPlugin)
        .add_input_context::<Char>();
    let res = InputSettings::read("assets/input.ron");
    match res {
        Ok(settings) => {
            info!("input found! inserting {settings:?}");
            app.insert_resource(settings);
        }
        Err(e) => match e.downcast_ref::<std::io::Error>() {
            Some(e) if e.kind() == std::io::ErrorKind::NotFound => {
                info!("file not found, writing default input");
                let _ = input.write("assets/input.ron");
                app.insert_resource(input);
            }
            Some(e) => {
                warn!("write error {e}");
            }
            None => {
                warn!("unknown error {e}");
            }
        },
    }
}
