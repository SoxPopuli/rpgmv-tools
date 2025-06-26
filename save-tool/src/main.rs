use clap::{Parser, ValueEnum};
use lib::save::{compress_json, decompress_json_pretty};
use std::{io::Write, path::PathBuf};

type AnyError = Box<dyn std::error::Error>;
type AnyResult<T> = Result<T, AnyError>;

#[derive(Debug, Clone, ValueEnum)]
enum Mode {
    #[value(alias = "d")]
    Decode,
    #[value(alias = "e")]
    Encode,
}

#[derive(Debug, Parser)]
#[command(version, about, arg_required_else_help(true))]
struct Args {
    #[arg(value_enum)]
    mode: Mode,
    file: PathBuf,
}

fn main() -> AnyResult<()> {
    let args = Args::parse();

    let file_data = std::fs::read_to_string(args.file)?;
    let output = match args.mode {
        Mode::Decode => decompress_json_pretty(&file_data)?,
        Mode::Encode => compress_json(&file_data)?,
    };

    std::io::stdout().lock().write_all(output.as_bytes())?;

    Ok(())
}
