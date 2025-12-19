use std::sync::Arc;

use crate::{
    app::{
        config::AppConfig,
        connection_manager::{ConnectionManager, DeviceStatusStrings},
    },
    internal::*,
    resource::APP_ICON,
};
use anyhow::Context;
use rust_i18n::t;
use tracing::log;
use windows::{
    Foundation::{Rect, Uri},
    System::Launcher,
    Win32::{
        Foundation::*,
        System::LibraryLoader::GetModuleHandleW,
        UI::{HiDpi::GetDpiForWindow, Shell::*, WindowsAndMessaging::*},
    },
    core::*,
};

#[derive(Debug, Clone)]
pub struct MenuStrings {
    pub connection_list: String,
    pub bluetooth_list: String,
    pub auto_connect: String,
    pub exit: String,
}

impl Default for MenuStrings {
    fn default() -> Self {
        Self {
            connection_list: "Open Connection List(&C)".to_string(),
            bluetooth_list: "Add Bluetooth Device(&B)".to_string(),
            auto_connect: "Auto Connect(&A)".to_string(),
            exit: "Exit(&X)".to_string(),
        }
    }
}

pub struct NotifyIcon {
    window: HWND,
    config: Arc<AppConfig>,
    data: NOTIFYICONDATAW,
    notify_icon_id: NOTIFYICONIDENTIFIER,
    manager: ConnectionManager,
    menu_str: MenuStrings,
}

#[allow(unused)]
impl NotifyIcon {
    const IDM_EXIT: u32 = 1001;
    const IDM_CONNECTION: u32 = 1002;
    const IDM_DEVICES: u32 = 1003;
    const IDM_AUTO_CONNECT: u32 = 1004;

    pub fn new(
        window: HWND,
        config: Arc<AppConfig>,
        callback_message: u32,
        strings: MenuStrings,
    ) -> anyhow::Result<Self> {
        let module = unsafe { GetModuleHandleW(None) }
            .context("Fail to get HMODULE handle for the current application")?;
        let instance = HINSTANCE::from(module);

        let icon = unsafe { LoadIconW(Some(instance), PCWSTR::from_raw(APP_ICON as _)) }
            .unwrap_or(unsafe { LoadIconW(None, IDI_APPLICATION) }.unwrap());

        Ok(Self {
            window,
            data: NOTIFYICONDATAW {
                hWnd: window,
                uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP | NIF_SHOWTIP,
                uCallbackMessage: callback_message,
                Anonymous: NOTIFYICONDATAW_0 { uVersion: 4 },
                hIcon: icon,
                ..Default::default()
            }
            .into(),
            notify_icon_id: NOTIFYICONIDENTIFIER {
                cbSize: size_of::<NOTIFYICONIDENTIFIER>() as u32,
                hWnd: window,
                ..Default::default()
            },
            menu_str: strings,
            manager: ConnectionManager::new(
                window,
                config.clone(),
                DeviceStatusStrings {
                    picker_title: t!("connection_manager.picker_title").to_string(),
                    timeout: t!("connection_manager.timeout").to_string(),
                    connecting: t!("connection_manager.connecting").to_string(),
                    connected: t!("connection_manager.connected").to_string(),
                    denied_by_system: t!("connection_manager.denied_by_system").to_string(),
                    not_found: t!("connection_manager.not_found").to_string(),
                    unknown_reason: t!("connection_manager.unknown_reason").to_string(),
                    disconnected: t!("connection_manager.disconnected").to_string(),
                },
            )?,
            config,
        })
    }

