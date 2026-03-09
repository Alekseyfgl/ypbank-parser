use crate::error::{Error, Result};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// A single YPBank transaction shared by all supported formats.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    /// Unique transaction identifier.
    pub tx_id: u64,
    /// Transaction type.
    pub tx_type: TransactionType,
    /// Sender account or user identifier.
    pub from_user_id: u64,
    /// Receiver account or user identifier.
    pub to_user_id: u64,
    /// Transaction amount in the smallest currency units.
    pub amount: i64,
    /// Unix timestamp in milliseconds.
    pub timestamp: u64,
    /// Transaction status.
    pub status: TransactionStatus,
    /// Human-readable transaction description.
    pub description: String,
}

/// Type of a YPBank transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionType {
    /// Deposit into an account.
    Deposit,
    /// Transfer between accounts.
    Transfer,
    /// Withdrawal from an account.
    Withdrawal,
}

impl TransactionType {
    pub(crate) fn as_byte(self) -> u8 {
        match self {
            Self::Deposit => 0,
            Self::Transfer => 1,
            Self::Withdrawal => 2,
        }
    }

    pub(crate) fn from_byte(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Self::Deposit),
            1 => Ok(Self::Transfer),
            2 => Ok(Self::Withdrawal),
            _ => Err(Error::InvalidValue {
                field: "TX_TYPE",
                value: value.to_string(),
                details: String::from("expected 0, 1 or 2"),
            }),
        }
    }
}

impl FromStr for TransactionType {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self> {
        match value.trim() {
            "DEPOSIT" => Ok(Self::Deposit),
            "TRANSFER" => Ok(Self::Transfer),
            "WITHDRAWAL" => Ok(Self::Withdrawal),
            _ => Err(Error::InvalidValue {
                field: "TX_TYPE",
                value: value.to_owned(),
                details: String::from("expected DEPOSIT, TRANSFER or WITHDRAWAL"),
            }),
        }
    }
}

impl Display for TransactionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Deposit => f.write_str("DEPOSIT"),
            Self::Transfer => f.write_str("TRANSFER"),
            Self::Withdrawal => f.write_str("WITHDRAWAL"),
        }
    }
}

/// Processing status of a YPBank transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionStatus {
    /// The transaction completed successfully.
    Success,
    /// The transaction failed.
    Failure,
    /// The transaction is still pending.
    Pending,
}

impl TransactionStatus {
    pub(crate) fn as_byte(self) -> u8 {
        match self {
            Self::Success => 0,
            Self::Failure => 1,
            Self::Pending => 2,
        }
    }

    pub(crate) fn from_byte(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Self::Success),
            1 => Ok(Self::Failure),
            2 => Ok(Self::Pending),
            _ => Err(Error::InvalidValue {
                field: "STATUS",
                value: value.to_string(),
                details: String::from("expected 0, 1 or 2"),
            }),
        }
    }
}

impl FromStr for TransactionStatus {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self> {
        match value.trim() {
            "SUCCESS" => Ok(Self::Success),
            "FAILURE" => Ok(Self::Failure),
            "PENDING" => Ok(Self::Pending),
            _ => Err(Error::InvalidValue {
                field: "STATUS",
                value: value.to_owned(),
                details: String::from("expected SUCCESS, FAILURE or PENDING"),
            }),
        }
    }
}

impl Display for TransactionStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success => f.write_str("SUCCESS"),
            Self::Failure => f.write_str("FAILURE"),
            Self::Pending => f.write_str("PENDING"),
        }
    }
}
