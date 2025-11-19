use log::info;
use std::{mem::size_of, sync::Arc};

use windows::{
    Devices::Enumeration::{
        DeviceInformation, DevicePicker, DevicePickerDisplayStatusOptions, DeviceSelectedEventArgs,
    },
    Foundation::{Rect, TypedEventHandler, Uri},
    Media::Audio::AudioPlaybackConnection,
    System::Launcher,
    Win32::{
        Foundation::{COLORREF, GetLastError, HWND, LPARAM, LRESULT, POINT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            HiDpi::GetDpiForWindow,
            Shell::{
                IInitializeWithWindow, NIF_ICON, NIF_MESSAGE, NIM_ADD, NIM_DELETE, NIM_SETVERSION,
                NIN_SELECT, NOTIFYICONDATAW, NOTIFYICONDATAW_0, NOTIFYICONIDENTIFIER,
                Shell_NotifyIconGetRect, Shell_NotifyIconW,
            },
            WindowsAndMessaging::{
                AppendMenuW, CW_USEDEFAULT, CreatePopupMenu, CreateWindowExW, DefWindowProcW,
                DestroyMenu, DispatchMessageW, GetCursorPos, GetMessageW, GetSystemMetrics,
                HWND_TOPMOST, IDC_ARROW, IDI_APPLICATION, LAYERED_WINDOW_ATTRIBUTES_FLAGS,
                LWA_ALPHA, LoadCursorW, LoadIconW, MB_ICONINFORMATION, MB_OK, MF_SEPARATOR,
                MF_STRING, MSG, MessageBoxW, PostMessageW, PostQuitMessage, RegisterClassExW,
                RegisterClassW, SM_CXSCREEN, SM_CYSCREEN, SWP_HIDEWINDOW, SetForegroundWindow,
                SetLayeredWindowAttributes, SetMenuDefaultItem, SetWindowPos, TPM_RIGHTBUTTON,
                TrackPopupMenu, TranslateMessage, WINDOW_EX_STYLE, WM_APP, WM_COMMAND, WM_CREATE,
                WM_DESTROY, WM_LBUTTONUP, WM_NULL, WM_RBUTTONUP, WM_USER, WNDCLASSEXW, WNDCLASSW,
                WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOPMOST, WS_OVERLAPPEDWINDOW, WS_POPUP,
                WS_VISIBLE,
            },
        },
    },
    core::{HSTRING, IInspectable, Interface, PCWSTR, w},
};

const WM_NOTIFYICON: u32 = WM_APP + 1;

