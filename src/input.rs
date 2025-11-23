use crate::mario::Mario;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

#[derive(InputAction)]
#[action_output(f32)]
pub struct Move;
pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(EnhancedInputPlugin)
        .add_input_context::<Mario>();
}
