use std::env;
use std::fs::File;
use std::io::BufReader;
use std::process::ExitCode;
use ypbank_converter::{Error, Format, Transaction, read_transactions};

struct CompareArgs {
    file1: String,
    format1: Format,
    file2: String,
    format2: Format,
}

enum Command {
    Help,
    Run(CompareArgs),
}

fn main() -> ExitCode {
    match parse_command(env::args().skip(1)) {
        Ok(Command::Help) => {
            println!("{}", usage());
            ExitCode::SUCCESS
        }
        Ok(Command::Run(args)) => match run(args) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("Error: {error}");
                ExitCode::from(1)
            }
        },
        Err(error) => {
            eprintln!("Error: {error}");
            eprintln!("{}", usage());
            ExitCode::from(1)
        }
    }
}

fn run(args: CompareArgs) -> Result<(), Error> {
    let left = read_file(&args.file1, args.format1)?;
    let right = read_file(&args.file2, args.format2)?;

    if transaction_sets_equivalent(&left, &right) {
        println!(
            "The transaction records in '{}' and '{}' are identical.",
            args.file1, args.file2
        );
        return Ok(());
    }

    if let Some(message) = first_difference_message(&args.file1, &left, &args.file2, &right) {
        println!("{message}");
    }

    Err(Error::InvalidArgument(String::from(
        "transaction sets are different",
    )))
}

fn read_file(path: &str, format: Format) -> Result<Vec<Transaction>, Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    read_transactions(reader, format)
}

fn first_difference_message(
    file1: &str,
    left: &[Transaction],
    file2: &str,
    right: &[Transaction],
) -> Option<String> {
    let limit = left.len().max(right.len());

    for index in 0..limit {
        match (left.get(index), right.get(index)) {
            (Some(left_transaction), Some(right_transaction))
                if !transactions_equivalent(left_transaction, right_transaction) =>
            {
                let differing_fields = differing_fields(left_transaction, right_transaction);
                return Some(format!(
                    "Mismatch at record {}.\n'{}': {:?}\n'{}': {:?}\nDiffering fields: {}.",
                    index + 1,
                    file1,
                    left_transaction,
                    file2,
                    right_transaction,
                    differing_fields.join(", ")
                ));
            }
            (Some(left_transaction), None) => {
                return Some(format!(
                    "Mismatch at record {}: '{}' has extra transaction {:?}.",
                    index + 1,
                    file1,
                    left_transaction
                ));
            }
            (None, Some(right_transaction)) => {
                return Some(format!(
                    "Mismatch at record {}: '{}' has extra transaction {:?}.",
                    index + 1,
                    file2,
                    right_transaction
                ));
            }
            _ => {}
        }
    }

    None
}

fn differing_fields(left: &Transaction, right: &Transaction) -> Vec<String> {
    let mut fields = Vec::new();

    if left.tx_id != right.tx_id {
        fields.push(format!("TX_ID: {} != {}", left.tx_id, right.tx_id));
    }
    if left.tx_type != right.tx_type {
        fields.push(format!("TX_TYPE: {} != {}", left.tx_type, right.tx_type));
    }
    if left.from_user_id != right.from_user_id {
        fields.push(format!(
            "FROM_USER_ID: {} != {}",
            left.from_user_id, right.from_user_id
        ));
    }
    if left.to_user_id != right.to_user_id {
        fields.push(format!(
            "TO_USER_ID: {} != {}",
            left.to_user_id, right.to_user_id
        ));
    }
    if left.amount != right.amount {
        fields.push(format!("AMOUNT: {} != {}", left.amount, right.amount));
    }
    if left.timestamp != right.timestamp {
        fields.push(format!(
            "TIMESTAMP: {} != {}",
            left.timestamp, right.timestamp
        ));
    }
    if left.status != right.status {
        fields.push(format!("STATUS: {} != {}", left.status, right.status));
    }
    let left_description = normalize_description(&left.description);
    let right_description = normalize_description(&right.description);
    if left_description != right_description {
        fields.push(format!(
            "DESCRIPTION: {:?} != {:?}",
            left_description, right_description
        ));
    }

    fields
}

fn transaction_sets_equivalent(left: &[Transaction], right: &[Transaction]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right.iter())
            .all(|(left, right)| transactions_equivalent(left, right))
}

fn transactions_equivalent(left: &Transaction, right: &Transaction) -> bool {
    left.tx_id == right.tx_id
        && left.tx_type == right.tx_type
        && left.from_user_id == right.from_user_id
        && left.to_user_id == right.to_user_id
        && left.amount == right.amount
        && left.timestamp == right.timestamp
        && left.status == right.status
        && normalize_description(&left.description) == normalize_description(&right.description)
}

