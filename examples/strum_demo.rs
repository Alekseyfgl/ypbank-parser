//! Пример: как `strum` убирает boilerplate у enum'ов.
//!
//! Запуск:
//! `cargo run --example strum_demo`

use std::str::FromStr;
use strum::{Display, EnumString};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
enum TransactionTypeDemo {
    Deposit,
    Transfer,
    Withdrawal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
enum TransactionStatusDemo {
    Pending,
    Success,
    Failure,
}

fn main() {
    println!("`strum` полезен, когда enum нужно переводить в строку и обратно.");
    println!();

    let tx_type =
        TransactionTypeDemo::from_str("TRANSFER").expect("TRANSFER должен успешно распарситься");
    let status =
        TransactionStatusDemo::from_str("SUCCESS").expect("SUCCESS должен успешно распарситься");

    println!("Что сделал `EnumString`:");
    println!("\"TRANSFER\" -> {:?}", tx_type);
    println!("\"SUCCESS\"  -> {:?}", status);
    println!();

    println!("Что сделал `Display`:");
    println!("TransactionTypeDemo::Transfer -> {}", tx_type);
    println!("TransactionStatusDemo::Success -> {}", status);
    println!();

    println!("Почему это удобно:");
    println!("1. Не нужно руками писать `impl FromStr`.");
    println!("2. Не нужно руками писать `impl Display`.");
    println!("3. Меньше повторяющегося кода и меньше мест для опечаток.");
    println!();

    match TransactionStatusDemo::from_str("DONE") {
        Ok(value) => println!("Неожиданно распарсили значение: {value}"),
        Err(error) => {
            println!("Если строка неправильная, strum возвращает понятную ошибку:");
            println!("{error}");
        }
    }
}
