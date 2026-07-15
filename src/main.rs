use std::process::ExitCode;

use clap::Parser;

use base_cli::cli::Cli;

fn main() -> ExitCode {
    let cli = Cli::parse();
    let json_errors = cli.json;
    match base_cli::commands::run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            if json_errors {
                eprintln!(
                    "{}",
                    serde_json::json!({
                        "ok": false,
                        "error": format!("{error:#}")
                    })
                );
            } else {
                eprintln!("error: {error:#}");
            }
            ExitCode::FAILURE
        }
    }
}
