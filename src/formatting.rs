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
    // A shortcut macro for writing to 'target'
    macro_rules! put {
        ($($arg:tt)*) => {
            write!(target, $($arg)*)
        };
    }

    match logline {
        LogLine::Continuation(logline) => {
            let text = format!("   | {}", logline.text);

            match logline.severity {
                // If part of an error, print the whole message red, for visibility
                Some(Severity::Error | Severity::Critical) => put!("{}", text.fg::<ErrorColor>())?,
                _ => put!("{}", text.fg::<Default>())?,
            }
        }
        LogLine::Normal(logline) => {
            match logline.severity {
                Severity::Debug => put!("{}", " DBG".fg::<DebugColor>().bold())?,
                Severity::Info => put!("{}", "INFO".fg::<InfoColor>().bold())?,
                Severity::Warning => put!("{}", "WARN".fg::<WarningColor>().bold())?,
                Severity::Error => put!("{}", " ERR".fg::<ErrorColor>().bold())?,
                Severity::Critical => put!("{}", " ERR".fg::<ErrorColor>().bold())?,
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
