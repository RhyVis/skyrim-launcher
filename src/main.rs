use crate::config::Config;
use crate::process::{retry_find_process_by_name, wait_for_process_exit};
use crate::window::{
    get_current_process_window, maximize_window, minimize_window, retry_window_from_process_id,
    set_foreground_window,
};
use anyhow::Result;
use std::thread::sleep;
use std::time::Duration;

mod config;
mod console;
mod process;
mod window;

pub fn wait_exit(exit_code: i32) -> ! {
    println!("Waiting for exit...");
    std::io::stdin()
        .read_line(&mut String::new())
        .expect("Failed to read input");
    std::process::exit(exit_code);
}

fn main() -> Result<()> {
    info!("Loading configuration...");
    let config = Config::load()?;

    match process::run_executable(config.exe_path()) {
        Ok(code) => {
            if code != 0 {
                error!("Executable exited with code: {}", code);
                wait_exit(code);
            } else {
                info!("Executable ran successfully.");
            }
        }
        Err(e) => {
            error!("Failed to run executable: {}", e);
            wait_exit(1);
        }
    };

    sleep(Duration::from_secs(1));

    let pid = match retry_find_process_by_name(&config.name_process, 10) {
        Ok(pid) => pid,
        Err(err) => {
            error!(
                "Failed to find process {} after 10 retries: {}",
                config.name_process, err
            );
            wait_exit(1);
        }
    };
    info!("Found process {} with PID {}", config.name_process, pid);

    let hwnd = match retry_window_from_process_id(pid, 30) {
        Some(hwnd) => hwnd,
        None => {
            error!(
                "Failed to find window for process {} after 30 retries",
                config.name_process
            );
            wait_exit(1);
        }
    };

    if let Some(current_hwnd) = get_current_process_window() {
        if !minimize_window(current_hwnd) {
            warn!("Failed to minimize current process window");
        }
    } else {
        warn!("No current process window found");
    }

    if !set_foreground_window(hwnd) {
        warn!("Failed to set foreground window");
    }
    if !maximize_window(hwnd) {
        warn!("Failed to maximize window");
    }

    let exit_code = match wait_for_process_exit(pid) {
        Ok(code) => code,
        Err(e) => {
            error!("Error waiting for process exit: {}", e);
            wait_exit(1);
        }
    };
    if exit_code != 0 {
        error!("Process exited with code: {}", exit_code);
    } else {
        info!("Process exited successfully.");
    }

    Ok(())
}
