# pretty\_crontab

`pretty_crontab` is a Rust-based utility to parse your user’s crontab and either:

* **Pretty-print** each cron entry with human-readable schedules and colorized output
* Generate **histograms** showing distribution of jobs by hour, day-of-week, or month
* Provide **detailed breakdown** for a specific month (jobs per day and per-hour per day)

---

## Installation

Ensure you have Rust and Cargo installed. Then clone and build:

```bash
git clone https://github.com/nanovirux/pretty_crontab.git
cd pretty_crontab
cargo build --release
# Binary will be at target/release/pretty_crontab
```

You can optionally install it to your `$PATH`:

```bash
cp target/release/pretty_crontab /usr/local/bin/
```

---

## Usage

```text
Usage: pretty_crontab [OPTIONS]

Options:
  --chart                     Bar-chart of cron jobs per hour (0–23)
  --chart-dow                 Bar-chart of cron jobs per day-of-week (Sun–Sat)
  --chart-month               Bar-chart of cron jobs per month (Jan–Dec)
  --chart-month-detail <MONTH>  Detailed breakdown for a specific month (name or number)
  -h, --help                  Print help information
```

Run `pretty_crontab --help` to see this summary at any time.

---

## Modes & Examples

### 1. Pretty-print (default)

Print each cron entry with a human-readable schedule and colored output:

```bash
$ pretty_crontab
Schedule:   at 05:05 AM every Sunday in November
Command:    /usr/bin/backup-home
Schedule:   at 02:00 AM on December 31st
Command:    /usr/bin/year-end-task
```

### 2. Hourly histogram (`--chart`)

Shows distribution of cron jobs by hour (0–23), with wildcard `*` labeled `any` first:

```bash
$ pretty_crontab --chart

 hourly distribution of cron jobs

any │  3 ███
  00 │  5 █████
  01 │  0
  ...
  23 │  2 ██
```

### 3. Day-of-week histogram (`--chart-dow`)

Shows distribution by day-of-week, aligned neatly:

```bash
$ pretty_crontab --chart-dow

 weekday distribution of cron jobs

any       │  4 ████
Sunday    │ 13 █████████████
Monday    │  8 ████████
Tuesday   │  6 ██████
...       │ ...
Saturday  │  3 ███
```

### 4. Monthly histogram (`--chart-month`)

Shows distribution by month:

```bash
$ pretty_crontab --chart-month

 monthly distribution of cron jobs

any       │  6 ██████
January   │  5 █████
February  │  6 ██████
March     │  1 █
...
December  │  5 █████
```

### 5. Detailed monthly breakdown (`--chart-month-detail`)

Provide a breakdown for a given month (by name or number).  Displays:

* **Jobs per day-of-month**
* **Hourly breakdown** for each day

```bash
$ pretty_crontab --chart-month-detail March

Details for March (month 3)

 Day-of-month distribution

  1 │  2 ██
  2 │  5 █████
  ...
 31 │  1 █

 Hourly breakdown by day

Day  1: 2 jobs
    any │  1 █
     02 │  1 █

Day  2: 5 jobs
     00 │  2 ██
     12 │  1 █
    any │  2 ██

... (etc.)
```

---

## Flags Summary

| Flag                         | Description                                               |
| ---------------------------- | --------------------------------------------------------- |
| `--chart`                    | Histogram of jobs per hour                                |
| `--chart-dow`                | Histogram of jobs per day-of-week (Sun–Sat)               |
| `--chart-month`              | Histogram of jobs per month (Jan–Dec)                     |
| `--chart-month-detail MONTH` | Detailed breakdown for a specified month (name or number) |
| `-h`, `--help`               | Show help information                                     |

---

Built with Rust + [`clap`](https://crates.io/crates/clap) + [`termcolor`](https://crates.io/crates/termcolor).
Feel free to open an issue or PR for feature requests or bugs!
