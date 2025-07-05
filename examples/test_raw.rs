use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

fn main() {
    println!("Testing raw fs_usage output for /tmp...");
    println!("Try: touch /tmp/test.txt in another terminal\n");

    let mut cmd = Command::new("fs_usage");
    cmd.arg("-w")
        .arg("-f")
        .arg("filesys")
        .stdout(Stdio::piped())
        .stderr(Stdio::null());

    let mut child = cmd.spawn().expect("Failed to start fs_usage");
    let stdout = child.stdout.take().expect("Failed to get stdout");
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        if let Ok(line) = line {
            if line.contains("/tmp") {
                println!("RAW: {}", line);
            }
        }
    }
}