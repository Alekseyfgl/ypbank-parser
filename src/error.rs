use std::fmt::{Display, Formatter};
use std::io;

/// Convenient result alias used by the library.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors returned by YPBank parsers, serializers and CLI helpers.
#[derive(Debug)]
pub enum Error {
    /// Wrapper around I/O failures.
    Io(io::Error),
    /// Unknown format name passed from CLI or user input.
    UnknownFormat(String),
    /// Invalid command-line argument or unsupported value.
    InvalidArgument(String),
    /// CSV header does not match the specification.
    InvalidCsvHeader {
        /// Header line that was actually read from the file.
        found: String,
    },
    /// CSV line is malformed.
    InvalidCsvRecord {
        /// 1-based line number of the malformed CSV record.
        line: usize,
        /// Human-readable parsing details.
        details: String,
    },
    /// Text line is malformed.
    InvalidTextLine {
        /// 1-based line number of the malformed text line.
        line: usize,
        /// Human-readable parsing details.
        details: String,
    },
    /// Text record is incomplete or inconsistent.
    InvalidTextRecord {
        /// 1-based record number in the text file.
        record: usize,
        /// Human-readable parsing details.
        details: String,
    },
    /// A required field is missing from a record.
    MissingField {
        /// 1-based record number where the field is missing.
        record: usize,
        /// Name of the missing field.
        field: &'static str,
    },
    /// A field is repeated inside a text record.
    DuplicateField {
        /// 1-based record number where the field is duplicated.
        record: usize,
        /// Name of the duplicated field.
        field: &'static str,
    },
    /// Field value cannot be parsed or validated.
    InvalidValue {
        /// Name of the field with an invalid value.
        field: &'static str,
        /// Original textual or numeric representation.
        value: String,
        /// Human-readable validation details.
        details: String,
    },
    /// Binary magic is invalid.
    InvalidBinaryMagic {
        /// 1-based binary record number.
        record: usize,
        /// Four bytes actually read instead of the expected magic.
        found: [u8; 4],
    },
    /// Binary record size is smaller than the fixed body.
    InvalidBinaryRecordSize {
        /// 1-based binary record number.
        record: usize,
        /// Size value from the record header.
        size: u32,
    },
    /// Binary record body is malformed.
    InvalidBinaryRecord {
        /// 1-based binary record number.
        record: usize,
        /// Human-readable parsing details.
        details: String,
    },
    /// The input ended before a full structure could be read.
    UnexpectedEof {
        /// Structure that was being read when the input ended.
        context: &'static str,
    },
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "i/o error: {error}"),
            Self::UnknownFormat(value) => write!(f, "unknown format: {value}"),
            Self::InvalidArgument(details) => write!(f, "invalid argument: {details}"),
            Self::InvalidCsvHeader { found } => {
                write!(f, "invalid CSV header: {found}")
            }
            Self::InvalidCsvRecord { line, details } => {
                write!(f, "invalid CSV record at line {line}: {details}")
            }
            Self::InvalidTextLine { line, details } => {
                write!(f, "invalid text line {line}: {details}")
            }
            Self::InvalidTextRecord { record, details } => {
                write!(f, "invalid text record {record}: {details}")
            }
            Self::MissingField { record, field } => {
                write!(f, "record {record} is missing required field {field}")
            }
            Self::DuplicateField { record, field } => {
                write!(f, "record {record} contains duplicate field {field}")
            }
            Self::InvalidValue {
                field,
                value,
                details,
            } => write!(f, "invalid value for {field} ({value}): {details}"),
            Self::InvalidBinaryMagic { record, found } => {
                write!(f, "invalid binary magic in record {record}: {found:02X?}")
            }
            Self::InvalidBinaryRecordSize { record, size } => {
                write!(f, "invalid binary record size in record {record}: {size}")
            }
            Self::InvalidBinaryRecord { record, details } => {
                write!(f, "invalid binary record {record}: {details}")
            }
            Self::UnexpectedEof { context } => {
                write!(f, "unexpected end of input while reading {context}")
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}
