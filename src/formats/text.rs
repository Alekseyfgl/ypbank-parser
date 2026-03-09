use super::{
    escape_text_value, normalize_description_for_text_output, parse_i64_field, parse_text_value,
    parse_u64_field,
};
use crate::error::{Error, Result};
use crate::types::{Transaction, TransactionStatus, TransactionType};
use std::io::{BufRead, BufReader, BufWriter, Read, Write};

/// Reads transactions from the `YPBankText` format.
pub fn read_text<R: Read>(reader: R) -> Result<Vec<Transaction>> {
    let reader = BufReader::new(reader);
    let mut transactions = Vec::new();
    let mut builder = TextRecordBuilder::default();
    let mut record_number = 0usize;

    for (line_index, line_result) in reader.lines().enumerate() {
        let line_number = line_index + 1;
        let line = line_result?;
        let trimmed = line.trim();

        if trimmed.is_empty() {
            if !builder.is_empty() {
                record_number += 1;
                transactions.push(builder.build(record_number)?);
                builder = TextRecordBuilder::default();
            }
            continue;
        }

        if trimmed.starts_with('#') {
            continue;
        }

        let (key, value) = line.split_once(':').ok_or(Error::InvalidTextLine {
            line: line_number,
            details: String::from("expected KEY: VALUE"),
        })?;

        builder.insert(key.trim(), value.trim(), record_number + 1, line_number)?;
    }

    if !builder.is_empty() {
        record_number += 1;
        transactions.push(builder.build(record_number)?);
    }

    Ok(transactions)
}

/// Writes transactions in the `YPBankText` format.
pub fn write_text<W: Write>(writer: W, transactions: &[Transaction]) -> Result<()> {
    let mut writer = BufWriter::new(writer);

    for (index, transaction) in transactions.iter().enumerate() {
        if index > 0 {
            writeln!(writer)?;
        }

        let description = normalize_description_for_text_output(&transaction.description);

        writeln!(writer, "TX_ID: {}", transaction.tx_id)?;
        writeln!(writer, "TX_TYPE: {}", transaction.tx_type)?;
        writeln!(writer, "FROM_USER_ID: {}", transaction.from_user_id)?;
        writeln!(writer, "TO_USER_ID: {}", transaction.to_user_id)?;
        writeln!(writer, "AMOUNT: {}", transaction.amount)?;
        writeln!(writer, "TIMESTAMP: {}", transaction.timestamp)?;
        writeln!(writer, "STATUS: {}", transaction.status)?;
        writeln!(writer, "DESCRIPTION: {}", escape_text_value(&description))?;
    }

    writer.flush()?;
    Ok(())
}

#[derive(Default)]
struct TextRecordBuilder {
    tx_id: Option<u64>,
    tx_type: Option<TransactionType>,
    from_user_id: Option<u64>,
    to_user_id: Option<u64>,
    amount: Option<i64>,
    timestamp: Option<u64>,
    status: Option<TransactionStatus>,
    description: Option<String>,
}

impl TextRecordBuilder {
    fn is_empty(&self) -> bool {
        self.tx_id.is_none()
            && self.tx_type.is_none()
            && self.from_user_id.is_none()
            && self.to_user_id.is_none()
            && self.amount.is_none()
            && self.timestamp.is_none()
            && self.status.is_none()
            && self.description.is_none()
    }

