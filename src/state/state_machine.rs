use super::pet_state::PetState;
use std::time::Instant;

/// Minimum time a high-priority state (Error=6, Notify=5, PendingApproval=4)
/// must be displayed before allowing a downgrade to a lower-priority state.
const HIGH_PRIORITY_COOLDOWN_MS: u128 = 5000;

pub struct StateMachine {
    current: PetState,
    previous: Option<PetState>,
    /// When the current state was entered.
    entered_at: Instant,
    /// A lower-priority transition deferred during cooldown.
    pending: Option<PetState>,
}

impl StateMachine {
    pub fn new() -> Self {
        Self {
            current: PetState::Sleeping,
            previous: None,
            entered_at: Instant::now(),
            pending: None,
        }
    }

    /// Attempt to transition to a new state.
    ///
    /// - Higher-or-equal priority: applied immediately.
    /// - Lower priority during cooldown: deferred (returns false now, will
    ///   auto-apply when cooldown expires via [`tick`]).
    /// - Lower priority after cooldown: applied immediately.
    pub fn transition(&mut self, new_state: PetState) -> bool {
        if self.current == new_state {
            return false;
        }

        let priority = |s: &PetState| match s {
            PetState::Error => 6,
            PetState::Notify(_) => 5,
            PetState::PendingApproval => 4,
            PetState::SubAgentWorking => 3,
            PetState::Working => 2,
            PetState::Thinking => 1,
            PetState::Idle => 0,
            PetState::Sleeping => 0,
        };

        if priority(&new_state) >= priority(&self.current) {
            // Higher or equal priority: apply immediately, clear any pending.
            self.previous = Some(self.current.clone());
            self.current = new_state;
            self.entered_at = Instant::now();
            self.pending = None;
            true
        } else if self.entered_at.elapsed().as_millis() < HIGH_PRIORITY_COOLDOWN_MS {
            // Still in cooldown: defer the transition.
            self.pending = Some(new_state);
            false
        } else {
            // Cooldown expired: apply immediately.
            self.previous = Some(self.current.clone());
            self.current = new_state;
            self.entered_at = Instant::now();
            self.pending = None;
            true
        }
    }