// 定义菜单项 ID
const IDM_EXIT: u32 = 1001;
const IDM_SHOW: u32 = 1002;
const IDM_ABOUT: u32 = 1003;
const IDM_SHOW_SETTINGS: u32 = 1004;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let h_instance = unsafe { GetModuleHandleW(None) }?;
    let h_cursor = unsafe { LoadCursorW(None, IDC_ARROW) }?;
    let h_icon = unsafe { LoadIconW(None, IDI_APPLICATION) }?;

    let window_class = WNDCLASSEXW {
        cbSize: size_of::<WNDCLASSEXW>() as u32,
        hInstance: h_instance.into(),
        lpszClassName: w!("awindow"),
        hCursor: h_cursor,
        hIcon: h_icon,
        lpfnWndProc: Some(wnproc),
        ..Default::default()
    };

    let class_atom = unsafe { RegisterClassExW(&window_class) };
    anyhow::ensure!(
        class_atom != 0,
        "Register class fail, Error: {:?}",
        unsafe { GetLastError() }
    );

    let window = unsafe {
        CreateWindowExW(
            // WS_EX_NOACTIVATE | WS_EX_LAYERED | WS_EX_TOPMOST,
            WINDOW_EX_STYLE::default() | WS_EX_LAYERED,
            window_class.lpszClassName,
            w!("windows name"),
            WS_OVERLAPPEDWINDOW,
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
    unsafe { SetLayeredWindowAttributes(window, COLORREF(0), 255, LWA_ALPHA) }.unwrap();

    let mut msg = MSG::default();

    while unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {
        // info!("{}:{} got message {msg:?}", file!(), line!());

        // unsafe { TranslateMessage(&msg) };
        unsafe { DispatchMessageW(&msg) };
    }

    Ok(())
}

extern "system" fn wnproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // info!("{}:{} message: 0x{message:X} wparam: 0x{:X} lparam: 0x{:X}", file!(), line!(), wparam.0, lparam.0);

    let mut result = LRESULT(0);

    let notify_icon_data = NOTIFYICONDATAW {
        hWnd: window,
        uFlags: NIF_ICON | NIF_MESSAGE,
        uCallbackMessage: WM_NOTIFYICON,
        Anonymous: NOTIFYICONDATAW_0 { uVersion: 4 },
        hIcon: unsafe { LoadIconW(None, IDI_APPLICATION) }.unwrap(),
        ..Default::default()
    };

    match message {
        WM_CREATE => {
            info!("WM_CREATE");
            unsafe { Shell_NotifyIconW(NIM_ADD, &notify_icon_data) }.unwrap();
            unsafe { Shell_NotifyIconW(NIM_SETVERSION, &notify_icon_data) }.unwrap();

            // show_device_picker(window);
            // show_device_picker2(window);
        }
        WM_DESTROY => {
            info!("WM_DESTROY");
            unsafe { PostQuitMessage(0) };
            unsafe { Shell_NotifyIconW(NIM_DELETE, &notify_icon_data) }.unwrap();
        }
        WM_NOTIFYICON => {
            let lp = lparam.0 as u32;
            info!("WM_NOTIFYICON event: lp=0x{lp:X} wp=0x{:X}", wparam.0);
            match lp {
                WM_LBUTTONUP => {
                    info!("Notify Icon Left Button Up");
                }
                WM_RBUTTONUP => {
                    info!("Notify Icon Right Button Up");
                    show_context_menu(window);
                }
                NIN_SELECT => {
                    info!("Notify Icon Select");

                    show_device_picker(window);
                }
                _ => {}
            }
        }
        WM_COMMAND => {
            // 处理菜单项点击
            let menu_id = wparam.0 as u32;
            match menu_id {
                IDM_SHOW_SETTINGS => {
                    info!("打开蓝牙设备列表");
                    let uri =
                        Uri::CreateUri(&windows::core::HSTRING::from("ms-settings:bluetooth"))
                            .unwrap();
                    Launcher::LaunchUriAsync(&uri).unwrap();
                }
                IDM_SHOW => {
                    info!("显示窗口");
                }
                IDM_ABOUT => {
                    unsafe {
                        MessageBoxW(
                            Some(window),
                            w!("系统托盘示例程序"),
                            w!("关于"),
                            MB_OK | MB_ICONINFORMATION,
                        )
                    };
                }
                IDM_EXIT => {
                    unsafe { PostQuitMessage(0) };
                }
                _ => {}
            }
        }
        _ => result = unsafe { DefWindowProcW(window, message, wparam, lparam) },
    };

    result

    // LRESULT(0)
}

fn show_context_menu(hwnd: HWND) {
    // 创建弹出菜单
    let hmenu = unsafe { CreatePopupMenu() }.unwrap();

    // 添加菜单项
    unsafe {
        AppendMenuW(
            hmenu,
            MF_STRING,
            IDM_SHOW_SETTINGS as usize,
            w!("打开蓝牙设备列表"),
        )
    }
    .unwrap();
    unsafe { AppendMenuW(hmenu, MF_STRING, IDM_SHOW as usize, w!("显示(&S)")) }.unwrap();
    unsafe { AppendMenuW(hmenu, MF_STRING, IDM_ABOUT as usize, w!("关于(&A)")) }.unwrap();
    unsafe { AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null()) }.unwrap();
    unsafe { AppendMenuW(hmenu, MF_STRING, IDM_EXIT as usize, w!("退出(&X)")) }.unwrap();

    // 设置默认菜单项（加粗显示）
    unsafe { SetMenuDefaultItem(hmenu, IDM_SHOW, 0) }.unwrap();

    // 获取鼠标位置
    let mut pt = POINT { x: 0, y: 0 };
    unsafe { GetCursorPos(&mut pt) }.unwrap();

    // 需要先将窗口设为前台窗口，菜单才能正常工作
    unsafe { SetForegroundWindow(hwnd) }.unwrap();

    // 显示菜单
    unsafe { TrackPopupMenu(hmenu, TPM_RIGHTBUTTON, pt.x, pt.y, Some(0), hwnd, None) }.unwrap();

    // 点击菜单外区域时关闭菜单
    unsafe { PostMessageW(Some(hwnd), WM_NULL, WPARAM(0), LPARAM(0)) }.unwrap();

    // 销毁菜单
    let _ = unsafe { DestroyMenu(hmenu) }.unwrap();
}

