use std::{
    fs::File,
    io::{BufRead, BufReader, Lines, Read, Stdin},
    os::fd::AsRawFd,
    process::ChildStdout,
    str::FromStr,
};

use chrono::{TimeZone, Utc};

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

#[derive(Debug)]
pub struct ContinuationLogLine {
    // None means unknown, can happen if the log starts on a continuation
    pub severity: Option<Severity>,
    pub text: String,
}

#[derive(Debug)]
pub enum LogLine {
    Normal(NormalLogLine),
    Continuation(ContinuationLogLine),
}

impl LogLine {
    pub fn severity(&self) -> Option<Severity> {
        match self {
            LogLine::Normal(logline) => Some(logline.severity),
            LogLine::Continuation(logline) => logline.severity,
        }
    }
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
    severity: Option<Severity>, // Keeps track of the previous line's severity
}

impl<T: Read + AsRawFd> Iterator for LogParser<T> {
    type Item = LogLine;

    fn next(&mut self) -> Option<Self::Item> {
        let line = self.lines.next()?.expect("Failed to read a line");

        Some(match line.parse::<NormalLogLine>() {
            Ok(log_line) => {
                self.severity = Some(log_line.severity.clone());
                LogLine::Normal(log_line)
            }
            Err(_) => LogLine::Continuation(ContinuationLogLine {
                severity: self.severity,
                text: line,
            }),
        })
    }
}

pub fn parse_log(source: ParseSource) -> LogParser<impl Read + AsRawFd> {
    LogParser {
        lines: BufReader::new(source).lines(),
        severity: None,
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

    let datetime = Utc
        .datetime_from_str(&line[date_start..date_end], "%d-%b-%Y::%H:%M:%S%.3f")
        .ok()?;

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
        message_start = message_start + 2;
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