    /// Call each frame to apply deferred transitions or auto-downgrade
    /// expired high-priority states to Idle.
    pub fn tick(&mut self) {
        if self.entered_at.elapsed().as_millis() < HIGH_PRIORITY_COOLDOWN_MS {
            return;
        }

        // Cooldown expired — apply pending transition if any.
        if let Some(pending) = self.pending.take() {
            self.previous = Some(self.current.clone());
            self.current = pending;
            self.entered_at = Instant::now();
            return;
        }

        // No pending transition: auto-downgrade high-priority states.
        let priority = match &self.current {
            PetState::Error => 6,
            PetState::Notify(_) => 5,
            PetState::PendingApproval => 4,
            _ => return,
        };
        if priority >= 4 {
            self.previous = Some(self.current.clone());
            self.current = PetState::Idle;
            self.entered_at = Instant::now();
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
        assert!(sm.transition(PetState::Error)); // priority 6
        assert_eq!(*sm.current(), PetState::Error);
    }

    #[test]
    fn test_lower_priority_deferred_during_cooldown() {
        let mut sm = StateMachine::new();
        sm.transition(PetState::Error); // priority 6, starts cooldown
        // Immediately try lower priority: should be deferred, not applied.
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

    #[test]
    fn test_idle_transition() {
        let mut sm = StateMachine::new();
        assert!(sm.transition(PetState::Idle));
        assert_eq!(*sm.current(), PetState::Idle);
    }

    #[test]
    fn test_working_overrides_idle() {
        let mut sm = StateMachine::new();
        sm.transition(PetState::Idle); // priority 0
        assert!(sm.transition(PetState::Working)); // priority 2
        assert_eq!(*sm.current(), PetState::Working);
    }

    #[test]
    fn test_thinking_transition() {
        let mut sm = StateMachine::new();
        sm.transition(PetState::Idle);
        assert!(sm.transition(PetState::Thinking)); // priority 1
        assert_eq!(*sm.current(), PetState::Thinking);
    }

    #[test]
    fn test_working_overrides_thinking() {
        let mut sm = StateMachine::new();
        sm.transition(PetState::Thinking); // priority 1
        assert!(sm.transition(PetState::Working)); // priority 2
        assert_eq!(*sm.current(), PetState::Working);
    }

    #[test]
    fn test_pending_approval_overrides_working() {
        let mut sm = StateMachine::new();
        sm.transition(PetState::Working); // priority 2
        assert!(sm.transition(PetState::PendingApproval)); // priority 4
        assert_eq!(*sm.current(), PetState::PendingApproval);
    }

    #[test]
    fn test_higher_overrides_pending_approval() {
        let mut sm = StateMachine::new();
        sm.transition(PetState::PendingApproval); // priority 4
        // Thinking (priority 1) is deferred during cooldown.
        assert!(!sm.transition(PetState::Thinking));
        assert_eq!(*sm.current(), PetState::PendingApproval);
        // But Error (priority 6) overrides immediately.
        assert!(sm.transition(PetState::Error));
        assert_eq!(*sm.current(), PetState::Error);
    }

    // --- Tick / cooldown tests ---

    #[test]
    fn test_tick_noop_within_cooldown() {
        let mut sm = StateMachine::new();
        sm.transition(PetState::Error);
        sm.tick(); // Should do nothing — cooldown hasn't expired.
        assert_eq!(*sm.current(), PetState::Error);
    }

    #[test]
    fn test_tick_applies_pending_after_cooldown() {
        let mut sm = StateMachine::new();
        sm.transition(PetState::PendingApproval);
        // Defer a Thinking transition.
        sm.transition(PetState::Thinking);
        assert_eq!(*sm.current(), PetState::PendingApproval);

        // Simulate cooldown expiry.
        sm.entered_at = Instant::now() - std::time::Duration::from_millis(6000);
        sm.tick();
        assert_eq!(*sm.current(), PetState::Thinking);
    }

    #[test]
    fn test_tick_auto_downgrades_error_to_idle() {
        let mut sm = StateMachine::new();
        sm.transition(PetState::Error);
        assert_eq!(*sm.current(), PetState::Error);

        // Simulate cooldown expiry.
        sm.entered_at = Instant::now() - std::time::Duration::from_millis(6000);
        sm.tick();
        assert_eq!(*sm.current(), PetState::Idle);
    }

    #[test]
    fn test_tick_auto_downgrades_notify_to_idle() {
        let mut sm = StateMachine::new();
        sm.transition(PetState::Notify("test".to_string()));

        sm.entered_at = Instant::now() - std::time::Duration::from_millis(6000);
        sm.tick();
        assert_eq!(*sm.current(), PetState::Idle);
    }

    #[test]
    fn test_tick_auto_downgrades_pending_approval_to_idle() {
        let mut sm = StateMachine::new();
        sm.transition(PetState::PendingApproval);

        sm.entered_at = Instant::now() - std::time::Duration::from_millis(6000);
        sm.tick();
        assert_eq!(*sm.current(), PetState::Idle);
    }

    #[test]
    fn test_tick_no_auto_downgrade_for_low_priority() {
        // Low-priority states (Working, Thinking, Idle) should NOT auto-downgrade.
        let mut sm = StateMachine::new();
        sm.transition(PetState::Working);

        sm.entered_at = Instant::now() - std::time::Duration::from_millis(6000);
        sm.tick();
        assert_eq!(*sm.current(), PetState::Working);
    }

    #[test]
    fn test_higher_priority_clears_pending() {
        let mut sm = StateMachine::new();
        sm.transition(PetState::PendingApproval);
        sm.transition(PetState::Thinking); // Deferred.
        assert!(sm.pending.is_some());

        // Higher priority arrives — should clear the pending state.
        sm.transition(PetState::Error);
        assert!(sm.pending.is_none());
        assert_eq!(*sm.current(), PetState::Error);
    }

    #[test]
    fn test_error_stuck_scenario_fixed() {
        // Reproduce the original bug: Error → PostToolUse(Thinking) → Stop(Idle)
        // All should eventually resolve.
        let mut sm = StateMachine::new();
        sm.transition(PetState::Working);
        sm.transition(PetState::Error); // Tool fails

        // Next events arrive while cooldown is active.
        assert!(!sm.transition(PetState::Thinking)); // Deferred
        assert!(!sm.transition(PetState::Idle)); // Deferred (overwrites Thinking)
        assert_eq!(*sm.current(), PetState::Error);

        // Cooldown expires — pending Idle should be applied.
        sm.entered_at = Instant::now() - std::time::Duration::from_millis(6000);
        sm.tick();
        assert_eq!(*sm.current(), PetState::Idle);
    }
}
