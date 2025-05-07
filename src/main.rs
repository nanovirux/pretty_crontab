use std::process::Command;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use std::io::Write;  // for write! and writeln!

fn main() {
    let output = Command::new("crontab")
        .arg("-l")
        .output()
        .expect("Failed to execute crontab -l");

    if output.stdout.is_empty() {
        println!("No crontab entries found.");
        return;
    }

    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let crontab = String::from_utf8_lossy(&output.stdout);

    for line in crontab.lines() {
        if line.trim().starts_with('#') {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue))).unwrap();
            writeln!(&mut stdout, "{}", line).unwrap();
            stdout.reset().unwrap();
        } else {
            let cols: Vec<&str> = line.split_whitespace().collect();
            if cols.len() >= 6 {
                let minute  = cols[0];
                let hour    = cols[1];
                let dom     = cols[2];
                let month   = cols[3];
                let dow     = cols[4];
                let command = cols[5..].join(" ");

                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green))).unwrap();
                println!(
                    "Schedule:   {:<30}",
                    cron_to_human_readable(minute, hour, dom, month, dow)
                );
                stdout.reset().unwrap();

                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Magenta))).unwrap();
                println!("Command:   {}", command);
                stdout.reset().unwrap();
            }
        }
    }
}

/// Build the human-readable schedule string, with extra cases for month/day combos.
fn cron_to_human_readable(
    minute: &str,
    hour: &str,
    day_of_month: &str,
    month: &str,
    day_of_week: &str,
) -> String {
    // 1) time portion
    let time_part = match (minute, hour) {
        ("*", "*") => "every minute".to_string(),
        ("*", h)   => {
            let human_hour = hour_to_ampm_string(h);
            format!("every minute during the {} hour", human_hour)
        }
        (m, "*")   => {
            let mm: u8 = m.parse().unwrap_or(0);
            format!("every hour at {:02} minutes past", mm)
        }
        (m, h)     => format_time(h, m),
    };

    // 2) special: month=* + dom!=* + dow!=* 
    if month == "*" && day_of_month != "*" && day_of_week != "*" {
        let day_name = dow_name(day_of_week);
        return format!(
            "{} every month on {} and every {}",
            time_part,
            day_of_month_with_suffix(day_of_month),
            day_name
        );
    }

    // 3) special: month!=* + dom="*" + dow!=*
    if month != "*" && day_of_month == "*" && day_of_week != "*" {
        let day_name = dow_name(day_of_week);
        return format!(
            "{} every {} in {}",
            time_part,
            day_name,
            month_to_string(month)
        );
    }

    // 4) fallback chaining
    let mut desc = time_part;
    if month != "*" {
        desc.push_str(&format!(" on {}", month_to_string(month)));
    }
    if day_of_month != "*" {
        desc.push_str(&format!(" {}", day_of_month_with_suffix(day_of_month)));
    }
    if day_of_week != "*" {
        let day_name = dow_name(day_of_week);
        if day_of_month != "*" {
            desc.push_str(&format!(" and every {}", day_name));
        } else {
            desc.push_str(&format!(" every {}", day_name));
        }
    }
    desc
}

/// Helper to map 0–6 → weekday name
fn dow_name(day_of_week: &str) -> &'static str {
    match day_of_week {
        "0" => "Sunday",
        "1" => "Monday",
        "2" => "Tuesday",
        "3" => "Wednesday",
        "4" => "Thursday",
        "5" => "Friday",
        "6" => "Saturday",
        _   => "Unknown",
    }
}

fn format_time(hour: &str, minute: &str) -> String {
    let hh: u8 = hour.parse().unwrap_or(0);
    let mm: u8 = minute.parse().unwrap_or(0);
    if hh >= 12 {
        if hh == 12 {
            format!("at 12:{:02} PM", mm)
        } else {
            format!("at {:02}:{:02} PM", hh - 12, mm)
        }
    } else {
        if hh == 0 {
            format!("at 12:{:02} AM", mm)
        } else {
            format!("at {:02}:{:02} AM", hh, mm)
        }
    }
}

fn hour_to_ampm_string(hour: &str) -> String {
    let h: u8 = hour.parse().unwrap_or(0);
    let (h12, ampm) = if h == 0 {
        (12, "AM")
    } else if h < 12 {
        (h, "AM")
    } else if h == 12 {
        (12, "PM")
    } else {
        (h - 12, "PM")
    };
    format!("{:02} {}", h12, ampm)
}

fn month_to_string(month: &str) -> String {
    match month {
        "1"  => "January",
        "2"  => "February",
        "3"  => "March",
        "4"  => "April",
        "5"  => "May",
        "6"  => "June",
        "7"  => "July",
        "8"  => "August",
        "9"  => "September",
        "10" => "October",
        "11" => "November",
        "12" => "December",
        _    => "Unknown month",
    }
    .to_string()
}

fn day_of_month_with_suffix(day: &str) -> String {
    let num: u32 = day.parse().unwrap_or(0);
    let suffix = if num % 100 / 10 == 1 {
        "th"
    } else {
        match num % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        }
    };
    format!("{}{}", num, suffix)
}
