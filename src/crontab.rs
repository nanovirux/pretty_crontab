use std::process::Command;
use std::io::{self, Error};

pub fn get_crontab() -> Result<String, Error> {
    let output = Command::new("crontab")
        .arg("-l")
        .output()?;

    if !output.status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "Failed to fetch crontab"));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
