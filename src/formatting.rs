use crate::parser::LogLine;
use owo_colors::colors::{Blue, Default, Green, Magenta, Red, Yellow};
use owo_colors::Color;
use owo_colors::OwoColorize;

type DebugColor = Magenta;
type InfoColor = Green;
type WarningColor = Yellow;
type ErrorColor = Red;

pub fn print_logline(logline: &LogLine) {
    match logline.get_severity() {
        "DEBUG" => print!("{}", " DBG".fg::<DebugColor>().bold()),
        "INFO" => print!("{}", "INFO".fg::<InfoColor>().bold()),
        "WARNING" => print!("{}", "WARN".fg::<WarningColor>().bold()),
        "ERROR" => print!("{}", " ERR".fg::<ErrorColor>().bold()),
        x => print!("{}", x.fg::<Default>()),
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

    match logline.get_severity() {
        "DEBUG" => print_message::<DebugColor>(logline),
        "INFO" => print_message::<InfoColor>(logline),
        "WARNING" => print_message::<WarningColor>(logline),
        "ERROR" => print_message::<ErrorColor>(logline),
        _ => print_message::<Default>(logline),
    }

    print!("\n");
}

fn print_message<T: Color>(logline: &LogLine) {
    let msg = logline.get_message();

    if !is_multiline(msg) {
        if logline.get_severity() == "ERROR" {
            print!(" {}", msg.fg::<T>());
        } else {
            print!(" {}", msg);
        }
    } else {
        for line in msg.split('\n') {
            print!("\n   {}", "|".fg::<T>().bold());

            if logline.get_severity() == "ERROR" {
                print!(" {}", line.fg::<ErrorColor>());
            } else {
                print!(" {}", line);
            }
        }
    }
}

/// Returns true if the input string contains more than one line
fn is_multiline(text: &str) -> bool {
    text.find('\n').is_some()
}
