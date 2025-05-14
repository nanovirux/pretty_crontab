use std::{
    collections::{BTreeMap, HashMap},
    fs,
    process::Command,
};
use std::io::{self, Write};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use clap::Parser;

/// A cron viewer that pretty-prints your crontab or shows histograms by hour, weekday, or month.
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

    /// Filter cron entries by substring match
    #[arg(long, value_name = "PATTERN")]
    filter: Option<String>,
}

/// Normalize “@hourly”, “@daily”, etc., into five-field cron syntax; skip “@reboot”.
fn normalize_special_entry(line: &str) -> Option<String> {
    let mut parts = line.split_whitespace();
    let first = parts.next()?;
    let rest = parts.collect::<Vec<_>>().join(" ");
    let cron = match first {
        "@hourly"    => "0 * * * *",
        "@daily"     => "0 0 * * *",
        "@weekly"    => "0 0 * * 0",
        "@monthly"   => "0 0 1 * *",
        "@yearly" | "@annually" => "0 0 1 1 *",
        "@reboot"    => return None,
        _            => return None,
    };
    Some(format!("{} {}", cron, rest))
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    // Load raw crontab (from file or `crontab -l`)
    let raw = if let Some(path) = args.file.as_deref() {
        fs::read_to_string(path)?
    } else {
        let output = Command::new("crontab")
            .arg("-l")
            .output()
            .expect("Failed to run `crontab -l`");
        String::from_utf8_lossy(&output.stdout).into_owned()
    };

    // Strip blanks/comments, normalize @special, count dropped
    let mut normalized = Vec::new();
    let mut dropped = 0;
    for line in raw.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            dropped += 1;
            continue;
        }
        if let Some(expanded) = normalize_special_entry(t) {
            normalized.push(expanded);
        } else {
            normalized.push(t.to_string());
        }
    }

    // Apply --filter if given
    let filtered: Vec<String> = if let Some(pat) = &args.filter {
        normalized.into_iter().filter(|l| l.contains(pat)).collect()
    } else {
        normalized
    };
    let lines_ref: Vec<&str> = filtered.iter().map(|s| s.as_str()).collect();

    // Dispatch
    if let Some(month) = &args.chart_month_detail {
        draw_month_detail(&lines_ref, month);
    } else if args.chart {
        draw_hourly_histogram(&lines_ref);
    } else if args.chart_dow {
        draw_dow_histogram(&lines_ref);
    } else if args.chart_month {
        draw_month_histogram(&lines_ref);
    } else {
        pretty_print(&lines_ref);
    }

    eprintln!(
        "({} raw lines dropped, {} cron jobs parsed)",
        dropped,
        lines_ref.len()
    );
    Ok(())
}

/// Break an hour field like "0", "0-5", "*/2", "1,2,3" into individual 0–23 values.
fn parse_hour_field(field: &str) -> Vec<u8> {
    let mut out = Vec::new();
    for part in field.split(',') {
        if part == "*" {
            continue;
        }
        // Range "N-M"
        if let Some(idx) = part.find('-') {
            if let (Ok(s), Ok(e)) = (
                part[..idx].parse::<u8>(),
                part[idx + 1..].parse::<u8>(),
            ) {
                for h in s..=e {
                    if h < 24 {
                        out.push(h);
                    }
                }
                continue;
            }
        }
        // Step "*/S"
        if let Some(step) = part.strip_prefix("*/").and_then(|s| s.parse::<u8>().ok()) {
            let mut h = 0;
            while h < 24 {
                out.push(h);
                h += step;
            }
            continue;
        }
        // Single hour
        if let Ok(h) = part.parse::<u8>() {
            if h < 24 {
                out.push(h);
            }
        }
    }
    out.sort_unstable();
    out.dedup();
    out
}

/// Histogram of cron jobs per hour (0–23), wildcard 'any' first.
fn draw_hourly_histogram(lines: &[&str]) {
    let mut counts: HashMap<u8, usize> = HashMap::new();
    let mut wildcard = 0;

    for &l in lines {
        let cols: Vec<&str> = l.split_whitespace().collect();
        if cols.len() < 6 {
            continue;
        }
        let field = cols[1];
        if field == "*" {
            wildcard += 1;
        } else {
            for h in parse_hour_field(field) {
                *counts.entry(h).or_default() += 1;
            }
        }
    }

    println!("\n hourly distribution of cron jobs\n");
    if wildcard > 0 {
        println!("{:>3} │ {:<4} {}", "any", wildcard, "█".repeat(wildcard));
    }
    let mut hours: Vec<u8> = counts.keys().copied().collect();
    hours.sort_unstable();
    for h in hours {
        let c = counts[&h];
        println!("{:>3} │ {:<4} {}", format!("{:02}", h), c, "█".repeat(c));
    }
    println!();
}

