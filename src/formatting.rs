use std::io::Write;

use owo_colors::colors::{Blue, Default, Green, Magenta, Red, Yellow};
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

    // Shortcut for writing to 'target' with the current severity color
    macro_rules! putc {
        ($string:expr) => {
            match logline.severity() {
                Some(Severity::Debug) => put!("{}", $string.fg::<DebugColor>())?,
                Some(Severity::Info) => put!("{}", $string.fg::<InfoColor>())?,
                Some(Severity::Warning) => put!("{}", $string.fg::<WarningColor>())?,
                Some(Severity::Error) => put!("{}", $string.fg::<ErrorColor>())?,
                Some(Severity::Critical) => put!("{}", $string.fg::<ErrorColor>())?,
                None => put!("{}", $string.fg::<Default>())?,
            }
        };
    }

    match logline {
        LogLine::Continuation(logline) => {
            putc!("   ┃ ");

            match logline.severity {
                Some(Severity::Error | Severity::Critical) => {
                    putc!(logline.text);
                }
                _ => {
                    put!("{}", logline.text)?;
                }
            };
        }
        LogLine::Normal(logline) => {
            match logline.severity {
                Severity::Debug => putc!(" DBG".bold()),
                Severity::Info => putc!("INFO".bold()),
                Severity::Warning => putc!("WARN".bold()),
                Severity::Error => putc!(" ERR".bold()),
                Severity::Critical => putc!(" ERR".bold()),
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

            match logline.severity {
                // If part of an error, print the whole message red, for visibility
                Severity::Error | Severity::Critical => {
                    put!(": {}", logline.message.fg::<ErrorColor>())?
                }
                _ => put!(": {}", logline.message.fg::<Default>())?,
            }
        }
    }

    put!("\n")?;

    Ok(())
}
