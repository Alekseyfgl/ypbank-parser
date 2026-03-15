//! Пример: как `clap` упрощает CLI-приложения.
//!
//! Запуск:
//! `cargo run --example clap_demo -- --input records.csv --input-format csv --output-format binary --dry-run`
//!
//! Полезно попробовать и `--help`:
//! `cargo run --example clap_demo -- --help`

use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum FormatArg {
    Csv,
    Text,
    Binary,
}

impl FormatArg {
    fn extension(self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Text => "txt",
            Self::Binary => "bin",
        }
    }
}

#[derive(Debug, Parser)]
#[command(
    name = "clap_demo",
    version,
    about = "Показывает, как clap декларативно разбирает аргументы CLI",
    long_about = None
)]
struct Args {
    /// Путь к входному файлу
    #[arg(long)]
    input: String,

    /// Формат входных данных
    #[arg(long, value_enum)]
    input_format: FormatArg,

    /// Формат выходных данных
    #[arg(long, value_enum)]
    output_format: FormatArg,

    /// Просто распарсить и показать аргументы, ничего не выполняя
    #[arg(long, default_value_t = false)]
    dry_run: bool,
}

fn main() {
    let args = Args::parse();

    println!("clap уже превратил аргументы командной строки в типизированную структуру:");
    println!("{args:#?}");
    println!();

    println!("Что здесь полезно для новичка:");
    println!("1. Мы описали CLI как обычную Rust-структуру.");
    println!("2. `#[arg(long)]` автоматически создал флаги вроде `--input`.");
    println!("3. `ValueEnum` ограничил значения форматов: csv, text, binary.");
    println!("4. `--help` и сообщения об ошибках clap генерирует сам.");
    println!();

    let suggested_output = format!("converted.{}", args.output_format.extension());
    println!("Например, после разбора можно уже работать с нормальными типами, а не со строками.");
    println!(
        "Вход: {} ({:?}) -> выходной формат {:?} -> файл {}",
        args.input, args.input_format, args.output_format, suggested_output
    );

    if args.dry_run {
        println!();
        println!("Флаг --dry-run включен, поэтому это был только демонстрационный запуск.");
    }
}
