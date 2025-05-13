use std::{collections::{BTreeMap, HashMap}, fs, process::Command};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use std::io::{self, Write};
use clap::Parser;

/// A cron viewer that pretty-prints crontab entries or shows histograms and detailed monthly breakdown.
#[derive(Parser)]
#[command(
    name = "pretty_crontab",
    about = "Pretty-prints your crontab or shows histograms by hour, weekday, or month",
    long_about = None
)]
struct Args {
    /// Bar-chart of jobs per hour
    #[arg(long)]
    chart: bool,

    /// Bar-chart of jobs per day of week (Sun–Sat)
    #[arg(long = "chart-dow")]
    chart_dow: bool,

    /// Bar-chart of jobs per month (Jan–Dec)
    #[arg(long = "chart-month")]
    chart_month: bool,

    /// Detailed breakdown for a specific month (name or number)
    #[arg(long = "chart-month-detail", value_name = "MONTH")]
    chart_month_detail: Option<String>,

    /// Path to a specific cron file (defaults to `crontab -l`)
    #[arg(long = "file", value_name = "FILE")]
    file: Option<String>,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    // Load cron content from file or `crontab -l`
    let content = if let Some(path) = args.file.as_deref() {
        fs::read_to_string(path)?
    } else {
        let output = Command::new("crontab")
            .arg("-l")
            .output()
            .expect("Failed to execute `crontab -l`");
        String::from_utf8_lossy(&output.stdout).into_owned()
    };

    let lines: Vec<&str> = content
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
        .collect();

    if let Some(month_arg) = &args.chart_month_detail {
        draw_month_detail(&lines, month_arg);
    } else if args.chart {
        draw_hourly_histogram(&lines);
    } else if args.chart_dow {
        draw_dow_histogram(&lines);
    } else if args.chart_month {
        draw_month_histogram(&lines);
    } else {
        pretty_print(&lines);
    }

    Ok(())
}

/// Pretty-print each crontab entry with human-readable schedule and colorized output.
fn pretty_print(lines: &[&str]) {
    for &line in lines {
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() < 6 { continue; }
        let (m, h, dom, mon, dow) = (cols[0], cols[1], cols[2], cols[3], cols[4]);
        let command = cols[5..].join(" ");

        let mut out = StandardStream::stdout(ColorChoice::Always);
        out.set_color(ColorSpec::new().set_fg(Some(Color::Green))).unwrap();
        writeln!(
            &mut out,
            "Schedule:   {}",
            cron_to_human_readable(m, h, dom, mon, dow)
        )
        .unwrap();
        out.reset().unwrap();

        out.set_color(ColorSpec::new().set_fg(Some(Color::Magenta))).unwrap();
        writeln!(&mut out, "Command:    {}", command).unwrap();
        out.reset().unwrap();
    }
}

/// Draw a histogram of cron jobs per hour, ordered 0–23 with wildcard 'any' first.
fn draw_hourly_histogram(lines: &[&str]) {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for &l in lines {
        let cols: Vec<&str> = l.split_whitespace().collect();
        if cols.len() < 6 { continue; }
        *counts.entry(cols[1].to_string()).or_default() += 1;
    }

    println!("\n hourly distribution of cron jobs\n");

    if let Some(&count) = counts.get("*") {
        let bar = "█".repeat(count);
        println!("{:>3} │ {:<4} {}", "any", count, bar);
    }

    let mut hours: Vec<u8> = counts
        .keys()
        .filter_map(|k| if k != "*" { k.parse().ok() } else { None })
        .collect();
    hours.sort();

    for h in hours {
        let key = h.to_string();
        let count = counts.get(&key).copied().unwrap_or(0);
        let bar = "█".repeat(count);
        println!("{:>3} │ {:<4} {}", format!("{:>2}", h), count, bar);
    }
    println!();
}

/// Draw a histogram of cron jobs per day-of-week, aligned neatly like months.
fn draw_dow_histogram(lines: &[&str]) {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for &l in lines {
        let cols: Vec<&str> = l.split_whitespace().collect();
        if cols.len() < 6 { continue; }
        *counts.entry(cols[4].to_string()).or_default() += 1;
    }

    println!("\n weekday distribution of cron jobs\n");
    for (dow, &count) in &counts {
        let label = if dow == "*" {
            "any".to_string()
        } else {
            dow_name(dow).to_string()
        };
        let bar = "█".repeat(count);
        println!("{:>9} │ {:<4} {}", label, count, bar);
    }
    println!();
}

/// Draw a histogram of cron jobs per month in calendar order.
fn draw_month_histogram(lines: &[&str]) {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for &l in lines {
        let cols: Vec<&str> = l.split_whitespace().collect();
        if cols.len() < 6 { continue; }
        *counts.entry(cols[3].to_string()).or_default() += 1;
    }

    println!("\n monthly distribution of cron jobs\n");

    if let Some(&count) = counts.get("*") {
        let bar = "█".repeat(count);
        println!("{:>9} │ {:<4} {}", "any", count, bar);
    }

    for month_idx in 1..=12 {
        let key = month_idx.to_string();
        if let Some(&count) = counts.get(&key) {
            let label = month_name(&key);
            let bar = "█".repeat(count);
            println!("{:>9} │ {:<4} {}", label, count, bar);
        }
    }
    println!();
}