/// Pretty-print each crontab entry with human-readable schedule and color.
fn pretty_print(lines: &[&str]) {
    for &line in lines {
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() < 6 {
            continue;
        }
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

/// Histogram of cron jobs per day-of-week, expanding lists/ranges.
fn draw_dow_histogram(lines: &[&str]) {
    let mut counts = [0usize; 7];
    let mut wildcard = 0;

    for &l in lines {
        let cols: Vec<&str> = l.split_whitespace().collect();
        if cols.len() < 6 {
            continue;
        }
        let field = cols[4];
        if field == "*" {
            wildcard += 1;
        } else {
            for day in parse_dow_field(field) {
                counts[day as usize] += 1;
            }
        }
    }

    println!("\n weekday distribution of cron jobs\n");
    if wildcard > 0 {
        println!("{:>9} │ {:<4} {}", "any", wildcard, "█".repeat(wildcard));
    }
    for day in 0..7 {
        let c = counts[day];
        if c > 0 {
            println!(
                "{:>9} │ {:<4} {}",
                dow_name_num(day as u8),
                c,
                "█".repeat(c)
            );
        }
    }
    println!();
}

/// Parse a DOW field like "Mon", "1", "Mon-Fri", "Tue,Thu" into 0..6.
fn parse_dow_field(field: &str) -> Vec<u8> {
    let mut out = Vec::new();
    for part in field.split(',') {
        if let Some(idx) = part.find('-') {
            let start = &part[..idx];
            let end = &part[idx + 1..];
            if let (Some(s), Some(e)) = (parse_dow_value(start), parse_dow_value(end)) {
                let mut cur = s;
                loop {
                    out.push(cur);
                    if cur == e {
                        break;
                    }
                    cur = (cur + 1) % 7;
                }
            }
        } else if let Some(d) = parse_dow_value(part) {
            out.push(d);
        }
    }
    out
}

fn parse_dow_value(tok: &str) -> Option<u8> {
    match tok.to_lowercase().as_str() {
        "sun" | "0" => Some(0),
        "mon" | "1" => Some(1),
        "tue" | "2" => Some(2),
        "wed" | "3" => Some(3),
        "thu" | "4" => Some(4),
        "fri" | "5" => Some(5),
        "sat" | "6" => Some(6),
        _ => None,
    }
}

fn dow_name_num(d: u8) -> &'static str {
    match d {
        0 => "Sunday",
        1 => "Monday",
        2 => "Tuesday",
        3 => "Wednesday",
        4 => "Thursday",
        5 => "Friday",
        6 => "Saturday",
        _ => "Unknown",
    }
}

/// Histogram of cron jobs per month (1–12).
fn draw_month_histogram(lines: &[&str]) {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for &l in lines {
        let cols: Vec<&str> = l.split_whitespace().collect();
        if cols.len() < 6 {
            continue;
        }
        *counts.entry(cols[3].to_string()).or_default() += 1;
    }

    println!("\n monthly distribution of cron jobs\n");
    if let Some(&c) = counts.get("*") {
        println!("{:>9} │ {:<4} {}", "any", c, "█".repeat(c));
    }
    for month in 1..=12 {
        let key = month.to_string();
        if let Some(&c) = counts.get(&key) {
            println!(
                "{:>9} │ {:<4} {}",
                month_name(&key),
                c,
                "█".repeat(c)
            );
        }
    }
    println!();
}

/// Detailed breakdown for a specific month.
fn draw_month_detail(lines: &[&str], month_arg: &str) {
    let month_num: u8 = match month_arg.parse() {
        Ok(n) if (1..=12).contains(&n) => n,
        _ => match month_arg.to_lowercase().as_str() {
            "january"   => 1,
            "february"  => 2,
            "march"     => 3,
            "april"     => 4,
            "may"       => 5,
            "june"      => 6,
            "july"      => 7,
            "august"    => 8,
            "september" => 9,
            "october"   => 10,
            "november"  => 11,
            "december"  => 12,
            _ => {
                eprintln!("Unknown month: {}", month_arg);
                return;
            }
        },
    };

    let mut day_counts: BTreeMap<u8, usize> = BTreeMap::new();
    let mut hour_by_day: BTreeMap<u8, BTreeMap<String, usize>> = BTreeMap::new();

    for &l in lines {
        let cols: Vec<&str> = l.split_whitespace().collect();
        if cols.len() < 6 {
            continue;
        }
        if cols[3] != "*" && cols[3].parse::<u8>().ok() != Some(month_num) {
            continue;
        }
        if let Ok(dom) = cols[2].parse::<u8>() {
            *day_counts.entry(dom).or_default() += 1;
            let hour_label = if cols[1] == "*" {
                "any".to_string()
            } else {
                format!("{:02}", cols[1].parse::<u8>().unwrap_or(0))
            };
            *hour_by_day
                .entry(dom)
                .or_default()
                .entry(hour_label)
                .or_default() += 1;
        }
    }

    println!(
        "\nDetails for {} (month {})\n",
        month_name(&month_num.to_string()),
        month_num
    );
    println!(" Day-of-month distribution\n");
    for (&day, &c) in &day_counts {
        println!("{:>2} │ {:<4} {}", day, c, "█".repeat(c));
    }
    println!("\n Hourly breakdown by day\n");
    for (&day, hours) in &hour_by_day {
        let total = day_counts.get(&day).copied().unwrap_or(0);
        println!("Day {}: {} jobs", day, total);
        for (hr, &c) in hours {
            println!("  {:>3} │ {:<4} {}", hr, c, "█".repeat(c));
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
            dow_name_num(parse_dow_field(day_of_week).get(0).copied().unwrap_or(0))
        );
    }
    if month != "*" && day_of_month == "*" && day_of_week != "*" {
        let dow = parse_dow_field(day_of_week);
        let name = if dow.len() == 1 {
            dow_name_num(dow[0])
        } else {
            "multiple"
        };
        return format!("{} every {} in {}", time_part, name, month_name(month));
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
        let dow = parse_dow_field(day_of_week);
        let name = if dow.len() == 1 {
            dow_name_num(dow[0])
        } else {
            "multiple"
        };
        desc.push_str(&format!("{} {}", conj, name));
    }
    desc
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

fn month_name(m: &str) -> &'static str {
    match m {
        "1" => "January",
        "2" => "February",
        "3" => "March",
        "4" => "April",
        "5" => "May",
        "6" => "June",
        "7" => "July",
        "8" => "August",
        "9" => "September",
        "10" => "October",
        "11" => "November",
        "12" => "December",
        _ => "Unknown",
    }
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
