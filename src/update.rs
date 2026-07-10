use clap::Parser;
use std::process::{Command, ExitStatus};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Parser)]
#[command(name = "ask-bridge-update")]
#[command(about = "Internal updater for ask-bridge")]
struct Args {
    /// Parent process id to wait for before replacing binaries.
    #[arg(long, value_name = "PID")]
    parent_pid: Option<u32>,

    /// Maximum seconds to wait for the parent process to exit.
    #[arg(long, default_value_t = 30)]
    wait_seconds: u64,
}

fn main() {
    let args = Args::parse();

    if let Some(parent_pid) = args.parent_pid {
        wait_for_process_exit(parent_pid, args.wait_seconds);
    }

    if let Err(e) = run_update_command() {
        eprintln!("Update failed: {}", e);
        std::process::exit(1);
    }
}

fn wait_for_process_exit(parent_pid: u32, wait_seconds: u64) {
    let mut iterations = 0u64;
    let max_iterations = wait_seconds.saturating_mul(4);
    let start = Instant::now();

    while iterations < max_iterations {
        if !is_process_running(parent_pid) {
            let elapsed = Instant::now().duration_since(start).as_secs();
            println!(
                "ask-bridge-update: parent process {} has exited after {}s.",
                parent_pid, elapsed
            );
            return;
        }
        if iterations == 0 {
            println!(
                "ask-bridge-update: waiting for parent process {} to exit...",
                parent_pid
            );
        }
        iterations += 1;
        thread::sleep(Duration::from_millis(250));
    }

    println!(
        "ask-bridge-update: parent process {} is still running after {}s, continue anyway.",
        parent_pid, wait_seconds
    );
}

fn run_update_command() -> Result<(), String> {
    println!("ask-bridge-update: launching official installer...");

    #[cfg(target_os = "windows")]
    let status = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "irm https://raw.githubusercontent.com/doggy8088/ask-bridge/main/install.ps1 | iex",
        ])
        .status()
        .map_err(|e| format!("Failed to run Windows update command: {}", e))?;

    #[cfg(not(target_os = "windows"))]
    let status = Command::new("sh")
        .args([
            "-c",
            "curl -fsSL https://raw.githubusercontent.com/doggy8088/ask-bridge/main/install.sh | bash",
        ])
        .status()
        .map_err(|e| format!("Failed to run macOS/Linux update command: {}", e))?;

    report_status(status)
}

fn report_status(status: ExitStatus) -> Result<(), String> {
    if status.success() {
        println!("ask-bridge-update: update command completed.");
        Ok(())
    } else {
        Err(format!("Update command failed with exit status {}", status))
    }
}

#[cfg(target_os = "windows")]
fn is_process_running(pid: u32) -> bool {
    let filter = format!("/FI PID eq {}", pid);
    let output = Command::new("tasklist").args(["/NH", &filter]).output();

    let output = match output {
        Ok(output) => output,
        Err(_) => return false,
    };

    if !output.status.success() {
        return false;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.to_lowercase().contains("no tasks are running") {
            return false;
        }
        let pid_field = trimmed.split_whitespace().nth(1);
        if let Some(pid_field) = pid_field {
            if let Ok(value) = pid_field.parse::<u32>() {
                return value == pid;
            }
        }
    }

    false
}

#[cfg(not(target_os = "windows"))]
fn is_process_running(pid: u32) -> bool {
    let pid_text = pid.to_string();
    let output = Command::new("ps")
        .args(["-p", &pid_text, "-o", "pid="])
        .output();

    let output = match output {
        Ok(output) => output,
        Err(_) => return false,
    };

    if !output.status.success() {
        return false;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    text.trim() == pid_text
}
