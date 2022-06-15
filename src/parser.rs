use std::fs::File;
use std::io::{BufReader, Lines};
use std::ops::Range;

pub fn parse_log(lines: Lines<BufReader<File>>) -> LogParser {
    LogParser {
        lines: lines,
        buffer: None,
    }
}

pub struct LogParser {
    lines: Lines<BufReader<File>>,
    buffer: Option<String>,
}

#[derive(Debug)]
struct LogLineRanges {
    severity: Range<usize>,
    date: Range<usize>,
    logger: Range<usize>,
    thread: Range<usize>,
    message: usize, // Message is the rest of the line from this index
}

impl LogLineRanges {
    fn new() -> Self {
        Self {
            severity: 0..0,
            date: 0..0,
            logger: 0..0,
            thread: 0..0,
            message: 0,
        }
    }
}

impl LogParser {
    /// Give the next log line
    ///
    /// This will first give the buffered line, if any.
    fn take_next_line(&mut self) -> Option<String> {
        if let Some(line) = self.buffer.take() {
            return Some(line);
        }

        let read = self.lines.next()?;

        match read {
            Ok(line) => return Some(line),

            // Failed to read? Probably means a STDIN pipe was closed.
            Err(_) => return None,
        }
    }

    /// Put a line back into the line buffer
    ///
    /// The buffered line will be returned by take_next_line, basically allowing one line
    /// read-ahead
    fn untake_line(&mut self, line: String) {
        self.buffer = Some(line);
    }
}

impl Iterator for LogParser {
    type Item = LogLine;

    fn next(&mut self) -> Option<Self::Item> {
        let mut text = self.take_next_line()?;

        let pos = match parse_line(&text) {
            Some(x) => x,
            None => LogLineRanges::new(),
        };

        // This loop checks upcoming lines and adds them to the message of the current log line if
        // they can't be parsed
        loop {
            let next_line = match self.take_next_line() {
                Some(x) => x,
                None => break,
            };

            match parse_line(&next_line) {
                // The next line can be parsed as a log line, so we put it back
                Some(_) => {
                    self.untake_line(next_line);
                    break;
                }
                None => {
                    // Add the next line to the message of the curent line
                    text = format!("{}\n{}", text, next_line);
                }
            }
        }

        Some(LogLine {
            text: text,
            positions: pos,
        })
    }
}

#[derive(Debug)]
pub struct LogLine {
    text: String,
    positions: LogLineRanges,
}

impl LogLine {
    pub fn get_severity(&self) -> &str {
        let range = self.positions.severity.clone();
        return &self.text[range];
    }
    pub fn get_date(&self) -> &str {
        let range = self.positions.date.clone();
        return &self.text[range];
    }
    pub fn get_logger(&self) -> &str {
        let range = self.positions.logger.clone();
        return &self.text[range];
    }
    pub fn get_thread(&self) -> &str {
        let range = self.positions.thread.clone();
        return &self.text[range];
    }
    pub fn get_message(&self) -> &str {
        return &self.text[self.positions.message..];
    }
}

fn parse_line(line: &str) -> Option<LogLineRanges> {
    let mut pos = LogLineRanges::new();

    if line.chars().nth(0)? != '<' {
        return None;
    }

    pos.severity.start = 1;
    pos.severity.end = line.char_indices().find(|(_, x)| *x == '>')?.0;

    pos.date.start = pos.severity.end + 2;
    pos.date.end = pos.date.start
        + line[pos.date.start..]
            .char_indices()
            .find(|(_, x)| *x == ' ')?
            .0;

    pos.logger.start = pos.date.end + 1;
    pos.logger.end = pos.logger.start
        + line[pos.logger.start..]
            .char_indices()
            .find(|(_, x)| *x == ' ')?
            .0;

    pos.thread.start = pos.logger.end + 1;
    pos.thread.end = pos.thread.start
        + line[pos.thread.start..]
            .char_indices()
            .find(|(_, x)| *x == ':')?
            .0;

    pos.message = pos.thread.end + 4;

    if pos.message >= line.chars().count() {
        return None;
    }

    return Some(pos);
}