    fn insert(
        &mut self,
        key: &str,
        value: &str,
        record_number: usize,
        line_number: usize,
    ) -> Result<()> {
        match key {
            "TX_ID" => set_unique(
                &mut self.tx_id,
                parse_u64_field("TX_ID", value)?,
                record_number,
                "TX_ID",
            ),
            "TX_TYPE" => set_unique(
                &mut self.tx_type,
                value.parse::<TransactionType>()?,
                record_number,
                "TX_TYPE",
            ),
            "FROM_USER_ID" => set_unique(
                &mut self.from_user_id,
                parse_u64_field("FROM_USER_ID", value)?,
                record_number,
                "FROM_USER_ID",
            ),
            "TO_USER_ID" => set_unique(
                &mut self.to_user_id,
                parse_u64_field("TO_USER_ID", value)?,
                record_number,
                "TO_USER_ID",
            ),
            "AMOUNT" => set_unique(
                &mut self.amount,
                parse_i64_field("AMOUNT", value)?,
                record_number,
                "AMOUNT",
            ),
            "TIMESTAMP" => set_unique(
                &mut self.timestamp,
                parse_u64_field("TIMESTAMP", value)?,
                record_number,
                "TIMESTAMP",
            ),
            "STATUS" => set_unique(
                &mut self.status,
                value.parse::<TransactionStatus>()?,
                record_number,
                "STATUS",
            ),
            "DESCRIPTION" => set_unique(
                &mut self.description,
                parse_text_value("DESCRIPTION", value)?,
                record_number,
                "DESCRIPTION",
            ),
            _ => Err(Error::InvalidTextLine {
                line: line_number,
                details: format!("unknown field {key}"),
            }),
        }
    }

    fn build(self, record_number: usize) -> Result<Transaction> {
        Ok(Transaction {
            tx_id: self.tx_id.ok_or(Error::MissingField {
                record: record_number,
                field: "TX_ID",
            })?,
            tx_type: self.tx_type.ok_or(Error::MissingField {
                record: record_number,
                field: "TX_TYPE",
            })?,
            from_user_id: self.from_user_id.ok_or(Error::MissingField {
                record: record_number,
                field: "FROM_USER_ID",
            })?,
            to_user_id: self.to_user_id.ok_or(Error::MissingField {
                record: record_number,
                field: "TO_USER_ID",
            })?,
            amount: self.amount.ok_or(Error::MissingField {
                record: record_number,
                field: "AMOUNT",
            })?,
            timestamp: self.timestamp.ok_or(Error::MissingField {
                record: record_number,
                field: "TIMESTAMP",
            })?,
            status: self.status.ok_or(Error::MissingField {
                record: record_number,
                field: "STATUS",
            })?,
            description: self.description.ok_or(Error::MissingField {
                record: record_number,
                field: "DESCRIPTION",
            })?,
        })
    }
}

fn set_unique<T>(
    slot: &mut Option<T>,
    value: T,
    record_number: usize,
    field: &'static str,
) -> Result<()> {
    if slot.is_some() {
        return Err(Error::DuplicateField {
            record: record_number,
            field,
        });
    }

    *slot = Some(value);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn must<T>(result: Result<T>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("unexpected error: {error}"),
        }
    }

    #[test]
    fn text_roundtrip_preserves_transactions() {
        let input = concat!(
            "# comment\n",
            "STATUS: SUCCESS\n",
            "TX_ID: 5\n",
            "TX_TYPE: DEPOSIT\n",
            "FROM_USER_ID: 0\n",
            "TO_USER_ID: 77\n",
            "AMOUNT: 1500\n",
            "TIMESTAMP: 1700000000000\n",
            "DESCRIPTION: \"Hello, \\\"world\\\"\"\n",
        );

        let records = must(read_text(input.as_bytes()));
        let mut output = Vec::new();
        must(write_text(&mut output, &records));
        let parsed_again = must(read_text(output.as_slice()));

        assert_eq!(records, parsed_again);
        assert_eq!(records[0].description, "Hello, \"world\"");
    }

    #[test]
    fn text_duplicate_field_is_rejected() {
        let input = concat!(
            "TX_ID: 1\n",
            "TX_ID: 2\n",
            "TX_TYPE: DEPOSIT\n",
            "FROM_USER_ID: 0\n",
            "TO_USER_ID: 1\n",
            "AMOUNT: 10\n",
            "TIMESTAMP: 1\n",
            "STATUS: SUCCESS\n",
            "DESCRIPTION: \"x\"\n",
        );

        let error = read_text(input.as_bytes()).expect_err("expected failure");
        assert!(matches!(error, Error::DuplicateField { .. }));
    }
}
