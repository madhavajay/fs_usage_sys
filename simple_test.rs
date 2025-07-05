// Test fs_usage parsing without cargo
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

fn main() {
    println!("Testing fs_usage - will run for 10 seconds");
    println!("Create a file: touch /tmp/test123.txt");
    
    let mut cmd = Command::new("fs_usage");
    cmd.arg("-w")
        .arg("-f")
        .arg("filesys")
        .stdout(Stdio::piped())
        .stderr(Stdio::null());

    let mut child = cmd.spawn().expect("Failed to start fs_usage");
    let stdout = child.stdout.take().expect("Failed to get stdout");
    let reader = BufReader::new(stdout);
    
    let mut count = 0;
    for line in reader.lines() {
        if let Ok(line) = line {
            if line.contains("/tmp") {
                println!("Found /tmp line: {}", line);
                count += 1;
            }
            if count > 10 {
                break;
            }
        }
    }
    
    let _ = child.kill();
    println!("Done - saw {} /tmp events", count);
}