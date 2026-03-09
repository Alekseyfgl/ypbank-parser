#![deny(missing_docs)]
//! Library for reading, writing and converting YPBank transaction files.
//!
//! The crate exposes a shared transaction model plus parsers/serializers for
//! CSV, text and binary YPBank formats. All public read/write entry points are
//! generic over [`std::io::Read`] and [`std::io::Write`].

mod error;
mod format;
mod formats;
mod types;

pub use crate::error::{Error, Result};
pub use crate::format::Format;
pub use crate::formats::{read_binary, read_csv, read_text, write_binary, write_csv, write_text};
pub use crate::types::{Transaction, TransactionStatus, TransactionType};

use std::io::{Read, Write};

/// Reads a list of transactions from any reader using the selected format.
pub fn read_transactions<R: Read>(reader: R, format: Format) -> Result<Vec<Transaction>> {
    match format {
        Format::Csv => read_csv(reader),
        Format::Text => read_text(reader),
        Format::Binary => read_binary(reader),
    }
}

/// Writes a list of transactions to any writer using the selected format.
pub fn write_transactions<W: Write>(
    writer: W,
    format: Format,
    transactions: &[Transaction],
) -> Result<()> {
    match format {
        Format::Csv => write_csv(writer, transactions),
        Format::Text => write_text(writer, transactions),
        Format::Binary => write_binary(writer, transactions),
    }
}

/// Converts transactions from one format to another.
pub fn convert<R: Read, W: Write>(
    reader: R,
    input_format: Format,
    writer: W,
    output_format: Format,
) -> Result<()> {
    let transactions = read_transactions(reader, input_format)?;
    write_transactions(writer, output_format, &transactions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn fixture_path(file_name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("Примеры_файлов")
            .join(file_name)
    }

    fn must<T>(result: Result<T>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("unexpected error: {error}"),
        }
    }

    fn must_io<T>(result: std::io::Result<T>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("unexpected i/o error: {error}"),
        }
    }

    fn sample_transaction() -> Transaction {
        Transaction {
            tx_id: 1_234_567_890,
            tx_type: TransactionType::Transfer,
            from_user_id: 11,
            to_user_id: 22,
            amount: -4_250,
            timestamp: 1_700_000_000_123,
            status: TransactionStatus::Pending,
            description: String::from("Invoice #42, partial refund"),
        }
    }

    #[test]
    fn dispatch_reads_and_writes_csv() {
        let transaction = sample_transaction();
        let mut output = Vec::new();

        must(write_transactions(
            &mut output,
            Format::Csv,
            std::slice::from_ref(&transaction),
        ));

        let parsed = must(read_transactions(output.as_slice(), Format::Csv));
        assert_eq!(parsed, vec![transaction]);
    }

    #[test]
    fn dispatch_reads_and_writes_text() {
        let transaction = sample_transaction();
        let mut output = Vec::new();

        must(write_transactions(
            &mut output,
            Format::Text,
            std::slice::from_ref(&transaction),
        ));

        let parsed = must(read_transactions(output.as_slice(), Format::Text));
        assert_eq!(parsed, vec![transaction]);
    }

    #[test]
    fn dispatch_reads_and_writes_binary() {
        let transaction = sample_transaction();
        let mut output = Vec::new();

        must(write_transactions(
            &mut output,
            Format::Binary,
            std::slice::from_ref(&transaction),
        ));

        let parsed = must(read_transactions(output.as_slice(), Format::Binary));
        assert_eq!(parsed, vec![transaction]);
    }

    #[test]
    fn sample_files_describe_identical_transactions() {
        let csv_data = must_io(fs::read_to_string(fixture_path("records_example.csv")));
        let text_data = must_io(fs::read_to_string(fixture_path("records_example.txt")));
        let bin_data = must_io(fs::read(fixture_path("records_example.bin")));

        let csv_records = must(read_csv(csv_data.as_bytes()));
        let text_records = must(read_text(text_data.as_bytes()));
        let bin_records = must(read_binary(bin_data.as_slice()));

        assert_transactions_equivalent(&csv_records, &text_records);
        assert_transactions_equivalent(&csv_records, &bin_records);
    }

    #[test]
    fn convert_reencodes_between_formats() {
        let csv_data = must_io(fs::read_to_string(fixture_path("records_example.csv")));
        let mut binary = Vec::new();
        must(convert(
            csv_data.as_bytes(),
            Format::Csv,
            &mut binary,
            Format::Binary,
        ));

        let binary_records = must(read_binary(binary.as_slice()));
        let csv_records = must(read_csv(csv_data.as_bytes()));

        assert_eq!(binary_records, csv_records);
    }

    fn assert_transactions_equivalent(left: &[Transaction], right: &[Transaction]) {
        assert_eq!(left.len(), right.len());
        for (left_record, right_record) in left.iter().zip(right.iter()) {
            assert!(
                crate::formats::transactions_equivalent(left_record, right_record),
                "records are not equivalent:\nleft: {:?}\nright: {:?}",
                left_record,
                right_record
            );
        }
    }
}
