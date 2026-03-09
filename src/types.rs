use crate::error::{Error, Result};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// Одна транзакция YPBank, общая для всех поддерживаемых форматов.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    /// Уникальный идентификатор транзакции.
    pub tx_id: u64,
    /// Тип транзакции.
    pub tx_type: TransactionType,
    /// Идентификатор отправителя.
    pub from_user_id: u64,
    /// Идентификатор получателя.
    pub to_user_id: u64,
    /// Сумма транзакции в минимальных денежных единицах.
    pub amount: i64,
    /// Unix-время в миллисекундах.
    pub timestamp: u64,
    /// Статус транзакции.
    pub status: TransactionStatus,
    /// Человекочитаемое описание транзакции.
    pub description: String,
}

/// Тип транзакции YPBank.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionType {
    /// Пополнение счета.
    Deposit,
    /// Перевод между счетами.
    Transfer,
    /// Списание со счета.
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
                details: String::from("ожидается 0, 1 или 2"),
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
                details: String::from("ожидается DEPOSIT, TRANSFER или WITHDRAWAL"),
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

/// Статус обработки транзакции YPBank.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionStatus {
    /// Транзакция успешно завершена.
    Success,
    /// Транзакция завершилась ошибкой.
    Failure,
    /// Транзакция еще находится в обработке.
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
                details: String::from("ожидается 0, 1 или 2"),
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
                details: String::from("ожидается SUCCESS, FAILURE или PENDING"),
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
