use std::env;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

fn main() {
    let args: Vec<String> = env::args().collect();
    
    println!("Testing fs_usage without path filtering...");
    
    let mut cmd = Command::new("fs_usage");
    cmd.arg("-w")
        .arg("-f")
        .arg("filesys")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    match cmd.spawn() {
        Ok(mut child) => {
            if let Some(stdout) = child.stdout.take() {
                let reader = BufReader::new(stdout);
                let mut count = 0;
                
                for line in reader.lines() {
                    if let Ok(line) = line {
                        if args.len() > 1 && line.contains(&args[1]) {
                            println!("MATCH: {}", line);
                            count += 1;
                        } else if args.len() == 1 {
                            println!("LINE: {}", line);
                            count += 1;
                        }
                        
                        if count >= 10 {
                            break;
                        }
                    }
                }
            }
            let _ = child.kill();
        }
        Err(e) => {
            println!("Failed to start fs_usage: {}", e);
            println!("Are you running with sudo?");
        }
    }
}