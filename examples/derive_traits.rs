//! Пример: зачем нужны #[derive(Debug, Clone, Copy, PartialEq, Eq)]
//!
//! Запуск: `cargo run --example derive_traits`

use ypbank_converter::TransactionStatus;

fn main() {
    let status = TransactionStatus::Success;

    // --- Debug: печать для отладки через {:?} или {:#?}
    println!("=== Debug ===");
    println!("status = {:?}", status);
    println!("красиво: {:#?}", status);

    // --- Copy: при присваивании копируется значение, исходная переменная не "переезжает"
    println!("\n=== Copy ===");
    let a = TransactionStatus::Pending;
    let b = a; // копия, не перемещение
    println!("a = {:?}, b = {:?} — оба валидны", a, b);

    // --- Clone: явное клонирование (для Copy-типов обычно не нужно, но возможно)
    println!("\n=== Clone ===");
    let c = status.clone();
    println!("status.clone() = {:?}", c);

    // --- PartialEq / Eq: сравнение через == и !=
    println!("\n=== PartialEq / Eq ===");
    let s1 = TransactionStatus::Success;
    let s2 = TransactionStatus::Success;
    let s3 = TransactionStatus::Failure;
    println!("s1 == s2: {}", s1 == s2);
    println!("s1 != s3: {}", s1 != s3);

    // Практика: использование в условиях и в структурах данных
    if status == TransactionStatus::Success {
        println!("\nТранзакция успешна.");
    }

    let statuses = [
        TransactionStatus::Success,
        TransactionStatus::Pending,
        TransactionStatus::Failure,
    ];
    let success_count = statuses
        .iter()
        .filter(|s| **s == TransactionStatus::Success)
        .count();
    println!("В массиве {} успешных статусов.", success_count);
}
