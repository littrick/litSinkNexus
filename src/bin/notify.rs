use log::info;

use windows::{
    core::w, Win32::{
        Foundation::{GetLastError, HWND, LPARAM, LRESULT, WPARAM}, Graphics::Gdi::ValidateRect, System::LibraryLoader::GetModuleHandleW, UI::{
            Shell::{
                Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW
            },
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, LoadCursorW, LoadIconW, PostQuitMessage, RegisterClassW, TranslateMessage, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, IDC_ARROW, IDI_APPLICATION, MSG, WINDOW_EX_STYLE, WM_CREATE, WM_DESTROY, WM_LBUTTONUP, WM_PAINT, WM_RBUTTONUP, WM_USER, WNDCLASSW, WS_EX_APPWINDOW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_OVERLAPPEDWINDOW, WS_EX_TOPMOST, WS_OVERLAPPEDWINDOW, WS_POPUP, WS_VISIBLE
            },
        }
    }
};

const WM_NOTIFYICON: u32 = WM_USER + 1;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let h_instance = unsafe { GetModuleHandleW(None) }?;
    let h_cursor = unsafe { LoadCursorW(None, IDC_ARROW) }?;
    let h_icon = unsafe { LoadIconW(None, IDI_APPLICATION) }?;

    let window_class = WNDCLASSW {
        hInstance: h_instance.into(),
        lpszClassName: w!("awindow"),
        hCursor: h_cursor,
        hIcon: h_icon,
        lpfnWndProc: Some(wnproc),
        style: CS_HREDRAW | CS_VREDRAW,
        ..Default::default()
    };

    let class_atom = unsafe { RegisterClassW(&window_class) };
    anyhow::ensure!(
        class_atom != 0,
        "Register class fail, Error: {:?}",
        unsafe { GetLastError() }
    );

    let _ = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            window_class.lpszClassName,
            w!("windows name"),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            Some(h_instance.into()),
            None,
        )
    }?;

    let mut msg = MSG::default();

    while unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {
        // info!("{}:{} got message {msg:?}", file!(), line!());

        // unsafe { TranslateMessage(&msg) };
        unsafe { DispatchMessageW(&msg) };
    }

    Ok(())
}

extern "system" fn wnproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // info!("{}:{} message: 0x{message:X}", file!(), line!());

    let mut result = LRESULT(0);

    match message {
        WM_PAINT => {
            println!("WM_PAINT");
            _ = unsafe { ValidateRect(Some(window), None) };
        }
        WM_CREATE => {
            info!("WM_CREATE");
        }
        WM_DESTROY => {
            info!("WM_DESTROY");
            unsafe { PostQuitMessage(0) };
        }
        _ => result = unsafe { DefWindowProcW(window, message, wparam, lparam) },
    };

    result

    // LRESULT(0)
}
