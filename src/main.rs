use std::process::ExitCode;

fn main() -> ExitCode {
    if let Err(err) = chronicle::cli::run() {
        eprintln!("{err}");
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}
