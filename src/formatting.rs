use crate::parser::{LogLine, Severity};
use owo_colors::colors::{Blue, Default, Green, Magenta, Red, Yellow};
use owo_colors::Color;
use owo_colors::OwoColorize;

type DebugColor = Magenta;
type InfoColor = Green;
type WarningColor = Yellow;
type ErrorColor = Red;

pub fn print_logline(logline: &LogLine) {
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
        Severity::DEBUG => print_message::<DebugColor>(logline),
        Severity::INFO => print_message::<InfoColor>(logline),
        Severity::WARNING => print_message::<WarningColor>(logline),
        Severity::ERROR => print_message::<ErrorColor>(logline),
        _ => print_message::<Default>(logline),
    }

    print!("\n");
}

fn print_message<T: Color>(logline: &LogLine) {
    let msg = logline.get_message();

    if !is_multiline(msg) {
        match logline.severity {
            Severity::ERROR => print!(" {}", msg.fg::<T>()),
            _ => print!(" {}", msg),
        }
    } else {
        for line in msg.split('\n') {
            print!("\n   {}", "|".fg::<T>().bold());

            match logline.severity {
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
