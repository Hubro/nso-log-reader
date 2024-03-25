use crate::parser::{LogLine, Severity};
use owo_colors::colors::{Blue, Default, Green, Magenta, Red, Yellow};
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
        LogLine::Continuation(logline) => {
            let text = format!("   | {}", logline.text);

            match logline.severity {
                // If part of an error, print the whole message red, for visibility
                Some(Severity::Error | Severity::Critical) => {
                    print!("{}", text.fg::<ErrorColor>())
                }
                _ => print!("{}", text.fg::<Default>()),
            }
        }
        LogLine::Normal(logline) => {
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
                    .datetime
                    .format(match dateformat {
                        DateFormat::Full => "%Y-%m-%d %H:%M:%S%.3f",
                        DateFormat::TimeOnly => "%H:%M %S%.3f",
                    })
                    .fg::<Blue>()
                    .bold()
            );

            print!(" {}", logline.logger_name.fg::<WarningColor>().bold());

            match logline.severity {
                // If part of an error, print the whole message red, for visibility
                Severity::Error | Severity::Critical => {
                    print!(": {}", logline.message.fg::<ErrorColor>())
                }
                _ => print!(": {}", logline.message.fg::<Default>()),
            }
        }
    }

    println!();
}
