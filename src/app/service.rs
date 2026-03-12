use clap::Parser;

use crate::cli::{args::Cli, fuzzy, handlers};

pub fn run() -> i32 {
    let args: Vec<String> = std::env::args().collect();
    let expanded_args = fuzzy::expand_command(args);

    let cli = Cli::try_parse_from(expanded_args.iter()).unwrap_or_else(|e| {
        e.exit();
    });

    exit_code_for(handlers::handle(cli))
}

fn exit_code_for(result: Result<(), crate::app::app_error::AppError>) -> i32 {
    match result {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("{error}");
            error.exit_code()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::exit_code_for;
    use crate::app::app_error::AppError;

    #[test]
    fn exit_code_for_success_is_zero() {
        assert_eq!(exit_code_for(Ok(())), 0);
    }

    #[test]
    fn exit_code_for_usage_errors_is_two() {
        assert_eq!(exit_code_for(Err(AppError::usage("bad args"))), 2);
    }

    #[test]
    fn exit_code_for_operational_errors_is_one() {
        assert_eq!(exit_code_for(Err(AppError::message("boom"))), 1);
    }
}
