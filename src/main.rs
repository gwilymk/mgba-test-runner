use io::Write;
use regex::Regex;
use std::{
    io,
    io::BufRead,
    io::BufReader,
    process::{Command, Stdio},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ThisError {
    #[error("tests failed")]
    TestsFailed,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let file_to_run = args.get(1).expect("you should provide file to run");

    let mut command = Command::new("xvfb-run")
        .arg("mgba-qt")
        .args(&["-l", "31", "-d", "-C", "logToStdout=1"])
        .arg(file_to_run)
        .stdout(Stdio::piped())
        .spawn()?;
    let stdout = command.stdout.take().expect("expected stdout to exist");

    let reader = BufReader::new(stdout);

    let regex = Regex::new(r"^\[(.*)\] GBA Debug: (.*)$").unwrap();

    for line in reader.lines().filter_map(|line| line.ok()) {
        if let Some(captures) = regex.captures(line.as_str()) {
            let log_level = &captures[1];
            let out = &captures[2];
            if out.ends_with("...") {
                print!("{}", out);
                io::stdout().flush()?;
            } else {
                println!("{}", out);
            }

            if log_level == "FATAL" {
                command.kill()?;
                return Err(Box::new(ThisError::TestsFailed));
            }
            if out == "Tests finished successfully" {
                command.kill()?;
                return Ok(());
            }
        } else {
            println!("{}", line);
        }
    }

    Ok(())
}
