use anyhow::Context;
use tracing::log;
use windows::{
    Foundation::{Rect, Uri},
    System::Launcher,
    Win32::{
        Foundation::*,
        UI::{HiDpi::GetDpiForWindow, Shell::*, WindowsAndMessaging::*},
    },
    core::*,
};

use crate::{app::connection_manager::ConnectionManager, internal::*};

#[derive(Debug)]
pub struct MenuStrings {
    bluetooth_list: String,
    connection_list: String,
    exit: String,
}

impl Default for MenuStrings {
    fn default() -> Self {
        Self {
            bluetooth_list: "Add Bluetooth Device(&B)".to_string(),
            connection_list: "Open Connection List(&C)".to_string(),
            exit: "Exit(&X)".to_string(),
        }
    }
}

pub struct NotifyIcon {
    window: HWND,
    notify_icon_data: NOTIFYICONDATAW,
    notify_icon_id: NOTIFYICONIDENTIFIER,
    manager: ConnectionManager,
    menu: HMENU,
}

impl NotifyIcon {
    const IDM_EXIT: u32 = 1001;
    const IDM_CONNECTION: u32 = 1002;
    const IDM_DEVICES: u32 = 1003;

    pub fn new(window: HWND, callback_message: u32, strings: MenuStrings) -> anyhow::Result<Self> {
        let hmenu = unsafe { CreatePopupMenu() }.context("Failed to create popup menu")?;

        unsafe {
            AppendMenuW(
                hmenu,
                MF_STRING,
                Self::IDM_DEVICES as usize,
                PCWSTR::from_raw(HSTRING::from(strings.bluetooth_list).as_ptr()),
            )
        }?;

        unsafe {
            AppendMenuW(
                hmenu,
                MF_STRING,
                Self::IDM_CONNECTION as usize,
                PCWSTR::from_raw(HSTRING::from(strings.connection_list).as_ptr()),
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

        Ok(Self {
            window,
            notify_icon_data: NOTIFYICONDATAW {
                hWnd: window,
                uFlags: NIF_ICON | NIF_MESSAGE,
                uCallbackMessage: callback_message,
                Anonymous: NOTIFYICONDATAW_0 { uVersion: 4 },
                hIcon: unsafe { LoadIconW(None, IDI_APPLICATION) }.unwrap(),
                ..Default::default()
            },
            notify_icon_id: NOTIFYICONIDENTIFIER {
                cbSize: size_of::<NOTIFYICONIDENTIFIER>() as u32,
                hWnd: window,
                ..Default::default()
            },
            menu: hmenu,
            manager: ConnectionManager::new(window, Default::default())?,
        })
    }

    pub fn show_menu(&self) -> anyhow::Result<()> {
        let mut point = POINT::default();
        unsafe { GetCursorPos(&mut point) }.context("Failed to get cursor position")?;

        // 需要先将窗口设为前台窗口，菜单才能正常工作
        unsafe { SetForegroundWindow(self.window) }.warn("Fail to set foreground window");

        unsafe {
            TrackPopupMenu(
                self.menu,
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

    pub fn add(&self) -> anyhow::Result<()> {
        unsafe { Shell_NotifyIconW(NIM_ADD, &self.notify_icon_data) }
            .context("Failed to add tray icon")?;

        unsafe { Shell_NotifyIconW(NIM_SETVERSION, &self.notify_icon_data) }
            .context("Fail to set NotifyIcon's Version")?;

        Ok(())
    }

    pub fn delete(&self) -> anyhow::Result<()> {
        unsafe { Shell_NotifyIconW(NIM_DELETE, &self.notify_icon_data) }
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
