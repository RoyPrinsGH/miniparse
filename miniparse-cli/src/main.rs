use std::{
    fs::{self},
    path::PathBuf,
};

use anyhow::anyhow;
use clap::{Parser, ValueEnum};
use env_logger::Builder as LoggerBuilder;
use log::LevelFilter;

#[derive(Debug, Clone, ValueEnum)]
enum Verbosity {
    Warnings,
    Silent,
    Debug,
}

/// Simple cli tool to introspect .ini files
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// File path of the .ini file
    #[arg(short, long)]
    path: PathBuf,

    /// Section name. Leave empty for global section.
    #[arg(short, long)]
    section: Option<String>,

    /// Key name
    #[arg(short, long)]
    key: String,

    /// Silent mode
    #[arg(value_enum, default_value_t = Verbosity::Warnings)]
    verbosity: Verbosity,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match args.verbosity {
        Verbosity::Silent => (),
        Verbosity::Warnings => LoggerBuilder::new().filter(None, LevelFilter::Warn).init(),
        Verbosity::Debug => LoggerBuilder::new().filter(None, LevelFilter::Debug).init(),
    }

    if let Some(extension) = args.path.extension() {
        if extension != "ini" {
            log::warn!("Specified file does not have an .ini extension!");
        }
    } else {
        log::warn!("Specified file does not have an .ini extension!");
    };

    // Try to read the file regardless

    let contents = fs::read_to_string(args.path)?;

    let ini_file = miniparse_lib::parse(contents.as_str())?;

    let found_section = match args.section {
        None => ini_file.get_global_section(),
        Some(name) => ini_file.get_section_by_name(name.as_str()),
    };

    let Some(section) = found_section else {
        return Err(anyhow!("The given ini file did not contain the specified section"))?;
    };

    match section.get_value_by_key(&args.key) {
        Some(value) => print!("{value}"),
        None => return Err(anyhow!("The given section did not contain the specified key"))?,
    }

    Ok(())
}
