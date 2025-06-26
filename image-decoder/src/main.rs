use clap::Parser;
use lib::encryption_key::EncryptionKey;
use std::{
    fs::File,
    io::{Write, stdout},
    path::PathBuf,
};

type AnyError = Box<dyn std::error::Error>;
type AnyResult<T> = Result<T, AnyError>;

#[derive(Debug, Parser)]
#[command(version, about, arg_required_else_help(true))]
struct Args {
    #[arg(long, short = 'k')]
    encryption_key: Option<String>,

    file: PathBuf,
}

fn main() -> AnyResult<()> {
    let args = Args::parse();

    let file = File::open(args.file).expect("Failed to open file");

    let key = match args.encryption_key {
        Some(key) => Some(EncryptionKey::from_hex_str(&key)?),
        None => None,
    };

    let decrypted = lib::image::decrypt(key, file)?;

    stdout().lock().write_all(&decrypted)?;

    Ok(())
}
