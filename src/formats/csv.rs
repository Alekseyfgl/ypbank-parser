use super::{
    escape_csv_field, normalize_description_for_text_output, parse_i64_field, parse_u64_field,
    trim_line_end,
};
use crate::error::{Error, Result};
use crate::types::{Transaction, TransactionStatus, TransactionType};
use std::io::{BufRead, BufReader, BufWriter, Read, Write};

const HEADER: &str = "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION";

/// Reads transactions from the `YPBankCsv` format.
pub fn read_csv<R: Read>(reader: R) -> Result<Vec<Transaction>> {
    let mut reader = BufReader::new(reader);
    let mut header_line = String::new();
    let mut line_number = 0usize;

    loop {
        header_line.clear();
        let bytes_read = reader.read_line(&mut header_line)?;
        if bytes_read == 0 {
            return Err(Error::InvalidCsvHeader {
                found: String::from("<empty file>"),
            });
        }

        line_number += 1;
        if !trim_line_end(&header_line).trim().is_empty() {
            break;
        }
    }

    let actual_header = trim_line_end(&header_line).trim_start_matches('\u{feff}');
    if actual_header != HEADER {
        return Err(Error::InvalidCsvHeader {
            found: actual_header.to_owned(),
        });
    }

    let mut transactions = Vec::new();
    let mut line = String::new();

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line)?;
        if bytes_read == 0 {
            break;
        }

        line_number += 1;
        let content = trim_line_end(&line);
        if content.trim().is_empty() {
            continue;
        }

        let fields = split_csv_record(content).map_err(|details| Error::InvalidCsvRecord {
            line: line_number,
            details,
        })?;

        if fields.len() != 8 {
            return Err(Error::InvalidCsvRecord {
                line: line_number,
                details: format!("expected 8 fields, found {}", fields.len()),
            });
        }

        let transaction = Transaction {
            tx_id: parse_u64_field("TX_ID", &fields[0])?,
            tx_type: fields[1].parse::<TransactionType>()?,
            from_user_id: parse_u64_field("FROM_USER_ID", &fields[2])?,
            to_user_id: parse_u64_field("TO_USER_ID", &fields[3])?,
            amount: parse_i64_field("AMOUNT", &fields[4])?,
            timestamp: parse_u64_field("TIMESTAMP", &fields[5])?,
            status: fields[6].parse::<TransactionStatus>()?,
            description: fields[7].clone(),
        };

        transactions.push(transaction);
    }

    Ok(transactions)
}

/// Writes transactions in the `YPBankCsv` format.
pub fn write_csv<W: Write>(writer: W, transactions: &[Transaction]) -> Result<()> {
    let mut writer = BufWriter::new(writer);
    writeln!(writer, "{HEADER}")?;

    for transaction in transactions {
        let description = normalize_description_for_text_output(&transaction.description);
        writeln!(
            writer,
            "{},{},{},{},{},{},{},{}",
            transaction.tx_id,
            transaction.tx_type,
            transaction.from_user_id,
            transaction.to_user_id,
            transaction.amount,
            transaction.timestamp,
            transaction.status,
            escape_csv_field(&description),
        )?;
    }

    writer.flush()?;
    Ok(())
}

fn split_csv_record(line: &str) -> std::result::Result<Vec<String>, String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut chars = line.chars().peekable();
    let mut in_quotes = false;

    while let Some(ch) = chars.next() {
        match ch {
            '"' if in_quotes => {
                if matches!(chars.peek(), Some('"')) {
                    current.push('"');
                    chars.next();
                } else {
                    in_quotes = false;
                }
            }
            '"' if current.is_empty() => {
                in_quotes = true;
            }
            ',' if !in_quotes => {
                fields.push(std::mem::take(&mut current));
            }
            _ => current.push(ch),
        }
    }

    if in_quotes {
        return Err(String::from("unterminated quoted field"));
    }

    fields.push(current);
    Ok(fields)
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
    fn csv_roundtrip_preserves_transactions() {
        let input = concat!(
            "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION\n",
            "42,TRANSFER,1,2,-50,1700000000000,PENDING,\"Quoted \"\"text\"\", with comma\"\n",
        );

        let records = must(read_csv(input.as_bytes()));
        let mut output = Vec::new();
        must(write_csv(&mut output, &records));
        let parsed_again = must(read_csv(output.as_slice()));

        assert_eq!(records, parsed_again);
        assert_eq!(records[0].description, "Quoted \"text\", with comma");
    }

    #[test]
    fn csv_header_is_validated() {
        let error = read_csv(b"bad\n1,2,3\n".as_slice()).expect_err("expected failure");

        assert!(matches!(error, Error::InvalidCsvHeader { .. }));
    }
}
