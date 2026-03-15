use std::error::Error as _;
use std::io;
use ypbank_converter::Error;

fn main() {
    let io_error = io::Error::new(io::ErrorKind::NotFound, "файл не найден");
    let app_error = Error::from(io_error);

    println!("Наша ошибка: {app_error}");

    match app_error.source() {
        Some(source) => println!("Исходная причина внутри source(): {source}"),
        None => println!("Внутренней причины нет"),
    }
}
