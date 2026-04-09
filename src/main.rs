use std::process::ExitCode;

fn main() -> ExitCode {
    match scrutt::cli::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
