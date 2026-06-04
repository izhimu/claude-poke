use crate::state::PetState;
use std::collections::HashMap;

/// Animation parameters for a single state.
#[derive(Debug, Clone)]
pub struct AnimationDef {
    pub sprite_name: String,
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

        // Default animation definitions — frame_count is auto-detected from the sprite sheet
        animations.insert(
            "Sleeping".to_string(),
            AnimationDef {
                sprite_name: "sleeping".to_string(),
                fps: 1.0,
                color: [100, 100, 200, 255], // blue
            },
        );
        animations.insert(
            "Idle".to_string(),
            AnimationDef {
                sprite_name: "waiting".to_string(),
                fps: 4.0,
                color: [100, 200, 100, 255], // green
            },
        );
        animations.insert(
            "Thinking".to_string(),
            AnimationDef {
                sprite_name: "thinking".to_string(),
                fps: 3.0,
                color: [150, 100, 200, 255], // purple
            },
        );
        animations.insert(
            "Working".to_string(),
            AnimationDef {
                sprite_name: "working".to_string(),
                fps: 8.0,
                color: [200, 150, 50, 255], // orange
            },
        );
        animations.insert(
            "PendingApproval".to_string(),
            AnimationDef {
                sprite_name: "pending_approval".to_string(),
                fps: 2.0,
                color: [200, 200, 100, 255], // yellow-orange
            },
        );
        animations.insert(
            "Notify".to_string(),
            AnimationDef {
                sprite_name: "notify".to_string(),
                fps: 6.0,
                color: [200, 50, 50, 255], // red
            },
        );
        animations.insert(
            "SubAgentWorking".to_string(),
            AnimationDef {
                sprite_name: "subagent".to_string(),
                fps: 6.0,
                color: [200, 200, 50, 255], // yellow
            },
        );
        animations.insert(
            "Error".to_string(),
            AnimationDef {
                sprite_name: "error".to_string(),
                fps: 2.0,
                color: [200, 50, 50, 255], // red
            },
        );

        Self { animations }
    }

    /// Get the animation definition for a given pet state.
    pub fn get(&self, state: &PetState) -> AnimationDef {
        let key = match state {
            PetState::Sleeping => "Sleeping",
            PetState::Idle => "Idle",
            PetState::Thinking => "Thinking",
            PetState::Working => "Working",
            PetState::PendingApproval => "PendingApproval",
            PetState::Notify(_) => "Notify",
            PetState::SubAgentWorking => "SubAgentWorking",
            PetState::Error => "Error",
        };

        self.animations
            .get(key)
            .cloned()
            .unwrap_or_else(|| AnimationDef {
                sprite_name: "unknown".to_string(),
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
