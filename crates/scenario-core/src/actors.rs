//! Synthetic actors used by scenarios.
//!
//! Scenarios are driven by named, role-carrying actors — Alice, Bob, Carol,
//! Issuer, Admin, Operator, Auditor, Unauthorized. Actor **identities and
//! roles are public protocol metadata**, not secrets: they name who does what,
//! and that naming is intentionally visible so scenarios, assertions, and
//! reports can reason about authorization. What is secret (keys, witnesses,
//! confidential balances) is never attached to an [`Actor`]; real credentials
//! exist only in configured integration/testnet environments and are injected
//! through the scenario context, never stored in scenario definitions or
//! fixtures.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::errors::{Error, Result};

/// A public actor identifier (e.g. `alice`, `issuer`, `auditor-1`).
///
/// Grammar mirrors [`crate::scenario_id::ScenarioId`] but is lowercase: actor
/// identifiers are common nouns, not registry codes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct ActorId(String);

impl ActorId {
    /// Maximum length of an actor identifier.
    pub const MAX_LEN: usize = 64;

    fn validate(raw: &str) -> std::result::Result<(), String> {
        if raw.is_empty() {
            return Err("must not be empty".to_string());
        }
        if raw.len() > Self::MAX_LEN {
            return Err(format!("must be at most {} characters", Self::MAX_LEN));
        }
        let bytes = raw.as_bytes();
        for (i, &b) in bytes.iter().enumerate() {
            if b.is_ascii_lowercase() || b.is_ascii_digit() {
                continue;
            }
            if b == b'-' {
                let prev_ok = i > 0 && (bytes[i - 1].is_ascii_lowercase() || bytes[i - 1].is_ascii_digit());
                let next_ok = i + 1 < bytes.len() && (bytes[i + 1].is_ascii_lowercase() || bytes[i + 1].is_ascii_digit());
                if !(prev_ok && next_ok) {
                    return Err("`-` must separate two alphanumeric runs".to_string());
                }
                continue;
            }
            return Err(format!(
                "character `{}` not allowed (lowercase a-z, 0-9, `-`)",
                raw[i..].chars().next().unwrap_or('?')
            ));
        }
        Ok(())
    }

    /// Construct a validated actor identifier.
    pub fn new(raw: &str) -> Result<Self> {
        Self::validate(raw).map_err(|why| Error::InvalidId(format!("actor `{raw}`"), why))?;
        Ok(ActorId(raw.to_string()))
    }

    /// The identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ActorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::str::FromStr for ActorId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        ActorId::new(s)
    }
}

impl<'de> Deserialize<'de> for ActorId {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        ActorId::new(&raw).map_err(serde::de::Error::custom)
    }
}

/// The authorization posture a scenario actor carries in the scenario model.
///
/// A role is a *declared* posture: scenarios pair roles with capabilities and
/// operations, then assert that the system under test honors or rejects them
/// accordingly. The model never assumes a role grants anything on its own.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Role {
    /// Token issuer — protocol-defined administrative authority.
    Issuer,
    /// Administrative operator of the token or environment.
    Admin,
    /// Routine operations performer.
    Operator,
    /// Read/verify authority (audit trail, public data).
    Auditor,
    /// Ordinary end user.
    #[default]
    User,
    /// A user with no granted authority — used by negative scenarios.
    Unauthorized,
}

impl Role {
    /// Stable machine name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Role::Issuer => "issuer",
            Role::Admin => "admin",
            Role::Operator => "operator",
            Role::Auditor => "auditor",
            Role::User => "user",
            Role::Unauthorized => "unauthorized",
        }
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Role {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "issuer" => Ok(Role::Issuer),
            "admin" => Ok(Role::Admin),
            "operator" => Ok(Role::Operator),
            "auditor" => Ok(Role::Auditor),
            "user" => Ok(Role::User),
            "unauthorized" | "unauthorized-user" => Ok(Role::Unauthorized),
            other => Err(format!("unknown role `{other}`")),
        }
    }
}

/// A scenario actor: public identity, role, and public metadata only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Actor {
    /// Public actor identifier.
    pub id: ActorId,
    /// Declared role.
    pub role: Role,
    /// Optional human-readable label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Arbitrary *public* metadata (e.g. registration state, token links).
    /// Never store keys, seeds, witnesses, or confidential values here.
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

