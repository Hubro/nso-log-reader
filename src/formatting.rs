use crate::parser::{LogLine, Severity};
use owo_colors::colors::{Blue, Default, Green, Magenta, Red, Yellow};
use owo_colors::Color;
use owo_colors::OwoColorize;

type DebugColor = Magenta;
type InfoColor = Green;
type WarningColor = Yellow;
type ErrorColor = Red;

pub fn print_logline(logline: &LogLine) {
    match logline {
        LogLine::Invalid(logline) => {
            print_message::<Default>(&Severity::INFO, &logline.text);
        }
        LogLine::Valid(logline) => {
            match logline.severity {
                Severity::DEBUG => print!("{}", " DBG".fg::<DebugColor>().bold()),
                Severity::INFO => print!("{}", "INFO".fg::<InfoColor>().bold()),
                Severity::WARNING => print!("{}", "WARN".fg::<WarningColor>().bold()),
                Severity::ERROR => print!("{}", " ERR".fg::<ErrorColor>().bold()),
                Severity::CRITICAL => print!("{}", " ERR".fg::<ErrorColor>().bold()),
            };

            print!(
                " {}",
                logline
                    .get_date()
                    .format("%H:%M %S%.3f")
                    .fg::<Blue>()
                    .bold()
            );

            print!(" {}", logline.get_logger().fg::<WarningColor>().bold());

            match logline.severity {
                Severity::DEBUG => {
                    print_message::<DebugColor>(&logline.severity, logline.get_message())
                }
                Severity::INFO => {
                    print_message::<InfoColor>(&logline.severity, logline.get_message())
                }
                Severity::WARNING => {
                    print_message::<WarningColor>(&logline.severity, logline.get_message())
                }
                Severity::ERROR => {
                    print_message::<ErrorColor>(&logline.severity, logline.get_message())
                }
                _ => print_message::<Default>(&logline.severity, logline.get_message()),
            }
        }
    }

    print!("\n");
}

fn print_message<T: Color>(severity: &Severity, message: &str) {
    if !is_multiline(message) {
        match severity {
            Severity::ERROR => print!(" {}", message.fg::<T>()),
            _ => print!(" {}", message),
        }
    } else {
        for line in message.split('\n') {
            print!("\n   {}", "|".fg::<T>().bold());

            match severity {
                Severity::ERROR => print!(" {}", line.fg::<ErrorColor>()),
                _ => print!(" {}", line),
            }
        }
    }
}

/// Returns true if the input string contains more than one line
fn is_multiline(text: &str) -> bool {
    text.find('\n').is_some()
}
