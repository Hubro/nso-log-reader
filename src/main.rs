use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::exit;

use clap::{CommandFactory, Parser};
use subprocess::Exec;

use crate::formatting::{print_logline, DateFormat};
use crate::parser::parse_log;
use crate::pattern_matching::match_pattern;

mod formatting;
mod parser;
mod pattern_matching;

const HELP_TEXT: &str = "
    Input one or more patterns to match a log file to read. The selected log
    file has to match every pattern you input.

    Example:

    $ nso-log-reader cfs l3vpn
";

#[derive(Debug, Parser)]
#[clap(about = HELP_TEXT)]
struct Args {
    /// Read a NSO log file by matching substrings
    #[clap(value_parser, multiple_values = true)]
    patterns: Vec<String>,

    /// The path to a log file to parse
    #[clap(short, long, value_parser = file_exists)]
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
}

impl Args {
    fn custom_parse() -> Self {
        let mut args = Args::parse();

        if args.follow {
            args.time = true;
        }

        args
    }

    /// Creates and returns a command for running this application with these arguments
    ///
    /// Only includes options, as that's currently the only use case.
    fn make_cmd(self: &Self) -> Exec {
        let self_cmd = std::env::args().next().unwrap();
        let mut cmd = Exec::cmd(self_cmd);

        if self.follow {
            cmd = cmd.arg("-f");
        }
        if self.cat {
            cmd = cmd.arg("-c");
        }
        if self.time {
            cmd = cmd.arg("-t");
        }

        cmd
    }
}

fn main() {
    let args = Args::custom_parse();

    if args.logfile.is_some() {
        let result = parse_from_file(&args);

        if let Err(error) = result {
            println!("{}", error);
            exit(1);
        }
        return;
    }

    if args.patterns.len() > 0 {
        let result = parse_from_pattern(&args);

        if let Err(error) = result {
            println!("{}", error);
            exit(1);
        }
        return;
    }

    // No arguments given and STDIN is a TTY, just print help and exit
    if atty::is(atty::Stream::Stdin) {
        return Args::command().print_help().unwrap();
    }

    return parse_from_stdin(&args);
}

fn parse_from_stdin(args: &Args) {
    let bufreader = BufReader::new(std::io::stdin());

    dbg!(args.time);

    for line in parse_log(bufreader.lines()) {
        print_logline(
            &line,
            match args.time {
                true => &DateFormat::TimeOnly,
                false => &DateFormat::Full,
            },
        );
    }
}

fn parse_from_file(args: &Args) -> subprocess::Result<()> {
    let filepath = args.logfile.as_ref().unwrap();

    let self_cmd = args.make_cmd();

    if args.follow {
        let tail_cmd = Exec::cmd("tail").args(&["-f", "-n", "100", filepath]);

        (tail_cmd | self_cmd).join().map(|_| ())
    } else if args.cat {
        let cat_cmd = Exec::cmd("cat").arg(filepath);

        (cat_cmd | self_cmd).join().map(|_| ())
    } else {
        let cat_cmd = Exec::cmd("cat").arg(filepath);
        let pager_cmd = Exec::cmd("less").arg("-SR");

        (cat_cmd | self_cmd | pager_cmd).join().map(|_| ())
    }
}

fn parse_from_pattern(args: &Args) -> Result<(), String> {
    let patterns = &args.patterns;

    let matches = match match_pattern(patterns) {
        Ok(x) => x,
        Err(x) => return Err(format!("Failed to search for log files: {}", x)),
    };

    match matches.len() {
        0 => Err(format!("Pattern {:?} matched no log files", patterns)),
        2.. => {
            let file_list = matches
                .iter()
                .map(|x| "- ".to_string() + x)
                .collect::<Vec<_>>()
                .join("\n");

            Err(format!(
                "Pattern matched more than one file:\n{}",
                file_list
            ))
        }
        _ => {
            let filepath = format!(
                "{}/logs/{}",
                std::env::var("NSO_RUN_DIR").unwrap(),
                &matches[0]
            );

            let new_args = Args {
                patterns: vec![],
                logfile: Some(filepath),
                follow: args.follow,
                cat: args.cat,
                time: args.time,
            };

            return parse_from_file(&new_args).map_err(|e| e.to_string());
        }
    }
}

fn file_exists(filepath: &str) -> Result<String, String> {
    if Path::new(filepath).exists() {
        Ok(String::from(filepath))
    } else {
        Err("File does not exist".to_string())
    }
}
