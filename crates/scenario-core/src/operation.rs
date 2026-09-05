//! The operation vocabulary of a Confidential Token scenario.
//!
//! Operations are **intents**, not implementations: an [`Operation`] says
//! *who* asked *what* about *which token*, in types the scenario model can
//! carry, replay, and assert over. Translating an operation into a simulator
//! call, a Soroban transaction, or a proof request is the job of the adapter
//! crates — never of this module. This crate must not encode protocol
//! semantics beyond the shape of the workflow (register, deposit, merge,
//! confidential transfer, withdraw), because implementing those semantics is
//! exactly what crucible-simulator and crucible-prover own.
//!
//! Amounts that the Confidential Token protocol keeps private — confidential
//! transfer amounts — ride on [`ConfidentialAmount`], whose `Debug`/`Display`
//! render only a redaction marker so a stray `println!("{op:?}")` cannot leak
//! a private amount into a log or CI artifact. Scenario *definition* files
//! may still carry the expected value (a test author knows it); runtime
//! results and observations must not, which the observation and reporting
//! layers enforce.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::actors::ActorId;
use crate::errors::{Error, Result};

/// A public token identifier within a scenario (e.g. `ct-usdc`).
///
/// Tokens are named by the scenario; whether the token actually exists in the
/// system under test is a question for fixtures and adapters, not for this
/// newtype.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TokenId(String);

impl TokenId {
    /// Construct a validated token identifier (lowercase slug).
    pub fn new(raw: &str) -> Result<Self> {
        ActorId::new(raw)
            .map(|_| TokenId(raw.to_string()))
            .map_err(|_| {
                Error::InvalidId(
                    format!("token `{raw}`"),
                    "expected lowercase a-z, 0-9, `-`".to_string(),
                )
            })
    }

    /// The identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for TokenId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::str::FromStr for TokenId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        TokenId::new(s)
    }
}

impl Serialize for TokenId {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for TokenId {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        TokenId::new(&raw).map_err(serde::de::Error::custom)
    }
}

/// An operation's stable identifier within a scenario (e.g. `op-1`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OperationId(String);

impl OperationId {
    /// Construct a validated operation identifier.
    pub fn new(raw: &str) -> Result<Self> {
        ActorId::new(raw)
            .map(|_| OperationId(raw.to_string()))
            .map_err(|_| {
                Error::InvalidId(
                    format!("operation `{raw}`"),
                    "expected lowercase a-z, 0-9, `-`".to_string(),
                )
            })
    }

    /// The identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for OperationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::str::FromStr for OperationId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        OperationId::new(s)
    }
}

impl Serialize for OperationId {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for OperationId {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        OperationId::new(&raw).map_err(serde::de::Error::custom)
    }
}

/// A public amount (deposits, withdrawals, and public balances).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Amount(u64);

impl Amount {
    /// The zero amount.
    pub const ZERO: Amount = Amount(0);

    /// Construct an amount.
    pub const fn new(value: u64) -> Self {
        Amount(value)
    }

    /// The raw value.
    pub const fn get(self) -> u64 {
        self.0
    }

    /// Whether this is zero.
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }
}

impl std::fmt::Display for Amount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::ops::Add for Amount {
    type Output = Amount;

    fn add(self, rhs: Amount) -> Amount {
        Amount(self.0.saturating_add(rhs.0))
    }
}

impl std::ops::Sub for Amount {
    type Output = Amount;

    fn sub(self, rhs: Amount) -> Amount {
        Amount(self.0.saturating_sub(rhs.0))
    }
}

/// An amount the Confidential Token protocol keeps private.
///
/// `Debug` and `Display` render only a redaction marker. Serialization is
/// plain so *scenario definition files* (which a test author writes with the
/// expected value in hand) can persist it; the observation and reporting
/// layers redact confidential values before anything reaches a log, event, or
/// report.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ConfidentialAmount(u64);

impl ConfidentialAmount {
    /// Construct a confidential amount.
    pub const fn new(value: u64) -> Self {
        ConfidentialAmount(value)
    }

    /// The raw value. Only trusted scenario code (the executor and the
    /// in-memory oracle) may read this; never render it to output.
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl std::fmt::Debug for ConfidentialAmount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[redacted-confidential-amount]")
    }
}

impl std::fmt::Display for ConfidentialAmount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[redacted]")
    }
}

/// The typed workflow step a scenario performs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationKind {
    /// Register an account/actor for confidential use.
    Register {
        /// The account being registered.
        account: ActorId,
    },
    /// Move a public amount into confidential balance for `recipient`.
    Deposit {
        /// Token being deposited into.
        token: TokenId,
        /// Whose confidential balance receives the funds.
        recipient: ActorId,
        /// Public deposit amount.
        amount: Amount,
    },
    /// Consolidate confidential balances of `owner`.
    Merge {
        /// Token whose balances are merged.
        token: TokenId,
        /// Owner of the balances being merged.
        owner: ActorId,
    },
    /// Confidential transfer between registered users.
    Transfer {
        /// Token being transferred.
        token: TokenId,
        /// Sender of the funds.
        sender: ActorId,
        /// Recipient of the funds.
        recipient: ActorId,
        /// Confidential (private) transfer amount.
        amount: ConfidentialAmount,
    },
    /// Move confidential balance back into a public balance.
    Withdraw {
        /// Token being withdrawn from.
        token: TokenId,
        /// Owner withdrawing.
        owner: ActorId,
        /// Public withdrawal amount.
        amount: Amount,
    },
}

