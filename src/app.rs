mod connection_manager;
mod context;
mod notify_icon;

use anyhow::Context;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};
use tracing::{Level, log, span, trace_span};
use windows::{
    Devices::Enumeration::DevicePicker,
    Foundation::*,
    Media::Audio::*,
    Win32::{
        Foundation::*,
        Graphics::Gdi::UpdateWindow,
        System::LibraryLoader::GetModuleHandleW,
        UI::{Input::KeyboardAndMouse::VK_ESCAPE, WindowsAndMessaging::*},
    },
    core::*,
};

use crate::{
    app::notify_icon::{MenuStrings, NotifyIcon},
    prelude::*,
};

#[derive(Default, Clone)]
pub struct Application {
    window: HWND,
    notify_icon: NotifyIcon,
}

impl Application {
    const CLASS_NAME: &str = env!("CARGO_PKG_NAME");
    const WM_NOTIFYICON: u32 = WM_USER + 1;

    #[tracing::instrument]
    pub fn new() -> Self {
        Default::default()
    }

    // #[tracing::instrument]
    pub fn run(&self) -> anyhow::Result<()> {
        let module = unsafe { GetModuleHandleW(None) }
            .context("Fail to get HMODULE handle for the current application")?;

        let instance = HINSTANCE::from(module);

        let class_name = PCWSTR::from_raw(HSTRING::from(Self::CLASS_NAME).as_ptr());
        let cursor =
            unsafe { LoadCursorW(None, IDC_ARROW) }.context("Failed to load arrow cursor")?;

        let window_class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            hCursor: cursor,
            hInstance: instance,
            lpszClassName: class_name,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(Self::wndproc),
            ..Default::default()
        };

        let atom = unsafe { RegisterClassExW(&window_class) };
        if atom == 0 {
            return win_error("Failed to register window class");
        }

        let window = unsafe {
            CreateWindowExW(
                // WINDOW_EX_STYLE::default(),
                WS_EX_LAYERED,
                class_name,
                w!(""),
                WS_POPUP,
                0,
                0,
                GetSystemMetrics(SM_CXSCREEN),
                GetSystemMetrics(SM_CYSCREEN),
                None,
                None,
                Some(instance),
                Some(self as *const _ as *const _),
            )
        }
        .context("Failed to create application window")?;

        // let _ = unsafe { ShowWindow(window, SW_SHOW) };
        let _ = unsafe { UpdateWindow(window) };

        let mut msg = MSG::default();

        while (unsafe { GetMessageW(&mut msg, None, 0, 0) }).into() {
            let span = trace_span!("message_loop");
            let _guard = span.enter();
            let _ = unsafe { TranslateMessage(&msg) };
            unsafe { DispatchMessageW(&msg) };
        }

        Ok(())
    }

    // #[tracing::instrument]
    fn handle_message(&self, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match message {
            WM_DESTROY => {
                self.notify_icon.delete().unwrap();
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
                log::info!("Notify Icon Message: {}", lparam.0 as u32);
                self.notify_icon.handle_message(lparam.0 as u32).unwrap();
            }
            WM_COMMAND => {
                self.notify_icon
                    .handle_command((wparam.0 & 0xffff) as u32)
                    .unwrap();
            }
            _ => {}
        }

        unsafe { DefWindowProcW(self.window, message, wparam, lparam) }
    }

    #[tracing::instrument]
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
                        NotifyIcon::new(window, Self::WM_NOTIFYICON, MenuStrings::default())
                            .unwrap();

                    notify_icon.add().unwrap();
                    (*this).notify_icon = notify_icon;

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
