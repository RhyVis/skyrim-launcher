use crate::{debug, warn};
use windows::core::BOOL;
use windows::{
    Win32::Foundation::{HWND, LPARAM},
    Win32::System::Threading::GetCurrentProcessId,
    Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowThreadProcessId, SW_MAXIMIZE, SW_MINIMIZE, SetForegroundWindow,
        ShowWindow,
    },
};

struct EnumWindowsData {
    process_id: u32,
    hwnd: Option<HWND>,
}

extern "system" fn enum_windows_callback(hwnd: HWND, l_param: LPARAM) -> BOOL {
    unsafe {
        let data = l_param.0 as *mut EnumWindowsData;
        let mut process_id = 0u32;

        GetWindowThreadProcessId(hwnd, Some(&mut process_id));

        if process_id == (*data).process_id {
            (*data).hwnd = Some(hwnd);
            return false.into();
        }
    }

    true.into()
}

pub fn get_window_from_process_id(process_id: u32) -> Option<HWND> {
    let mut data = EnumWindowsData {
        process_id,
        hwnd: None,
    };

    unsafe {
        EnumWindows(
            Some(enum_windows_callback),
            LPARAM((&mut data as *mut EnumWindowsData) as isize),
        )
        .ok();
    }

    data.hwnd
}

pub fn retry_window_from_process_id(process_id: u32, retries: u32) -> Option<HWND> {
    for attempt in 0..retries {
        debug!("Attempt {}", attempt);
        if let Some(hwnd) = get_window_from_process_id(process_id) {
            return Some(hwnd);
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    None
}

pub fn maximize_window(hwnd: HWND) -> bool {
    unsafe { ShowWindow(hwnd, SW_MAXIMIZE).as_bool() }
}

pub fn minimize_window(hwnd: HWND) -> bool {
    unsafe { ShowWindow(hwnd, SW_MINIMIZE).as_bool() }
}

pub fn get_current_process_window() -> Option<HWND> {
    let process_id = unsafe { GetCurrentProcessId() };
    get_window_from_process_id(process_id)
}

pub fn maximize_current_process_window() {
    if let Some(hwnd) = get_current_process_window() {
        if maximize_window(hwnd) {
            return;
        }
    }
    warn!("Failed to maximize current process window");
}

pub fn minimize_current_process_window() {
    if let Some(hwnd) = get_current_process_window() {
        if minimize_window(hwnd) {
            return;
        }
    }
    warn!("Failed to minimize current process window");
}

pub fn set_foreground_window(hwnd: HWND) -> bool {
    unsafe { SetForegroundWindow(hwnd).into() }
}
