use super::pet_state::PetState;

pub struct StateMachine {
    current: PetState,
    previous: Option<PetState>,
}

impl StateMachine {
    pub fn new() -> Self {
        Self {
            current: PetState::Sleeping,
            previous: None,
        }
    }

    /// Attempt to transition to a new state.
    /// Returns true if the transition was applied.
    /// Higher priority states override lower ones; equal priority also overrides.
    pub fn transition(&mut self, new_state: PetState) -> bool {
        if self.current == new_state {
            return false;
        }

        let priority = |s: &PetState| match s {
            PetState::Error => 5,
            PetState::Notify(_) => 4,
            PetState::SubAgentWorking => 3,
            PetState::Working => 2,
            PetState::Thinking => 1,
            PetState::Waiting => 0,
            PetState::Sleeping => 0,
            PetState::Stopped => 0,
        };

        if priority(&new_state) >= priority(&self.current) {
            self.previous = Some(self.current.clone());
            self.current = new_state;
            true
        } else {
            false
        }
    }

    pub fn current(&self) -> &PetState {
        &self.current
    }

    #[allow(dead_code)]
    pub fn previous(&self) -> Option<&PetState> {
        self.previous.as_ref()
    }
}

impl Default for StateMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let sm = StateMachine::new();
        assert_eq!(*sm.current(), PetState::Sleeping);
        assert!(sm.previous().is_none());
    }

    #[test]
    fn test_basic_transition() {
        let mut sm = StateMachine::new();
        assert!(sm.transition(PetState::Working));
        assert_eq!(*sm.current(), PetState::Working);
        assert_eq!(*sm.previous().unwrap(), PetState::Sleeping);
    }

    #[test]
    fn test_same_state_no_transition() {
        let mut sm = StateMachine::new();
        assert!(!sm.transition(PetState::Sleeping));
    }

    #[test]
    fn test_higher_priority_overrides() {
        let mut sm = StateMachine::new();
        sm.transition(PetState::Working); // priority 2
        assert!(sm.transition(PetState::Error)); // priority 5
        assert_eq!(*sm.current(), PetState::Error);
    }

    #[test]
    fn test_lower_priority_rejected() {
        let mut sm = StateMachine::new();
        sm.transition(PetState::Error); // priority 5
        assert!(!sm.transition(PetState::Working)); // priority 2
        assert_eq!(*sm.current(), PetState::Error);
    }

    #[test]
    fn test_equal_priority_overrides() {
        let mut sm = StateMachine::new();
        sm.transition(PetState::Working); // priority 2
        assert!(sm.transition(PetState::SubAgentWorking)); // priority 3
        assert_eq!(*sm.current(), PetState::SubAgentWorking);
    }

    #[test]
    fn test_notify_overrides_working() {
        let mut sm = StateMachine::new();
        sm.transition(PetState::Working);
        assert!(sm.transition(PetState::Notify("test".to_string())));
        assert_eq!(*sm.current(), PetState::Notify("test".to_string()));
    }
}
