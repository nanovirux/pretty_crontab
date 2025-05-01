use crate::crontab::get_crontab;       // Import the `get_crontab` function from the `crontab` module
use crate::formatter::print_pretty_crontab; // Import the `print_pretty_crontab` function from the `formatter` module

mod crontab;    // Declare the `crontab` module
mod formatter;  // Declare the `formatter` module
mod utils;      // Declare the `utils` module (if you're using it elsewhere)

fn main() {
    match crontab::get_crontab() {
        Ok(crontab) => {
            println!("Crontab fetched successfully!");
            print_pretty_crontab(&crontab);
        },
        Err(err) => {
            eprintln!("Error fetching crontab: {}", err);
        }
    }
}

