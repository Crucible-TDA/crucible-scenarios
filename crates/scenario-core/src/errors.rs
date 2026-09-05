//! Central error type for the scenario domain model.
//!
//! [`Error`] describes problems *constructing, validating, or describing*
//! scenarios — invalid identifiers, duplicate IDs, unsupported capabilities,
//! malformed definitions. It is deliberately distinct from
//! [`crate::failure::Failure`], which classifies what happened *while
//! executing* a scenario against a system under test. Mixing the two is a
//! common source of confused reports: a scenario-definition mistake is a
//! defect in the test harness, while a classified failure is a finding about
//! the system under test.

use thiserror::Error;

/// Result alias for scenario-domain operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors raised while building or validating scenario artifacts.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    /// A scenario ID, actor ID, or other identifier violates the identifier
    /// grammar (see [`crate::scenario_id::ScenarioId`]).
    #[error("invalid identifier `{0}`: {1}")]
    InvalidId(String, String),

    /// An identifier that must be unique was supplied twice.
    #[error("duplicate identifier `{0}`")]
    DuplicateId(String),

    /// Metadata is missing a required field or holds inconsistent values.
    #[error("invalid scenario metadata: {0}")]
    InvalidMetadata(String),

    /// A scenario references an actor that has not been declared.
    #[error("unknown actor `{0}`")]
    UnknownActor(String),

    /// A scenario references a token that has not been declared.
    #[error("unknown token `{0}`")]
    UnknownToken(String),

    /// An operation references state that has not been declared.
    #[error("unknown state reference `{0}`")]
    UnknownState(String),

    /// An expectation, assertion, or invariant references an unknown target.
    #[error("unknown reference in `{0}`: `{1}`")]
    UnknownReference(&'static str, String),

    /// A capability that is required by a scenario is not available in the
    /// chosen environment or context.
    #[error("capability `{0}` required but unavailable")]
    UnavailableCapability(String),

    /// A scenario requests an execution environment it does not support.
    #[error("environment `{0}` unsupported for this scenario")]
    UnsupportedEnvironment(String),

    /// A seed is malformed or outside the accepted range.
    #[error("invalid seed `{0}`: {1}")]
    InvalidSeed(String, String),

    /// A scenario declares a timeout that is not a positive duration.
    #[error("invalid timeout: {0}")]
    InvalidTimeout(String),

    /// Serialization failed (scenario/vector/result encoding or decoding).
    #[error("serialization error: {0}")]
    Serialization(String),

    /// An internal inconsistency that indicates a bug in this repository
    /// rather than a finding about the system under test.
    #[error("internal error: {0}")]
    Internal(String),
}

impl Error {
    /// Short, stable machine name used by classification and reporting.
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidId(..) => "INVALID_ID",
            Self::DuplicateId(..) => "DUPLICATE_ID",
            Self::InvalidMetadata(..) => "INVALID_METADATA",
            Self::UnknownActor(..) => "UNKNOWN_ACTOR",
            Self::UnknownToken(..) => "UNKNOWN_TOKEN",
            Self::UnknownState(..) => "UNKNOWN_STATE",
            Self::UnknownReference(..) => "UNKNOWN_REFERENCE",
            Self::UnavailableCapability(..) => "UNAVAILABLE_CAPABILITY",
            Self::UnsupportedEnvironment(..) => "UNSUPPORTED_ENVIRONMENT",
            Self::InvalidSeed(..) => "INVALID_SEED",
            Self::InvalidTimeout(..) => "INVALID_TIMEOUT",
            Self::Serialization(..) => "SERIALIZATION",
            Self::Internal(..) => "INTERNAL",
        }
    }
}

/// Shorthand for turning a [`serde_json::Error`] into [`Error::Serialization`].
pub fn serde_err(context: &str) -> impl FnOnce(serde_json::Error) -> Error + '_ {
    move |e| Error::Serialization(format!("{context}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_code_is_stable_and_machine_readable() {
        assert_eq!(
            Error::InvalidId("x".into(), "why".into()).code(),
            "INVALID_ID"
        );
        assert_eq!(Error::DuplicateId("x".into()).code(), "DUPLICATE_ID");
        assert_eq!(Error::Internal("boom".into()).code(), "INTERNAL");
    }

    #[test]
    fn error_displays_human_readable_message() {
        let e = Error::InvalidId("ct-xfer".into(), "must be uppercase".into());
        assert_eq!(
            e.to_string(),
            "invalid identifier `ct-xfer`: must be uppercase"
        );
    }
}
