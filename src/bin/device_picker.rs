use windows::{
    Devices::Enumeration::DevicePicker,
    Foundation::*,
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::LibraryLoader::GetModuleHandleA,
        UI::{
            HiDpi::*, Input::KeyboardAndMouse::VK_ESCAPE, Shell::IInitializeWithWindow,
            WindowsAndMessaging::*,
        },
    },
    core::*,
};

use tracing::log;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    log::info!("Starting Device Picker Sample...");

    // unsafe { SetProcessDpiAwareness(PROCESS_PER_MONITOR_DPI_AWARE) }?;

    let hmodule = unsafe { GetModuleHandleA(None) }?;
    let hinstace = HINSTANCE::from(hmodule);
    let window_class: PCWSTR = w!("demo_window_class");

    register_class(hinstace, window_class)?;
    create_window(hinstace, window_class)?;
    message_loop()?;
    Ok(())
}

fn register_class(hinstance: HINSTANCE, class_name: PCWSTR) -> anyhow::Result<()> {
    let wc = WNDCLASSEXW {
        cbSize: size_of::<WNDCLASSEXW>() as u32,
        hCursor: unsafe { LoadCursorW(None, IDC_ARROW) }?,
        hInstance: hinstance,
        lpszClassName: class_name,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(wndproc),
        // hbrBackground: HBRUSH(unsafe { GetStockObject(WHITE_BRUSH) }.0), // set white background brush
        ..Default::default()
    };

    let atom = unsafe { RegisterClassExW(&wc) };
    anyhow::ensure!(atom != 0, "Failed to register window class");
    Ok(())
}

fn create_window(hinstance: HINSTANCE, class_name: PCWSTR) -> anyhow::Result<HWND> {
    let windows_title = w!("Device Picker Sample");

    let screen_width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
    let screen_height = unsafe { GetSystemMetrics(SM_CYSCREEN) };

    let (x, y, w, h) = {
        (0, 0, screen_width, screen_height)
        // (CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT)
        // (100, 100, screen_width - 200, screen_height - 200)
    };

    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            class_name,
            windows_title,
            WS_POPUP | WS_VISIBLE,
            x,
            y,
            w,
            h,
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

/// Show device picker at given rect
fn show_picker(window: HWND, rect: Rect) -> anyhow::Result<()> {
    let device_picker = DevicePicker::new()?;

    log::debug!(
        "Picker: X={} Y={} Width={} Height={}",
        rect.X,
        rect.Y,
        rect.Width,
        rect.Height,
    );

    unsafe {
        device_picker
            .cast::<IInitializeWithWindow>()
            .unwrap()
            .Initialize(window)
    }?;

    device_picker.Show(rect)?;
    // device_picker.ShowWithPlacement(rect, Placement::Left)?;

    Ok(())
}

/// Paint rectangle at given rect with random color
fn paint_rect(window: HWND, rect: RECT, color: Option<COLORREF>) -> anyhow::Result<()> {
    log::debug!(
        "Painting: left={} top={} right={} bottom={} ",
        rect.left,
        rect.top,
        rect.right,
        rect.bottom,
    );

    let hdc = unsafe { GetDC(Some(window)) };
    anyhow::ensure!(!hdc.is_invalid(), "GetDC failed");

    // random color
    let color = color.unwrap_or(COLORREF(rand::random::<u32>() & 0x00FFFFFF));
    let hbrush = unsafe { CreateSolidBrush(color) };

    unsafe { FillRect(hdc, &rect, hbrush) };
    let _ = unsafe { DeleteObject(hbrush.into()) };
    unsafe { ReleaseDC(Some(window), hdc) };

    Ok(())
}

/// Show picker and paint rectangle at current cursor position
fn show_position(window: HWND) {
    const WIDTH: i32 = 200;
    const HEIGHT: i32 = 200;

    let mut pt = POINT { x: 0, y: 0 };
    unsafe { GetCursorPos(&mut pt) }.unwrap();

    let mut client_pt = pt;
    unsafe { ScreenToClient(window, &mut client_pt) }.unwrap();

    log::debug!("ScreenCursor: X={} Y={}", pt.x, pt.y);
    log::debug!("ClientCursor: X={} Y={}", client_pt.x, client_pt.y);

    // show picker at cursor position
    let scale = unsafe { GetDpiForWindow(window) } as f32 / USER_DEFAULT_SCREEN_DPI as f32;
    show_picker(
        window,
        Rect {
            X: pt.x as f32 / scale,
            Y: pt.y as f32 / scale,
            Width: 0.0,
            Height: 0.0,
        },
    )
    .unwrap();

    // paint rectangle at cursor position
    paint_rect(
        window,
        RECT {
            left: client_pt.x - (WIDTH / 2),
            top: client_pt.y - (HEIGHT / 2),
            right: client_pt.x + (WIDTH / 2),
            bottom: client_pt.y + (HEIGHT / 2),
        },
        None,
    )
    .unwrap();
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let mut res = LRESULT(0);
    // log::debug!("wndproc: message(0x{:X}) wparam(0x{:X}) lparam(0x{:X})", message, wparam.0, lparam.0);

    match message {
        WM_LBUTTONUP => {
            log::debug!(
                "WM_LBUTTONDOWN: wparam(0x{:X}) lparam(0x{:X})",
                wparam.0,
                lparam.0
            );

            // Click left button to show picker at current cursor position
            show_position(window);
        }
        WM_RBUTTONUP => {
            // Handle right button to quit application
            unsafe { PostQuitMessage(0) };
        }
        WM_KEYDOWN => {
            // Handle ESC key to quit application

            log::debug!(
                "WM_KEYDOWN: wparam(0x{:X}) lparam(0x{:X})",
                wparam.0,
                lparam.0
            );
            if wparam.0 as u32 == VK_ESCAPE.0.into() {
                unsafe { PostQuitMessage(0) };
            }
        }
        _ => res = unsafe { DefWindowProcW(window, message, wparam, lparam) },
    }

    res
}
