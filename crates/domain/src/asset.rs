use thiserror::Error;

/// Lifecycle status of an Asset (source of truth in domain).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetStatus {
    WaitingReview,
    Approved,
    Rejected,
    Duplicate,
    Superseded,
}

impl AssetStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::WaitingReview => "waiting_review",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Duplicate => "duplicate",
            Self::Superseded => "superseded",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "waiting_review" => Some(Self::WaitingReview),
            "approved" => Some(Self::Approved),
            "rejected" => Some(Self::Rejected),
            "duplicate" => Some(Self::Duplicate),
            "superseded" => Some(Self::Superseded),
            _ => None,
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AssetTransitionError {
    #[error("invalid asset transition from {from} via {action}")]
    Invalid {
        from: &'static str,
        action: &'static str,
    },
}

/// Pure transitions for Review actions.
pub fn approve(from: AssetStatus) -> Result<AssetStatus, AssetTransitionError> {
    match from {
        AssetStatus::WaitingReview => Ok(AssetStatus::Approved),
        other => Err(AssetTransitionError::Invalid {
            from: other.as_str(),
            action: "approve",
        }),
    }
}

pub fn reject(from: AssetStatus) -> Result<AssetStatus, AssetTransitionError> {
    match from {
        AssetStatus::WaitingReview => Ok(AssetStatus::Rejected),
        other => Err(AssetTransitionError::Invalid {
            from: other.as_str(),
            action: "reject",
        }),
    }
}

pub fn mark_duplicate(from: AssetStatus) -> Result<AssetStatus, AssetTransitionError> {
    match from {
        AssetStatus::WaitingReview => Ok(AssetStatus::Duplicate),
        other => Err(AssetTransitionError::Invalid {
            from: other.as_str(),
            action: "mark_duplicate",
        }),
    }
}

pub fn supersede(from: AssetStatus) -> Result<AssetStatus, AssetTransitionError> {
    match from {
        AssetStatus::WaitingReview | AssetStatus::Approved => Ok(AssetStatus::Superseded),
        other => Err(AssetTransitionError::Invalid {
            from: other.as_str(),
            action: "supersede",
        }),
    }
}

/// Library may only expose approved assets.
pub fn is_library_visible(status: AssetStatus) -> bool {
    status == AssetStatus::Approved
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approve_only_from_waiting_review() {
        assert_eq!(
            approve(AssetStatus::WaitingReview).unwrap(),
            AssetStatus::Approved
        );
        assert!(approve(AssetStatus::Rejected).is_err());
        assert!(approve(AssetStatus::Approved).is_err());
    }

    #[test]
    fn reject_and_duplicate_only_from_waiting() {
        assert_eq!(
            reject(AssetStatus::WaitingReview).unwrap(),
            AssetStatus::Rejected
        );
        assert_eq!(
            mark_duplicate(AssetStatus::WaitingReview).unwrap(),
            AssetStatus::Duplicate
        );
        assert!(reject(AssetStatus::Approved).is_err());
    }

    #[test]
    fn library_only_approved() {
        assert!(is_library_visible(AssetStatus::Approved));
        assert!(!is_library_visible(AssetStatus::WaitingReview));
        assert!(!is_library_visible(AssetStatus::Rejected));
    }
}
