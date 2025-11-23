//! Utilities for implementing character controllers.
//! temp here until avian merges this and I dont have to patch it in

pub mod move_and_slide;

/// Re-exports common types related to character controller functionality.
pub mod prelude {
    pub use super::move_and_slide::{
        MoveAndSlide, MoveAndSlideConfig, MoveAndSlideHitData, MoveAndSlideOutput,
    };
}
