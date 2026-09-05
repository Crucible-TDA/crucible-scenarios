//! What a scenario observes — and what may be shown.
//!
//! Observations are the only channel through which a scenario sees the
//! effects of its operations (transaction results, state snapshots, events,
//! proof verification results, commitment ids). They carry an explicit
//! [`Visibility`] classification:
//!
//! * `Public` — protocol-defined public information (addresses, statuses,
//!   public amounts, verification results). Safe in logs and reports.
//! * `Private` — protocol-confidential values (confidential balances,
//!   transfer amounts, commitment secrets).
//! * `Sensitive` — test infrastructure secrets (tokens, credentials).
//! * `Internal` — harness internals useful for debugging a run.
//!
//! Only `Public` observations are serialized with their value. Everything
//! else serializes as a `[REDACTED]` marker, and `Debug` never prints private
//! values either — so privacy is enforced by the type, not by author
//! discipline. The in-memory [`Observation::value`] accessor remains
//! available to the trusted executor and assertion code, which is exactly
//! where independent oracles need it, and those values never cross the
//! serialization boundary.

use serde::Serialize;

/// Visibility classification of an observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Visibility {
    /// Protocol-defined public information — safe in logs and reports.
    Public,
    /// Protocol-confidential value (balance, amount, commitment secret).
    Private,
    /// Test-infrastructure secret (credentials, endpoints).
    Sensitive,
    /// Harness-internal detail, useful only while debugging the runner.
    Internal,
}

impl Visibility {
    /// Stable machine name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Visibility::Public => "public",
            Visibility::Private => "private",
            Visibility::Sensitive => "sensitive",
            Visibility::Internal => "internal",
        }
    }

    /// Whether values with this classification may enter persistent reports.
    pub const fn is_public(self) -> bool {
        matches!(self, Visibility::Public)
    }
}

impl Default for Visibility {
    fn default() -> Self {
        Visibility::Public
    }
}

impl std::fmt::Display for Visibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The marker that replaces non-public values at serialization time.
pub const REDACTED_MARKER: &str = "[REDACTED]";

/// A simple typed value carried by an observation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum ObservationValue {
    /// Textual value.
    Text(String),
    /// Signed integer value.
    Integer(i64),
    /// Unsigned integer value.
    Unsigned(u64),
    /// Boolean value.
    Boolean(bool),
    /// Structural value (state digests, JSON payloads).
    Object(serde_json::Value),
}

impl ObservationValue {
    /// Wrap a string.
    pub fn text(value: impl Into<String>) -> Self {
        ObservationValue::Text(value.into())
    }

    /// Wrap an unsigned integer.
    pub fn unsigned(value: u64) -> Self {
        ObservationValue::Unsigned(value)
    }
}

impl From<String> for ObservationValue {
    fn from(v: String) -> Self {
        ObservationValue::Text(v)
    }
}

impl From<&str> for ObservationValue {
    fn from(v: &str) -> Self {
        ObservationValue::Text(v.to_string())
    }
}

impl From<u64> for ObservationValue {
    fn from(v: u64) -> Self {
        ObservationValue::Unsigned(v)
    }
}

impl From<i64> for ObservationValue {
    fn from(v: i64) -> Self {
        ObservationValue::Integer(v)
    }
}

impl From<i32> for ObservationValue {
    fn from(v: i32) -> Self {
        ObservationValue::Integer(v as i64)
    }
}

impl From<u32> for ObservationValue {
    fn from(v: u32) -> Self {
        ObservationValue::Unsigned(v as u64)
    }
}

impl From<bool> for ObservationValue {
    fn from(v: bool) -> Self {
        ObservationValue::Boolean(v)
    }
}

/// One observed fact about a scenario run.
#[derive(Clone, PartialEq, Eq)]
pub struct Observation {
    /// Stable key (e.g. `op-1.status`, `op-1.alice.balance`).
    pub key: String,
    /// Visibility classification.
    pub classification: Visibility,
    /// The observed value. In-memory only for non-public observations: never
    /// cross the serialization boundary with a raw private value.
    value: ObservationValue,
}

impl std::fmt::Debug for Observation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Debug renders the safe representation, never a raw private value:
        // a stray `{:?}` in a log or panic message cannot leak confidential
        // data.
        f.debug_struct("Observation")
            .field("key", &self.key)
            .field("classification", &self.classification)
            .field("value", &self.safe_display())
            .finish()
    }
}

impl Observation {
    /// Record a public observation.
    pub fn public(key: impl Into<String>, value: impl Into<ObservationValue>) -> Self {
        Observation {
            key: key.into(),
            classification: Visibility::Public,
            value: value.into(),
        }
    }

