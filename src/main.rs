use anyhow::{anyhow, Error};
use io::Write;
use nix::{sys::signal, unistd::Pid};
use regex::Regex;
use std::{
    convert::TryInto,
    io,
    io::BufRead,
    io::BufReader,
    process::{Child, Command, Stdio},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let file_to_run = args.get(1).expect("you should provide file to run");

    let mut command = Command::new("mgba-qt")
        .args(&["-l", "31", "-d", "-C", "logToStdout=1"])
        .arg(file_to_run)
        .stdout(Stdio::piped())
        .spawn()?;

    monitor_mgba(&mut command)?;

    Ok(())
}

fn monitor_mgba(command: &mut Child) -> Result<(), Error> {
    let stdout = command.stdout.take().expect("expected stdout to exist");

    let reader = BufReader::new(stdout);

    let regex = Regex::new(r"^\[(.*)\] GBA Debug: (.*)$").unwrap();

    let mut tests_result = Ok(());

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
                send_sigint_to_process(command)?;
                tests_result = Err(anyhow!("Tests failed"));
            }
            if out == "Tests finished successfully" {
                send_sigint_to_process(command)?;
                tests_result = Ok(());
            }
        } else {
            println!("{}", line);
        }
    }

    tests_result
}

fn send_sigint_to_process(child: &Child) -> Result<(), Error> {
    signal::kill(Pid::from_raw(child.id().try_into()?), signal::SIGINT)?;
    Ok(())
}
