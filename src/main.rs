use std::env;
use std::fs::File;
use std::io::{self, BufReader};
use std::process::ExitCode;
use ypbank_converter::{Error, Format, convert};

struct ConvertArgs {
    input: String,
    input_format: Format,
    output_format: Format,
}

enum Command {
    Help,
    Run(ConvertArgs),
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

fn run(args: ConvertArgs) -> Result<(), Error> {
    let input_file = File::open(&args.input)?;
    let reader = BufReader::new(input_file);
    let stdout = io::stdout();
    let handle = stdout.lock();

    convert(reader, args.input_format, handle, args.output_format)
}

fn parse_command<I>(args: I) -> Result<Command, Error>
where
    I: Iterator<Item = String>,
{
    let args: Vec<String> = args.collect();
    if args.is_empty() || args.iter().any(|value| value == "--help" || value == "-h") {
        return Ok(Command::Help);
    }

    let mut input = None;
    let mut input_format = None;
    let mut output_format = None;
    let mut index = 0usize;

    while index < args.len() {
        match args[index].as_str() {
            "--input" => {
                input = Some(argument_value(&args, &mut index, "--input")?.to_owned());
            }
            "--input-format" => {
                input_format = Some(argument_value(&args, &mut index, "--input-format")?.parse()?);
            }
            "--output-format" => {
                output_format =
                    Some(argument_value(&args, &mut index, "--output-format")?.parse()?);
            }
            other => {
                return Err(Error::InvalidArgument(format!("unknown argument {other}")));
            }
        }

        index += 1;
    }

    Ok(Command::Run(ConvertArgs {
        input: input.ok_or_else(|| missing_argument("--input"))?,
        input_format: input_format.ok_or_else(|| missing_argument("--input-format"))?,
        output_format: output_format.ok_or_else(|| missing_argument("--output-format"))?,
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
    "Usage: ypbank_converter --input <file> --input-format <csv|text|binary> --output-format <csv|text|binary>"
}