/// Detailed breakdown: for given month, show jobs per day-of-month and per-hour for each day.
fn draw_month_detail(lines: &[&str], month_arg: &str) {
    let month_num: u8 = match month_arg.parse() {
        Ok(n) if (1..=12).contains(&n) => n,
        _ => match month_arg.to_lowercase().as_str() {
            "january" => 1, "february" => 2, "march" => 3, "april" => 4,
            "may" => 5, "june" => 6, "july" => 7, "august" => 8,
            "september" => 9, "october" => 10, "november" => 11, "december" => 12,
            _ => { eprintln!("Unknown month: {}", month_arg); return; }
        }
    };

    let mut day_counts: BTreeMap<u8, usize> = BTreeMap::new();
    let mut hour_by_day: BTreeMap<u8, BTreeMap<String, usize>> = BTreeMap::new();

    for &l in lines {
        let cols: Vec<&str> = l.split_whitespace().collect();
        if cols.len() < 6 { continue; }
        if cols[3] != "*" && cols[3].parse::<u8>().ok() != Some(month_num) { continue; }
        if let Ok(dom) = cols[2].parse::<u8>() {
            *day_counts.entry(dom).or_default() += 1;
            let hour_label = if cols[1] == "*" {
                "any".to_string()
            } else {
                format!("{:02}", cols[1].parse::<u8>().unwrap_or(0))
            };
            let hours_map = hour_by_day.entry(dom).or_default();
            *hours_map.entry(hour_label).or_default() += 1;
        }
    }

    println!("\nDetails for {} (month {})\n", month_name(&month_num.to_string()), month_num);
    println!(" Day-of-month distribution\n");
    for (&day, &count) in &day_counts {
        let bar = "█".repeat(count);
        println!("{:>2} │ {:<4} {}", day, count, bar);
    }
    println!("\n Hourly breakdown by day\n");
    for (&day, hours) in &hour_by_day {
        println!("Day {}: {} jobs", day, day_counts.get(&day).copied().unwrap_or(0));
        for (hour, &count) in hours {
            let bar = "█".repeat(count);
            println!("  {:>3} │ {:<4} {}", hour, count, bar);
        }
        println!();
    }
}

/// Convert cron fields to a human-readable schedule string.
fn cron_to_human_readable(
    minute: &str,
    hour: &str,
    day_of_month: &str,
    month: &str,
    day_of_week: &str,
) -> String {
    let time_part = match (minute, hour) {
        ("*", "*") => "every minute".into(),
        ("*", h) => {
            let human_hour = hour_to_ampm_string(h);
            format!("every minute during the {} hour", human_hour)
        }
        (m, "*") => {
            let mm: u8 = m.parse().unwrap_or(0);
            format!("every hour at {:02} minutes past", mm)
        }
        (m, h) => format_time(h, m),
    };
    if month == "*" && day_of_month != "*" && day_of_week != "*" {
        return format!(
            "{} every month on {} and every {}",
            time_part,
            day_of_month_with_suffix(day_of_month),
            dow_name(day_of_week)
        );
    }
    if month != "*" && day_of_month == "*" && day_of_week != "*" {
        return format!(
            "{} every {} in {}",
            time_part,
            dow_name(day_of_week),
            month_name(month)
        );
    }
    let mut desc = time_part;
    if month != "*" {
        desc.push_str(&format!(" on {}", month_name(month)));
    }
    if day_of_month != "*" {
        desc.push_str(&format!(" {}", day_of_month_with_suffix(day_of_month)));
    }
    if day_of_week != "*" {
        let conj = if day_of_month != "*" { " and every" } else { " every" };
        desc.push_str(&format!("{} {}", conj, dow_name(day_of_week)));
    }
    desc
}

/// Format hh:mm AM/PM
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

/// Convert hour to "HH AM/PM" string
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

/// Map month number to name
fn month_name(m: &str) -> &'static str {
    match m {
        "1"  => "January",  "2"  => "February", "3"  => "March",
        "4"  => "April",    "5"  => "May",      "6"  => "June",
        "7"  => "July",     "8"  => "August",   "9"  => "September",
        "10" => "October",  "11" => "November", "12" => "December",
        _     => "Unknown",
    }
}

/// Add ordinal suffix to day-of-month
fn day_of_month_with_suffix(day: &str) -> String {
    let num: u32 = day.parse().unwrap_or(0);
    let suffix = if num % 100 / 10 == 1 {
        "th"
    } else {
        match num % 10 { 1 => "st", 2 => "nd", 3 => "rd", _ => "th" }
    };
    format!("{}{}", num, suffix)
}

/// Map day-of-week number to name
fn dow_name(d: &str) -> &'static str {
    match d {
        "0" => "Sunday",   "1" => "Monday",   "2" => "Tuesday",
        "3" => "Wednesday","4" => "Thursday","5" => "Friday",
        "6" => "Saturday", _    => "Unknown",
    }
}
