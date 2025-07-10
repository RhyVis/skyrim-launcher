use crate::wait_exit;
use crate::{debug, error};
use anyhow::Result;
use std::mem::size_of;
use std::path::Path;
use std::process::Command;
use windows::Win32::Foundation::{HANDLE, STILL_ACTIVE, WAIT_OBJECT_0};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{
    GetExitCodeProcess, INFINITE, OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_SYNCHRONIZE,
    PROCESS_VM_READ, WaitForSingleObject,
};
use windows::core::Free;

pub fn run_executable(exe_path: impl AsRef<Path>) -> Result<i32> {
    let path = exe_path.as_ref();
    if !path.exists() {
        return Err(anyhow::anyhow!("Executables not found: {:?}", path));
    }

    let mut cmd = Command::new(path);

    let output = cmd.spawn()?.wait()?;

    match output.code() {
        Some(code) => Ok(code),
        None => Err(anyhow::anyhow!("Process terminated by signal")),
    }
}

pub fn retry_find_process_by_name(name: &str, retries: u32) -> Result<u32> {
    let mut attempts = 0;
    loop {
        match find_process_by_name(name) {
            Ok(pid) => return Ok(pid),
            Err(e) if attempts < retries => {
                attempts += 1;
                debug!("Attempt {}: {}", attempts, e);
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
            Err(e) => return Err(e),
        }
    }
}

pub fn find_process_by_name(name: &str) -> Result<u32> {
    let iter = ProcessIter::new()?;
    for process in iter {
        if process.name.to_lowercase() == name.to_lowercase() {
            return Ok(process.pid);
        }
    }
    Err(anyhow::anyhow!("Process not found: {}", name))
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
}

pub struct ProcessIter {
    index: usize,
    process_info: ProcessInfo,
    process_snapshot: HANDLE,
    process_entry: PROCESSENTRY32W,
}

impl ProcessIter {
    pub fn new() -> Result<Self> {
        let h_process_snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)? };
        if h_process_snapshot.is_invalid() {
            error!("Failed to create process snapshot");
            wait_exit(1);
        }

        let mut pe = PROCESSENTRY32W::default();
        pe.dwSize = size_of::<PROCESSENTRY32W>() as u32;

        unsafe { Process32FirstW(h_process_snapshot, &mut pe)? };

        let pid = pe.th32ProcessID;
        let name = get_exe_name_from_pe(&pe);

        Ok(Self {
            index: 0,
            process_info: ProcessInfo { pid, name },
            process_snapshot: h_process_snapshot,
            process_entry: pe,
        })
    }
}

impl Iterator for ProcessIter {
    type Item = ProcessInfo;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        if self.index == 1 {
            return Some(self.process_info.clone());
        }

        let mut pe = self.process_entry;

        unsafe { Process32NextW(self.process_snapshot, &mut pe).ok()? };

        if pe.th32ProcessID == 0 {
            return None;
        }

        let pid = pe.th32ProcessID;
        let name = get_exe_name_from_pe(&pe);
        Some(ProcessInfo { pid, name })
    }
}

pub fn wait_for_process_exit(pid: u32) -> Result<i32> {
    let mut h_process = unsafe {
        OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_READ | PROCESS_SYNCHRONIZE,
            false,
            pid,
        )?
    };

    let wait_result = unsafe { WaitForSingleObject(h_process, INFINITE) };

    if wait_result != WAIT_OBJECT_0 {
        return Err(anyhow::anyhow!("Wait failed: {}", wait_result.0));
    }

    let mut exit_code: u32 = 0;
    if unsafe { GetExitCodeProcess(h_process, &mut exit_code).is_ok() } {
        unsafe { h_process.free() };
        if exit_code == STILL_ACTIVE.0 as u32 {
            return Err(anyhow::anyhow!("The process is still running"));
        }
        Ok(exit_code as i32)
    } else {
        unsafe { h_process.free() };
        Err(anyhow::anyhow!(
            "Failed to get exit code for process {}",
            pid
        ))
    }
}

fn get_exe_name_from_pe(pe: &PROCESSENTRY32W) -> String {
    let name_bytes = &pe.szExeFile;
    let null_pos = name_bytes
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(name_bytes.len());

    String::from_utf16_lossy(&name_bytes[..null_pos])
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_process_iter() {
        let mut iter = ProcessIter::new().expect("Failed to create process iterator");

        while let Some(process) = iter.next() {
            println!("Process ID: {}, Name: {}", process.pid, process.name);
        }
    }
}
