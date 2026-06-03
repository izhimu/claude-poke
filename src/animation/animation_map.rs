use crate::state::PetState;
use std::collections::HashMap;

/// Animation parameters for a single state.
#[derive(Debug, Clone)]
pub struct AnimationDef {
    pub sprite_name: String,
    pub frame_count: u32,
    pub fps: f32,
    pub color: [u8; 4], // placeholder color when no sprite sheet
}

/// Maps PetState to animation definitions.
pub struct AnimationMap {
    animations: HashMap<String, AnimationDef>,
}

impl AnimationMap {
    pub fn new() -> Self {
        let mut animations = HashMap::new();

        // Default animation definitions matching the design doc
        animations.insert(
            "Sleeping".to_string(),
            AnimationDef {
                sprite_name: "sleeping".to_string(),
                frame_count: 6,
                fps: 1.0,
                color: [100, 100, 200, 255], // blue
            },
        );
        animations.insert(
            "Waiting".to_string(),
            AnimationDef {
                sprite_name: "waiting".to_string(),
                frame_count: 8,
                fps: 4.0,
                color: [100, 200, 100, 255], // green
            },
        );
        animations.insert(
            "Working".to_string(),
            AnimationDef {
                sprite_name: "working".to_string(),
                frame_count: 8,
                fps: 8.0,
                color: [200, 150, 50, 255], // orange
            },
        );
        animations.insert(
            "Thinking".to_string(),
            AnimationDef {
                sprite_name: "thinking".to_string(),
                frame_count: 6,
                fps: 3.0,
                color: [150, 100, 200, 255], // purple
            },
        );
        animations.insert(
            "Notify".to_string(),
            AnimationDef {
                sprite_name: "notify".to_string(),
                frame_count: 6,
                fps: 6.0,
                color: [200, 50, 50, 255], // red
            },
        );
        animations.insert(
            "SubAgentWorking".to_string(),
            AnimationDef {
                sprite_name: "subagent".to_string(),
                frame_count: 8,
                fps: 6.0,
                color: [200, 200, 50, 255], // yellow
            },
        );
        animations.insert(
            "Error".to_string(),
            AnimationDef {
                sprite_name: "error".to_string(),
                frame_count: 4,
                fps: 2.0,
                color: [200, 50, 50, 255], // red
            },
        );
        animations.insert(
            "Stopped".to_string(),
            AnimationDef {
                sprite_name: "stopped".to_string(),
                frame_count: 4,
                fps: 2.0,
                color: [128, 128, 128, 255], // gray
            },
        );

        Self { animations }
    }

    /// Get the animation definition for a given pet state.
    pub fn get(&self, state: &PetState) -> AnimationDef {
        let key = match state {
            PetState::Sleeping => "Sleeping",
            PetState::Waiting => "Waiting",
            PetState::Working => "Working",
            PetState::Thinking => "Thinking",
            PetState::Notify(_) => "Notify",
            PetState::SubAgentWorking => "SubAgentWorking",
            PetState::Error => "Error",
            PetState::Stopped => "Stopped",
        };

        self.animations
            .get(key)
            .cloned()
            .unwrap_or_else(|| AnimationDef {
                sprite_name: "unknown".to_string(),
                frame_count: 1,
                fps: 1.0,
                color: [128, 128, 128, 255],
            })
    }
}

impl Default for AnimationMap {
    fn default() -> Self {
        Self::new()
    }
}