impl Actor {
    /// Create an actor with the given public identity and role.
    pub fn new(id: ActorId, role: Role) -> Self {
        Actor {
            id,
            role,
            display_name: None,
            metadata: BTreeMap::new(),
        }
    }

    /// Attach a human-readable label.
    pub fn with_display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    /// Attach one item of public metadata.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// An ordered collection of the actors declared by a scenario.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActorSet(BTreeMap<ActorId, Actor>);

impl ActorSet {
    /// An empty actor set.
    pub fn new() -> Self {
        ActorSet(BTreeMap::new())
    }

    /// Register an actor. Duplicate registration is an error: a scenario that
    /// declares the same actor twice has a definition bug, not two actors.
    pub fn register(&mut self, actor: Actor) -> Result<()> {
        let id = actor.id.clone();
        if self.0.insert(actor.id.clone(), actor).is_some() {
            return Err(Error::DuplicateId(format!("actor `{id}`")));
        }
        Ok(())
    }

    /// Look up an actor by public id.
    pub fn get(&self, id: &ActorId) -> Option<&Actor> {
        self.0.get(id)
    }

    /// Look up an actor, returning [`Error::UnknownActor`] when absent.
    pub fn require(&self, id: &ActorId) -> Result<&Actor> {
        self.0.get(id).ok_or_else(|| Error::UnknownActor(id.to_string()))
    }

    /// Iterate actors in deterministic (sorted-id) order.
    pub fn iter(&self) -> impl Iterator<Item = &Actor> {
        self.0.values()
    }

    /// Number of declared actors.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether no actors are declared.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl std::fmt::Display for ActorSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let names = self.iter().map(|a| a.id.to_string()).collect::<Vec<_>>().join(", ");
        f.write_str(&names)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actor_ids_are_lowercase_slug_shapes() {
        for id in ["alice", "bob", "issuer", "carol-2", "unauthorized-user"] {
            assert!(ActorId::new(id).is_ok());
        }
        for id in ["", "Alice", "al ice", "alice_1", "-alice", "alice-", "--", "a--b"] {
            assert!(ActorId::new(id).is_err());
        }
    }

    #[test]
    fn roles_parse_and_round_trip() {
        for role in [
            Role::Issuer,
            Role::Admin,
            Role::Operator,
            Role::Auditor,
            Role::User,
            Role::Unauthorized,
        ] {
            assert_eq!(role.to_string().parse::<Role>().unwrap(), role);
        }
        assert_eq!("unauthorized-user".parse::<Role>().unwrap(), Role::Unauthorized);
        assert_eq!(serde_json::to_string(&Role::Auditor).unwrap(), "\"auditor\"");
    }

    #[test]
    fn actor_carries_only_public_data() {
        let alice = Actor::new(ActorId::new("alice").unwrap(), Role::User)
            .with_display_name("Alice")
            .with_metadata("registered", "true");
        assert_eq!(alice.role, Role::User);
        assert_eq!(alice.metadata.get("registered").unwrap(), "true");
    }

    #[test]
    fn actor_set_detects_duplicates() {
        let mut set = ActorSet::new();
        set.register(Actor::new(ActorId::new("alice").unwrap(), Role::User)).unwrap();
        assert!(set.register(Actor::new(ActorId::new("alice").unwrap(), Role::Admin)).is_err());
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn actor_set_require_and_iterate_deterministically() {
        let mut set = ActorSet::new();
        set.register(Actor::new(ActorId::new("bob").unwrap(), Role::User)).unwrap();
        set.register(Actor::new(ActorId::new("alice").unwrap(), Role::User)).unwrap();
        let names: Vec<_> = set.iter().map(|a| a.id.to_string()).collect();
        assert_eq!(names, vec!["alice", "bob"]);
        let missing = ActorId::new("carol").unwrap();
        assert!(set.require(&missing).is_err());
        assert_eq!(set.require(&ActorId::new("bob").unwrap()).unwrap().id.to_string(), "bob");
    }

    #[test]
    fn actor_serde_round_trip() {
        let actor = Actor::new(ActorId::new("issuer").unwrap(), Role::Issuer)
            .with_metadata("token", "ct-1");
        let json = serde_json::to_string(&actor).unwrap();
        let back: Actor = serde_json::from_str(&json).unwrap();
        assert_eq!(back, actor);
    }
}
