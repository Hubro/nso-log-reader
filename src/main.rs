use std::cmp::Ordering;
use std::fs::File;
use std::io::{copy, BufRead, BufReader};
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
    #[clap(value_parser, multiple_values = true)]
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

    /// Creates and returns a command for running this application with these arguments
    ///
    /// Only includes options, as that's currently the only use case.
    fn make_cmd(&self) -> Exec {
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

    if !args.patterns.is_empty() {
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

    parse_from_stdin(&args)
}

fn parse_from_stdin(args: &Args) {
    let bufreader = BufReader::new(std::io::stdin());

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
    let filepath = Path::new(args.logfile.as_ref().unwrap());
    let filename = filepath.file_name().unwrap().to_str().unwrap();

    let self_cmd = args.make_cmd().stdin(File::open(filepath).unwrap());

    if args.follow {
        let tail_cmd = Exec::cmd("tail").args(&["-f", "-n", "100", filepath.to_str().unwrap()]);

        (tail_cmd | self_cmd).join().map(|_| ())
    } else if args.cat {
        self_cmd.join().map(|_| ())
    } else {
        let mut self_cmd_stdout = self_cmd.stream_stdout().unwrap();

        let mut prompt = format!("Reading log: {}", filename);
        prompt = prompt.replace(":", "\\:");
        prompt = prompt.replace(".", "\\.");
        prompt = prompt.replace("?", "\\?");

        prompt = format!("{} ?e(END):[page %dm/%D] [%Pt\\%].", prompt);

        let pager_cmd = Exec::cmd("less")
            .arg("-SR")
            .arg("+G")
            .arg("--header=0,5")
            .arg(format!("--prompt={}", prompt));

        let mut pager_cmd_stdin = pager_cmd.stream_stdin().unwrap();

        let _ = copy(&mut self_cmd_stdout, &mut pager_cmd_stdin);

        Ok(())
    }
}

fn parse_from_pattern(args: &Args) -> Result<(), String> {
    let patterns = &args.patterns;

    let mut matches = match match_pattern(patterns) {
        Ok(x) => x,
        Err(x) => return Err(format!("Failed to search for log files: {}", x)),
    };

    matches.sort_by(|a, b| match a.len().cmp(&b.len()) {
        Ordering::Equal => a.cmp(&b),
        x => x,
    });

    if args.print_matches {
        println!(
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
        );

        return Ok(());
    }

    let filepath = format!(
        "{}/logs/{}",
        std::env::var("NSO_RUN_DIR").unwrap(),
        &matches[0],
    );

    let new_args = Args {
        patterns: vec![],
        logfile: Some(filepath),
        follow: args.follow,
        cat: args.cat,
        time: args.time,
        print_matches: args.print_matches,
    };

    parse_from_file(&new_args).map_err(|e| e.to_string())
}

fn file_exists(filepath: &str) -> Result<String, String> {
    if Path::new(filepath).exists() {
        Ok(String::from(filepath))
    } else {
        Err("File does not exist".to_string())
    }
}
