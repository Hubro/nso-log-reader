use std::fs::File;
use std::io::{stdin, Write};
use std::path::Path;
use std::process::exit;

use clap::{CommandFactory, Parser};
use subprocess::Exec;

mod formatting;
use formatting::{print_logline, DateFormat};
mod parser;
use parser::{parse_log, ParseSource};
mod pattern_matching;
use pattern_matching::match_pattern;
mod tail;
use tail::tail;

const HELP_TEXT: &str = "
    Input one or more patterns to match a log file to read. The selected log file has to match
    every pattern you input. If multiple log files match, the one with the shortest name will be
    selected.

    Example:

    $ nso-log-reader cfs l3vpn
";

#[derive(Debug, Parser)]
#[clap(about = HELP_TEXT)]
struct Args {
    /// Read a NSO log file by matching substrings
    #[clap(value_parser)]
    patterns: Vec<String>,

    /// The path to a log file to parse
    #[clap(short = 'F', long, value_parser = file_exists)]
    logfile: Option<String>,

    /// Tail the file rather than paging it
    #[clap(short, long)]
    follow: bool,

    /// Print the entire file rather than paging it
    #[clap(short, long)]
    cat: bool,

    /// Show only the time, not the full date (implied when using "-f")
    #[clap(short, long)]
    time: bool,

    /// Print matches and exit, useful for troubleshooting
    #[clap(long)]
    print_matches: bool,
}

impl Args {
    fn custom_parse() -> Self {
        let mut args = Args::parse();

        if args.follow {
            args.time = true;
        }

        args
    }
}

fn main() {
    let args = Args::custom_parse();

    if let Err(error) = run_program(args) {
        // Write the error to STDERR
        eprintln!("{}", error);
        exit(1);
    }
}

fn run_program(args: Args) -> Result<(), String> {
    let filename: String;
    let source: ParseSource;
    let mut target: Box<dyn std::io::Write>;

    //
    // Figure out the source
    //

    if let Some(logfile) = args.logfile {
        filename = Path::new(&logfile)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        if args.follow {
            source = tail(&logfile)?.into();
        } else {
            source = File::open(&logfile).map_err(|err| err.to_string())?.into();
        }
    } else if !args.patterns.is_empty() {
        let matches = match_pattern(&args.patterns)?;

        if args.print_matches {
            match matches.len() {
                0 => println!("No matches"),
                _ => println!(
                    "{}",
                    matches
                        .iter()
                        .enumerate()
                        .map(|(i, x)| {
                            if i == 0 {
                                "* ".to_string() + x
                            } else {
                                "- ".to_string() + x
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                ),
            };

            return Ok(());
        }

        let best_match = matches.first().ok_or("No matches")?;

        let filepath = format!(
            "{}/logs/{}",
            std::env::var("NSO_RUN_DIR").unwrap(),
            best_match,
        );
        filename = Path::new(&filepath)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        if args.follow {
            source = tail(&filepath)?.into();
        } else {
            source = File::open(&filepath).map_err(|err| err.to_string())?.into();
        }
    } else if atty::is(atty::Stream::Stdin) {
        // No logfile arguments and STDIN is a TTY, just print help msg and exit
        return Args::command().print_help().map_err(|err| err.to_string());
    } else {
        filename = "(STDIN)".into();
        source = stdin().into();
    }

    //
    // Figure out the target
    //
    // (--follow implies --cat)
    //
    if args.cat || args.follow {
        target = Box::new(std::io::stdout());
    } else {
        target = Box::new(pager(&filename)?);
    }

    //
    // Parse away!
    //

    for logline in parse_log(source) {
        print_logline(
            &logline,
            &mut target,
            match args.time {
                true => &DateFormat::TimeOnly,
                false => &DateFormat::Full,
            },
        )
        .map_err(|err| err.to_string())?;
    }

    Ok(())
}

/// Parses a log file from the logfile command line option
fn pager(filename: &str) -> Result<impl Write, String> {
    let mut prompt = format!("Reading log: {}", filename);
    prompt = prompt.replace(':', "\\:");
    prompt = prompt.replace('.', "\\.");
    prompt = prompt.replace('?', "\\?");

    prompt = format!("{} ?e(END):[page %dm/%D] [%Pt\\%].", prompt);

    let pager_cmd = Exec::cmd("less")
        .arg("-SR")
        .arg("+G")
        .arg(format!("--prompt={}", prompt));

    pager_cmd.stream_stdin().map_err(|err| err.to_string())
}

fn file_exists(filepath: &str) -> Result<String, String> {
    if Path::new(filepath).exists() {
        Ok(String::from(filepath))
    } else {
        Err("File does not exist".to_string())
    }
}