fn normalize_description(value: &str) -> String {
    if !(value.len() >= 2 && value.starts_with('"') && value.ends_with('"')) {
        return value.to_owned();
    }

    match parse_quoted_description(value) {
        Some(parsed) => parsed,
        None => value.to_owned(),
    }
}

fn parse_quoted_description(value: &str) -> Option<String> {
    let mut chars = value[1..value.len() - 1].chars();
    let mut result = String::new();

    while let Some(ch) = chars.next() {
        if ch != '\\' {
            result.push(ch);
            continue;
        }

        let escaped = chars.next()?;
        match escaped {
            '\\' => result.push('\\'),
            '"' => result.push('"'),
            'n' => result.push('\n'),
            'r' => result.push('\r'),
            't' => result.push('\t'),
            _ => return None,
        }
    }

    Some(result)
}

fn parse_command<I>(args: I) -> Result<Command, Error>
where
    I: Iterator<Item = String>,
{
    let args: Vec<String> = args.collect();
    if args.is_empty() || args.iter().any(|value| value == "--help" || value == "-h") {
        return Ok(Command::Help);
    }

    let mut file1 = None;
    let mut format1 = None;
    let mut file2 = None;
    let mut format2 = None;
    let mut index = 0usize;

    while index < args.len() {
        match args[index].as_str() {
            "--file1" => {
                file1 = Some(argument_value(&args, &mut index, "--file1")?.to_owned());
            }
            "--format1" => {
                format1 = Some(argument_value(&args, &mut index, "--format1")?.parse()?);
            }
            "--file2" => {
                file2 = Some(argument_value(&args, &mut index, "--file2")?.to_owned());
            }
            "--format2" => {
                format2 = Some(argument_value(&args, &mut index, "--format2")?.parse()?);
            }
            other => {
                return Err(Error::InvalidArgument(format!("unknown argument {other}")));
            }
        }

        index += 1;
    }

    Ok(Command::Run(CompareArgs {
        file1: file1.ok_or_else(|| missing_argument("--file1"))?,
        format1: format1.ok_or_else(|| missing_argument("--format1"))?,
        file2: file2.ok_or_else(|| missing_argument("--file2"))?,
        format2: format2.ok_or_else(|| missing_argument("--format2"))?,
    }))
}

fn argument_value<'a>(args: &'a [String], index: &mut usize, flag: &str) -> Result<&'a str, Error> {
    *index += 1;
    args.get(*index)
        .map(String::as_str)
        .ok_or_else(|| Error::InvalidArgument(format!("missing value for {flag}")))
}

fn missing_argument(flag: &str) -> Error {
    Error::InvalidArgument(format!("missing required argument {flag}"))
}

fn usage() -> &'static str {
    "Usage: ypbank_compare --file1 <file> --format1 <csv|text|binary> --file2 <file> --format2 <csv|text|binary>"
}

#[cfg(test)]
mod tests {
    use super::*;
    use ypbank_converter::{TransactionStatus, TransactionType};

    fn sample_transaction() -> Transaction {
        Transaction {
            tx_id: 1,
            tx_type: TransactionType::Transfer,
            from_user_id: 10,
            to_user_id: 20,
            amount: 300,
            timestamp: 1_700_000_000_000,
            status: TransactionStatus::Success,
            description: String::from("Base transaction"),
        }
    }

    #[test]
    fn comparer_message_includes_changed_fields() {
        let left = sample_transaction();
        let mut right = sample_transaction();
        right.amount = 450;
        right.description = String::from("Updated transaction");

        let message = match first_difference_message(
            "left.csv",
            std::slice::from_ref(&left),
            "right.csv",
            std::slice::from_ref(&right),
        ) {
            Some(message) => message,
            None => panic!("difference should be reported"),
        };

        assert!(message.contains("Mismatch at record 1."));
        assert!(message.contains("AMOUNT: 300 != 450"));
        assert!(message.contains("DESCRIPTION: \"Base transaction\" != \"Updated transaction\""));
    }

    #[test]
    fn comparer_message_reports_extra_transaction() {
        let left = sample_transaction();

        let message = match first_difference_message(
            "left.csv",
            std::slice::from_ref(&left),
            "right.csv",
            &[],
        ) {
            Some(message) => message,
            None => panic!("difference should be reported"),
        };

        assert!(message.contains("left.csv"));
        assert!(message.contains("extra transaction"));
        assert!(message.contains("tx_id: 1"));
    }

    #[test]
    fn comparer_treats_wrapped_description_as_equivalent() {
        let left = sample_transaction();
        let mut right = sample_transaction();
        right.description = String::from("\"Base transaction\"");

        assert!(transactions_equivalent(&left, &right));
        assert!(transaction_sets_equivalent(
            std::slice::from_ref(&left),
            std::slice::from_ref(&right)
        ));
    }
}
