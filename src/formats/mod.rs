mod binary;
mod csv;
mod text;

pub use self::binary::{read_binary, write_binary};
pub use self::csv::{read_csv, write_csv};
pub use self::text::{read_text, write_text};

use crate::error::{Error, Result};
#[cfg(test)]
use crate::types::Transaction;

fn trim_line_end(line: &str) -> &str {
    line.trim_end_matches(['\r', '\n'])
}

fn parse_u64_field(field: &'static str, value: &str) -> Result<u64> {
    value.parse::<u64>().map_err(|error| Error::InvalidValue {
        field,
        value: value.to_owned(),
        details: error.to_string(),
    })
}

fn parse_i64_field(field: &'static str, value: &str) -> Result<i64> {
    value.parse::<i64>().map_err(|error| Error::InvalidValue {
        field,
        value: value.to_owned(),
        details: error.to_string(),
    })
}

fn escape_csv_field(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn escape_text_value(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 2);
    escaped.push('"');
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(ch),
        }
    }
    escaped.push('"');
    escaped
}

fn parse_text_value(field: &'static str, value: &str) -> Result<String> {
    if !(value.starts_with('"') && value.ends_with('"') && value.len() >= 2) {
        return Ok(value.to_owned());
    }

    let mut chars = value[1..value.len() - 1].chars();
    let mut result = String::new();

    while let Some(ch) = chars.next() {
        if ch != '\\' {
            result.push(ch);
            continue;
        }

        let escaped = chars.next().ok_or(Error::InvalidValue {
            field,
            value: value.to_owned(),
            details: String::from("незавершенная управляющая последовательность"),
        })?;

        match escaped {
            '\\' => result.push('\\'),
            '"' => result.push('"'),
            'n' => result.push('\n'),
            'r' => result.push('\r'),
            't' => result.push('\t'),
            _ => {
                return Err(Error::InvalidValue {
                    field,
                    value: value.to_owned(),
                    details: format!("неподдерживаемая управляющая последовательность \\{escaped}"),
                });
            }
        }
    }

    Ok(result)
}

fn normalize_description_for_text_output(value: &str) -> String {
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        match parse_text_value("DESCRIPTION", value) {
            Ok(parsed) => parsed,
            Err(_) => value.to_owned(),
        }
    } else {
        value.to_owned()
    }
}

#[cfg(test)]
pub(crate) fn transactions_equivalent(left: &Transaction, right: &Transaction) -> bool {
    left.tx_id == right.tx_id
        && left.tx_type == right.tx_type
        && left.from_user_id == right.from_user_id
        && left.to_user_id == right.to_user_id
        && left.amount == right.amount
        && left.timestamp == right.timestamp
        && left.status == right.status
        && normalize_description_for_text_output(&left.description)
            == normalize_description_for_text_output(&right.description)
}
