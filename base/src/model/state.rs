use serde::{Deserialize, Serialize};

/// Lifecycle states for versions per LFS-004
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum State {
    Draft,
    Review,
    Approved,
    Archived,
}

impl State {
    /// Check if transition from current state to target state is valid
    pub fn can_transition_to(&self, target: &State) -> bool {
        use State::*;

        match (self, target) {
            // From Draft
            (Draft, Review) => true,
            (Draft, Archived) => true, // Abandon

            // From Review
            (Review, Approved) => true,
            (Review, Draft) => true, // Send back
            (Review, Archived) => true,

            // From Approved
            (Approved, Archived) => true, // Deprecate

            // From Archived
            // No transitions from Archived (terminal state)

            // Self-transitions are always valid
            (a, b) if a == b => true,

            // All other transitions are invalid
            _ => false,
        }
    }

    /// Get all valid next states from the current state
    pub fn valid_next_states(&self) -> Vec<State> {
        use State::*;

        match self {
            Draft => vec![Draft, Review, Archived],
            Review => vec![Review, Draft, Approved, Archived],
            Approved => vec![Approved, Archived],
            Archived => vec![Archived],
        }
    }
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Draft => write!(f, "draft"),
            State::Review => write!(f, "review"),
            State::Approved => write!(f, "approved"),
            State::Archived => write!(f, "archived"),
        }
    }
}

impl std::str::FromStr for State {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "draft" => Ok(State::Draft),
            "review" => Ok(State::Review),
            "approved" => Ok(State::Approved),
            "archived" => Ok(State::Archived),
            _ => Err(format!("Invalid state: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_transitions() {
        // Draft transitions
        assert!(State::Draft.can_transition_to(&State::Review));
        assert!(State::Draft.can_transition_to(&State::Archived));
        assert!(!State::Draft.can_transition_to(&State::Approved));

        // Review transitions
        assert!(State::Review.can_transition_to(&State::Approved));
        assert!(State::Review.can_transition_to(&State::Draft));
        assert!(State::Review.can_transition_to(&State::Archived));

        // Approved transitions
        assert!(State::Approved.can_transition_to(&State::Archived));
        assert!(!State::Approved.can_transition_to(&State::Draft));
        assert!(!State::Approved.can_transition_to(&State::Review));

        // Archived is terminal
        assert!(!State::Archived.can_transition_to(&State::Draft));
        assert!(!State::Archived.can_transition_to(&State::Review));
        assert!(!State::Archived.can_transition_to(&State::Approved));
    }

    #[test]
    fn test_self_transitions() {
        assert!(State::Draft.can_transition_to(&State::Draft));
        assert!(State::Review.can_transition_to(&State::Review));
        assert!(State::Approved.can_transition_to(&State::Approved));
        assert!(State::Archived.can_transition_to(&State::Archived));
    }

    #[test]
    fn test_from_str() {
        assert_eq!("draft".parse::<State>().unwrap(), State::Draft);
        assert_eq!("review".parse::<State>().unwrap(), State::Review);
        assert_eq!("approved".parse::<State>().unwrap(), State::Approved);
        assert_eq!("archived".parse::<State>().unwrap(), State::Archived);

        assert!("invalid".parse::<State>().is_err());
    }

    #[test]
    fn test_display() {
        assert_eq!(State::Draft.to_string(), "draft");
        assert_eq!(State::Review.to_string(), "review");
        assert_eq!(State::Approved.to_string(), "approved");
        assert_eq!(State::Archived.to_string(), "archived");
    }
}
