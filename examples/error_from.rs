use std::fs::File;
use ypbank_converter::Error;

fn open_missing_file() -> Result<(), Error> {
    // `File::open` возвращает `std::io::Error`.
    // Оператор `?` автоматически превращает его в `ypbank_converter::Error`
    // благодаря `impl From<std::io::Error> for Error`.
    File::open("examples/no_such_file.txt")?;
    Ok(())
}

fn main() {
    match open_missing_file() {
        Ok(()) => println!("Файл открылся без ошибок"),
        Err(error) => {
            println!("Получили наш тип ошибки: {error}");
            println!("Полное значение: {error:?}");
        }
    }
}
