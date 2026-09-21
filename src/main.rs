use std::process::ExitCode;

use copyast::CopyastCommand;
use copyast::logger::{self, LogLevel};

const METADATA_FLAG: &str = "--yuntuns-plugin-metadata";

fn main() -> ExitCode {
    let raw_args = std::env::args().collect::<Vec<_>>();

    if raw_args.get(1).map(String::as_str) == Some(METADATA_FLAG) {
        print_plugin_metadata();
        return ExitCode::SUCCESS;
    }

    logger::init_from_env();
    if raw_args
        .iter()
        .any(|argument| matches!(argument.as_str(), "-q" | "--quiet"))
    {
        logger::set_level(LogLevel::Error);
    }

    match CopyastCommand::new().execute(&raw_args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            logger::error(&error.to_string());
            ExitCode::FAILURE
        }
    }
}

fn print_plugin_metadata() {
    println!("protocol=1");
    println!("name=copyast");
    println!("version={}", env!("CARGO_PKG_VERSION"));
    println!("description=Copy text files into one AI context file");
}
