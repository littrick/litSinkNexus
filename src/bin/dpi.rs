use std::env::set_var;

use tracing::log::*;
use windows::{
    Win32::{
        Foundation::*, System::LibraryLoader::GetModuleHandleA,
        UI::{HiDpi::{DPI_AWARENESS_SYSTEM_AWARE, GetDpiForWindow, PROCESS_PER_MONITOR_DPI_AWARE, SetProcessDpiAwareness, SetThreadDpiAwarenessContext}, WindowsAndMessaging::*},
    },
    core::*,
};

fn main() -> Result<()> {
    unsafe { set_var("RUST_LOG", "trace") };
    tracing_subscriber::fmt::init();

    unsafe {
        let instance = GetModuleHandleA(None)?;
        let window_class = w!("mywindow");

        let wc = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hInstance: instance.into(),
            lpszClassName: window_class,

            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            ..Default::default()
        };

        let atom = RegisterClassExW(&wc);
        println!("atom = {atom}");
        debug_assert!(atom != 0);

        let window = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            window_class,
            w!("Fuck world"),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            Some(instance.into()),
            None,
        )?;

        SetProcessDpiAwareness(PROCESS_PER_MONITOR_DPI_AWARE).unwrap();

        let mut message = MSG::default();

        while GetMessageA(&mut message, None, 0, 0).into() {
            DispatchMessageA(&message);

            // Test
            let dpi = GetDpiForWindow(message.hwnd);
            info!("DPI for window {:?}: {}", message.hwnd, dpi);
        }

        Ok(())
    }
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_CREATE => {
                println!("WM_CREATE");
                LRESULT(0)
            }
            WM_DESTROY => {
                println!("WM_DESTROY");
                PostQuitMessage(0);

                LRESULT(0)
            }
            _ => DefWindowProcA(window, message, wparam, lparam),
        }
    }
}