impl OperationKind {
    /// Coarse operation family (matches [`crate::tags::standard`]).
    pub const fn family(&self) -> &'static str {
        match self {
            OperationKind::Register { .. } => "register",
            OperationKind::Deposit { .. } => "deposit",
            OperationKind::Merge { .. } => "merge",
            OperationKind::Transfer { .. } => "transfer",
            OperationKind::Withdraw { .. } => "withdraw",
        }
    }

    /// Whether this operation carries a confidential (private) amount.
    pub const fn carries_confidential_amount(&self) -> bool {
        matches!(self, OperationKind::Transfer { .. })
    }

    /// The token the operation touches, when it touches one.
    pub fn token(&self) -> Option<&TokenId> {
        match self {
            OperationKind::Deposit { token, .. }
            | OperationKind::Merge { token, .. }
            | OperationKind::Transfer { token, .. }
            | OperationKind::Withdraw { token, .. } => Some(token),
            OperationKind::Register { .. } => None,
        }
    }
}

/// A single workflow step: who performs which kind of operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Operation {
    /// Stable identifier within the scenario (ordering reference).
    pub id: OperationId,
    /// The actor performing this operation.
    pub actor: ActorId,
    /// What is being done.
    pub kind: OperationKind,
    /// Optional public context keys (e.g. fixture/state references). Never
    /// store secret material here.
    #[serde(default)]
    pub context: BTreeMap<String, String>,
}

impl Operation {
    /// Construct an operation.
    pub fn new(id: OperationId, actor: ActorId, kind: OperationKind) -> Self {
        Operation {
            id,
            actor,
            kind,
            context: BTreeMap::new(),
        }
    }

    /// Attach a public context key.
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    /// Whether the operation carries a confidential amount.
    pub fn is_confidential(&self) -> bool {
        self.kind.carries_confidential_amount()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alice() -> ActorId {
        ActorId::new("alice").unwrap()
    }

    fn bob() -> ActorId {
        ActorId::new("bob").unwrap()
    }

    fn token() -> TokenId {
        TokenId::new("ct-usdc").unwrap()
    }

    #[test]
    fn identifiers_validate_shapes() {
        assert!(TokenId::new("ct-usdc").is_ok());
        assert!(TokenId::new("CT-USDC").is_err());
        assert!(OperationId::new("op-1").is_ok());
        assert!(OperationId::new("op 1").is_err());
    }

    #[test]
    fn kinds_expose_family_and_token() {
        let register = OperationKind::Register { account: alice() };
        assert_eq!(register.family(), "register");
        assert!(register.token().is_none());

        let transfer = OperationKind::Transfer {
            token: token(),
            sender: alice(),
            recipient: bob(),
            amount: ConfidentialAmount::new(30),
        };
        assert_eq!(transfer.family(), "transfer");
        assert_eq!(transfer.token().unwrap().as_str(), "ct-usdc");
        assert!(transfer.carries_confidential_amount());
        assert!(!register.carries_confidential_amount());
    }

    #[test]
    fn confidential_amount_never_debugs_or_displays_raw() {
        let amount = ConfidentialAmount::new(987_654_321);
        assert!(!format!("{amount:?}").contains("987654321"));
        assert!(!format!("{amount}").contains("987654321"));
        assert!(format!("{amount:?}").contains("redact"));
        // Internal access still works for the trusted executor/oracle.
        assert_eq!(amount.get(), 987_654_321);
    }

    #[test]
    fn amount_arithmetic_saturates() {
        assert_eq!(Amount::new(10) + Amount::new(5), Amount::new(15));
        assert_eq!(Amount::new(5) - Amount::new(10), Amount::ZERO);
        assert!(Amount::new(0).is_zero());
    }

    #[test]
    fn operation_round_trips_through_json() {
        let op = Operation::new(
            OperationId::new("op-1").unwrap(),
            alice(),
            OperationKind::Transfer {
                token: token(),
                sender: alice(),
                recipient: bob(),
                amount: ConfidentialAmount::new(30),
            },
        )
        .with_context("fixture", "state-s3");
        let json = serde_json::to_string(&op).unwrap();
        // The secret value is present in the definition file, as intended;
        // runtime observation/reporting layers are what redact.
        assert!(json.contains("\"amount\":30"));
        let back: Operation = serde_json::from_str(&json).unwrap();
        assert_eq!(back, op);
        assert!(back.is_confidential());
        assert_eq!(back.context.get("fixture").unwrap(), "state-s3");
    }

    #[test]
    fn register_operation_has_no_token() {
        let op = Operation::new(
            OperationId::new("op-1").unwrap(),
            alice(),
            OperationKind::Register { account: alice() },
        );
        assert!(!op.is_confidential());
        assert_eq!(op.kind.family(), "register");
    }
}
