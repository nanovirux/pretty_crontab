use std::process::Command;
use std::io::Error;

pub fn get_crontab() -> Result<String, Error> {
    let output = Command::new("crontab")
        .arg("-l")
        .output()?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

