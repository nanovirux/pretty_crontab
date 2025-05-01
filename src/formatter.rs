use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use std::io::Write;  // This import is necessary for `write!` to work with StandardStream

pub fn print_pretty_crontab(crontab: &str) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);

    for line in crontab.lines() {
        if line.trim().is_empty() {
            continue; // Skip empty lines
        }

        // Check if the line is a comment
        if line.trim().starts_with("#") {
            // Set color to light blue for comment lines
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue))).unwrap();
            write!(&mut stdout, "{:<20}", line).unwrap(); // Print comment in light blue
            stdout.reset().unwrap();  // Reset color for the rest of the line
            println!();  // Ensure a new line after the comment
        } else {
            // Print the crontab in the default formatting
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() > 5 {
                let time = &fields[0..5].join(" ");
                let command = fields[5..].join(" ");
                // Coloring the time part in green
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green))).unwrap();
                write!(&mut stdout, "{:<20}", time).unwrap();
                stdout.reset().unwrap();
                println!("{}", command);
            }
        }
    }
}

