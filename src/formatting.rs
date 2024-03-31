use std::io::Write;

use owo_colors::colors::{Blue, Green, Magenta, Red, Yellow};
use owo_colors::OwoColorize;

use crate::parser::{LogLine, Severity};

type DebugColor = Magenta;
type InfoColor = Green;
type WarningColor = Yellow;
type ErrorColor = Red;

#[derive(Debug)]
pub enum DateFormat {
    Full,
    TimeOnly,
}

pub fn print_logline(
    logline: &LogLine,
    target: &mut impl Write,
    dateformat: &DateFormat,
) -> std::io::Result<()> {
    // Shortcut for writing to 'target'
    macro_rules! put {
        ($($arg:tt)*) => {
            write!(target, $($arg)*)
        };
    }

    match logline {
        LogLine::Dangling(logline) => {
            put!("{}", logline.text)?;
        }
        LogLine::Normal(logline) => {
            // Shortcut for writing to 'target' with the current severity color
            macro_rules! putc {
                ($string:expr) => {
                    match logline.severity {
                        Severity::Debug => put!("{}", $string.fg::<DebugColor>())?,
                        Severity::Info => put!("{}", $string.fg::<InfoColor>())?,
                        Severity::Warning => put!("{}", $string.fg::<WarningColor>())?,
                        Severity::Error => put!("{}", $string.fg::<ErrorColor>())?,
                        Severity::Critical => put!("{}", $string.fg::<ErrorColor>())?,
                    }
                };
            }

            match logline.severity {
                Severity::Debug => putc!(" DBG".bold()),
                Severity::Info => putc!("INFO".bold()),
                Severity::Warning => putc!("WARN".bold()),
                Severity::Error => putc!(" ERR".bold()),
                Severity::Critical => putc!("CRIT".bold()),
            };

            put!(
                " {}",
                logline
                    .datetime
                    .format(match dateformat {
                        DateFormat::Full => "%Y-%m-%d %H:%M:%S%.3f",
                        DateFormat::TimeOnly => "%H:%M %S%.3f",
                    })
                    .fg::<Blue>()
                    .bold()
            )?;

            put!(" {}", logline.logger_name.fg::<WarningColor>().bold())?;
            put!(":")?;

            if !logline.message.contains('\n') {
                // Single-line message
                match logline.severity {
                    Severity::Error | Severity::Critical => {
                        putc!(logline.message.fg::<ErrorColor>());
                    }
                    _ => {
                        put!(" {}", logline.message)?;
                    }
                };
            } else {
                let line_count = logline.message.lines().count();

                // Multi-line log message, we draw a little box around it
                for (i, line) in logline.message.lines().enumerate() {
                    put!("\n")?;

                    if i < (line_count - 1) {
                        putc!("   │ ");
                    } else {
                        putc!("   ╰ ");
                    }

                    if matches!(logline.severity, Severity::Error | Severity::Critical) {
                        putc!(line);
                    } else {
                        put!("{}", line)?;
                    }
                }
            }
        }
    }

    put!("\n")?;

    Ok(())
}
