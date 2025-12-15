use windows::{
    Win32::{Foundation::*, System::LibraryLoader::*, UI::WindowsAndMessaging::*},
    core::*,
};

#[allow(non_snake_case, unused)]
fn main() {
    let module = unsafe { GetModuleHandleW(None).unwrap() };
    let instance = HINSTANCE::from(module);

    println!("registering window class");

    let hIcon = unsafe { LoadIconW(None, IDI_APPLICATION) }.unwrap();
    // let hIcon = unsafe { LoadIconW(Some(instance), w!("nexus.logo.ico")) }.unwrap();

    let hIcon = unsafe { LoadIconW(Some(instance), PCWSTR::from_raw(101 as _)) }.unwrap(); // 这里会失败，因为resource会被链接到lib，而不是bin

    let wc = WNDCLASSW {
        lpfnWndProc: Some(wndproc),
        hInstance: instance,
        lpszClassName: w!("my_window_class"),
        hIcon,
        ..Default::default()
    };
    unsafe { RegisterClassW(&wc) }.eq(&0).then(|| {
        println!("Register Class fail");
    });

    println!("creating window");
    let wnd = unsafe {
        CreateWindowExW(
            Default::default(),
            w!("my_window_class"),
            w!("My Window"),
            WS_OVERLAPPEDWINDOW,
            0,
            0,
            800,
            600,
            None,
            None,
            Some(instance),
            None,
        )
    }
    .unwrap();
    let _ = unsafe { ShowWindow(wnd, SW_SHOW) };

    println!("entering message loop");
    let mut msg = MSG::default();
    while unsafe { GetMessageW(&mut msg, Some(wnd), 0, 0) }.as_bool() {
        unsafe {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}
