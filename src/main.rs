use crate::config::Config;
use crate::console::clear_screen;
use crate::process::{retry_find_process_by_name, wait_for_process_exit};
use crate::window::{
    maximize_current_process_window, maximize_window, minimize_current_process_window,
    retry_window_from_process_id, set_foreground_window,
};
use anyhow::Result;
use std::thread::sleep;
use std::time::Duration;

mod config;
mod console;
mod process;
mod window;

pub fn wait_exit(exit_code: i32) -> ! {
    println!("Enter to exit...");
    std::io::stdin()
        .read_line(&mut String::new())
        .expect("Failed to read line");
    std::process::exit(exit_code);
}

fn main() -> Result<()> {
    maximize_current_process_window();

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
    clear_screen();

    info!("Trying to find process by name: {}", config.name_process);
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
    clear_screen();
    info!("Found process {} with PID {}", config.name_process, pid);

    info!("Trying to find window for process ID: {}", pid);
    let hwnd = match retry_window_from_process_id(pid, 60) {
        Some(hwnd) => hwnd,
        None => {
            error!(
                "Failed to find window for process {} after 60 retries",
                config.name_process
            );
            wait_exit(1);
        }
    };

    clear_screen();
    minimize_current_process_window();

    if !set_foreground_window(hwnd) {
        warn!("Failed to set foreground window");
    }
    if !maximize_window(hwnd) {
        warn!("Failed to maximize window");
    }

    info!("Found window, waiting for exit...");

    let exit_code = match wait_for_process_exit(pid) {
        Ok(code) => code,
        Err(e) => {
            error!("Error waiting for process exit: {}", e);
            wait_exit(1);
        }
    };

    maximize_current_process_window();
    clear_screen();

    if exit_code != 0 {
        error!("Process exited with code: {}", exit_code);
        wait_exit(exit_code);
    } else {
        info!("Process exited successfully.");
        sleep(Duration::from_secs(3));
    }

    Ok(())
}
