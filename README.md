# pretty\_crontab

`pretty_crontab` is a Rust utility to parse crontab files and:

* **Pretty-print** each cron entry with human-readable schedules and colorized output
* Generate **histograms** showing distribution of jobs by hour, day-of-week, or month
* Provide **detailed breakdown** for a specific month (jobs per day and per-hour per day)
* **Filter** entries by substring
* **Load** from a custom cron file

---

## Installation

Ensure you have Rust and Cargo installed. Then clone and build:

```bash
git clone https://github.com/nanovirux/pretty_crontab.git
cd pretty_crontab
cargo build --release
# Binary is at target/release/pretty_crontab
```

Optionally install it system-wide:

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
  --filter <PATTERN>          Show only entries containing the given substring
  --file <FILE>               Read cron entries from a file instead of `crontab -l`
  -h, --help                  Print help information
```

Run `pretty_crontab --help` to display this summary at any time.

---

## Modes & Examples

### 1. Default pretty-print

```bash
pretty_crontab
```

Prints each cron entry:

```
Schedule:   at 05:05 AM every Sunday in November
Command:    /usr/bin/backup-home
Schedule:   at 02:00 AM on December 31st
Command:    /usr/bin/year-end-task
```

### 2. Hourly histogram (`--chart`)

```bash
pretty_crontab --chart
```

```
 hourly distribution of cron jobs

any │  3 ███
  00 │  5 █████
  ...
  23 │  2 ██
```

### 3. Day-of-week histogram (`--chart-dow`)

```bash
pretty_crontab --chart-dow
```

```
 weekday distribution of cron jobs

any       │  4 ████
Sunday    │ 13 █████████████
Monday    │  8 ████████
...       │ ...
Saturday  │  3 ███
```

### 4. Monthly histogram (`--chart-month`)

```bash
pretty_crontab --chart-month
```

```
 monthly distribution of cron jobs

any       │  6 ██████
January   │  5 █████
...       │ ...
December  │  5 █████
```

### 5. Detailed monthly breakdown (`--chart-month-detail`)

```bash
pretty_crontab --chart-month-detail March
```

```
Details for March (month 3)

 Day-of-month distribution

  1 │  2 ██
  2 │  5 █████
  ...
 31 │  1 █

 Hourly breakdown by day

Day  1: 2 jobs
 any │ 1 █
 02  │ 1 █

Day  2: 5 jobs
 00  │ 2 ██
 any │ 3 ███
```

### 6. Filter entries (`--filter`)

Show only lines matching a substring:

```bash
pretty_crontab --filter backup
```

Runs any mode (default or charts) but only on entries containing “backup”.

### 7. Custom cron file (`--file`)

Read from a file instead of your personal crontab:

```bash
pretty_crontab --file /etc/cron.d/myjobs --chart-month
```

---

## Flags Summary

| Flag                         | Description                                         |
| ---------------------------- | --------------------------------------------------- |
| `--chart`                    | Histogram of jobs per hour                          |
| `--chart-dow`                | Histogram of jobs per day-of-week (Sun–Sat)         |
| `--chart-month`              | Histogram of jobs per month (Jan–Dec)               |
| `--chart-month-detail MONTH` | Detailed breakdown for a specified month            |
| `--filter PATTERN`           | Filter entries by substring match                   |
| `--file FILE`                | Read cron entries from FILE instead of `crontab -l` |
| `-h`, `--help`               | Show help information                               |

Built with Rust + [`clap`](https://crates.io/crates/clap)

Repository: [https://github.com/nanovirux/pretty\_crontab.git](https://github.com/nanovirux/pretty_crontab.git) + [`termcolor`](https://crates.io/crates/termcolor).
Feel free to open issues or PRs!