fn show_device_picker(hwnd: HWND) {
    info!("显示设备选择器");
    let selector = AudioPlaybackConnection::GetDeviceSelector().unwrap();
    let device_picker = Arc::new(DevicePicker::new().unwrap());
    let all_devices = DeviceInformation::FindAllAsyncAqsFilter(&selector)
        .unwrap()
        .join()
        .unwrap();
    info!(
        "Found {} audio playback devices",
        all_devices.Size().unwrap()
    );
    for device in all_devices {
        device_picker
            .SetDisplayStatus(&device, &HSTRING::new(), DevicePickerDisplayStatusOptions::None)
            .unwrap();
    }

    // 选择音频播放设备

    device_picker
        .Filter()
        .unwrap()
        .SupportedDeviceSelectors()
        .unwrap()
        .Append(&selector)
        .unwrap();

    device_picker
        .DeviceSelected(
            &TypedEventHandler::<DevicePicker, DeviceSelectedEventArgs>::new({
                let device_picker = device_picker.clone();
                move |sender, args| {
                    let devcie_info = args.as_ref().unwrap().SelectedDevice().unwrap();

                    info!(
                        "Selected device: {} - {}",
                        devcie_info.Name().unwrap(),
                        devcie_info.Id().unwrap()
                    );

                    device_picker
                        .SetDisplayStatus(
                            &devcie_info,
                            &HSTRING::from("连接中"),
                            DevicePickerDisplayStatusOptions::ShowProgress
                                | DevicePickerDisplayStatusOptions::ShowDisconnectButton,
                        )
                        .unwrap();

                    let connection = AudioPlaybackConnection::TryCreateFromId(
                        &args
                            .as_ref()
                            .unwrap()
                            .SelectedDevice()
                            .unwrap()
                            .Id()
                            .unwrap(),
                    )
                    .unwrap();
                    connection
                        .StateChanged(
                            &TypedEventHandler::<AudioPlaybackConnection, IInspectable>::new(
                                |sender, inspectable| {
                                    println!(
                                        "Connection state changed: {:?}",
                                        sender.as_ref().unwrap().State().unwrap()
                                    );

                                    Ok(())
                                },
                            ),
                        )
                        .unwrap();

                    connection.StartAsync().unwrap();
                    let res = connection.Open().unwrap();
                    info!("Connection opened: {:?}", res);

                    // device_picker
                    //     .SetDisplayStatus(
                    //         &devcie_info,
                    //         &HSTRING::from("已连接"),
                    //         DevicePickerDisplayStatusOptions::None,
                    //     )
                    //     .unwrap();

                    Ok(())
                }
            }),
        )
        .unwrap();

    // let iniitialize_with_window: IInitializeWithWindow = device_picker.cast().unwrap();
    // unsafe { iniitialize_with_window.Initialize(hwnd) }.unwrap();

    let notify_icon_id = NOTIFYICONIDENTIFIER {
        cbSize: size_of::<NOTIFYICONIDENTIFIER>() as u32,
        hWnd: hwnd,
        ..Default::default()
    };
    let dpi = unsafe { GetDpiForWindow(hwnd) };
    info!("DPI: {}", dpi);

    let rect = unsafe { Shell_NotifyIconGetRect(&notify_icon_id as *const _) }.unwrap();

    let selection = Rect {
        X: rect.left as f32 * 96.0 / dpi as f32,
        Y: rect.top as f32 * 96.0 / dpi as f32,
        Width: (rect.right - rect.left) as f32 * 96.0 / dpi as f32,
        Height: (rect.bottom - rect.top) as f32 * 96.0 / dpi as f32,
    };

    unsafe {
        SetWindowPos(
            hwnd,
            None,
            0,
            0,
            GetSystemMetrics(SM_CXSCREEN),
            GetSystemMetrics(SM_CYSCREEN),
            SWP_HIDEWINDOW,
        )
    }
    .unwrap();
    unsafe { SetForegroundWindow(hwnd) }.unwrap();

    info!("Selection rect: {:?}", selection);
    device_picker.Show(selection).unwrap();
}
