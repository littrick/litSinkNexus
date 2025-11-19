mod connection_manager;
mod notify_icon;

use anyhow::Context;
use tracing::{log, trace_span};
use windows::{
    Win32::{
        Foundation::*,
        Graphics::Gdi::UpdateWindow,
        System::LibraryLoader::GetModuleHandleW,
        UI::{Input::KeyboardAndMouse::VK_ESCAPE, WindowsAndMessaging::*},
    },
    core::*,
};

use crate::{app::notify_icon::NotifyIcon, internal::*};

pub struct Application {
    window: HWND,
    notify_icon: Option<NotifyIcon>,
}

impl Application {
    const WM_NOTIFYICON: u32 = WM_USER + 1;

    pub fn run() -> anyhow::Result<Self> {
        let window = HWND::default();
        let app = Self {
            window,
            notify_icon: None,
        };

        let module = unsafe { GetModuleHandleW(None) }
            .context("Fail to get HMODULE handle for the current application")?;

        let instance = HINSTANCE::from(module);

        let cursor =
            unsafe { LoadCursorW(None, IDC_ARROW) }.context("Failed to load arrow cursor")?;

        let class_name = HSTRING::from(Self::class_name());
        let window_class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            hCursor: cursor,
            hInstance: instance,
            lpszClassName: PCWSTR::from_raw(class_name.as_ptr()),
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(Self::wndproc),
            ..Default::default()
        };

        log::debug!("Registering window class {:?}", Self::class_name());
        let atom = unsafe { RegisterClassExW(&window_class) };
        if atom == 0 {
            return win_error("Failed to register window class");
        }

        let window = unsafe {
            let class_name = HSTRING::from(Self::class_name());
            CreateWindowExW(
                WS_EX_LAYERED,
                PCWSTR::from_raw(class_name.as_ptr()),
                w!(""),
                WS_POPUP,
                0,
                0,
                GetSystemMetrics(SM_CXSCREEN),
                GetSystemMetrics(SM_CYSCREEN),
                None,
                None,
                Some(instance),
                Some(&app as *const _ as *const _),
            )
        }
        // .context("Failed to create application window")?;
        .context("Failed to create application window")?;

        // let _ = unsafe { ShowWindow(window, SW_SHOW) };
        let _ = unsafe { UpdateWindow(window) };

        let mut msg = MSG::default();
        while (unsafe { GetMessageW(&mut msg, None, 0, 0) }).into() {
            let _ = unsafe { TranslateMessage(&msg) };
            unsafe { DispatchMessageW(&msg) };
        }

        Ok(app)
    }

    // #[tracing::instrument]
    fn handle_message(&self, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match message {
            WM_DESTROY => {
                self.notify_icon.as_ref().unwrap().delete().unwrap();
                unsafe { PostQuitMessage(0) };
            }
            WM_KEYDOWN => {
                // Handle ESC key to quit application
                let s = trace_span!("WM_KEYDOWN");
                let _guard = s.enter();

                log::info!("Key down: {:?}", wparam.0 as u32);
                if wparam.0 as u32 == VK_ESCAPE.0.into() {
                    unsafe { PostQuitMessage(0) };
                }
            }
            Self::WM_NOTIFYICON => {
                self.notify_icon
                    .as_ref()
                    .unwrap()
                    .handle_message(lparam.0 as u32)
                    .unwrap();
            }
            WM_COMMAND => {
                self.notify_icon
                    .as_ref()
                    .unwrap()
                    .handle_command((wparam.0 & 0xffff) as u32)
                    .unwrap();
            }
            _ => {
                return unsafe { DefWindowProcW(self.window, message, wparam, lparam) };
            }
        }

        // unsafe { DefWindowProcW(self.window, message, wparam, lparam) }
        LRESULT(0)
    }

    fn class_name() -> &'static str {
        std::any::type_name::<Self>()
    }

    // #[tracing::instrument]
    extern "system" fn wndproc(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        unsafe {
            if message == WM_CREATE {
                let createstruct = &*(lparam.0 as *const CREATESTRUCTW);
                let this = createstruct.lpCreateParams as *mut Self;
                if !this.is_null() {
                    (*this).window = window;
                    let notify_icon =
                        NotifyIcon::new(window, Self::WM_NOTIFYICON, Default::default()).unwrap();

                    notify_icon.add().unwrap();
                    (*this).notify_icon = Some(notify_icon);

                    SetWindowLongPtrW(window, GWLP_USERDATA, this as isize);
                }
            } else {
                let this = GetWindowLongPtrW(window, GWLP_USERDATA) as *mut Self;
                if !this.is_null() {
                    return (*this).handle_message(message, wparam, lparam);
                }
            }
            DefWindowProcW(window, message, wparam, lparam)
        }
    }
}
