use crate::mario::Mario;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;

#[derive(Resource, Reflect, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct InputSettings {
    pub left: Vec<Binding>,
    pub right: Vec<Binding>,
    pub jump: Vec<Binding>,
    pub run: Vec<Binding>,
    pub crouch: Vec<Binding>,
}

impl InputSettings {
    fn read(path: &str) -> Result<Self, Box<dyn Error>> {
        let string = fs::read_to_string(path)?;
        let settings = ron::from_str(&string)?;
        Ok(settings)
    }

    fn write(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let string = ron::ser::to_string_pretty(self, Default::default())?;
        fs::write(path, string)?;
        Ok(())
    }

    fn clear(&mut self) {
        self.left.clear();
        self.right.clear();
        self.jump.clear();
        self.run.clear();
        self.crouch.clear();
    }
}

impl Default for InputSettings {
    fn default() -> Self {
        Self {
            left: vec![KeyCode::KeyA.into(), KeyCode::ArrowLeft.into()],
            right: vec![KeyCode::KeyD.into(), KeyCode::ArrowRight.into()],
            jump: vec![KeyCode::Space.into()],
            run: vec![KeyCode::ShiftLeft.into()],
            crouch: vec![KeyCode::KeyS.into(), KeyCode::ArrowDown.into()],
        }
    }
}
#[derive(InputAction)]
#[action_output(f32)]
pub struct Move;

#[derive(InputAction)]
#[action_output(bool)]
pub struct Jump;
pub(crate) fn plugin(app: &mut App) {
    let input = InputSettings::default();
    app.add_plugins(EnhancedInputPlugin)
        .add_input_context::<Mario>();
    let res = InputSettings::read("assets/input.ron");
    match res {
        Ok(settings) => {
            info!("input found! inserting");
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
