use chrono::offset::{Local, Utc};
use chrono::{DateTime, NaiveDateTime};
use std::io::{BufReader, Lines, Read};
use std::ops::Range;

pub fn parse_log<T: Read>(lines: Lines<BufReader<T>>) -> LogParser<T> {
    LogParser {
        lines,
        buffer: None,
    }
}

pub struct LogParser<T: Read> {
    lines: Lines<BufReader<T>>,
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

impl<T: Read> LogParser<T> {
    /// Give the next log line
    ///
    /// This will first give the buffered line, if any.
    fn take_next_line(&mut self) -> Option<String> {
        if let Some(line) = self.buffer.take() {
            return Some(line);
        }

        let read = self.lines.next()?;

        match read {
            Ok(line) => Some(line),

            // Failed to read? Probably means a STDIN pipe was closed.
            Err(_) => None,
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

impl<T: Read> Iterator for LogParser<T> {
    type Item = LogLine;

    fn next(&mut self) -> Option<Self::Item> {
        let mut text = self.take_next_line()?;

        let positions = match parse_line(&text) {
            Some(x) => x,
            None => LogLineRanges::new(),
        };

        // This loop checks upcoming lines and adds them to the message of the current log line if
        // they can't be parsed
        while let Some(next_line) = self.take_next_line() {
            match parse_line(&next_line) {
                // The next line can be parsed as a log line, so we put it back
                Some(_) => {
                    self.untake_line(next_line);
                    break;
                }
                None => {
                    // Ignore empty lines
                    if !next_line.trim().is_empty() {
                        // Add the next line to the message of the curent line
                        text = format!("{}\n{}", text, next_line);
                    }
                }
            }
        }

        if positions.message == 0 {
            return Some(LogLine::Invalid(InvalidLogLine { text }));
        }

        let severity: Severity = match positions.severity.end {
            0 => Severity::Info, // Default to INFO if the line can't be parsed
            _ => text[positions.severity.clone()].into(),
        };

        Some(LogLine::Valid(ValidLogLine {
            text,
            severity,
            positions,
        }))
    }
}

pub enum Severity {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

impl From<&str> for Severity {
    fn from(text: &str) -> Self {
        match text {
            "DEBUG" => Severity::Debug,
            "INFO" => Severity::Info,
            "WARNING" => Severity::Warning,
            "ERROR" => Severity::Error,
            "CRITICAL" => Severity::Critical,
            _ => panic!("Unexpected severity: {}", text),
        }
    }
}

pub struct ValidLogLine {
    text: String,
    pub severity: Severity,
    positions: LogLineRanges,
}

impl ValidLogLine {
    pub fn get_date(&self) -> DateTime<Local> {
        let datetime_text = &self.text[self.positions.date.clone()];

        let test = datetime_text.split('.').next().unwrap();

        let naivedatetime =
            match NaiveDateTime::parse_from_str(datetime_text, "%d-%b-%Y::%H:%M:%S%.3f") {
                Ok(datetime) => datetime,
                Err(e) => {
                    panic!("Fatal error, failed to parse time {:?}: {}", test, e);
                }
            };

        let utcdatetime = DateTime::<Utc>::from_utc(naivedatetime, Utc);

        DateTime::from(utcdatetime)
    }
    pub fn get_logger(&self) -> &str {
        let range = self.positions.logger.clone();
        &self.text[range]
    }
    pub fn get_message(&self) -> &str {
        &self.text[self.positions.message..]
    }
}

fn parse_line(line: &str) -> Option<LogLineRanges> {
    let mut pos = LogLineRanges::new();

    if line.chars().next()? != '<' {
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
    pos.thread.end = pos.thread.start + line[pos.thread.start..].find(": ")?;

    pos.message = pos.thread.end + 4;

    if pos.message >= line.chars().count() {
        return None;
    }

    Some(pos)
}

pub struct InvalidLogLine {
    pub text: String,
}

pub enum LogLine {
    Valid(ValidLogLine),
    Invalid(InvalidLogLine),
}
