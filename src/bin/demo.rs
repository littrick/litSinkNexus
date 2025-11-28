use ::windows::{
    Win32::{
        Foundation::*, Graphics::Gdi::UpdateWindow, System::LibraryLoader::GetModuleHandleA,
        UI::WindowsAndMessaging::*,
    },
    core::*,
};

fn main() -> anyhow::Result<()> {
    let hmodule = unsafe { GetModuleHandleA(None) }?;
    let hinstace = HINSTANCE::from(hmodule);

    let window_class: PCWSTR = w!("demo_window_class");

    init_class(hinstace, window_class)?;
    init_window(hinstace, window_class)?;

    message_loop()?;
    Ok(())
}

fn init_class(hinstance: HINSTANCE, class_name: PCWSTR) -> anyhow::Result<()> {
    let wc = WNDCLASSEXW {
        cbSize: size_of::<WNDCLASSEXW>() as u32,
        hCursor: unsafe { LoadCursorW(None, IDC_ARROW) }?,
        hInstance: hinstance,
        lpszClassName: class_name,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(wndproc),
        ..Default::default()
    };

    let atom = unsafe { RegisterClassExW(&wc) };
    anyhow::ensure!(atom != 0, "Failed to register window class");
    Ok(())
}

fn init_window(hinstance: HINSTANCE, class_name: PCWSTR) -> anyhow::Result<HWND> {
    let windows_title = w!("Demo Window");

    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            class_name,
            windows_title,
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            0,
            CW_USEDEFAULT,
            0,
            None,
            None,
            Some(hinstance),
            None,
        )
    }?;
    let _ = unsafe { ShowWindow(hwnd, SW_SHOW) };
    let _ = unsafe { UpdateWindow(hwnd) };
    Ok(hwnd)
}

fn message_loop() -> anyhow::Result<()> {
    unsafe {
        let mut message = MSG::default();

        while GetMessageW(&mut message, None, 0, 0).into() {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
    Ok(())
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
