use std::{
    fs::File,
    io::{BufRead, BufReader, Lines, Read, Stdin},
    os::fd::AsRawFd,
    process::ChildStdout,
    str::FromStr,
    time::Duration,
};

use chrono::NaiveDateTime;
use timeout_readwrite::TimeoutReadExt;

#[derive(Clone, Copy, Debug)]
pub enum Severity {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug)]
pub struct NormalLogLine {
    pub severity: Severity,
    pub datetime: chrono::DateTime<chrono::Utc>,
    pub logger_name: String,
    pub thread: String,
    pub message: String,
}

impl FromStr for NormalLogLine {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_line(s).ok_or(())
    }
}

/// A log line that couldn't be parsed and also couldn't be associated with a previous log line
///
/// This happens when the log starts with a cut-off multi-line log message, common when parsing
/// from "tail".
///
#[derive(Debug)]
pub struct DanglingLogLine {
    pub text: String,
}

#[derive(Debug)]
pub enum LogLine {
    Normal(NormalLogLine),
    Dangling(DanglingLogLine),
}

pub enum ParseSource {
    Stdin(Stdin),
    /// Filename, file
    File(File),
    /// Filename, tail stdout
    Tail(ChildStdout),
}

impl From<Stdin> for ParseSource {
    fn from(stdin: Stdin) -> Self {
        Self::Stdin(stdin)
    }
}

impl From<File> for ParseSource {
    fn from(file: File) -> Self {
        Self::File(file)
    }
}

impl From<ChildStdout> for ParseSource {
    fn from(tail_stdout: ChildStdout) -> Self {
        Self::Tail(tail_stdout)
    }
}

impl Read for ParseSource {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            ParseSource::Stdin(stdin) => stdin.read(buf),
            ParseSource::File(file) => file.read(buf),
            ParseSource::Tail(tail_stdout) => tail_stdout.read(buf),
        }
    }
}

impl AsRawFd for ParseSource {
    fn as_raw_fd(&self) -> std::os::unix::prelude::RawFd {
        match self {
            ParseSource::Stdin(stdin) => stdin.as_raw_fd(),
            ParseSource::File(file) => file.as_raw_fd(),
            ParseSource::Tail(tail_stdout) => tail_stdout.as_raw_fd(),
        }
    }
}

pub struct LogParser<T: Read + AsRawFd> {
    lines: Lines<BufReader<T>>,
    /// Holds the *next* log message, since we need to read ahead to see if the next line is part
    /// of the current log message
    buffer: Option<NormalLogLine>,
}

impl<T: Read + AsRawFd> Iterator for LogParser<T> {
    type Item = LogLine;

    fn next(&mut self) -> Option<Self::Item> {
        let mut log_message: NormalLogLine = if let Some(log_message) = self.buffer.take() {
            log_message
        } else {
            let line = loop {
                match self.lines.next() {
                    Some(Ok(line)) => break line,

                    // Do nothing, wait for the next log line to be emitted. This can happen while
                    // tailing a file or while parsing from STDIN.
                    Some(Err(e)) if e.kind() == std::io::ErrorKind::TimedOut => {}

                    // Let's panic, just to find out which errors can happen here
                    Some(Err(e)) => panic!("Fatal error: {}", e),

                    // End of iterator
                    None => return None,
                };
            };

            match line.parse::<NormalLogLine>() {
                Ok(log_message) => log_message,
                Err(_) => {
                    return Some(LogLine::Dangling(DanglingLogLine { text: line }));
                }
            }
        };

        // Read ahead to grab any lines that belong to the same log message. (Any line that can't
        // be parsed as a new log message.)
        loop {
            let next_line = match self.lines.next() {
                Some(Ok(line)) => line,

                // If we time out, that means we're waiting for new log messages. The means there
                // are definitely no more lines associated with the current log message.
                Some(Err(e)) if e.kind() == std::io::ErrorKind::TimedOut => {
                    return Some(LogLine::Normal(log_message))
                }

                // Let's panic, just to find out which errors can happen here
                Some(Err(e)) => panic!("Fatal error: {}", e),

                // End of iterator
                None => return Some(LogLine::Normal(log_message)),
            };

            match next_line.parse::<NormalLogLine>() {
                Ok(next_log_message) => {
                    self.buffer = Some(next_log_message);
                    return Some(LogLine::Normal(log_message));
                }
                Err(_) => {
                    // Add next_line as a new line to the end of log_message.message
                    log_message.message.push('\n');
                    log_message.message.push_str(&next_line);
                }
            }
        }
    }
}

pub fn parse_log(source: ParseSource) -> LogParser<impl Read + AsRawFd> {
    LogParser {
        lines: BufReader::new(source.with_timeout(Duration::from_millis(10))).lines(),
        buffer: None,
    }
}

fn parse_line(line: &str) -> Option<NormalLogLine> {
    if line.chars().next()? != '<' {
        return None;
    }

    let severity_start = 1;
    let severity_end = line.char_indices().find(|(_, x)| *x == '>')?.0;

    let severity = match &line[severity_start..severity_end] {
        "DEBUG" => Severity::Debug,
        "INFO" => Severity::Info,
        "WARN" => Severity::Warning,
        "WARNING" => Severity::Warning,
        "ERR" => Severity::Error,
        "ERROR" => Severity::Error,
        "CRIT" => Severity::Critical,
        "CRITICAL" => Severity::Critical,
        _ => return None,
    };

    let date_start = severity_end + 2;
    let date_end = date_start
        + line[date_start..]
            .char_indices()
            .find(|(_, x)| *x == ' ')?
            .0;

    let datetime =
        NaiveDateTime::parse_from_str(&line[date_start..date_end], "%d-%b-%Y::%H:%M:%S%.3f")
            .ok()?
            .and_utc();

    let logger_name_start = date_end + 1;
    let logger_name_end = logger_name_start
        + line[logger_name_start..]
            .char_indices()
            .find(|(_, x)| *x == ' ')?
            .0;

    let logger_name = line[logger_name_start..logger_name_end].to_string();

    let thread_start = logger_name_end + 1;
    let thread_end = thread_start + line[thread_start..].find(": ")?;

    let thread = line[thread_start..thread_end].to_string();
    let mut message_start = thread_end + 2;

    // ncs-python-vm-*.log (for some reason) uses ": - " as the message delimiter, but
    // ncs-python-vm.log doesn't
    if &line[message_start..message_start + 2] == "- " {
        message_start += 2;
    }

    if message_start >= line.chars().count() {
        return None;
    }

    let message = line[message_start..].to_string();

    Some(NormalLogLine {
        severity,
        datetime,
        logger_name,
        thread,
        message,
    })
}