    /// Record a private observation (confidential protocol value).
    pub fn private(key: impl Into<String>, value: impl Into<ObservationValue>) -> Self {
        Observation {
            key: key.into(),
            classification: Visibility::Private,
            value: value.into(),
        }
    }

    /// Record a sensitive observation (infrastructure secret).
    pub fn sensitive(key: impl Into<String>, value: impl Into<ObservationValue>) -> Self {
        Observation {
            key: key.into(),
            classification: Visibility::Sensitive,
            value: value.into(),
        }
    }

    /// Record an internal observation (harness detail).
    pub fn internal(key: impl Into<String>, value: impl Into<ObservationValue>) -> Self {
        Observation {
            key: key.into(),
            classification: Visibility::Internal,
            value: value.into(),
        }
    }

    /// The raw value. Trusted scenario code only; never render to output.
    pub fn value(&self) -> &ObservationValue {
        &self.value
    }

    /// The value this observation may safely expose, or a redaction marker
    /// when the observation is not public.
    pub fn safe_display(&self) -> String {
        if self.classification.is_public() {
            match &self.value {
                ObservationValue::Text(t) => t.clone(),
                ObservationValue::Integer(i) => i.to_string(),
                ObservationValue::Unsigned(u) => u.to_string(),
                ObservationValue::Boolean(b) => b.to_string(),
                ObservationValue::Object(o) => o.to_string(),
            }
        } else {
            format!("{REDACTED_MARKER}:{}", self.classification)
        }
    }
}

impl Serialize for Observation {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        // Structural redaction: private/sensitive/internal values are never
        // emitted, only a marker. This is enforced here, at the type, so no
        // downstream serializer can accidentally include the raw value.
        let mut state = serializer.serialize_struct("Observation", 3)?;
        state.serialize_field("key", &self.key)?;
        state.serialize_field("classification", &self.classification)?;
        if self.classification.is_public() {
            state.serialize_field("value", &self.value)?;
        } else {
            state.serialize_field("value", REDACTED_MARKER)?;
        }
        state.end()
    }
}

/// An ordered log of observations for a scenario run.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct ObservationLog {
    observations: Vec<Observation>,
}

impl ObservationLog {
    /// An empty log.
    pub fn new() -> Self {
        ObservationLog {
            observations: Vec::new(),
        }
    }

    /// Append an observation.
    pub fn push(&mut self, observation: Observation) {
        self.observations.push(observation);
    }

    /// Iterate observations in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = &Observation> {
        self.observations.iter()
    }

    /// Number of observations.
    pub fn len(&self) -> usize {
        self.observations.len()
    }

    /// Whether the log is empty.
    pub fn is_empty(&self) -> bool {
        self.observations.is_empty()
    }

    /// Whether every observation is public (used by reporting to decide what
    /// may be written verbatim).
    pub fn all_public(&self) -> bool {
        self.observations.iter().all(|o| o.classification.is_public())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_observations_serialize_with_values() {
        let status = Observation::public("op-1.status", "succeeded");
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"value\":\"succeeded\""));
        assert!(json.contains("\"classification\":\"public\""));
        assert_eq!(status.safe_display(), "succeeded");
    }

    #[test]
    fn private_observations_never_serialize_raw_values() {
        let balance = Observation::private("op-1.alice.balance", 42_000);
        let json = serde_json::to_string(&balance).unwrap();
        assert!(!json.contains("42000"), "private value leaked: {json}");
        assert!(json.contains("\"value\":\"[REDACTED]\""));
        assert_eq!(balance.safe_display(), "[REDACTED]:private");
        // In-memory value remains available to the trusted executor/assertions.
        assert_eq!(balance.value(), &ObservationValue::Integer(42_000));
    }

    #[test]
    fn debug_of_private_observation_is_redacted() {
        let secret = Observation::private("op-1.amount", 987_654);
        let debug = format!("{secret:?}");
        assert!(!debug.contains("987654"), "debug leaked the value: {debug}");
    }

    #[test]
    fn classification_flags_are_exact() {
        assert!(Visibility::Public.is_public());
        for v in [Visibility::Private, Visibility::Sensitive, Visibility::Internal] {
            assert!(!v.is_public());
        }
    }

    #[test]
    fn observation_log_tracks_all_public() {
        let mut log = ObservationLog::new();
        log.push(Observation::public("a", true));
        assert!(log.all_public());
        log.push(Observation::private("b", 1));
        assert!(!log.all_public());
        assert_eq!(log.len(), 2);
        let json = serde_json::to_string(&log).unwrap();
        assert!(!json.contains("\"classification\":\"private\",\"value\":1"));
    }
}
