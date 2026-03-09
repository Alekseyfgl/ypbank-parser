use crate::error::{Error, Result};
use crate::types::{Transaction, TransactionStatus, TransactionType};
use std::io::{BufWriter, ErrorKind, Read, Write};

const MAGIC: [u8; 4] = *b"YPBN";
const FIXED_BODY_SIZE: usize = 46;

/// Reads transactions from the `YPBankBin` format.
pub fn read_binary<R: Read>(reader: R) -> Result<Vec<Transaction>> {
    let mut reader = reader;
    let mut transactions = Vec::new();
    let mut record_number = 0usize;

    loop {
        let mut first = [0u8; 1];
        let bytes_read = reader.read(&mut first)?;
        if bytes_read == 0 {
            break;
        }

        let mut magic = [0u8; 4];
        magic[0] = first[0];
        read_exact_with_context(&mut reader, &mut magic[1..], "binary record magic")?;

        record_number += 1;
        if magic != MAGIC {
            return Err(Error::InvalidBinaryMagic {
                record: record_number,
                found: magic,
            });
        }

        let record_size = read_u32(&mut reader, "binary record size")?;
        if record_size < FIXED_BODY_SIZE as u32 {
            return Err(Error::InvalidBinaryRecordSize {
                record: record_number,
                size: record_size,
            });
        }

        let mut body = vec![0u8; record_size as usize];
        read_exact_with_context(&mut reader, &mut body, "binary record body")?;
        transactions.push(parse_binary_body(&body, record_number)?);
    }

    Ok(transactions)
}

/// Writes transactions in the `YPBankBin` format.
pub fn write_binary<W: Write>(writer: W, transactions: &[Transaction]) -> Result<()> {
    let mut writer = BufWriter::new(writer);

    for (index, transaction) in transactions.iter().enumerate() {
        let description = transaction.description.as_bytes();
        let description_len =
            u32::try_from(description.len()).map_err(|_| Error::InvalidBinaryRecord {
                record: index + 1,
                details: String::from("description is too long to fit into u32"),
            })?;
        let record_size = u32::try_from(FIXED_BODY_SIZE + description.len()).map_err(|_| {
            Error::InvalidBinaryRecord {
                record: index + 1,
                details: String::from("record body is too large to fit into u32"),
            }
        })?;

        writer.write_all(&MAGIC)?;
        writer.write_all(&record_size.to_be_bytes())?;
        writer.write_all(&transaction.tx_id.to_be_bytes())?;
        writer.write_all(&[transaction.tx_type.as_byte()])?;
        writer.write_all(&transaction.from_user_id.to_be_bytes())?;
        writer.write_all(&transaction.to_user_id.to_be_bytes())?;
        writer.write_all(&transaction.amount.to_be_bytes())?;
        writer.write_all(&transaction.timestamp.to_be_bytes())?;
        writer.write_all(&[transaction.status.as_byte()])?;
        writer.write_all(&description_len.to_be_bytes())?;
        writer.write_all(description)?;
    }

    writer.flush()?;
    Ok(())
}

fn parse_binary_body(body: &[u8], record_number: usize) -> Result<Transaction> {
    let mut reader = BinaryBodyReader::new(body, record_number);

    let tx_id = reader.read_u64("TX_ID")?;
    let tx_type = TransactionType::from_byte(reader.read_u8("TX_TYPE")?)?;
    let from_user_id = reader.read_u64("FROM_USER_ID")?;
    let to_user_id = reader.read_u64("TO_USER_ID")?;
    let amount = reader.read_i64("AMOUNT")?;
    let timestamp = reader.read_u64("TIMESTAMP")?;
    let status = TransactionStatus::from_byte(reader.read_u8("STATUS")?)?;
    let description_len = reader.read_u32("DESC_LEN")? as usize;

    if reader.remaining() != description_len {
        return Err(Error::InvalidBinaryRecord {
            record: record_number,
            details: format!(
                "DESC_LEN is {description_len}, but {} bytes remain in record body",
                reader.remaining()
            ),
        });
    }

    let raw_description = reader.read_bytes(description_len, "DESCRIPTION")?;
    let description = String::from_utf8(raw_description.to_vec()).map_err(|error| {
        Error::InvalidBinaryRecord {
            record: record_number,
            details: format!("DESCRIPTION is not valid UTF-8: {error}"),
        }
    })?;

    Ok(Transaction {
        tx_id,
        tx_type,
        from_user_id,
        to_user_id,
        amount,
        timestamp,
        status,
        description,
    })
}

