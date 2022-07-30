use crate::parser::{LogLine, Severity};
use owo_colors::colors::{Blue, Default, Green, Magenta, Red, Yellow};
use owo_colors::Color;
use owo_colors::OwoColorize;

type DebugColor = Magenta;
type InfoColor = Green;
type WarningColor = Yellow;
type ErrorColor = Red;

#[derive(Debug)]
pub enum DateFormat {
    Full,
    TimeOnly,
}

pub fn print_logline(logline: &LogLine, dateformat: &DateFormat) {
    match logline {
        LogLine::Invalid(logline) => {
            print_message::<Default>(&Severity::Info, &logline.text);
        }
        LogLine::Valid(logline) => {
            match logline.severity {
                Severity::Debug => print!("{}", " DBG".fg::<DebugColor>().bold()),
                Severity::Info => print!("{}", "INFO".fg::<InfoColor>().bold()),
                Severity::Warning => print!("{}", "WARN".fg::<WarningColor>().bold()),
                Severity::Error => print!("{}", " ERR".fg::<ErrorColor>().bold()),
                Severity::Critical => print!("{}", " ERR".fg::<ErrorColor>().bold()),
            };

            print!(
                " {}",
                logline
                    .get_date()
                    .format(match dateformat {
                        DateFormat::Full => "%Y-%m-%d %H:%M:%S%.3f",
                        DateFormat::TimeOnly => "%H:%M %S%.3f",
                    })
                    .fg::<Blue>()
                    .bold()
            );

            print!(" {}", logline.get_logger().fg::<WarningColor>().bold());

            match logline.severity {
                Severity::Debug => {
                    print_message::<DebugColor>(&logline.severity, logline.get_message())
                }
                Severity::Info => {
                    print_message::<InfoColor>(&logline.severity, logline.get_message())
                }
                Severity::Warning => {
                    print_message::<WarningColor>(&logline.severity, logline.get_message())
                }
                Severity::Error => {
                    print_message::<ErrorColor>(&logline.severity, logline.get_message())
                }
                _ => print_message::<Default>(&logline.severity, logline.get_message()),
            }
        }
    }

    println!();
}

fn print_message<T: Color>(severity: &Severity, message: &str) {
    if !is_multiline(message) {
        match severity {
            Severity::Error => print!(" {}", message.fg::<T>()),
            _ => print!(" {}", message),
        }
    } else {
        for line in message.split('\n') {
            print!("\n   {}", "|".fg::<T>().bold());

            match severity {
                Severity::Error => print!(" {}", line.fg::<ErrorColor>()),
                _ => print!(" {}", line),
            }
        }
    }
}

/// Returns true if the input string contains more than one line
fn is_multiline(text: &str) -> bool {
    text.find('\n').is_some()
}
