use clap::Parser;

const MIN_JOBS: usize = 1;
const MAX_JOBS: usize = 64;
const MIN_TIMEOUT_SECONDS: u64 = 1;
const MAX_TIMEOUT_SECONDS: u64 = 300;

fn parse_jobs(value: &str) -> Result<usize, String> {
    let jobs = value
        .parse::<usize>()
        .map_err(|_| format!("jobs must be an integer between {MIN_JOBS} and {MAX_JOBS}"))?;

    if (MIN_JOBS..=MAX_JOBS).contains(&jobs) {
        Ok(jobs)
    } else {
        Err(format!("jobs must be between {MIN_JOBS} and {MAX_JOBS}"))
    }
}

fn parse_timeout(value: &str) -> Result<u64, String> {
    let timeout = value.parse::<u64>().map_err(|_| {
        format!(
            "timeout must be an integer between {MIN_TIMEOUT_SECONDS} and {MAX_TIMEOUT_SECONDS} seconds"
        )
    })?;

    if (MIN_TIMEOUT_SECONDS..=MAX_TIMEOUT_SECONDS).contains(&timeout) {
        Ok(timeout)
    } else {
        Err(format!(
            "timeout must be between {MIN_TIMEOUT_SECONDS} and {MAX_TIMEOUT_SECONDS} seconds"
        ))
    }
}

#[derive(Debug, Parser)]
#[command(
    name = "media-resolver",
    version,
    about = "Resolve one or more supported links.",
    long_about = None
)]
pub struct Args {
    /// Links to resolve. Use '-' to read one link per line from stdin.
    #[arg(value_name = "LINK")]
    pub inputs: Vec<String>,

    /// Maximum number of concurrent jobs.
    #[arg(
        short = 'j',
        long = "jobs",
        default_value_t = 4,
        value_parser = parse_jobs
    )]
    pub jobs: usize,

    /// Emit machine-readable JSON.
    #[arg(long)]
    pub json: bool,

    /// Emit indented JSON. Implies --json.
    #[arg(long)]
    pub pretty: bool,

    /// Include every available resource variant in output.
    #[arg(long)]
    pub all_variants: bool,

    /// Per-request timeout in seconds.
    #[arg(long, default_value_t = 15, value_parser = parse_timeout)]
    pub timeout: u64,

    /// Disable ANSI colors. Accepted for scripting compatibility.
    #[arg(long)]
    pub no_color: bool,

    /// Print request lifecycle diagnostics to stderr.
    #[arg(short, long)]
    pub verbose: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jobs_parser_accepts_bounds() {
        assert_eq!(parse_jobs("1"), Ok(1));
        assert_eq!(parse_jobs("64"), Ok(64));
    }

    #[test]
    fn jobs_parser_rejects_out_of_range_values() {
        assert!(parse_jobs("0").is_err());
        assert!(parse_jobs("65").is_err());
        assert!(parse_jobs("not-a-number").is_err());
    }

    #[test]
    fn timeout_parser_accepts_bounds() {
        assert_eq!(parse_timeout("1"), Ok(1));
        assert_eq!(parse_timeout("300"), Ok(300));
    }

    #[test]
    fn timeout_parser_rejects_out_of_range_values() {
        assert!(parse_timeout("0").is_err());
        assert!(parse_timeout("301").is_err());
        assert!(parse_timeout("not-a-number").is_err());
    }
}
