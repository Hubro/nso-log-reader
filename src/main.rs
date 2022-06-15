use crate::parser::parse_log;
use std::io::BufRead;

mod parser;

fn main() {
    let testpath = "/home/tomas/nso-run-common/logs/ncs-python-vm-tnso-resource-allocator.log";

    let file = std::fs::File::open(testpath).unwrap();
    let bufreader = std::io::BufReader::new(file);

    for line in parse_log(bufreader.lines()).take(10) {
        print!("\n");
        println!("Severity: {}", line.get_severity());
        println!("Date: {}", line.get_date());
        println!("Logger: {}", line.get_logger());
        println!("Thread: {}", line.get_thread());
        println!("Message: {}", line.get_message());
        print!("\n");
    }
}
