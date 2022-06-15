use crate::formatting::print_logline;
use crate::parser::parse_log;
use std::io::BufRead;

mod formatting;
mod parser;

fn main() {
    let bufreader = std::io::BufReader::new(std::io::stdin());

    for line in parse_log(bufreader.lines()) {
        print_logline(&line);
    }
}
