use std::{fs, path::Path};

use resvg::{tiny_skia, usvg};
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

fn load_icon(hinstance: HINSTANCE, resource_id: u16) -> anyhow::Result<HICON> {
    let icon = unsafe {
        LoadImageW(
            Some(hinstance),
            PCWSTR(resource_id as _),
            IMAGE_ICON,
            0,
            0,
            LR_DEFAULTSIZE,
        )
    }?;
    Ok(HICON(icon.0))
}

fn svg2bitmap<P: AsRef<Path>>(svg: P) -> (Vec<u8>, u32, u32) {
    let svg_tree = usvg::Tree::from_str(
        fs::read_to_string(svg).unwrap().as_str(),
        &Default::default(),
    )
    .unwrap();

    let (w, h) = {
        let size = svg_tree.size();
        (size.width() as u32, size.height() as u32)
    };

    let mut pixmap = tiny_skia::Pixmap::new(w, h).unwrap();

    resvg::render(&svg_tree, Default::default(), &mut pixmap.as_mut());

    (pixmap.data().to_vec(), w, h)
}

fn load_icon_svg()-> anyhow::Result<HICON> {

    let (bitmap_data, width, height) = svg2bitmap("assets/logo.svg");
    
    let icon = CreateIcon()

    todo!()
}

fn init_class(hinstance: HINSTANCE, class_name: PCWSTR) -> anyhow::Result<()> {
    let icon = unsafe {
        LoadImageW(
            Some(hinstance),
            w!("logo.ico"),
            IMAGE_ICON,
            0,
            0,
            LR_LOADFROMFILE,
        )
    }?;
    let icon2 = load_icon(hinstance, 1).unwrap();

    let wc = WNDCLASSEXW {
        cbSize: size_of::<WNDCLASSEXW>() as u32,
        hCursor: unsafe { LoadCursorW(None, IDC_ARROW) }?,
        hInstance: hinstance,
        lpszClassName: class_name,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(wndproc),
        hIcon: HICON(icon.0),
        // hIcon: unsafe { LoadIconW(None, IDI_APPLICATION) }.unwrap(),
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