fn read_u32<R: Read>(reader: &mut R, context: &'static str) -> Result<u32> {
    let mut bytes = [0u8; 4];
    read_exact_with_context(reader, &mut bytes, context)?;
    Ok(u32::from_be_bytes(bytes))
}

fn read_exact_with_context<R: Read>(
    reader: &mut R,
    buffer: &mut [u8],
    context: &'static str,
) -> Result<()> {
    match reader.read_exact(buffer) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::UnexpectedEof => {
            Err(Error::UnexpectedEof { context })
        }
        Err(error) => Err(Error::Io(error)),
    }
}

struct BinaryBodyReader<'a> {
    body: &'a [u8],
    offset: usize,
    record: usize,
}

impl<'a> BinaryBodyReader<'a> {
    fn new(body: &'a [u8], record: usize) -> Self {
        Self {
            body,
            offset: 0,
            record,
        }
    }

    fn read_u8(&mut self, field: &'static str) -> Result<u8> {
        let bytes = self.read_bytes(1, field)?;
        Ok(bytes[0])
    }

    fn read_u32(&mut self, field: &'static str) -> Result<u32> {
        let bytes = self.read_bytes(4, field)?;
        let mut array = [0u8; 4];
        array.copy_from_slice(bytes);
        Ok(u32::from_be_bytes(array))
    }

    fn read_u64(&mut self, field: &'static str) -> Result<u64> {
        let bytes = self.read_bytes(8, field)?;
        let mut array = [0u8; 8];
        array.copy_from_slice(bytes);
        Ok(u64::from_be_bytes(array))
    }

    fn read_i64(&mut self, field: &'static str) -> Result<i64> {
        let bytes = self.read_bytes(8, field)?;
        let mut array = [0u8; 8];
        array.copy_from_slice(bytes);
        Ok(i64::from_be_bytes(array))
    }

    fn read_bytes(&mut self, length: usize, field: &'static str) -> Result<&'a [u8]> {
        if self.remaining() < length {
            return Err(Error::InvalidBinaryRecord {
                record: self.record,
                details: format!(
                    "field {field} needs {length} bytes, but only {} remain",
                    self.remaining()
                ),
            });
        }

        let start = self.offset;
        self.offset += length;
        Ok(&self.body[start..self.offset])
    }

    fn remaining(&self) -> usize {
        self.body.len().saturating_sub(self.offset)
    }
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
    fn binary_roundtrip_preserves_transactions() {
        let transaction = Transaction {
            tx_id: 7,
            tx_type: TransactionType::Withdrawal,
            from_user_id: 99,
            to_user_id: 0,
            amount: -300,
            timestamp: 1700000000000,
            status: TransactionStatus::Success,
            description: String::from("ATM withdrawal"),
        };

        let mut output = Vec::new();
        must(write_binary(
            &mut output,
            std::slice::from_ref(&transaction),
        ));

        let parsed = must(read_binary(output.as_slice()));
        assert_eq!(parsed, vec![transaction]);
    }

    #[test]
    fn binary_roundtrip_preserves_outer_quotes_in_description() {
        let transaction = Transaction {
            tx_id: 8,
            tx_type: TransactionType::Deposit,
            from_user_id: 0,
            to_user_id: 42,
            amount: 1_500,
            timestamp: 1_700_000_000_001,
            status: TransactionStatus::Pending,
            description: String::from("\"VIP\""),
        };

        let mut output = Vec::new();
        must(write_binary(
            &mut output,
            std::slice::from_ref(&transaction),
        ));

        let parsed = must(read_binary(output.as_slice()));
        assert_eq!(parsed, vec![transaction]);
    }

    #[test]
    fn binary_magic_is_validated() {
        let bytes = b"FAIL\x00\x00\x00\x2e".to_vec();
        let error = read_binary(bytes.as_slice()).expect_err("expected failure");

        assert!(matches!(error, Error::InvalidBinaryMagic { .. }));
    }
}
