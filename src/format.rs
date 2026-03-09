use crate::error::{Error, Result};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// Supported YPBank file formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// `YPBankCsv` format.
    Csv,
    /// `YPBankText` format.
    Text,
    /// `YPBankBin` format.
    Binary,
}

impl FromStr for Format {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "csv" | "ypbankcsv" => Ok(Self::Csv),
            "text" | "txt" | "ypbanktext" => Ok(Self::Text),
            "binary" | "bin" | "ypbankbin" => Ok(Self::Binary),
            _ => Err(Error::UnknownFormat(value.to_owned())),
        }
    }
}

impl Display for Format {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Csv => f.write_str("csv"),
            Self::Text => f.write_str("text"),
            Self::Binary => f.write_str("binary"),
        }
    }
}
