use crate::{
    app::{
        config::AppConfig,
        notify_icon::{MenuStrings, NotifyIcon},
    },
    internal::*,
};
use anyhow::Context;
use rust_i18n::t;
use std::{cell::LazyCell, sync::Arc};
use tracing::log;
use windows::{
    Win32::{
        Foundation::*, Graphics::Gdi::UpdateWindow, System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::*,
    },
    core::*,
};

pub struct Application {
    window: HWND,
    config: Arc<AppConfig>,
    notify_icon: Option<NotifyIcon>,
}

impl Application {
    const CLASS_NAME: LazyCell<&'static str> = LazyCell::new(|| std::any::type_name::<Self>());
    const WINDOW_NAME: LazyCell<&'static str> = LazyCell::new(|| "LitAudioSinkNexusHiddenWindow");
    const WM_NOTIFYICON: u32 = WM_USER + 1;
    const WM_SHOW_PICKER: u32 = WM_USER + 2;
    const WM_TASKBAR_CREATED: LazyCell<u32> =
        LazyCell::new(|| unsafe { RegisterWindowMessageW(w!("TaskbarCreated")) });

    pub fn run(config: AppConfig) -> anyhow::Result<Self> {
        let window = HWND::default();
        let app = Self {
            window,
            config: Arc::new(config),
            notify_icon: None,
        };

        match app.find_exists() {
            Ok(wnd) => {
                log::info!("Found existing application window, activating...");
                unsafe { PostMessageW(Some(wnd), Self::WM_SHOW_PICKER, WPARAM(0), LPARAM(0)) }?;
            }
            Err(_) => {
                app.main_loop()?;
            }
        }

        Ok(app)
    }
}

impl Application {
    fn main_loop(&self) -> anyhow::Result<()> {
        let module = unsafe { GetModuleHandleW(None) }
            .context("Fail to get HMODULE handle for the current application")?;

        let instance = HINSTANCE::from(module);

        let cursor =
            unsafe { LoadCursorW(None, IDC_ARROW) }.context("Failed to load arrow cursor")?;

        let wnd_class = HSTRING::from(*Self::CLASS_NAME);
        let window_class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            hCursor: cursor,
            hInstance: instance,
            lpszClassName: PCWSTR::from_raw(wnd_class.as_ptr()),
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(Self::wndproc),
            ..Default::default()
        };

        log::debug!("Registering window class {:?}", *Self::CLASS_NAME);
        let atom = unsafe { RegisterClassExW(&window_class) };
        if atom == 0 {
            return win_error("Failed to register window class");
        }

        let wnd_name = HSTRING::from(*Self::WINDOW_NAME);
        let window = unsafe {
            let wnd_class = HSTRING::from(*Self::CLASS_NAME);
            CreateWindowExW(
                WS_EX_LAYERED,
                PCWSTR::from_raw(wnd_class.as_ptr()),
                PCWSTR::from_raw(wnd_name.as_ptr()),
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
        // .context("Failed to create application window")?;
        .context("Failed to create application window")?;

        // let _ = unsafe { ShowWindow(window, SW_SHOW) };
        let _ = unsafe { UpdateWindow(window) };

        let mut msg = MSG::default();
        while (unsafe { GetMessageW(&mut msg, None, 0, 0) }).into() {
            let _ = unsafe { TranslateMessage(&msg) };
            unsafe { DispatchMessageW(&msg) };
        }
        Ok(())
    }

    fn find_exists(&self) -> anyhow::Result<HWND> {
        let wnd_class = HSTRING::from(*Self::CLASS_NAME);
        let wnd_name = HSTRING::from(*Self::WINDOW_NAME);
        let wnd = unsafe {
            FindWindowW(
                PCWSTR::from_raw(wnd_class.as_ptr()),
                PCWSTR::from_raw(wnd_name.as_ptr()),
            )
        }?;
        Ok(wnd)
    }

    // #[tracing::instrument]
    fn handle_message(&self, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match message {
            WM_DESTROY => {
                self.notify_icon.as_ref().unwrap().delete().unwrap();
                unsafe { PostQuitMessage(0) };
            }
            Self::WM_NOTIFYICON => {
                self.notify_icon
                    .as_ref()
                    .unwrap()
                    .handle_message(lparam.0 as u32)
                    .unwrap();
            }
            Self::WM_SHOW_PICKER => {
                let x = unsafe { GetSystemMetrics(SM_CXSCREEN) };
                let y = unsafe { GetSystemMetrics(SM_CYSCREEN) };
                self.notify_icon
                    .as_ref()
                    .unwrap()
                    .show_picker(x, y)
                    .warn("Fail to Show Picker");
            }
            WM_COMMAND => {
                self.notify_icon
                    .as_ref()
                    .unwrap()
                    .handle_command((wparam.0 & 0xffff) as u32)
                    .unwrap();
            }
            msg if msg == *Self::WM_TASKBAR_CREATED => {
                // when explorer.exe restarts, the taskbar is recreated, need to re-add the notify icon
                log::debug!("Taskbar recreated, re-adding notify icon");
                self.notify_icon.as_ref().unwrap().add().unwrap();
            }
            _ => {
                return unsafe { DefWindowProcW(self.window, message, wparam, lparam) };
            }
        }

        // unsafe { DefWindowProcW(self.window, message, wparam, lparam) }
        LRESULT(0)
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
                    let notify_icon = NotifyIcon::new(
                        window,
                        (*this).config.clone(),
                        Self::WM_NOTIFYICON,
                        MenuStrings {
                            bluetooth_list: t!("notify_icon.bluetooth_list").to_string(),
                            connection_list: t!("notify_icon.connection_list").to_string(),
                            exit: t!("notify_icon.exit").to_string(),
                            ..Default::default()
                        },
                    )
                    .unwrap();

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
