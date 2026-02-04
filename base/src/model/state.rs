use serde::{Deserialize, Serialize};

/// Lifecycle states for versions per LFS-004
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum State {
    /// Default state for new versions.
    Draft,
    /// Pending review.
    Review,
    /// Accepted/vetted version.
    Approved,
    /// Auto-set when a draft is superseded by a new version.
    Discarded,
    /// Locks the object against further updates.
    Sealed,
    /// Deprecated version.
    Archived,
}

impl State {
    /// Check if transition from current state to target state is valid
    pub fn can_transition_to(&self, target: &State) -> bool {
        use State::*;

        match (self, target) {
            // From Draft
            (Draft, Review) => true,
            (Draft, Discarded) => true,
            (Draft, Sealed) => true,
            (Draft, Archived) => true, // Abandon

            // From Review
            (Review, Approved) => true,
            (Review, Draft) => true, // Send back
            (Review, Discarded) => true,
            (Review, Sealed) => true,
            (Review, Archived) => true,

            // From Approved
            (Approved, Sealed) => true,
            (Approved, Archived) => true, // Deprecate

            // From Discarded
            (Discarded, Archived) => true,

            // From Sealed
            // No transitions from Sealed (terminal state)

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
            Draft => vec![Draft, Review, Discarded, Sealed, Archived],
            Review => vec![Review, Draft, Approved, Discarded, Sealed, Archived],
            Approved => vec![Approved, Sealed, Archived],
            Discarded => vec![Discarded, Archived],
            Sealed => vec![Sealed],
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
            State::Discarded => write!(f, "discarded"),
            State::Sealed => write!(f, "sealed"),
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
            "discarded" => Ok(State::Discarded),
            "sealed" => Ok(State::Sealed),
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
        assert!(State::Draft.can_transition_to(&State::Discarded));
        assert!(State::Draft.can_transition_to(&State::Sealed));
        assert!(State::Draft.can_transition_to(&State::Archived));
        assert!(!State::Draft.can_transition_to(&State::Approved));

        // Review transitions
        assert!(State::Review.can_transition_to(&State::Approved));
        assert!(State::Review.can_transition_to(&State::Draft));
        assert!(State::Review.can_transition_to(&State::Discarded));
        assert!(State::Review.can_transition_to(&State::Sealed));
        assert!(State::Review.can_transition_to(&State::Archived));

        // Approved transitions
        assert!(State::Approved.can_transition_to(&State::Sealed));
        assert!(State::Approved.can_transition_to(&State::Archived));
        assert!(!State::Approved.can_transition_to(&State::Draft));
        assert!(!State::Approved.can_transition_to(&State::Review));

        // Discarded transitions
        assert!(State::Discarded.can_transition_to(&State::Archived));
        assert!(!State::Discarded.can_transition_to(&State::Draft));
        assert!(!State::Discarded.can_transition_to(&State::Review));
        assert!(!State::Discarded.can_transition_to(&State::Approved));

        // Sealed transitions
        assert!(!State::Sealed.can_transition_to(&State::Draft));
        assert!(!State::Sealed.can_transition_to(&State::Review));
        assert!(!State::Sealed.can_transition_to(&State::Approved));
        assert!(!State::Sealed.can_transition_to(&State::Archived));

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
        assert!(State::Discarded.can_transition_to(&State::Discarded));
        assert!(State::Sealed.can_transition_to(&State::Sealed));
        assert!(State::Archived.can_transition_to(&State::Archived));
    }

    #[test]
    fn test_from_str() {
        assert_eq!("draft".parse::<State>().unwrap(), State::Draft);
        assert_eq!("review".parse::<State>().unwrap(), State::Review);
        assert_eq!("approved".parse::<State>().unwrap(), State::Approved);
        assert_eq!("discarded".parse::<State>().unwrap(), State::Discarded);
        assert_eq!("sealed".parse::<State>().unwrap(), State::Sealed);
        assert_eq!("archived".parse::<State>().unwrap(), State::Archived);

        assert!("invalid".parse::<State>().is_err());
    }

    #[test]
    fn test_display() {
        assert_eq!(State::Draft.to_string(), "draft");
        assert_eq!(State::Review.to_string(), "review");
        assert_eq!(State::Approved.to_string(), "approved");
        assert_eq!(State::Discarded.to_string(), "discarded");
        assert_eq!(State::Sealed.to_string(), "sealed");
        assert_eq!(State::Archived.to_string(), "archived");
    }
}
