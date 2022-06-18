use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::exit;

use clap::{CommandFactory, Parser};
use subprocess::Exec;

use crate::formatting::print_logline;
use crate::parser::parse_log;
use crate::pattern_matching::match_pattern;

mod formatting;
mod parser;
mod pattern_matching;

#[derive(Debug, Parser)]
struct Args {
    /// Read a NSO log file by fuzzy pattern
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
}

fn main() {
    let clap_args = Args::parse();

    if clap_args.logfile.is_some() {
        let result = parse_from_file(&clap_args);

        if let Err(error) = result {
            println!("{}", error);
            exit(1);
        }
        return;
    }

    if clap_args.patterns.len() > 0 {
        let result = parse_from_pattern(&clap_args);

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

    return parse_from_stdin();
}

fn parse_from_stdin() {
    let bufreader = BufReader::new(std::io::stdin());

    for line in parse_log(bufreader.lines()) {
        print_logline(&line);
    }
}

fn parse_from_file(args: &Args) -> subprocess::Result<()> {
    let filepath = args.logfile.as_ref().unwrap();

    if args.follow {
        let tail_cmd = Exec::cmd("tail").args(&["-f", filepath]);
        let self_cmd = Exec::cmd(std::env::args().next().unwrap());

        (tail_cmd | self_cmd).join().map(|_| ())
    } else {
        let cat_cmd = Exec::cmd("cat").arg(filepath);
        let self_cmd = Exec::cmd(std::env::args().next().unwrap());
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
