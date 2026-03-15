# Примеры (`examples/`)

В этой папке лежат небольшие учебные примеры, которые показывают отдельные идеи из проекта и соседних crates.

## Как запускать examples

Общий шаблон:

```bash
cargo run --example <имя_файла_без_rs>
```

Если самому примеру нужно передать аргументы командной строки, после имени примера нужен дополнительный `--`:

```bash
cargo run --example <example_name> -- <аргументы_примера>
```

## Список примеров

### `derive_traits`

Показывает, зачем типу полезны `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`.

Запуск:

```bash
cargo run --example derive_traits
```

### `error_from`

Показывает, как `std::io::Error` автоматически превращается в `ypbank_converter::Error` через `From` и оператор `?`.

Запуск:

```bash
cargo run --example error_from
```

Пример специально пытается открыть несуществующий файл `examples/no_such_file.txt`, чтобы показать обработку ошибки.

### `error_source`

Показывает, как у пользовательской ошибки можно получить исходную причину через `source()`.

Запуск:

```bash
cargo run --example error_source
```

### `strum_demo`

Показывает, как `strum` помогает автоматически реализовать преобразование enum <-> строка.

Запуск:

```bash
cargo run --example strum_demo
```

### `clap_demo`

Показывает, как `clap` декларативно разбирает аргументы командной строки.

Справка:

```bash
cargo run --example clap_demo -- --help
```

Демонстрационный запуск:

```bash
cargo run --example clap_demo -- \
  --input records.csv \
  --input-format csv \
  --output-format binary \
  --dry-run
```

Здесь первый `--` нужен для `cargo`, а всё после него получает уже сам example.
