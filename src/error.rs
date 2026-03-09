use std::fmt::{Display, Formatter};
use std::io;

/// Удобный псевдоним результата, используемый библиотекой.
pub type Result<T> = std::result::Result<T, Error>;

/// Ошибки, которые возвращают парсеры, сериализаторы и CLI-утилиты YPBank.
#[derive(Debug)]
pub enum Error {
    /// Обертка над ошибками ввода-вывода.
    Io(io::Error),
    /// Неизвестное имя формата, переданное из CLI или пользовательского ввода.
    UnknownFormat(String),
    /// Некорректный аргумент командной строки или неподдерживаемое значение.
    InvalidArgument(String),
    /// Заголовок CSV не соответствует спецификации.
    InvalidCsvHeader {
        /// Строка заголовка, фактически прочитанная из файла.
        found: String,
    },
    /// Строка CSV имеет неверный формат.
    InvalidCsvRecord {
        /// Номер некорректной CSV-записи, начиная с 1.
        line: usize,
        /// Человекочитаемые подробности ошибки парсинга.
        details: String,
    },
    /// Текстовая строка имеет неверный формат.
    InvalidTextLine {
        /// Номер некорректной текстовой строки, начиная с 1.
        line: usize,
        /// Человекочитаемые подробности ошибки парсинга.
        details: String,
    },
    /// Текстовая запись неполная или противоречивая.
    InvalidTextRecord {
        /// Номер записи в текстовом файле, начиная с 1.
        record: usize,
        /// Человекочитаемые подробности ошибки парсинга.
        details: String,
    },
    /// В записи отсутствует обязательное поле.
    MissingField {
        /// Номер записи, где отсутствует поле, начиная с 1.
        record: usize,
        /// Имя отсутствующего поля.
        field: &'static str,
    },
    /// Поле повторяется внутри текстовой записи.
    DuplicateField {
        /// Номер записи, где поле дублируется, начиная с 1.
        record: usize,
        /// Имя дублирующегося поля.
        field: &'static str,
    },
    /// Значение поля не удалось распарсить или провалидировать.
    InvalidValue {
        /// Имя поля с некорректным значением.
        field: &'static str,
        /// Исходное текстовое или числовое представление.
        value: String,
        /// Человекочитаемые подробности валидации.
        details: String,
    },
    /// Сигнатура бинарной записи некорректна.
    InvalidBinaryMagic {
        /// Номер бинарной записи, начиная с 1.
        record: usize,
        /// Четыре байта, фактически прочитанные вместо ожидаемой сигнатуры.
        found: [u8; 4],
    },
    /// Размер бинарной записи меньше фиксированной части тела.
    InvalidBinaryRecordSize {
        /// Номер бинарной записи, начиная с 1.
        record: usize,
        /// Значение размера из заголовка записи.
        size: u32,
    },
    /// Тело бинарной записи имеет неверный формат.
    InvalidBinaryRecord {
        /// Номер бинарной записи, начиная с 1.
        record: usize,
        /// Человекочитаемые подробности ошибки парсинга.
        details: String,
    },
    /// Входные данные закончились до чтения полной структуры.
    UnexpectedEof {
        /// Структура, которая читалась в момент окончания входных данных.
        context: &'static str,
    },
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "ошибка ввода-вывода: {error}"),
            Self::UnknownFormat(value) => write!(f, "неизвестный формат: {value}"),
            Self::InvalidArgument(details) => write!(f, "некорректный аргумент: {details}"),
            Self::InvalidCsvHeader { found } => {
                write!(f, "некорректный заголовок CSV: {found}")
            }
            Self::InvalidCsvRecord { line, details } => {
                write!(f, "некорректная CSV-запись в строке {line}: {details}")
            }
            Self::InvalidTextLine { line, details } => {
                write!(f, "некорректная текстовая строка {line}: {details}")
            }
            Self::InvalidTextRecord { record, details } => {
                write!(f, "некорректная текстовая запись {record}: {details}")
            }
            Self::MissingField { record, field } => {
                write!(f, "в записи {record} отсутствует обязательное поле {field}")
            }
            Self::DuplicateField { record, field } => {
                write!(f, "запись {record} содержит повторяющееся поле {field}")
            }
            Self::InvalidValue {
                field,
                value,
                details,
            } => write!(f, "некорректное значение поля {field} ({value}): {details}"),
            Self::InvalidBinaryMagic { record, found } => {
                write!(
                    f,
                    "некорректная сигнатура бинарной записи {record}: {found:02X?}"
                )
            }
            Self::InvalidBinaryRecordSize { record, size } => {
                write!(f, "некорректный размер бинарной записи {record}: {size}")
            }
            Self::InvalidBinaryRecord { record, details } => {
                write!(f, "некорректная бинарная запись {record}: {details}")
            }
            Self::UnexpectedEof { context } => {
                write!(f, "неожиданный конец входных данных при чтении {context}")
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}