    pub fn show_menu(&self) -> anyhow::Result<()> {
        let mut point = POINT::default();
        unsafe { GetCursorPos(&mut point) }.context("Failed to get cursor position")?;

        // 需要先将窗口设为前台窗口，菜单才能正常工作
        unsafe { SetForegroundWindow(self.window) }.warn("Fail to set foreground window");

        let hmenu = unsafe { CreatePopupMenu() }.context("Failed to create popup menu")?;
        let strings = self.menu_str.clone();

        unsafe {
            AppendMenuW(
                hmenu,
                MF_STRING,
                Self::IDM_CONNECTION as usize,
                PCWSTR::from_raw(HSTRING::from(strings.connection_list).as_ptr()),
            )
        }?;

        unsafe {
            AppendMenuW(
                hmenu,
                MF_STRING,
                Self::IDM_DEVICES as usize,
                PCWSTR::from_raw(HSTRING::from(strings.bluetooth_list).as_ptr()),
            )
        }?;

        let checked = {
            if self.config.auto_connect() {
                MF_CHECKED
            } else {
                MF_UNCHECKED
            }
        };
        unsafe {
            AppendMenuW(
                hmenu,
                MF_STRING | checked,
                Self::IDM_AUTO_CONNECT as usize,
                PCWSTR::from_raw(HSTRING::from(strings.auto_connect).as_ptr()),
            )
        }?;

        unsafe { AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null()) }.unwrap();

        unsafe {
            AppendMenuW(
                hmenu,
                MF_STRING,
                Self::IDM_EXIT as usize,
                PCWSTR::from_raw(HSTRING::from(strings.exit).as_ptr()),
            )
        }?;

        unsafe {
            TrackPopupMenu(
                hmenu,
                TPM_BOTTOMALIGN,
                point.x,
                point.y,
                Some(0),
                self.window,
                None,
            )
        }
        .warn("Fail to popup menu");

        Ok(())
    }

    pub fn show_picker(&self, x: i32, y: i32) -> anyhow::Result<()> {
        self.manager
            .show(Rect {
                X: x as f32,
                Y: y as f32,
                Width: 0.0,
                Height: 0.0,
            })
            .context("Fail to show device picker")
    }

    pub fn add(&self) -> anyhow::Result<()> {
        unsafe { Shell_NotifyIconW(NIM_ADD, &self.data) }.context("Failed to add tray icon")?;
        unsafe { Shell_NotifyIconW(NIM_SETVERSION, &self.data) }
            .context("Fail to set NotifyIcon's Version")?;

        Ok(())
    }

    pub fn delete(&self) -> anyhow::Result<()> {
        unsafe { Shell_NotifyIconW(NIM_DELETE, &self.data) }
            .context("Failed to remove tray icon")?;
        Ok(())
    }

    pub fn handle_message(&self, message: u32) -> anyhow::Result<()> {
        // log::debug!("Notify Icon Message: {}", message);
        match message {
            WM_RBUTTONUP => {
                log::debug!("Notify Icon Right Button Up");
                self.show_menu()?;
            }
            NIN_SELECT => {
                // Need NIM_SETVERSION to 4 to receive this message
                log::debug!("Notify Icon Select");
                self.show_connection_list()?;
            }
            _ => {}
        }
        Ok(())
    }

    pub fn handle_command(&self, message_id: u32) -> anyhow::Result<()> {
        match message_id {
            Self::IDM_DEVICES => {
                let uri =
                    Uri::CreateUri(&windows::core::HSTRING::from("ms-settings:bluetooth")).unwrap();
                Launcher::LaunchUriAsync(&uri).unwrap();
            }
            Self::IDM_CONNECTION => {
                log::info!("Open connection list menu item clicked");
                self.show_connection_list()?;
            }
            Self::IDM_AUTO_CONNECT => {
                let auto_connect = self.config.auto_connect();
                self.config.set_auto_connect(!auto_connect);
                log::info!("Auto Connect set to {}", !auto_connect);
            }
            Self::IDM_EXIT => {
                unsafe { PostQuitMessage(0) };
            }
            _ => {}
        }
        Ok(())
    }

    fn show_connection_list(&self) -> anyhow::Result<()> {
        let rect = unsafe { Shell_NotifyIconGetRect(&self.notify_icon_id) }
            .context("Fail to get notify icon rect")?;
        let scale = unsafe { GetDpiForWindow(self.window) } as f32 / USER_DEFAULT_SCREEN_DPI as f32;

        self.manager
            .show(Rect {
                X: rect.left as f32 / scale,
                Y: rect.top as f32 / scale,
                Width: (rect.right - rect.left) as f32 / scale,
                Height: (rect.bottom - rect.top) as f32 / scale,
            })
            .context("Fail to show popup of connections")
    }
}
