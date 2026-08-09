mod args;
mod client;
mod output;
mod runner;

use std::{
    io::{self, BufRead, IsTerminal},
    process::ExitCode,
    time::Duration,
};

use clap::Parser;

use crate::{
    args::Args,
    output::{print_human, print_json, trim_variants},
};

#[tokio::main]
async fn main() -> ExitCode {
    let args = Args::parse();

    let inputs = match collect_inputs(args.inputs) {
        Ok(inputs) if !inputs.is_empty() => inputs,
        Ok(_) => {
            eprintln!("no links were provided");
            return ExitCode::from(2);
        }
        Err(error) => {
            eprintln!("failed to read input: {error}");
            return ExitCode::from(2);
        }
    };

    let operation = runner::run(
        inputs,
        args.jobs,
        Duration::from_secs(args.timeout),
        args.verbose,
    );

    let mut results = tokio::select! {
        result = operation => match result {
            Ok(results) => results,
            Err(error) => {
                eprintln!("failed to initialize the remote client: {error}");
                return ExitCode::from(1);
            }
        },
        signal = tokio::signal::ctrl_c() => {
            if let Err(error) = signal {
                eprintln!("failed to listen for cancellation: {error}");
                return ExitCode::from(1);
            }
            eprintln!("cancelled");
            return ExitCode::from(130);
        }
    };

    trim_variants(&mut results, args.all_variants);
    let has_failures = results
        .iter()
        .any(|result| matches!(result.state, runner::ResultState::Failed));

    let output_result = if args.json || args.pretty {
        print_json(&results, args.pretty).map_err(|error| error.to_string())
    } else {
        let color = !args.no_color && io::stderr().is_terminal();
        print_human(&results, color);
        Ok(())
    };

    if let Err(error) = output_result {
        eprintln!("failed to serialize output: {error}");
        return ExitCode::from(1);
    }

    if has_failures {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn collect_inputs(raw: Vec<String>) -> io::Result<Vec<String>> {
    raw.into_iter().try_fold(Vec::new(), |mut output, input| {
        if input == "-" {
            for line in io::stdin().lock().lines() {
                let value = line?.trim().to_owned();
                if !value.is_empty() {
                    output.push(value);
                }
            }
        } else {
            let value = input.trim();
            if !value.is_empty() {
                output.push(value.to_owned());
            }
        }

        Ok(output)
    })
}
